// src/web/handlers/linkedin_handlers.rs - Updated for standard responses

use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::linkedin_analysis::job_analyzer::JobAnalyzer;
use crate::linkedin_analysis::{JobAnalysisRequest, JobAnalysisResponse};
use crate::web::types::{
    DisplayFormat, DisplaySection, StandardErrorResponse, StandardRequest, WithConversationId,
};
use crate::web::TextResponse;

use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

pub async fn analyze_job_fit_handler(
    request: Json<StandardRequest<JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<TextResponse>, Json<StandardErrorResponse>> {
    // Changed return type
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    info!(
        "User {} (tenant: {}) analyzing job fit for person: {}",
        user.email, tenant.tenant_name, request.data.person_name
    );

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

fn create_job_analysis_display_format(response: &JobAnalysisResponse) -> DisplayFormat {
    let mut sections = Vec::new();

    // Job content section
    if let Some(job_content) = &response.job_content {
        sections.push(DisplaySection {
            title: "Job Position".to_string(),
            content: format!(
                "{} at {} ({})",
                job_content.title, job_content.company, job_content.location
            ),
            score: None,
            points: None,
        });
    }

    // Analysis section with key points extraction
    if let Some(analysis) = &response.fit_analysis {
        let analysis_points = extract_key_points(analysis);

        sections.push(DisplaySection {
            title: "Fit Analysis".to_string(),
            content: if analysis.len() > 200 {
                format!("{}...", &analysis[..200])
            } else {
                analysis.clone()
            },
            score: Some("good".to_string()), // You could implement actual scoring logic
            points: Some(analysis_points),
        });
    }

    DisplayFormat {
        format_type: "analysis".to_string(),
        sections: Some(sections),
    }
}

fn extract_key_points(analysis: &str) -> Vec<String> {
    // Simple extraction logic - look for bullet points or numbered items
    analysis
        .lines()
        .filter(|line| {
            line.trim().starts_with("•")
                || line.trim().starts_with("-")
                || line.trim().starts_with("*")
                || line
                    .trim_start()
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_digit())
        })
        .map(|line| {
            line.trim()
                .trim_start_matches("•")
                .trim_start_matches("-")
                .trim_start_matches("*")
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start_matches(".")
                .trim()
                .to_string()
        })
        .filter(|point| !point.is_empty())
        .take(5) // Limit to 5 key points
        .collect()
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
                "Verify the LinkedIn job URL is accessible".to_string(),
                "The job post may be behind authentication or no longer available".to_string(),
                "Try a different job posting URL".to_string(),
            ],
        )
    } else if error_msg.contains("job matching API") || error_msg.contains("API error") {
        (
            "API_ERROR".to_string(),
            vec![
                "The AI analysis service is temporarily unavailable".to_string(),
                "Try again in a few moments".to_string(),
                "Contact support if the problem persists".to_string(),
            ],
        )
    } else {
        (
            "ANALYSIS_ERROR".to_string(),
            vec![
                "Try again in a few moments".to_string(),
                "Check that the job URL is valid and accessible".to_string(),
                "Contact support if the problem persists".to_string(),
            ],
        )
    }
}
