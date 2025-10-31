// src/web/file_handlers.rs - Updated for new tenant structure

use crate::app_log;
use crate::auth::AuthenticatedUser;
use crate::core::FsOps;
use crate::database::{get_tenant_folder_path, DatabaseConfig};
use crate::web::types::{
    ActionResponse, SaveFileRequest, StandardErrorResponse, StandardRequest, WithConversationId,
};
use async_recursion::async_recursion;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;

impl AuthenticatedUser {
    /// Ensure person directory exists for this user
    pub async fn ensure_person_exists(
        &self,
        config: &crate::web::types::ServerConfig,
        _db_config: &DatabaseConfig,
    ) -> Result<(), anyhow::Error> {
        // Extract person name from email (before @)
        let person_name = self.firebase_user.email.split('@').next().unwrap_or("user");
        let normalized_person = crate::utils::normalize_person_name(person_name);

        // Use new tenant folder path
        let tenant_data_dir = get_tenant_folder_path(&self.firebase_user.email, &config.data_dir);
        let person_dir = tenant_data_dir.join(&normalized_person);

        // Ensure directories exist
        FsOps::ensure_dir_exists(&person_dir).await?;

        // Create default files if they don't exist
        let cv_params_path = person_dir.join("cv_params.toml");
        if !cv_params_path.exists() {
            // Use core TemplateEngine to create person files
            let template_engine = crate::core::TemplateEngine::new(config.templates_dir.clone())?;
            template_engine
                .create_person_from_templates(
                    &normalized_person,
                    &tenant_data_dir,
                    Some(person_name),
                )
                .await?;
        }

        Ok(())
    }
}

pub async fn get_tenant_file_content_handler(
    path: String,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<String, Status> {
    let tenant = auth.tenant();

    // Security: Only allow .typ and .toml files
    if !path.ends_with(".typ") && !path.ends_with(".toml") {
        app_log!(warn, "Unauthorized file access attempt: {}", path);
        return Err(Status::Forbidden);
    }

    app_log!(
        info,
        "User {} (tenant: {}) requesting file: {}",
        auth.user().email,
        tenant.tenant_name,
        path
    );

    // Use new tenant folder path
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let file_path = tenant_data_dir.join(&path);

    // Security: Ensure the file is within tenant directory
    if !file_path.starts_with(&tenant_data_dir) {
        app_log!(warn, "Path traversal attempt: {}", path);
        return Err(Status::Forbidden);
    }

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            app_log!(
                info,
                "File content served: {} for tenant: {}",
                path,
                tenant.tenant_name
            );
            Ok(content)
        }
        Err(e) => {
            app_log!(error, "Failed to read file {}: {}", file_path.display(), e);
            Err(Status::NotFound)
        }
    }
}

pub async fn save_tenant_file_content_handler(
    request: Json<StandardRequest<SaveFileRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    // Security: Only allow .typ and .toml files
    if !request.data.path.ends_with(".typ") && !request.data.path.ends_with(".toml") {
        app_log!(
            warn,
            "Unauthorized file save attempt: {}",
            request.data.path
        );
        return Err(Json(StandardErrorResponse::new(
            "File type not allowed".to_string(),
            "FORBIDDEN_FILE_TYPE".to_string(),
            vec![
                "Only .typ and .toml files can be edited".to_string(),
                "Use appropriate endpoints for other file types".to_string(),
            ],
            conversation_id,
        )));
    }

    app_log!(
        info,
        "User {} (tenant: {}) saving file: {}",
        auth.user().email,
        tenant.tenant_name,
        request.data.path
    );

    // Use new tenant folder path
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let file_path = tenant_data_dir.join(&request.data.path);

    // Security: Ensure the file is within tenant directory
    if !file_path.starts_with(&tenant_data_dir) {
        app_log!(warn, "Path traversal attempt: {}", request.data.path);
        return Err(Json(StandardErrorResponse::new(
            "Invalid file path".to_string(),
            "INVALID_PATH".to_string(),
            vec![
                "File path must be within your tenant directory".to_string(),
                "Contact support if you believe this is an error".to_string(),
            ],
            conversation_id,
        )));
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if let Err(e) = FsOps::ensure_dir_exists(parent).await {
            app_log!(
                error,
                "Failed to create directory {}: {}",
                parent.display(),
                e
            );
            return Err(Json(StandardErrorResponse::new(
                "Failed to create directory structure".to_string(),
                "DIRECTORY_CREATE_ERROR".to_string(),
                vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                conversation_id,
            )));
        }
    }

    match tokio::fs::write(&file_path, &request.data.content).await {
        Ok(_) => {
            app_log!(
                info,
                "File saved: {} for tenant: {}",
                request.data.path,
                tenant.tenant_name
            );

            let next_actions = vec![
                "Generate CV with updated content".to_string(),
                "Preview changes in CV".to_string(),
                "Save additional files if needed".to_string(),
            ];

            let response = ActionResponse::success(
                format!("File '{}' saved successfully", request.data.path),
                "saved".to_string(),
                conversation_id,
            )
            .with_next_actions(next_actions);

            Ok(Json(response))
        }
        Err(e) => {
            app_log!(error, "Failed to save file {}: {}", file_path.display(), e);
            Err(Json(StandardErrorResponse::new(
                "Failed to save file".to_string(),
                "FILE_SAVE_ERROR".to_string(),
                vec![
                    "Check file permissions".to_string(),
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}

pub async fn get_tenant_files_handler(
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    // Auto-create person if doesn't exist
    if let Err(e) = auth.ensure_person_exists(config, db_config).await {
        app_log!(error, "Failed to ensure person exists: {}", e);
    }

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    // Build file tree for tenant's directory only if it exists
    match build_file_tree(&tenant_data_dir).await {
        Ok(tree) => {
            let tree_value = serde_json::to_value(tree).unwrap_or_default();
            Ok(Json(tree_value))
        }
        Err(e) => {
            app_log!(
                error,
                "Failed to build file tree for tenant {}: {}",
                auth.tenant().tenant_name,
                e
            );
            Err(Status::InternalServerError)
        }
    }
}

#[async_recursion]
async fn build_file_tree(
    dir_path: &std::path::Path,
) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    use tokio::fs;
    let mut tree = HashMap::new();
    if !dir_path.exists() {
        return Ok(tree);
    }
    let mut entries = fs::read_dir(dir_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().await?;
        if metadata.is_dir() {
            let children = build_file_tree(&path).await?;
            tree.insert(
                name,
                serde_json::json!({
                    "type": "folder",
                    "children": children
                }),
            );
        } else if name.ends_with(".typ") || name.ends_with(".toml") {
            tree.insert(
                name,
                serde_json::json!({
                    "type": "file",
                    "size": metadata.len(),
                    "modified": metadata.modified().ok()
                }),
            );
        }
    }
    Ok(tree)
}

// Add wrapper function for tenant-aware file tree
pub async fn get_tenant_file_tree(
    email: &str,
    tenant_data_path: &std::path::PathBuf,
) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    let tenant_path = get_tenant_folder_path(email, tenant_data_path);
    build_file_tree(&tenant_path).await
}

