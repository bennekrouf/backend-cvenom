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
        request.data.person_name
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

    // Load person's CV data as CvJson (UPDATED)
    let cv_data = match load_person_cv_data(&request.data.person_name, &tenant_data_dir).await {
        Ok(data) => data,
        Err(e) => {
            app_log!(
                error,
                "Failed to load CV data for {}: {}",
                request.data.person_name,
                e
            );
            return Err(Json(StandardErrorResponse::new(
                format!(
                    "Person '{}' not found or CV data incomplete",
                    request.data.person_name
                ),
                "PERSON_NOT_FOUND".to_string(),
                vec![
                    format!(
                        "Create person '{}' first using the create endpoint",
                        request.data.person_name
                    ),
                    "Check the person name spelling".to_string(),
                    "Use 'Show collaborators' to see available persons".to_string(),
                ],
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
                request.data.person_name,
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
                request.data.person_name,
                user.email,
                tenant.tenant_name,
                error_msg
            );

            let (error_code, suggestions) = categorize_error(&error_msg, &request.data.person_name);
            Err(Json(StandardErrorResponse::new(
                error_msg,
                error_code,
                suggestions,
                conversation_id,
            )))
        }
    }
}

// UPDATED: Load person CV data as CvJson instead of String
async fn load_person_cv_data(
    person_name: &str,
    tenant_data_dir: &std::path::Path,
) -> Result<CvJson> {
    let person_dir = tenant_data_dir.join(person_name);
    let toml_path = person_dir.join("cv_params.toml");
    let typst_path = person_dir.join("experiences_en.typ"); // Default to English

    if !toml_path.exists() {
        return Err(anyhow::anyhow!(
            "Person directory not found: cv_params.toml missing"
        ));
    }

    if !typst_path.exists() {
        return Err(anyhow::anyhow!(
            "Person directory not found: experiences_en.typ missing"
        ));
    }

    // Use CvConverter to load from files
    CvConverter::from_files(&toml_path, &typst_path)
        .map_err(|e| anyhow::anyhow!("Failed to load CV data: {}", e))
}

fn categorize_error(error_msg: &str, person_name: &str) -> (String, Vec<String>) {
    if error_msg.contains("Person directory not found")
        || error_msg.contains("cv_params.toml missing")
        || error_msg.contains("experiences_en.typ missing")
    {
        (
            "PERSON_NOT_FOUND".to_string(),
            vec![
                format!(
                    "Create person '{}' first using the create endpoint",
                    person_name
                ),
                "Check the person name spelling".to_string(),
                "Use 'Show collaborators' to see available persons".to_string(),
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
                "Ensure the person's CV data is complete".to_string(),
                "Contact support if the problem persists".to_string(),
            ],
        )
    }
}

