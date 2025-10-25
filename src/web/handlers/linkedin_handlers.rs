// src/web/handlers/linkedin_handlers.rs
use crate::auth::AuthenticatedUser;
use crate::database::TenantService;
use crate::linkedin_analysis::job_analyzer::JobAnalyzer;
use crate::linkedin_analysis::JobAnalysisRequest;
use crate::web::types::{StandardErrorResponse, StandardRequest, TextResponse, WithConversationId};
use crate::web::{DatabaseConfig, ServerConfig};
use anyhow::Result;
use rocket::serde::json::Json;
use rocket::{post, State};
use tracing::{error, info};

#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit_handler(
    request: Json<StandardRequest<JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<TextResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    info!(
        "User {} (tenant: {}) requesting job fit analysis for {}",
        user.email, tenant.tenant_name, request.data.person_name
    );

    // Database setup
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Database connection failed".to_string(),
                "DATABASE_ERROR".to_string(),
                vec!["Try again in a few moments".to_string()],
                conversation_id,
            )));
        }
    };

    let tenant_service = TenantService::new(pool);
    let tenant_data_dir = match tenant_service
        .ensure_tenant_data_dir(&config.data_dir, tenant)
        .await
    {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to ensure tenant data directory: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Failed to access tenant data directory".to_string(),
                "TENANT_DIR_ERROR".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )));
        }
    };

    // Initialize job analyzer
    let analyzer = match JobAnalyzer::new() {
        Ok(analyzer) => analyzer,
        Err(e) => {
            error!("Failed to initialize job analyzer: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Service configuration error".to_string(),
                "SERVICE_CONFIG_ERROR".to_string(),
                vec![
                    "Ensure job matching API is available".to_string(),
                    "Contact system administrator".to_string(),
                ],
                conversation_id,
            )));
        }
    };

    // Perform analysis
    let analysis_response = analyzer
        .analyze_job_fit(request.data.clone(), &tenant_data_dir)
        .await;

    if analysis_response.success {
        info!(
            "Successfully analyzed job fit for {} by {} (tenant: {})",
            request.data.person_name, user.email, tenant.tenant_name
        );

        // Return simple text response for chat frontend
        let analysis_text = analysis_response.fit_analysis.unwrap_or_else(|| {
            "Job analysis completed but no detailed analysis was returned.".to_string()
        });

        Ok(Json(TextResponse::success(analysis_text, conversation_id)))
    } else {
        let error_msg = analysis_response
            .error
            .unwrap_or_else(|| "Unknown analysis error".to_string());

        error!(
            "Job analysis failed for {} by {} (tenant: {}): {}",
            request.data.person_name, user.email, tenant.tenant_name, error_msg
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

fn categorize_error(error_msg: &str, person_name: &str) -> (String, Vec<String>) {
    if error_msg.contains("Person directory not found") {
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
