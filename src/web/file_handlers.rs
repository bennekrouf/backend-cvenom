// src/web/file_handlers.rs - Updated for standard responses

use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::web::types::{
    ActionResponse, SaveFileRequest, StandardErrorResponse, StandardRequest, WithConversationId,
};
use async_recursion::async_recursion;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;
use tracing::{error, info, warn};

impl AuthenticatedUser {
    /// Ensure person directory exists for this user
    pub async fn ensure_person_exists(
        &self,
        config: &crate::web::types::ServerConfig,
        db_config: &DatabaseConfig,
    ) -> Result<(), anyhow::Error> {
        let pool = db_config.pool()?;
        let tenant_service = TenantService::new(pool);

        // Extract person name from email (before @)
        let person_name = self.firebase_user.email.split('@').next().unwrap_or("user");
        let normalized_person = crate::utils::normalize_person_name(person_name);

        tenant_service
            .create_default_person(
                &config.data_dir,
                &config.templates_dir,
                &self.tenant,
                &normalized_person,
                Some(person_name), // Pass original name for display
            )
            .await?;

        Ok(())
    }
}

pub async fn get_tenant_file_content_handler(
    path: String,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<String, Status> {
    let tenant = auth.tenant();

    // Security: Only allow .typ and .toml files
    if !path.ends_with(".typ") && !path.ends_with(".toml") {
        warn!("Unauthorized file access attempt: {}", path);
        return Err(Status::Forbidden);
    }

    info!(
        "User {} (tenant: {}) requesting file: {}",
        auth.user().email,
        tenant.tenant_name,
        path
    );

    // Get tenant-specific data directory
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Status::InternalServerError);
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
            return Err(Status::InternalServerError);
        }
    };

    let file_path = tenant_data_dir.join(&path);

    // Security: Ensure the file is within tenant directory
    if !file_path.starts_with(&tenant_data_dir) {
        warn!("Path traversal attempt: {}", path);
        return Err(Status::Forbidden);
    }

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            info!(
                "File content served: {} for tenant: {}",
                path, tenant.tenant_name
            );
            Ok(content)
        }
        Err(e) => {
            error!("Failed to read file {}: {}", file_path.display(), e);
            Err(Status::NotFound)
        }
    }
}

pub async fn save_tenant_file_content_handler(
    request: Json<StandardRequest<SaveFileRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    // Security: Only allow .typ and .toml files
    if !request.data.path.ends_with(".typ") && !request.data.path.ends_with(".toml") {
        warn!("Unauthorized file save attempt: {}", request.data.path);
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

    info!(
        "User {} (tenant: {}) saving file: {}",
        auth.user().email,
        tenant.tenant_name,
        request.data.path
    );

    // Get tenant-specific data directory
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

    let file_path = tenant_data_dir.join(&request.data.path);

    // Security: Ensure the file is within tenant directory
    if !file_path.starts_with(&tenant_data_dir) {
        warn!("Path traversal attempt: {}", request.data.path);
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
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            error!("Failed to create directory {}: {}", parent.display(), e);
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
            info!(
                "File saved: {} for tenant: {}",
                request.data.path, tenant.tenant_name
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
            error!("Failed to save file {}: {}", file_path.display(), e);
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
        error!("Failed to ensure person exists: {}", e);
    }
    let tenant = auth.tenant();

    info!(
        "User {} (tenant: {}) requesting file tree",
        auth.user().email,
        tenant.tenant_name
    );

    // Get tenant-specific data directory (don't create if it doesn't exist)
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);

            return Err(Status::InternalServerError);
        }
    };

    let tenant_service = TenantService::new(pool);
    let tenant_data_dir = tenant_service.get_tenant_data_dir(&config.data_dir, tenant);

    // Build file tree for tenant's directory only if it exists
    match build_file_tree(&tenant_data_dir).await {
        Ok(tree) => {
            let tree_value = serde_json::to_value(tree).unwrap_or_default();

            Ok(Json(tree_value))
        }
        Err(e) => {
            error!(
                "Failed to build file tree for tenant {}: {}",
                tenant.tenant_name, e
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
