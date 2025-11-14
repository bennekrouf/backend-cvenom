// src/web/handlers/cv_handlers/translate.rs
//! CV translation handler

use crate::auth::AuthenticatedUser;
use crate::core::ServiceClient;
use crate::types::cv_data::{CvConverter, CvJson};
use crate::types::response::TranslateResponse;
use crate::web::types::{DataResponse, StandardErrorResponse};
use graflog::app_log;
use rocket::data::ToByteUnit;
use rocket::serde::json::Json;
use rocket::{Data, State};

pub async fn translate_cv_handler(
    data: Data<'_>,
    target_lang: Option<&str>,
    _auth: AuthenticatedUser,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<TranslateResponse>>, Json<StandardErrorResponse>> {
    let target_lang = target_lang.unwrap_or("en");

    // Parse the uploaded data as CvJson
    let file_content = data.open(10_i32.bytes()).into_bytes().await.map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Failed to read data: {}", e),
            "DATA_READ_ERROR".to_string(),
            vec!["Check request format".to_string()],
            None,
        ))
    })?;

    // Try to parse as CvJson
    let cv_data: CvJson = serde_json::from_slice(&file_content).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Invalid CV data format: {}", e),
            "INVALID_CV_DATA".to_string(),
            vec!["Ensure CV data is in correct JSON format".to_string()],
            None,
        ))
    })?;

    let service_client = match ServiceClient::new(cv_service_url.inner().clone(), 30) {
        Ok(client) => client,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                format!("Service initialization failed: {}", e),
                "SERVICE_INIT_FAILED".to_string(),
                vec!["Contact system administrator".to_string()],
                None,
            )))
        }
    };

    // Call cv-import service for translation
    match service_client.translate_cv(&cv_data, target_lang).await {
        Ok(translated_cv) => {
            // Convert translated CvJson back to Typst content
            let translated_typst = match CvConverter::to_typst(&translated_cv, target_lang) {
                Ok(typst) => typst,
                Err(e) => {
                    app_log!(error, "Failed to convert translated CV to Typst: {}", e);
                    return Err(Json(StandardErrorResponse::new(
                        "Translation conversion failed".to_string(),
                        "CONVERSION_ERROR".to_string(),
                        vec!["Try again later".to_string()],
                        None,
                    )));
                }
            };

            let translate_response = TranslateResponse {
                translated_content: translated_typst,
                status: "success".to_string(),
            };

            Ok(Json(DataResponse::success(
                format!("Translation to {} completed successfully", target_lang),
                translate_response,
                None,
            )))
        }
        Err(e) => Err(Json(StandardErrorResponse::new(
            format!("Translation failed: {}", e),
            "TRANSLATION_FAILED".to_string(),
            vec!["Check CV data format".to_string()],
            None,
        ))),
    }
}
