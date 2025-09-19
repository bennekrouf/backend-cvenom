// src/web/handlers/person_handlers.rs - CORRECTED version

use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::utils::normalize_person_name;
use crate::web::types::{
    ActionResponse, CreatePersonRequest, DeletePersonRequest, StandardErrorResponse,
    StandardRequest, UploadForm, WithConversationId,
};
use crate::TemplateProcessor;

use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

pub async fn create_person_handler(
    request: Json<StandardRequest<CreatePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = normalize_person_name(&request.data.person);
    let conversation_id = request.conversation_id();

    info!(
        "User {} (tenant: {}) creating person: {}",
        user.email, tenant.tenant_name, request.data.person
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

    let template_processor = TemplateProcessor::new(config.templates_dir.clone());

    match template_processor.create_person_from_templates(
        &normalized_person,
        &tenant_data_dir,
        Some(&request.data.person),
    ) {
        Ok(_) => {
            info!(
                "Person directory created for {} by {} (tenant: {})",
                request.data.person, user.email, tenant.tenant_name
            );

            let next_actions = vec![
                format!("Upload profile picture for {}", request.data.person),
                format!("Edit CV parameters in {}/cv_params.toml", normalized_person),
                format!(
                    "Update work experience in {}/experiences_en.typ",
                    normalized_person
                ),
                format!("Generate CV PDF for {}", request.data.person),
            ];

            let response = ActionResponse::success(
                format!(
                    "Collaborator '{}' created successfully",
                    request.data.person
                ),
                "created".to_string(),
                conversation_id,
            )
            .with_next_actions(next_actions);

            Ok(Json(response))
        }
        Err(e) => {
            error!(
                "Person creation error for {} (tenant: {}): {}",
                request.data.person, tenant.tenant_name, e
            );

            let error_msg = if e.to_string().contains("already exists") {
                format!("Collaborator '{}' already exists", request.data.person)
            } else {
                "Failed to create collaborator directory".to_string()
            };

            Err(Json(StandardErrorResponse::new(
                error_msg,
                "PERSON_CREATE_ERROR".to_string(),
                vec![
                    "Try a different person name".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}

pub async fn delete_person_handler(
    request: Json<StandardRequest<DeletePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = normalize_person_name(&request.data.person);
    let conversation_id = request.conversation_id();

    info!(
        "User {} (tenant: {}) deleting person: {}",
        user.email, tenant.tenant_name, normalized_person
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

    let person_dir = tenant_data_dir.join(&normalized_person);

    if !person_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Collaborator '{}' not found", request.data.person),
            "PERSON_NOT_FOUND".to_string(),
            vec![
                "Check the person name spelling".to_string(),
                "Use 'Show collaborators' to see available persons".to_string(),
            ],
            conversation_id,
        )));
    }

    match tokio::fs::remove_dir_all(&person_dir).await {
        Ok(_) => {
            info!(
                "Person directory deleted: {} by {} (tenant: {})",
                normalized_person, user.email, tenant.tenant_name
            );

            let response = ActionResponse::success(
                format!(
                    "Collaborator '{}' deleted successfully",
                    request.data.person
                ),
                "deleted".to_string(),
                conversation_id,
            );

            Ok(Json(response))
        }
        Err(e) => {
            error!(
                "Failed to delete person directory {}: {}",
                person_dir.display(),
                e
            );

            Err(Json(StandardErrorResponse::new(
                "Failed to delete collaborator".to_string(),
                "DELETE_ERROR".to_string(),
                vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}

pub async fn upload_picture_handler(
    mut upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
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
            return Err(Json(StandardErrorResponse::new(
                "Database connection failed".to_string(),
                "DATABASE_ERROR".to_string(),
                vec!["Try again in a few moments".to_string()],
                None,
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
                None,
            )));
        }
    };

    let person_dir = tenant_data_dir.join(&normalized_person);
    if !person_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Collaborator '{}' not found", upload.person),
            "PERSON_NOT_FOUND".to_string(),
            vec![
                format!("Create collaborator '{}' first", upload.person),
                "Check the person name spelling".to_string(),
            ],
            None,
        )));
    }

    let content_type = upload.file.content_type();
    let is_image = content_type.map_or(false, |ct| {
        ct.is_png() || ct.is_jpeg() || ct.top() == "image"
    });

    if !is_image {
        return Err(Json(StandardErrorResponse::new(
            "Invalid file type".to_string(),
            "INVALID_FILE_TYPE".to_string(),
            vec![
                "Please upload an image file (PNG, JPG, etc.)".to_string(),
                "Supported formats: PNG, JPEG, JPG".to_string(),
            ],
            None,
        )));
    }

    let target_path = person_dir.join("profile.png");

    match upload.file.persist_to(&target_path).await {
        Ok(_) => {
            info!(
                "Profile picture uploaded for {} by {} (tenant: {})",
                upload.person, user.email, tenant.tenant_name
            );

            let next_actions = vec![
                format!("Generate CV PDF for {}", upload.person),
                format!("Update CV parameters for {}", upload.person),
            ];

            let response = ActionResponse::success(
                format!(
                    "Profile picture uploaded successfully for {}",
                    upload.person
                ),
                "uploaded".to_string(),
                None,
            )
            .with_next_actions(next_actions);

            Ok(Json(response))
        }
        Err(e) => {
            error!(
                "File upload error for {} (tenant: {}): {}",
                upload.person, tenant.tenant_name, e
            );

            Err(Json(StandardErrorResponse::new(
                "Failed to upload profile picture".to_string(),
                "UPLOAD_ERROR".to_string(),
                vec![
                    "Try again with a different image".to_string(),
                    "Ensure the image file is not corrupted".to_string(),
                ],
                None,
            )))
        }
    }
}

