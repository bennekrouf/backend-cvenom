// src/web/handlers/cv_handlers/cover_letter.rs
//! Cover letter generation handler
//!
//!   POST /cover-letter
//!   Body: { profile, lang, job_description }
//!   → Reads CV data, calls LLM via cv-import service, returns cover letter text.
//!   → Costs 20 credits (same as CV generation).

use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::core::ServiceClient;
use crate::types::cv_data::CvConverter;
use crate::web::handlers::payment_handlers::check_and_deduct_credits;
use crate::web::types::{DataResponse, StandardErrorResponse, StandardRequest, WithConversationId};
use crate::web::ServerConfig;
use graflog::app_log;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

// ── Request / Response ────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CoverLetterRequest {
    /// Profile name to read CV data from.
    pub profile: String,
    /// Language for the cover letter ("en" or "fr").
    pub lang: String,
    /// LinkedIn or other job posting description pasted by the user.
    pub job_description: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CoverLetterResult {
    pub cover_letter: String,
    pub lang: String,
    pub profile: String,
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn cover_letter_handler(
    request: Json<StandardRequest<CoverLetterRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<CoverLetterResult>>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();
    let data = &request.data;

    // Cover letter uses LLM — 20 credits (same as CV generation)
    check_and_deduct_credits(&user.email, 20, conversation_id.clone(), "cover_letter").await?;

    app_log!(
        info,
        "User {} (tenant: {}) requesting cover letter for profile '{}' in '{}'",
        user.email,
        tenant.tenant_name,
        data.profile,
        data.lang
    );

    // Validate job description is not empty
    if data.job_description.trim().is_empty() {
        return Err(Json(StandardErrorResponse::new(
            "Job description is required".to_string(),
            "MISSING_JOB_DESCRIPTION".to_string(),
            vec!["Paste the job posting text into the job description field".to_string()],
            conversation_id,
        )));
    }

    // Resolve profile directory
    let tenant_data_dir = get_tenant_folder_path(&user.email, &config.data_dir);
    let profile_dir = tenant_data_dir.join(&data.profile);

    if !profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' not found", data.profile),
            "PROFILE_NOT_FOUND".to_string(),
            vec!["Check the profile name and try again".to_string()],
            conversation_id,
        )));
    }

    // Load CV data — prefer language-specific experiences file, fall back to generic
    let toml_path = profile_dir.join("cv_params.toml");
    let lang_typst = profile_dir.join(format!("experiences_{}.typ", data.lang));
    let typst_path = if lang_typst.exists() {
        lang_typst
    } else {
        profile_dir.join("experiences.typ")
    };

    let cv_data = match CvConverter::from_files(&toml_path, &typst_path) {
        Ok(d) => d,
        Err(e) => {
            app_log!(
                error,
                "Failed to load CV data for cover letter (profile: {}): {}",
                data.profile,
                e
            );
            return Err(Json(StandardErrorResponse::new(
                "Failed to load CV data from profile".to_string(),
                "CV_LOAD_ERROR".to_string(),
                vec!["Ensure the profile has valid CV data".to_string()],
                conversation_id,
            )));
        }
    };

    // Initialise service client
    let service_client = match ServiceClient::new(cv_service_url.inner().clone(), 60) {
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

    // Call the cv-import service
    match service_client
        .generate_cover_letter(&cv_data, &data.job_description, &data.lang)
        .await
    {
        Ok(cover_letter) => {
            app_log!(
                info,
                "Cover letter generated for profile '{}' in '{}' by {} (tenant: {})",
                data.profile,
                data.lang,
                user.email,
                tenant.tenant_name
            );
            Ok(Json(DataResponse::success(
                "Cover letter generated successfully".to_string(),
                CoverLetterResult {
                    cover_letter,
                    lang: data.lang.clone(),
                    profile: data.profile.clone(),
                },
                conversation_id,
            )))
        }
        Err(e) => {
            app_log!(
                error,
                "Cover letter generation failed for profile '{}' by {} (tenant: {}): {}",
                data.profile,
                user.email,
                tenant.tenant_name,
                e
            );
            Err(Json(StandardErrorResponse::new(
                format!("Cover letter generation failed: {}", e),
                "COVER_LETTER_FAILED".to_string(),
                vec![
                    "Check that the job description is readable".to_string(),
                    "Try again in a few moments".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}
