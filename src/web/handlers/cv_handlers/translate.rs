// src/web/handlers/cv_handlers/translate.rs
//! CV translation handler
use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::core::ServiceClient;
use crate::types::cv_data::CvConverter;
use crate::types::response::TranslateResponse;
use crate::web::types::{DataResponse, StandardErrorResponse, StandardRequest, WithConversationId};
use crate::web::ServerConfig;
use graflog::app_log;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;

#[derive(Deserialize)]
pub struct TranslateCvRequest {
    pub profile_name: String,
    pub target_lang: String,
}

pub async fn translate_cv_handler(
    request: Json<StandardRequest<TranslateCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<TranslateResponse>>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    app_log!(
        info,
        "User {} (tenant: {}) requesting CV translation for profile: {} to language: {}",
        user.email,
        tenant.tenant_name,
        request.data.profile_name,
        request.data.target_lang
    );

    // Load CV data from profile
    let tenant_data_dir = get_tenant_folder_path(&user.email, &config.data_dir);
    let profile_dir = tenant_data_dir.join(&request.data.profile_name);
    let toml_path = profile_dir.join("cv_params.toml");
    let typst_path = profile_dir.join("experiences.typ");

    // Verify profile exists
    if !profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' not found", request.data.profile_name),
            "PROFILE_NOT_FOUND".to_string(),
            vec!["Check the profile name and try again".to_string()],
            conversation_id,
        )));
    }

    // Load CV data from profile files
    let cv_data = match CvConverter::from_files(&toml_path, &typst_path) {
        Ok(data) => data,
        Err(e) => {
            app_log!(
                error,
                "Failed to load CV data from profile {}: {}",
                request.data.profile_name,
                e
            );
            return Err(Json(StandardErrorResponse::new(
                "Failed to load CV data from profile".to_string(),
                "CV_LOAD_ERROR".to_string(),
                vec![
                    "Ensure the profile has valid CV data".to_string(),
                    "Try regenerating the profile".to_string(),
                ],
                conversation_id,
            )));
        }
    };

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

    // Call cv-import service for translation
    match service_client
        .translate_cv(&cv_data, &request.data.target_lang)
        .await
    {
        Ok(translated_cv) => {
            // Convert translated CvJson back to Typst content
            let translated_typst =
                match CvConverter::to_typst(&translated_cv, &request.data.target_lang) {
                    Ok(typst) => typst,
                    Err(e) => {
                        app_log!(error, "Failed to convert translated CV to Typst: {}", e);
                        return Err(Json(StandardErrorResponse::new(
                            "Translation conversion failed".to_string(),
                            "CONVERSION_ERROR".to_string(),
                            vec!["Try again later".to_string()],
                            conversation_id,
                        )));
                    }
                };

            app_log!(
                info,
                "Successfully translated CV for profile {} to {} by {} (tenant: {})",
                request.data.profile_name,
                request.data.target_lang,
                user.email,
                tenant.tenant_name
            );

            let translate_response = TranslateResponse {
                translated_content: translated_typst,
                status: "success".to_string(),
            };

            Ok(Json(DataResponse::success(
                format!(
                    "Translation to {} completed successfully",
                    request.data.target_lang
                ),
                translate_response,
                conversation_id,
            )))
        }
        Err(e) => {
            app_log!(
                error,
                "Translation failed for profile {} by {} (tenant: {}): {}",
                request.data.profile_name,
                user.email,
                tenant.tenant_name,
                e
            );
            Err(Json(StandardErrorResponse::new(
                format!("Translation failed: {}", e),
                "TRANSLATION_FAILED".to_string(),
                vec![
                    "Check CV data format".to_string(),
                    "Try again in a few moments".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}
