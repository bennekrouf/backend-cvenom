// src/web/person_handlers.rs
use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::utils::normalize_person_name;
use crate::web::types::*;
use crate::TemplateProcessor;

use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

pub async fn create_person_handler(
    request: Json<CreatePersonRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CreatePersonResponse>, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = normalize_person_name(&request.person);

    info!(
        "User {} (tenant: {}) creating person: {}",
        user.email, tenant.tenant_name, request.person
    );

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

    let template_processor = TemplateProcessor::new(config.templates_dir.clone());

    match template_processor.create_person_from_templates(
        &normalized_person,
        &tenant_data_dir,
        Some(&request.person),
    ) {
        Ok(_) => {
            let person_dir = tenant_data_dir.join(&normalized_person);
            info!(
                "Person directory created for {} by {} (tenant: {})",
                request.person, user.email, tenant.tenant_name
            );

            Ok(Json(CreatePersonResponse {
                success: true,
                message: format!(
                    "Person directory created successfully for {}",
                    request.person
                ),
                person_dir: person_dir.to_string_lossy().to_string(),
                created_by: Some(user.email.clone()),
                tenant: tenant.tenant_name.clone(),
            }))
        }
        Err(e) => {
            error!(
                "Person creation error for {} (tenant: {}): {}",
                request.person, tenant.tenant_name, e
            );
            Err(Status::InternalServerError)
        }
    }
}

pub async fn delete_person_handler(
    request: Json<DeletePersonRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<DeletePersonResponse>, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = normalize_person_name(&request.person);

    info!(
        "User {} (tenant: {}) deleting person: {}",
        user.email, tenant.tenant_name, normalized_person
    );

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

    let person_dir = tenant_data_dir.join(&normalized_person);

    if !person_dir.exists() {
        return Ok(Json(DeletePersonResponse {
            success: false,
            message: format!("Person '{}' not found", request.person),
            deleted_person: request.person.clone(),
            tenant: tenant.tenant_name.clone(),
        }));
    }

    match tokio::fs::remove_dir_all(&person_dir).await {
        Ok(_) => {
            info!(
                "Person directory deleted: {} by {} (tenant: {})",
                normalized_person, user.email, tenant.tenant_name
            );
            Ok(Json(DeletePersonResponse {
                success: true,
                message: format!("Person '{}' deleted successfully", request.person),
                deleted_person: request.person.clone(),
                tenant: tenant.tenant_name.clone(),
            }))
        }
        Err(e) => {
            error!(
                "Failed to delete person directory {}: {}",
                person_dir.display(),
                e
            );
            Err(Status::InternalServerError)
        }
    }
}

pub async fn upload_picture_handler(
    mut upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<UploadResponse>, Status> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = normalize_person_name(&upload.person);

    info!(
        "User {} (tenant: {}) uploading picture for {}",
        user.email, tenant.tenant_name, upload.person
    );

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

    let person_dir = tenant_data_dir.join(&normalized_person);
    if !person_dir.exists() {
        return Ok(Json(UploadResponse {
            success: false,
            message: format!("Person directory not found: {}", upload.person),
            file_path: String::new(),
            tenant: tenant.tenant_name.clone(),
        }));
    }

    let content_type = upload.file.content_type();
    let is_image = content_type.map_or(false, |ct| {
        ct.is_png() || ct.is_jpeg() || ct.top() == "image"
    });

    if !is_image {
        return Ok(Json(UploadResponse {
            success: false,
            message: "Invalid file type. Please upload an image file (PNG, JPG, etc.)".to_string(),
            file_path: String::new(),
            tenant: tenant.tenant_name.clone(),
        }));
    }

    let target_path = person_dir.join("profile.png");

    match upload.file.persist_to(&target_path).await {
        Ok(_) => {
            info!(
                "Profile picture uploaded for {} by {} (tenant: {})",
                upload.person, user.email, tenant.tenant_name
            );
            Ok(Json(UploadResponse {
                success: true,
                message: format!(
                    "Profile picture uploaded successfully for {}",
                    upload.person
                ),
                file_path: target_path.to_string_lossy().to_string(),
                tenant: tenant.tenant_name.clone(),
            }))
        }
        Err(e) => {
            error!(
                "File upload error for {} (tenant: {}): {}",
                upload.person, tenant.tenant_name, e
            );
            Err(Status::InternalServerError)
        }
    }
}
