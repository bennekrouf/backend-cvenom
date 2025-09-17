// src/web/handlers/linkedin_handlers.rs
use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::linkedin_analysis::{JobAnalysisRequest, JobAnalysisResponse, JobAnalyzer};
use crate::web::types::ErrorResponse;

use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

pub async fn analyze_job_fit_handler(
    request: Json<JobAnalysisRequest>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<JobAnalysisResponse>, Json<ErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();

    info!(
        "User {} (tenant: {}) analyzing job fit for person: {}",
        user.email, tenant.tenant_name, request.person_name
    );

    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Json(ErrorResponse {
                success: false,
                error: "Database connection failed".to_string(),
                error_code: "DATABASE_ERROR".to_string(),
                suggestions: vec!["Try again in a few moments".to_string()],
            }));
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
            return Err(Json(ErrorResponse {
                success: false,
                error: "Failed to access tenant data directory".to_string(),
                error_code: "TENANT_DIR_ERROR".to_string(),
                suggestions: vec!["Contact system administrator".to_string()],
            }));
        }
    };

    // Initialize job analyzer
    let analyzer = match JobAnalyzer::new() {
        Ok(analyzer) => analyzer,
        Err(e) => {
            error!("Failed to initialize job analyzer: {}", e);
            return Err(Json(ErrorResponse {
                success: false,
                error: "Service configuration error".to_string(),
                error_code: "SERVICE_CONFIG_ERROR".to_string(),
                suggestions: vec![
                    "Ensure SEMANTIC_API_KEY environment variable is set".to_string(),
                    "Contact system administrator".to_string(),
                ],
            }));
        }
    };

    // Perform analysis
    let analysis_response = analyzer
        .analyze_job_fit(request.clone().into_inner(), &tenant_data_dir)
        .await;

    if analysis_response.success {
        info!(
            "Successfully analyzed job fit for {} by {} (tenant: {})",
            request.person_name, user.email, tenant.tenant_name
        );
        Ok(Json(analysis_response))
    } else {
        let error_msg = analysis_response
            .error
            .unwrap_or_else(|| "Unknown analysis error".to_string());

        error!(
            "Job analysis failed for {} by {} (tenant: {}): {}",
            request.person_name, user.email, tenant.tenant_name, error_msg
        );

        let (error_code, suggestions) = if error_msg.contains("Person directory not found") {
            (
                "PERSON_NOT_FOUND",
                vec![
                    format!(
                        "Create person '{}' first using the create endpoint",
                        request.person_name
                    ),
                    "Check the person name spelling".to_string(),
                ],
            )
        } else if error_msg.contains("Failed to scrape")
            || error_msg.contains("extract job content")
        {
            (
                "SCRAPING_ERROR",
                vec![
                    "Verify the LinkedIn job URL is accessible".to_string(),
                    "The job post may be behind authentication or no longer available".to_string(),
                    "Try a different job posting URL".to_string(),
                ],
            )
        } else if error_msg.contains("Semantic API") {
            (
                "API_ERROR",
                vec![
                    "The AI analysis service is temporarily unavailable".to_string(),
                    "Try again in a few moments".to_string(),
                ],
            )
        } else {
            (
                "ANALYSIS_ERROR",
                vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
            )
        };

        Err(Json(ErrorResponse {
            success: false,
            error: error_msg,
            error_code: error_code.to_string(),
            suggestions,
        }))
    }
}
