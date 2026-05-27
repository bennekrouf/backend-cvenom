// src/web/handlers/cv_handlers/portfolio.rs
//! Portfolio PDF generation: calls AI to generate [[projects]] from the profile,
//! writes them into the profile's cv_params.toml, then compiles with Typst.

use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::{FsOps, ServiceClient, TemplateEngine};
use crate::web::handlers::payment_handlers::check_and_deduct_credits;
use crate::utils::{normalize_language, normalize_profile_name};
use crate::web::types::WithConversationId;
use crate::web::types::{
    GeneratePdfResponse, ResponseType, ServerConfig, StandardErrorResponse, StandardRequest,
};
use crate::{CvConfig, CvGenerator};
use crate::types::cv_data::CvConverter;
use graflog::{app_log, app_span};
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GeneratePortfolioRequest {
    pub profile: String,
    pub lang: Option<String>,
    /// Override the template; defaults to "portfolio"
    pub template: Option<String>,
}

pub async fn generate_portfolio_handler(
    request: Json<StandardRequest<GeneratePortfolioRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<GeneratePdfResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    check_and_deduct_credits(&user.email, 20, conversation_id.clone(), "portfolio_generation")
        .await?;

    let span = app_span!("portfolio_generation",
        user_email = %user.email,
        tenant = %tenant.tenant_name,
        profile = %request.data.profile,
    );
    let _enter = span.enter();

    let template_manager = TemplateEngine::new(config.templates_dir.clone()).map_err(|e| {
        err("TEMPLATE_INIT_ERROR", format!("Template system error: {}", e), conversation_id.clone())
    })?;

    let lang = normalize_language(request.data.lang.as_deref());
    let normalized_profile = normalize_profile_name(&request.data.profile);
    let template_id = request.data.template.as_deref().unwrap_or("portfolio").to_string();

    if template_manager.get_template(&template_id).is_none() {
        return Err(err(
            "TEMPLATE_NOT_FOUND",
            format!("Template '{}' not found", template_id),
            conversation_id,
        ));
    }

    let tenant_data_dir = get_tenant_folder_path(&user.email, &config.data_dir);
    FsOps::ensure_dir_exists(&tenant_data_dir).await.map_err(|e| {
        err("TENANT_DIR_ERROR", format!("Failed to access tenant directory: {}", e), conversation_id.clone())
    })?;

    let profile_dir = tenant_data_dir.join(&normalized_profile);
    if !profile_dir.exists() {
        return Err(err(
            "PROFILE_NOT_FOUND",
            format!("Profile '{}' not found", request.data.profile),
            conversation_id,
        ));
    }

    // ── 1. Load existing CV data from profile ─────────────────────────────────
    let toml_path = profile_dir.join("cv_params.toml");

    // Try language-specific experiences file, then generic fallback
    let exp_path = {
        let lang_specific = profile_dir.join(format!("experiences_{}.typ", lang));
        let generic_en   = profile_dir.join("experiences_en.typ");
        let legacy       = profile_dir.join("experiences.typ");
        if lang_specific.exists() { lang_specific }
        else if generic_en.exists() { generic_en }
        else if legacy.exists() { legacy }
        else { lang_specific } // pass non-existent; from_files handles gracefully
    };

    let cv_data = match CvConverter::from_files(&toml_path, &exp_path) {
        Ok(data) => data,
        Err(e) => {
            // If experiences file is missing try loading with the toml as fallback
            // so at minimum we get name / skills / summary for the AI prompt
            app_log!(warn, "Could not load full CV data ({}), retrying without experiences", e);
            CvConverter::from_files(&toml_path, &toml_path).map_err(|e2| {
                err("PROFILE_LOAD_ERROR", format!("Failed to load profile data: {}", e2), conversation_id.clone())
            })?
        }
    };

    // ── 2. Call AI service to generate [[projects]] TOML ─────────────────────
    let service_client = ServiceClient::new(cv_service_url.inner().clone(), 120).map_err(|e| {
        err("SERVICE_CLIENT_ERROR", format!("Failed to create service client: {}", e), conversation_id.clone())
    })?;

    app_log!(info, "Calling AI service to generate portfolio projects for '{}'", normalized_profile);

    let projects_toml = service_client
        .generate_portfolio_content(&cv_data, &lang)
        .await
        .map_err(|e| {
            err(
                "AI_SERVICE_ERROR",
                format!("Portfolio AI generation failed: {}", e),
                conversation_id.clone(),
            )
        })?;

    app_log!(info, "AI generated {} chars of projects TOML", projects_toml.len());

    // ── 3. Merge generated projects into the profile's cv_params.toml ────────
    if !projects_toml.trim().is_empty() {
        let existing_toml = std::fs::read_to_string(&toml_path).unwrap_or_default();

        // Strip any existing [[projects]] blocks and everything after them
        let base_toml = strip_projects_section(&existing_toml);
        let updated_toml = format!(
            "{}\n\n# Projects generated by AI — edit freely\n{}\n",
            base_toml.trim_end(),
            projects_toml
        );

        if let Err(e) = std::fs::write(&toml_path, &updated_toml) {
            app_log!(warn, "Could not save generated projects to cv_params.toml: {}", e);
            // Non-fatal: proceed with compilation using what's already in the file
        } else {
            app_log!(info, "Saved generated projects to {}", toml_path.display());
        }
    }

    // ── 4. Compile portfolio PDF ──────────────────────────────────────────────
    let cv_config = CvConfig::new(&normalized_profile, &lang)
        .with_template(template_id)
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    match CvGenerator::new(cv_config) {
        Ok(generator) => match generator.generate().await {
            Ok(output_path) => {
                let filename = output_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("portfolio.pdf")
                    .to_string();

                let base_url = env::var("PUBLIC_BASE_URL")
                    .unwrap_or_else(|_| "https://api.cvenom.com".to_string());

                app_log!(info, "Portfolio generated: {}", filename);

                let download_url = format!("{}/outputs/{}", base_url, filename);
                crate::email::send_email(
                    &auth.user().email,
                    crate::email::EmailKind::PortfolioReady {
                        profile: normalized_profile.clone(),
                        filename: filename.clone(),
                        download_url: download_url.clone(),
                    },
                    &lang,
                );
                crate::email::notify_admin(
                    crate::email::EmailKind::AdminActivity {
                        user_email: auth.user().email.clone(),
                        action: "Portfolio generated".to_string(),
                        detail: format!("profile={}", normalized_profile),
                    },
                );

                // Persist user's preferred language
                if let Ok(pool) = db_config.pool() {
                    let email = auth.user().email.clone();
                    let preferred = lang.clone();
                    let pool = pool.clone();
                    tokio::spawn(async move {
                        let repo = crate::core::database::TenantRepository::new(&pool);
                        if let Err(e) = repo.update_preferred_lang(&email, &preferred).await {
                            graflog::app_log!(warn, "update_preferred_lang failed for {}: {}", email, e);
                        }
                    });
                }

                Ok(Json(GeneratePdfResponse {
                    response_type: ResponseType::File,
                    success: true,
                    message: "Portfolio generated successfully".to_string(),
                    download_url,
                    filename,
                    profile: normalized_profile,
                    conversation_id,
                }))
            }
            Err(e) => {
                app_log!(error, "Portfolio compilation failed: {}", e);
                Err(err("GENERATION_ERROR", format!("Portfolio compilation failed: {}", e), conversation_id))
            }
        },
        Err(e) => Err(err("CONFIG_ERROR", format!("Generator init failed: {}", e), conversation_id)),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn err(code: &str, msg: String, cid: Option<String>) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(msg, code.to_string(), vec![], cid))
}

/// Remove all `[[projects]]` blocks from TOML content.
/// Everything from the first `[[projects]]` line to end-of-string is stripped.
fn strip_projects_section(toml: &str) -> String {
    if let Some(idx) = toml.find("\n[[projects]]") {
        toml[..idx].to_string()
    } else if toml.starts_with("[[projects]]") {
        String::new()
    } else {
        toml.to_string()
    }
}
