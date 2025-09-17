// src/web/file_handlers.rs
use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::web::types::*;
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
        config: &ServerConfig,
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
    config: &State<ServerConfig>,
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
    request: Json<SaveFileRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    let tenant = auth.tenant();

    // Security: Only allow .typ and .toml files
    if !request.path.ends_with(".typ") && !request.path.ends_with(".toml") {
        warn!("Unauthorized file save attempt: {}", request.path);
        return Err(Status::Forbidden);
    }

    info!(
        "User {} (tenant: {}) saving file: {}",
        auth.user().email,
        tenant.tenant_name,
        request.path
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

    let file_path = tenant_data_dir.join(&request.path);

    // Security: Ensure the file is within tenant directory
    if !file_path.starts_with(&tenant_data_dir) {
        warn!("Path traversal attempt: {}", request.path);
        return Err(Status::Forbidden);
    }

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            error!("Failed to create directory {}: {}", parent.display(), e);
            return Err(Status::InternalServerError);
        }
    }

    match tokio::fs::write(&file_path, &request.content).await {
        Ok(_) => {
            info!(
                "File saved: {} for tenant: {}",
                request.path, tenant.tenant_name
            );
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "File saved successfully"
            })))
        }
        Err(e) => {
            error!("Failed to save file {}: {}", file_path.display(), e);
            Err(Status::InternalServerError)
        }
    }
}

pub async fn get_tenant_files_handler(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
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
        Ok(tree) => Ok(Json(serde_json::to_value(tree).unwrap_or_default())),
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
