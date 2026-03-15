// src/web/handlers/cv_handlers/save_optimized.rs
//! POST /save-optimized — persists an optimized CvJson under a new profile name.

use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::types::cv_data::CvJson;
use crate::utils::{normalize_language, normalize_profile_name};
use crate::web::handlers::cv_handlers::helpers::save_profile_cv_data;
use crate::web::types::WithConversationId;
use crate::web::types::{ActionResponse, ServerConfig, StandardErrorResponse, StandardRequest};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SaveOptimizedRequest {
    /// Desired name for the new profile (will be normalized: lowercase, spaces → underscores).
    pub profile_name: String,
    /// Serialised CvJson returned by the `/optimize` endpoint in `optimized_cv_json`.
    pub cv_json: String,
    /// Language for the Typst experiences file (defaults to "en").
    pub lang: Option<String>,
}

pub async fn save_optimized_handler(
    request: Json<StandardRequest<SaveOptimizedRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let profile = normalize_profile_name(&request.data.profile_name);
    let lang = normalize_language(request.data.lang.as_deref());
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    // Parse the serialised CvJson back into a strongly-typed struct
    let cv_data: CvJson = serde_json::from_str(&request.data.cv_json).map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Invalid CV JSON: {}", e),
            "INVALID_CV_JSON".to_string(),
            vec!["Ensure cv_json contains the value returned by /optimize".to_string()],
            conversation_id.clone(),
        ))
    })?;

    // Write cv_params.toml + experiences_{lang}.typ into the new profile directory
    if let Err(e) = save_profile_cv_data(&profile, &tenant_data_dir, &cv_data, &lang).await {
        app_log!(error, "Failed to save optimized profile '{}': {}", profile, e);
        return Err(Json(StandardErrorResponse::new(
            format!("Failed to save profile: {}", e),
            "SAVE_FAILED".to_string(),
            vec!["Check disk space and permissions".to_string()],
            conversation_id,
        )));
    }

    app_log!(info, "Saved optimized profile '{}' (lang: {})", profile, lang);

    Ok(Json(ActionResponse::success(
        format!("Profile '{}' created successfully", profile),
        "profile_created".to_string(),
        conversation_id,
    )))
}
