// src/web/handlers/cv_handlers/optimize.rs
//! CV optimization handler — optimizes via LLM then saves files + generates PDF.

use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::{FsOps, ServiceClient, TemplateEngine};
use crate::web::handlers::payment_handlers::check_and_deduct_credits;
use crate::types::cv_data::{CvConverter, CvJson};
use crate::types::response::OptimizeResponse;
use crate::utils::{normalize_language, normalize_profile_name};
use crate::web::types::WithConversationId;
use crate::web::types::{
    DataResponse, PdfResponse, ServerConfig, StandardErrorResponse, StandardRequest,
};
use crate::{CvConfig, CvGenerator};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

use super::helpers::{load_profile_cv_data, normalize_template, save_profile_cv_data};

/// Request body shared by both optimize endpoints.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct OptimizeCvRequest {
    /// Serialised CvJson of the profile to optimise.
    /// If omitted the handler will load the CV data from the profile directory on disk.
    pub cv_json: Option<String>,
    /// Public job posting URL (LinkedIn, Indeed, company careers page …)
    pub job_url: String,
    /// Raw job description text — when provided, scraping is skipped entirely.
    pub job_description: Option<String>,
    /// Profile name — used to save the optimised result back to disk.
    pub profile: String,
    /// Language for the Typst experiences file (defaults to "en").
    pub lang: Option<String>,
    /// CV template for PDF generation (defaults to "default").
    pub template: Option<String>,
}

// ── Shared optimization pipeline ──────────────────────────────────────────────

/// Runs the full optimization pipeline:
/// 1. Call cv-import service  (scrape job → keyword extraction → ATS rewrite)
/// 2. Save optimized TOML + Typst back to the profile directory on disk
///
/// Returns the enriched `OptimizeResponse` and the resolved `CvJson`.
async fn run_optimization(
    cv_data: &CvJson,
    profile: &str,
    lang: &str,
    job_url: &str,
    job_description: Option<&str>,
    tenant_data_dir: &std::path::Path,
    cv_service_url: &str,
    conversation_id: Option<String>,
) -> Result<(OptimizeResponse, CvJson), Json<StandardErrorResponse>> {
    // ── 1. Init service client ────────────────────────────────────────────────
    let service_client = match ServiceClient::new(cv_service_url.to_string(), 30) {
        Ok(c) => c,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("Service initialization failed: {}", e),
                "SERVICE_INIT_FAILED".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )));
        }
    };

    // ── 2. Call cv-import optimization service ────────────────────────────────
    let optimization_response = match service_client.optimize_cv(cv_data, job_url, job_description).await {
        Ok(r) => r,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("CV optimization failed: {}", e),
                "OPTIMIZATION_FAILED".to_string(),
                vec![
                    "Verify the job URL is publicly accessible".to_string(),
                    "Ensure the CV data is valid JSON".to_string(),
                ],
                conversation_id,
            )));
        }
    };

    let optimized_cv_json = optimization_response.optimized_cv.clone();

    // ── 3. Convert optimized CvJson → Typst (for the response payload) ────────
    let optimized_typst = match CvConverter::to_typst(&optimized_cv_json, lang) {
        Ok(t) => t,
        Err(e) => {
            app_log!(error, "Failed to convert optimized CV to Typst: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Optimization conversion failed".to_string(),
                "CONVERSION_ERROR".to_string(),
                vec!["Try again later".to_string()],
                conversation_id,
            )));
        }
    };

    // ── 4. Persist optimized files to disk ────────────────────────────────────
    if let Err(e) = save_profile_cv_data(profile, tenant_data_dir, &optimized_cv_json, lang).await
    {
        app_log!(
            error,
            "Failed to save optimized CV files for profile {}: {}",
            profile,
            e
        );
        return Err(Json(StandardErrorResponse::new(
            format!("Failed to save optimized CV: {}", e),
            "SAVE_FAILED".to_string(),
            vec!["Check disk space and permissions".to_string()],
            conversation_id,
        )));
    }

    app_log!(
        info,
        "Optimized CV saved to disk — profile: {}, lang: {}",
        profile,
        lang
    );

    let response = OptimizeResponse {
        optimized_typst,
        job_title: optimization_response.job_title,
        company_name: optimization_response.company_name,
        optimizations: optimization_response.optimizations,
        keyword_analysis: optimization_response.keyword_analysis,
        saved: true,
        status: optimization_response.status,
    };

    Ok((response, optimized_cv_json))
}

// ── POST /optimize ─────────────────────────────────────────────────────────────

pub async fn optimize_cv_handler(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<OptimizeResponse>>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let lang = normalize_language(request.data.lang.as_deref());
    let profile = normalize_profile_name(&request.data.profile);
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    let cv_data: CvJson = match &request.data.cv_json {
        Some(json_str) => serde_json::from_str(json_str).map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Invalid CV JSON format: {}", e),
                "INVALID_CV_JSON".to_string(),
                vec!["Ensure CV data is in correct JSON format".to_string()],
                conversation_id.clone(),
            ))
        })?,
        None => load_profile_cv_data(&profile, &tenant_data_dir).await.map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Failed to load CV data for profile '{}': {}", profile, e),
                "PROFILE_LOAD_FAILED".to_string(),
                vec![
                    "Ensure the profile exists and has valid cv_params.toml and experiences_en.typ files".to_string(),
                ],
                conversation_id.clone(),
            ))
        })?,
    };

    // Optimization uses Cohere command-r7b (~$0.0003/call × 8.3 markup) — 1 credit
    check_and_deduct_credits(&auth.user().email, 1, conversation_id.clone()).await?;

    let (response, _) = run_optimization(
        &cv_data,
        &profile,
        &lang,
        &request.data.job_url,
        request.data.job_description.as_deref(),
        &tenant_data_dir,
        cv_service_url.inner(),
        conversation_id.clone(),
    )
    .await?;

    Ok(Json(DataResponse::success(
        format!(
            "CV optimized for \"{}\" at {} and saved to profile",
            response.job_title, response.company_name
        ),
        response,
        conversation_id,
    )))
}

// ── POST /optimize-and-generate ────────────────────────────────────────────────

pub async fn optimize_and_generate_handler(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    _db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<PdfResponse, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let lang = normalize_language(request.data.lang.as_deref());
    let profile = normalize_profile_name(&request.data.profile);
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    let cv_data: CvJson = match &request.data.cv_json {
        Some(json_str) => serde_json::from_str(json_str).map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Invalid CV JSON format: {}", e),
                "INVALID_CV_JSON".to_string(),
                vec!["Ensure CV data is in correct JSON format".to_string()],
                conversation_id.clone(),
            ))
        })?,
        None => load_profile_cv_data(&profile, &tenant_data_dir).await.map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Failed to load CV data for profile '{}': {}", profile, e),
                "PROFILE_LOAD_FAILED".to_string(),
                vec![
                    "Ensure the profile exists and has valid cv_params.toml and experiences_en.typ files".to_string(),
                ],
                conversation_id.clone(),
            ))
        })?,
    };

    // Optimization uses Cohere command-r7b (~$0.0003/call × 8.3 markup) — 1 credit
    check_and_deduct_credits(&auth.user().email, 1, conversation_id.clone()).await?;

    // ── Step 1: Optimize + save ───────────────────────────────────────────────
    let (optimize_resp, _) = run_optimization(
        &cv_data,
        &profile,
        &lang,
        &request.data.job_url,
        request.data.job_description.as_deref(),
        &tenant_data_dir,
        cv_service_url.inner(),
        conversation_id.clone(),
    )
    .await?;

    // ── Step 2: Generate PDF from freshly-saved profile ───────────────────────
    let template_manager = match TemplateEngine::new(config.templates_dir.clone()) {
        Ok(m) => m,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("Template system error: {}", e),
                "TEMPLATE_INIT_ERROR".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )));
        }
    };

    let template_id = normalize_template(request.data.template.as_deref(), &template_manager);

    let profile_dir = tenant_data_dir.join(&profile);
    if !profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile directory not found after save: {}", profile),
            "PROFILE_DIR_MISSING".to_string(),
            vec!["Internal error — contact support".to_string()],
            conversation_id,
        )));
    }

    if let Err(e) = FsOps::ensure_dir_exists(&config.output_dir).await {
        return Err(Json(StandardErrorResponse::new(
            format!("Output directory error: {}", e),
            "OUTPUT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            conversation_id,
        )));
    }

    let cv_config = CvConfig::new(&profile, &lang)
        .with_template(template_id)
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    let generator = match CvGenerator::new(cv_config) {
        Ok(g) => g,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("CV generator init failed: {}", e),
                "CONFIG_ERROR".to_string(),
                vec!["Verify the profile exists".to_string()],
                conversation_id,
            )));
        }
    };

    match generator.generate_pdf_data().await {
        Ok((pdf_data, _filename)) => {
            // Build a descriptive ATS filename
            let safe_company = optimize_resp
                .company_name
                .to_lowercase()
                .replace(' ', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            let ats_filename = format!("{}_ats_{}_{}.pdf", profile, safe_company, lang);

            app_log!(
                info,
                "ATS-optimized PDF generated: {} ({} bytes)",
                ats_filename,
                pdf_data.len()
            );
            Ok(PdfResponse::with_filename(pdf_data, ats_filename))
        }
        Err(e) => Err(Json(StandardErrorResponse::new(
            format!("PDF generation failed: {}", e),
            "GENERATION_ERROR".to_string(),
            vec![
                "Verify all required profile files exist".to_string(),
                "Check the error details above".to_string(),
            ],
            conversation_id,
        ))),
    }
}
