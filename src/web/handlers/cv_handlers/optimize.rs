// src/web/handlers/cv_handlers/optimize.rs
//! CV optimization handler
use crate::auth::AuthenticatedUser;
use crate::core::ServiceClient;
use crate::types::cv_data::{CvConverter, CvJson};
use crate::types::response::OptimizeResponse;
use crate::web::types::WithConversationId;
use crate::web::types::{DataResponse, OptimizeCvRequest, StandardErrorResponse, StandardRequest};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::State;

pub async fn optimize_cv_handler(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    _auth: AuthenticatedUser,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<OptimizeResponse>>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();

    // Parse cv_json as CvJson instead of raw string
    let cv_data: CvJson = serde_json::from_str(&request.data.cv_json).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Invalid CV JSON format: {}", e),
            "INVALID_CV_JSON".to_string(),
            vec!["Ensure CV data is in correct JSON format".to_string()],
            conversation_id.clone(),
        ))
    })?;

    let service_client = match ServiceClient::new(cv_service_url.inner().clone(), 30) {
        Ok(client) => client,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("Service initialization failed: {}", e),
                "SERVICE_INIT_FAILED".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )))
        }
    };

    // Call cv-import service for optimization
    match service_client
        .optimize_cv(&cv_data, &request.data.job_url)
        .await
    {
        Ok(optimization_response) => {
            // Convert optimized CvJson to Typst
            let optimized_typst =
                match CvConverter::to_typst(&optimization_response.optimized_cv, "en") {
                    Ok(typst) => typst,
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

            let response = OptimizeResponse {
                optimized_typst,
                job_title: optimization_response.job_title,
                company_name: optimization_response.company_name,
                status: optimization_response.status,
            };

            Ok(Json(DataResponse::success(
                "CV optimization completed successfully".to_string(),
                response,
                conversation_id,
            )))
        }
        Err(e) => Err(Json(StandardErrorResponse::new(
            format!("CV optimization failed: {}", e),
            "OPTIMIZATION_FAILED".to_string(),
            vec!["Check job URL validity".to_string()],
            conversation_id,
        ))),
    }
}
