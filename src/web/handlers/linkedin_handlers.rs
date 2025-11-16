// src/web/handlers/linkedin_handlers.rs - Fixed to use CvJson

use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::{FsOps, ServiceClient};
use crate::linkedin_analysis::JobAnalysisRequest;
use crate::types::cv_data::{CvConverter, CvJson}; // Add CvJson imports
use crate::web::types::{StandardErrorResponse, StandardRequest, TextResponse, WithConversationId};
use crate::web::ServerConfig;
use anyhow::Result;
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::{post, State};

#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit_handler(
    request: Json<StandardRequest<JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<TextResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    app_log!(
        info,
        "User {} (tenant: {}) requesting job fit analysis for {}",
        user.email,
        tenant.tenant_name,
        request.data.profile_name
    );

    // Use new tenant folder path
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    // Ensure directory exists
    if let Err(e) = FsOps::ensure_dir_exists(&tenant_data_dir).await {
        app_log!(error, "Failed to create tenant directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to access tenant data directory".to_string(),
            "TENANT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            conversation_id,
        )));
    }

    // Initialize service client
    let service_client = match ServiceClient::new(cv_service_url.inner().clone(), 400) {
        Ok(client) => client,
        Err(e) => {
            app_log!(error, "Failed to initialize service client: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Service configuration error".to_string(),
                "SERVICE_CONFIG_ERROR".to_string(),
                vec![
                    "Ensure cv_import service is available".to_string(),
                    "Contact system administrator".to_string(),
                ],
                conversation_id,
            )));
        }
    };

    // Load profile's CV data as CvJson (UPDATED)
    // In the load_profile_cv_data function error handling (around line 9565)
    let cv_data = match load_profile_cv_data(&request.data.profile_name, &tenant_data_dir).await {
        Ok(data) => data,
        Err(e) => {
            app_log!(
                error,
                "Failed to load CV data for {}: {}",
                request.data.profile_name,
                e
            );

            let error_message = e.to_string();
            let (error_code, suggestions) =
                categorize_cv_error(&error_message, &request.data.profile_name);

            return Err(Json(StandardErrorResponse::new(
                format!(
                    "Profile '{}' has invalid CV data: {}",
                    request.data.profile_name, error_message
                ),
                error_code,
                suggestions,
                conversation_id,
            )));
        }
    };

    // Call cv_import service for job matching (UPDATED to use CvJson)
    match service_client
        .match_job(&cv_data, &request.data.job_url)
        .await
    {
        Ok(match_response) => {
            app_log!(
                info,
                "Successfully analyzed job fit for {} by {} (tenant: {})",
                request.data.profile_name,
                user.email,
                tenant.tenant_name
            );
            // Use the analysis field from JobMatchResponse
            Ok(Json(TextResponse::success(
                match_response.analysis,
                conversation_id,
            )))
        }
        Err(e) => {
            let error_msg = format!("Job analysis failed: {}", e);
            app_log!(
                error,
                "Job analysis failed for {} by {} (tenant: {}): {}",
                request.data.profile_name,
                user.email,
                tenant.tenant_name,
                error_msg
            );

            let (error_code, suggestions) =
                categorize_error(&error_msg, &request.data.profile_name);
            Err(Json(StandardErrorResponse::new(
                error_msg,
                error_code,
                suggestions,
                conversation_id,
            )))
        }
    }
}

// UPDATED: Load profile CV data as CvJson instead of String
async fn load_profile_cv_data(
    profile_name: &str,
    tenant_data_dir: &std::path::Path,
) -> Result<CvJson> {
    let profile_dir = tenant_data_dir.join(profile_name);
    let toml_path = profile_dir.join("cv_params.toml");
    let typst_path = profile_dir.join("experiences_en.typ"); // Default to English
    app_log!(info, "Looking for profile at: {}", profile_dir.display());
    app_log!(info, "TOML exists: {}", toml_path.exists());
    app_log!(info, "Typst exists: {}", typst_path.exists());

    if !toml_path.exists() {
        return Err(anyhow::anyhow!(
            "Profile directory not found: cv_params.toml missing"
        ));
    }

    if !typst_path.exists() {
        return Err(anyhow::anyhow!(
            "Profile directory not found: experiences_en.typ missing"
        ));
    }

    // Use CvConverter to load from files
    CvConverter::from_files(&toml_path, &typst_path)
        .map_err(|e| anyhow::anyhow!("Failed to load CV data: {}", e))
}

fn categorize_cv_error(error_msg: &str, profile_name: &str) -> (String, Vec<String>) {
    if error_msg.contains("Missing") && error_msg.contains("section") {
        let missing_section = extract_missing_section(error_msg);
        (
            "MISSING_CV_SECTION".to_string(),
            vec![
                format!("Add [{}] section to cv_params.toml", missing_section),
                "Check cv_params.toml structure matches expected format".to_string(),
                "Re-upload your CV or recreate the profile".to_string(),
            ],
        )
    } else if error_msg.contains("Missing") && error_msg.contains("field") {
        let missing_field = extract_missing_field(error_msg);
        (
            "MISSING_CV_FIELD".to_string(),
            vec![
                format!("Add '{}' field to cv_params.toml", missing_field),
                "Check required fields are present".to_string(),
                "Edit cv_params.toml manually or re-upload CV".to_string(),
            ],
        )
    } else if error_msg.contains("Invalid") || error_msg.contains("malformed") {
        (
            "INVALID_CV_FORMAT".to_string(),
            vec![
                "Check cv_params.toml syntax is valid".to_string(),
                "Verify TOML structure is correct".to_string(),
                "Re-upload your CV to regenerate files".to_string(),
            ],
        )
    } else if error_msg.contains("cv_params.toml missing")
        || error_msg.contains("experiences_en.typ missing")
    {
        (
            "PROFILE_INCOMPLETE".to_string(),
            vec![
                format!(
                    "Create profile '{}' first using the create endpoint",
                    profile_name
                ),
                "Check the profile name spelling".to_string(),
                "Use 'Show profiles' to see available profiles".to_string(),
            ],
        )
    } else {
        (
            "CV_DATA_ERROR".to_string(),
            vec![
                "Check CV data structure and content".to_string(),
                "Try recreating the profile".to_string(),
                "Contact support if the problem persists".to_string(),
            ],
        )
    }
}

fn categorize_error(error_msg: &str, profile_name: &str) -> (String, Vec<String>) {
    // Handle CV data structure errors first
    if error_msg.contains("Missing") && error_msg.contains("section") {
        let missing_section = extract_missing_section(error_msg);
        (
            "MISSING_CV_SECTION".to_string(),
            vec![
                format!("Add [{}] section to cv_params.toml", missing_section),
                "Check cv_params.toml structure matches expected format".to_string(),
                "Re-upload your CV or recreate the profile".to_string(),
            ],
        )
    } else if error_msg.contains("Missing") && error_msg.contains("field") {
        let missing_field = extract_missing_field(error_msg);
        (
            "MISSING_CV_FIELD".to_string(),
            vec![
                format!("Add '{}' field to cv_params.toml", missing_field),
                "Check required fields are present".to_string(),
                "Edit cv_params.toml manually or re-upload CV".to_string(),
            ],
        )
    } else if error_msg.contains("Invalid") || error_msg.contains("malformed") {
        (
            "INVALID_CV_FORMAT".to_string(),
            vec![
                "Check cv_params.toml syntax is valid".to_string(),
                "Verify TOML structure is correct".to_string(),
                "Re-upload your CV to regenerate files".to_string(),
            ],
        )
    } else if error_msg.contains("Profile directory not found")
        || error_msg.contains("cv_params.toml missing")
        || error_msg.contains("experiences_en.typ missing")
    {
        (
            "PROFILE_NOT_FOUND".to_string(),
            vec![
                format!(
                    "Create profile '{}' first using the create endpoint",
                    profile_name
                ),
                "Check the profile name spelling".to_string(),
                "Use 'Show profiles' to see available profiles".to_string(),
            ],
        )
    } else if error_msg.contains("Failed to scrape") || error_msg.contains("extract job content") {
        (
            "SCRAPING_ERROR".to_string(),
            vec![
                "Ensure the job URL is accessible and public".to_string(),
                "Try a different LinkedIn job URL".to_string(),
                "Check if the job posting is still active".to_string(),
            ],
        )
    } else if error_msg.contains("timeout") || error_msg.contains("network") {
        (
            "NETWORK_ERROR".to_string(),
            vec![
                "Check your internet connection".to_string(),
                "Try again in a few moments".to_string(),
                "Verify the LinkedIn URL is correct".to_string(),
            ],
        )
    } else {
        (
            "ANALYSIS_ERROR".to_string(),
            vec![
                "Try again with a different job URL".to_string(),
                "Ensure the profile's CV data is complete".to_string(),
                "Contact support if the problem persists".to_string(),
            ],
        )
    }
}

fn extract_missing_section(error_msg: &str) -> String {
    // Extract section name from "Missing personal section" or similar
    if let Some(start) = error_msg.find("Missing ") {
        if let Some(end) = error_msg[start + 8..].find(" section") {
            return error_msg[start + 8..start + 8 + end].to_string();
        }
    }
    "required".to_string()
}

fn extract_missing_field(error_msg: &str) -> String {
    // Extract field name from "Missing field 'name'" or similar
    if let Some(start) = error_msg.find("field '") {
        if let Some(end) = error_msg[start + 7..].find("'") {
            return error_msg[start + 7..start + 7 + end].to_string();
        }
    } else if let Some(start) = error_msg.find("Missing ") {
        if let Some(end) = error_msg[start + 8..].find(" ") {
            return error_msg[start + 8..start + 8 + end].to_string();
        }
    }
    "required".to_string()
}
