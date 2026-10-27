// src/web/handlers/cv_handlers/optimize.rs
//! CV optimization handler — optimizes via LLM then saves files + generates PDF.

use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::{profile_store, FsOps, ServiceClient, TemplateEngine};
use crate::core::ai_region;
use crate::web::handlers::payment_handlers::{check_and_deduct_ai_credits, refund_credits};
use crate::types::cv_data::{CvConverter, CvJson};
use crate::types::response::OptimizeResponse;
use crate::utils::{normalize_language, normalize_profile_name, truncate_str};
use crate::web::types::WithConversationId;
use crate::web::types::{
    DataResponse, GeneratePdfResponse, ResponseType, ServerConfig, StandardErrorResponse, StandardRequest,
};
use crate::{CvConfig, CvGenerator};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;
use std::env;

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
    /// optimize-and-generate only: name of the new profile that receives the
    /// tailored CV. Defaults to `<profile>-<company>-<job-title>`. The base
    /// profile is never modified.
    pub save_as: Option<String>,
    /// The posting's job title, when the caller already knows it (e.g. from a
    /// job board). Preferred over the title extracted from the posting; names
    /// the PDF and the tailored profile.
    pub job_title: Option<String>,
    /// The hiring company, when the caller already knows it. Preferred over
    /// the company extracted from the posting.
    pub company: Option<String>,
}

// ── Shared optimization pipeline ──────────────────────────────────────────────

/// Runs the optimization pipeline:
/// 1. Call cv-import service  (scrape job → keyword extraction → ATS rewrite)
/// Returns the enriched `OptimizeResponse` and the resolved `CvJson`.
/// Disk persistence is the caller's responsibility.
async fn run_optimization(
    cv_data: &CvJson,
    lang: &str,
    job_url: &str,
    job_description: Option<&str>,
    known_title: Option<&str>,
    known_company: Option<&str>,
    cv_service_url: &str,
    ai_region: Option<&str>,
    conversation_id: Option<String>,
) -> Result<(OptimizeResponse, CvJson), Json<StandardErrorResponse>> {
    // ── 1. Init service client ────────────────────────────────────────────────
    let service_client = match ServiceClient::new_in_region(cv_service_url.to_string(), 30, ai_region) {
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
    let optimization_response = match service_client.optimize_cv(cv_data, job_url, job_description, lang).await {
        Ok(r) => r,
        Err(e) => {
            let err_str = e.to_string();
            return Err(Json(StandardErrorResponse::new(
                format!("CV optimization failed: {}", err_str),
                "OPTIMIZATION_FAILED".to_string(),
                optimization_suggestions(&err_str, job_description.is_some()),
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

    // Serialise the optimized CvJson so the frontend can pass it to /save-optimized
    let optimized_cv_json_str = serde_json::to_string(&optimized_cv_json).ok();

    // The caller's values come from the posting itself, so they win. Otherwise:
    // older cv-import versions leave these empty when a job_description is
    // passed; the keyword pass has them.
    let ka = optimization_response.keyword_analysis.as_ref();
    let job_title = given(known_title)
        .unwrap_or_else(|| non_empty_or(optimization_response.job_title, ka.map(|k| &k.job_title)));
    let company_name = given(known_company)
        .unwrap_or_else(|| non_empty_or(optimization_response.company_name, ka.map(|k| &k.company)));

    let response = OptimizeResponse {
        optimized_typst,
        optimized_cv_json: optimized_cv_json_str,
        job_title,
        company_name,
        optimizations: optimization_response.optimizations,
        keyword_analysis: optimization_response.keyword_analysis,
        before_score: optimization_response.before_score,
        after_score: optimization_response.after_score,
        warnings: optimization_response.warnings,
        saved: false,
        status: optimization_response.status,
    };

    Ok((response, optimized_cv_json))
}

/// Suggestions matching where the optimization failed, not a generic list.
fn optimization_suggestions(err: &str, has_job_description: bool) -> Vec<String> {
    let mut out = Vec::new();
    if err.contains("returned the CV unchanged") {
        out.push("The AI could not tailor this CV to the posting; no credits were charged".to_string());
        out.push("Try again, or pass a fuller job description".to_string());
    } else if err.contains("job description") || err.contains("Job page") || err.contains("job content") {
        if !has_job_description {
            out.push("The job page could not be read; pass the posting's text as job_description".to_string());
        }
    } else if err.contains("Failed to parse optimized CV") {
        out.push("The AI returned a malformed CV; no credits were charged — try again".to_string());
    }
    if out.is_empty() {
        out.push("Try again in a few moments; no credits were charged".to_string());
    }
    out
}

fn given(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|v| !v.is_empty()).map(str::to_string)
}

/// Base of the tailored PDF's name: `{company}_{job-title}`, or the profile
/// with whichever of the two is known, or `{profile}_ats`.
fn ats_base_name(profile: &str, company: &str, job_title: &str) -> String {
    let slug = |s: &str| -> String {
        s.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    };
    let base = match (slug(company), slug(job_title)) {
        (c, t) if !c.is_empty() && !t.is_empty() => format!("{}_{}", c, t),
        (c, _) if !c.is_empty() => format!("{}_{}", profile, c),
        (_, t) if !t.is_empty() => format!("{}_{}", profile, t),
        _ => format!("{}_ats", profile),
    };
    // Cap at a reasonable filename length
    if base.len() > 60 { truncate_str(&base, 60).trim_end_matches('-').to_string() } else { base }
}

fn non_empty_or(value: String, fallback: Option<&String>) -> String {
    if value.trim().is_empty() {
        fallback.map(|f| f.trim().to_string()).unwrap_or_default()
    } else {
        value
    }
}

fn optimized_message(job_title: &str, company: &str) -> String {
    match (job_title.trim(), company.trim()) {
        ("", "") => "CV optimized for the job posting".to_string(),
        (t, "") => format!("CV optimized for \"{}\"", t),
        ("", c) => format!("CV optimized for a role at {}", c),
        (t, c) => format!("CV optimized for \"{}\" at {}", t, c),
    }
}

// ── POST /optimize ─────────────────────────────────────────────────────────────

pub async fn optimize_cv_handler(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
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
        None => load_profile_cv_data(&profile, &tenant_data_dir, &lang).await.map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Failed to load CV data for profile '{}': {}", profile, e),
                "PROFILE_LOAD_FAILED".to_string(),
                vec![
                    "Ensure the profile exists and has valid cv_params.toml and experiences_<lang>.typ files".to_string(),
                ],
                conversation_id.clone(),
            ))
        })?,
    };

    // Optimization — 5 credits by default (¼ of a CV generation), scaled in an AI region
    let cost = check_and_deduct_ai_credits(&auth, ai_region::OPTIMIZE, cv_service_url.inner(), conversation_id.clone(), "optimize").await?;

    let (response, _) = match run_optimization(
        &cv_data,
        &lang,
        &request.data.job_url,
        request.data.job_description.as_deref(),
        request.data.job_title.as_deref(),
        request.data.company.as_deref(),
        cv_service_url.inner(),
        auth.ai_region(),
        conversation_id.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            refund_credits(&auth.user().email, cost, "optimize").await;
            return Err(e);
        }
    };

    crate::email::send_email_with_prefs(
        &auth.user().email,
        crate::email::EmailKind::AtsResults {
            profile: profile.clone(),
            job_title: response.job_title.clone(),
            company: response.company_name.clone(),
            before_score: response.before_score.map(|s| s as u8),
            after_score: response.after_score.map(|s| s as u8),
        },
        &lang,
        auth.email_prefs(),
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

    Ok(Json(DataResponse::success(
        optimized_message(&response.job_title, &response.company_name),
        response,
        conversation_id,
    )))
}

// ── POST /optimize-and-generate ────────────────────────────────────────────────

pub async fn optimize_and_generate_handler(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<GeneratePdfResponse>, Json<StandardErrorResponse>> {
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
        None => load_profile_cv_data(&profile, &tenant_data_dir, &lang).await.map_err(|e| {
            Json(StandardErrorResponse::new(
                format!("Failed to load CV data for profile '{}': {}", profile, e),
                "PROFILE_LOAD_FAILED".to_string(),
                vec![
                    "Ensure the profile exists and has valid cv_params.toml and experiences_<lang>.typ files".to_string(),
                ],
                conversation_id.clone(),
            ))
        })?,
    };

    // Optimization — 5 credits by default (¼ of a CV generation), scaled in an AI region
    let cost = check_and_deduct_ai_credits(&auth, ai_region::OPTIMIZE, cv_service_url.inner(), conversation_id.clone(), "optimize").await?;

    // ── Step 1: Optimize ─────────────────────────────────────────────────────
    let (optimize_resp, optimized_cv_data) = match run_optimization(
        &cv_data,
        &lang,
        &request.data.job_url,
        request.data.job_description.as_deref(),
        request.data.job_title.as_deref(),
        request.data.company.as_deref(),
        cv_service_url.inner(),
        auth.ai_region(),
        conversation_id.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            refund_credits(&auth.user().email, cost, "optimize").await;
            return Err(e);
        }
    };

    // ── Step 1b: Save the tailored CV as a NEW profile ───────────────────────
    // The base profile stays untouched: copy it (pictures, logos, other
    // languages, styling), then write the tailored files into the copy.
    let base_profile = profile;
    let profile = match request.data.save_as.as_deref().map(normalize_profile_name) {
        Some(name) if !name.trim().is_empty() => {
            if tenant_data_dir.join(&name).exists() {
                return Err(Json(StandardErrorResponse::new(
                    format!("Profile '{}' already exists", name),
                    "PROFILE_EXISTS".to_string(),
                    vec!["Choose another save_as name, or omit it to get one derived from the job".to_string()],
                    conversation_id,
                )));
            }
            name
        }
        _ => profile_store::tailored_profile_name(
            &tenant_data_dir,
            &base_profile,
            &optimize_resp.company_name,
            &optimize_resp.job_title,
        ),
    };

    if let Err(e) = profile_store::duplicate(&tenant_data_dir, &base_profile, &profile).await {
        return Err(Json(StandardErrorResponse::new(
            format!("Failed to create tailored profile '{}': {}", profile, e),
            "SAVE_FAILED".to_string(),
            vec![],
            conversation_id,
        )));
    }
    // From here on, any failure removes the half-made profile.
    let discard = |reason: StandardErrorResponse| {
        let dir = tenant_data_dir.join(&profile);
        async move {
            if let Err(e) = FsOps::remove_dir_all(&dir).await {
                app_log!(warn, "Could not remove failed tailored profile {}: {}", dir.display(), e);
            }
            Json(reason)
        }
    };

    if let Err(e) = save_profile_cv_data(&profile, &tenant_data_dir, &optimized_cv_data, &lang).await {
        app_log!(error, "Failed to save optimized CV for profile {}: {}", profile, e);
        return Err(discard(StandardErrorResponse::new(
            format!("Failed to save optimized CV: {}", e),
            "SAVE_FAILED".to_string(),
            vec![],
            conversation_id,
        ))
        .await);
    }
    app_log!(info, "Tailored CV saved as new profile {} (base {}, lang {})", profile, base_profile, lang);

    // Remember which job post this profile was tailored for.
    let job_source = profile_store::JobSource::new(
        &request.data.job_url,
        &optimize_resp.company_name,
        &optimize_resp.job_title,
    );
    if let Err(e) = profile_store::write_job_source(&tenant_data_dir, &profile, &job_source).await {
        app_log!(warn, "Could not record job source for profile {}: {}", profile, e);
    }

    // ── Step 2: Generate PDF from freshly-saved profile ───────────────────────
    let template_manager = match TemplateEngine::new(config.templates_dir.clone()) {
        Ok(m) => m,
        Err(e) => {
            return Err(discard(StandardErrorResponse::new(
                format!("Template system error: {}", e),
                "TEMPLATE_INIT_ERROR".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )).await);
        }
    };

    let template_id = normalize_template(request.data.template.as_deref(), &template_manager);

    let profile_dir = tenant_data_dir.join(&profile);
    if !profile_dir.exists() {
        return Err(discard(StandardErrorResponse::new(
            format!("Profile directory not found after save: {}", profile),
            "PROFILE_DIR_MISSING".to_string(),
            vec!["Internal error — contact support".to_string()],
            conversation_id,
        )).await);
    }

    if let Err(e) = FsOps::ensure_dir_exists(&config.output_dir).await {
        return Err(discard(StandardErrorResponse::new(
            format!("Output directory error: {}", e),
            "OUTPUT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            conversation_id,
        )).await);
    }

    let cv_config = CvConfig::new(&profile, &lang)
        .with_template(template_id)
        .with_data_dir(tenant_data_dir.clone())
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    let generator = match CvGenerator::new(cv_config) {
        Ok(g) => g,
        Err(e) => {
            return Err(discard(StandardErrorResponse::new(
                format!("CV generator init failed: {}", e),
                "CONFIG_ERROR".to_string(),
                vec!["Verify the profile exists".to_string()],
                conversation_id,
            )).await);
        }
    };

    match generator.generate().await {
        Ok(output_path) => {
            // Descriptive ATS filename: "{company}_{job-title}_{lang}.pdf"
            let base = ats_base_name(&profile, &optimize_resp.company_name, &optimize_resp.job_title);
            let ats_filename = format!("{}_{}.pdf", base, lang);

            // Rename the output file to the ATS filename in the output directory
            let final_path = config.output_dir.join(&ats_filename);
            if let Err(e) = std::fs::rename(&output_path, &final_path) {
                app_log!(warn, "Failed to rename optimized PDF to {}: {}", ats_filename, e);
            }

            app_log!(
                info,
                "ATS-optimized PDF generated and saved as: {}",
                ats_filename
            );

            let base_url = env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "https://api.cvenom.com".to_string());
            let pdf_url = match crate::web::downloads::publish(&config.output_dir, &ats_filename, &base_url) {
                Ok(url) => url,
                Err(e) => {
                    app_log!(error, "Could not publish a download link for {}: {}", ats_filename, e);
                    return Err(discard(StandardErrorResponse::new(
                        "The CV was generated but its download link could not be created".to_string(),
                        "DOWNLOAD_LINK_FAILED".to_string(),
                        vec![],
                        None,
                    )).await);
                }
            };

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
                message: format!(
                    "Tailored CV saved as new profile '{}' (base '{}' unchanged) and generated",
                    profile, base_profile
                ),
                download_url: pdf_url,
                filename: ats_filename,
                profile,
                job_url: job_source.url,
                conversation_id,
            }))
        }
        Err(e) => Err(discard(StandardErrorResponse::new(
            format!("PDF generation failed: {}", e),
            "GENERATION_ERROR".to_string(),
            vec![
                "Verify all required profile files exist".to_string(),
                "Check the error details above".to_string(),
            ],
            conversation_id,
        ))
        .await),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_never_has_empty_quotes_or_dangling_at() {
        // Regression: was `CV optimized for "" at ` with a pasted job description.
        assert_eq!(optimized_message("", ""), "CV optimized for the job posting");
        assert_eq!(optimized_message("Rust Engineer", ""), "CV optimized for \"Rust Engineer\"");
        assert_eq!(optimized_message("", "Proton"), "CV optimized for a role at Proton");
        assert_eq!(optimized_message("Rust Engineer", "Proton"), "CV optimized for \"Rust Engineer\" at Proton");
    }

    #[test]
    fn filename_keeps_whatever_is_known() {
        assert_eq!(ats_base_name("jd", "Proton AG", "Rust Engineer"), "proton-ag_rust-engineer");
        assert_eq!(ats_base_name("jd", "Proton", ""), "jd_proton");
        // Regression: a title without a company gave "jd_ats".
        assert_eq!(ats_base_name("jd", "", "Rust Engineer"), "jd_rust-engineer");
        assert_eq!(ats_base_name("jd", " ", ""), "jd_ats");
    }

    #[test]
    fn caller_values_are_used_only_when_non_blank() {
        assert_eq!(given(Some(" Rust Engineer ")), Some("Rust Engineer".into()));
        assert_eq!(given(Some("  ")), None);
        assert_eq!(given(None), None);
    }

    #[test]
    fn empty_title_falls_back_to_keyword_analysis() {
        let extracted = "Senior Rust Engineer".to_string();
        assert_eq!(non_empty_or(String::new(), Some(&extracted)), "Senior Rust Engineer");
        assert_eq!(non_empty_or("Scraped".into(), Some(&extracted)), "Scraped");
        assert_eq!(non_empty_or(" ".into(), None), "");
    }
}
