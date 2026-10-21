// src/web/handlers/cv_handlers/profile_versions.rs
//! Duplicate a profile, list / restore its saved versions, and read the job
//! post a tailored profile was made for.

use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::core::profile_store;
use crate::utils::normalize_profile_name;
use crate::web::types::WithConversationId;
use crate::web::types::{ActionResponse, DataResponse, ServerConfig, StandardErrorResponse, StandardRequest};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DuplicateProfileRequest {
    pub profile: String,
    pub new_profile: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RestoreProfileRequest {
    pub profile: String,
    /// Version id from list versions; the newest when omitted.
    pub version: Option<String>,
}

fn err(message: String, code: &str, conversation_id: Option<String>) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(message, code.to_string(), vec![], conversation_id))
}

pub async fn duplicate_profile_handler(
    request: Json<StandardRequest<DuplicateProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let source = normalize_profile_name(&request.data.profile);
    let dest = normalize_profile_name(&request.data.new_profile);
    if dest.trim().is_empty() {
        return Err(err("new_profile must not be empty".into(), "INVALID_INPUT", conversation_id));
    }
    let tenant_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    profile_store::duplicate(&tenant_dir, &source, &dest)
        .await
        .map_err(|e| err(e.to_string(), "DUPLICATE_FAILED", conversation_id.clone()))?;
    Ok(Json(ActionResponse::success(
        format!("Profile '{}' copied to '{}'", source, dest),
        "profile_created".to_string(),
        conversation_id,
    )))
}

pub async fn list_profile_versions_handler(
    profile: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<DataResponse<Vec<String>>>, Json<StandardErrorResponse>> {
    let profile = normalize_profile_name(&profile);
    let tenant_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let versions = profile_store::list_versions(&tenant_dir, &profile)
        .await
        .map_err(|e| err(e.to_string(), "VERSIONS_FAILED", None))?;
    Ok(Json(DataResponse::success(
        format!("{} saved version(s) of '{}', newest first", versions.len(), profile),
        versions,
        None,
    )))
}

pub async fn restore_profile_version_handler(
    request: Json<StandardRequest<RestoreProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let profile = normalize_profile_name(&request.data.profile);
    let tenant_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let restored = profile_store::restore(&tenant_dir, &profile, request.data.version.as_deref())
        .await
        .map_err(|e| err(e.to_string(), "RESTORE_FAILED", conversation_id.clone()))?;
    Ok(Json(ActionResponse::success(
        format!(
            "Profile '{}' restored to version {} (the state before the restore was saved as a new version)",
            profile, restored
        ),
        "profile_restored".to_string(),
        conversation_id,
    )))
}

/// The job post a tailored profile was generated from (`data` is null when none).
pub async fn get_profile_job_handler(
    profile: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<DataResponse<Option<profile_store::JobSource>>>, Json<StandardErrorResponse>> {
    let profile = normalize_profile_name(&profile);
    let tenant_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    if !tenant_dir.join(&profile).join("cv_params.toml").is_file() {
        return Err(err(format!("Profile '{}' not found", profile), "NOT_FOUND", None));
    }
    let job = profile_store::read_job_source(&tenant_dir, &profile)
        .await
        .map_err(|e| err(e.to_string(), "READ_FAILED", None))?;
    let message = if job.is_some() { "Job source found" } else { "Profile was not made from a job post" };
    Ok(Json(DataResponse::success(message.to_string(), job, None)))
}
