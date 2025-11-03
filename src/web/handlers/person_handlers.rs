// src/web/handlers/person_handlers.rs - Updated with new tenant structure

use graflog::app_log;
use crate::auth::AuthenticatedUser;
use crate::core::FsOps;
use crate::database::{get_tenant_folder_path, DatabaseConfig};
use crate::web::types::{
    ActionResponse, CreatePersonRequest, DeletePersonRequest, StandardErrorResponse,
    StandardRequest, UploadForm, WithConversationId,
};
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::State;

pub async fn create_person_handler(
    request: Json<StandardRequest<CreatePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = FsOps::normalize_person_name(&request.data.person);
    let conversation_id = request.conversation_id();

    app_log!(
        info,
        "Creating person: {} for tenant: {} (user: {}) [{}]",
        normalized_person,
        tenant.tenant_name,
        user.email,
        conversation_id.clone().unwrap_or_default()
    );

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    // Ensure the directory exists
    if let Err(e) = FsOps::ensure_dir_exists(&tenant_data_dir).await {
        app_log!(error, "Failed to create tenant directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to create tenant directory".to_string(),
            "TENANT_ERROR".to_string(),
            vec!["Contact support if this persists".to_string()],
            conversation_id,
        )));
    }

    // Use core TemplateEngine
    let template_engine = match crate::core::TemplateEngine::new(config.templates_dir.clone()) {
        Ok(engine) => engine,
        Err(e) => {
            app_log!(error, "Failed to create template engine: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Template engine initialization failed".to_string(),
                "TEMPLATE_ERROR".to_string(),
                vec!["Contact support".to_string()],
                conversation_id,
            )));
        }
    };

    if let Err(e) = template_engine
        .create_person_from_templates(
            &normalized_person,
            &tenant_data_dir,
            Some(&request.data.person),
        )
        .await
    {
        app_log!(error, "Failed to create person: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to create person".to_string(),
            "CREATION_ERROR".to_string(),
            vec!["Try again or contact support".to_string()],
            conversation_id,
        )));
    }

    app_log!(info, "Successfully created person: {}", normalized_person);

    Ok(Json(ActionResponse::success(
        format!("Person '{}' created successfully", request.data.person),
        "created".to_string(),
        conversation_id,
    )))
}

pub async fn list_persons_handler(
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<Vec<String>>, Json<StandardErrorResponse>> {
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    match FsOps::list_persons(&tenant_data_dir).await {
        Ok(persons) => Ok(Json(persons)),
        Err(e) => {
            app_log!(error, "Failed to list persons: {}", e);
            Err(Json(StandardErrorResponse::new(
                "Failed to list persons".to_string(),
                "LIST_ERROR".to_string(),
                vec!["Try again or contact support".to_string()],
                None,
            )))
        }
    }
}

pub async fn delete_person_handler(
    request: Json<StandardRequest<DeletePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let normalized_person = FsOps::normalize_person_name(&request.data.person);
    let conversation_id = request.conversation_id();

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let person_dir = tenant_data_dir.join(&normalized_person);

    if !person_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Person '{}' not found", request.data.person),
            "NOT_FOUND".to_string(),
            vec!["Check the person name and try again".to_string()],
            conversation_id,
        )));
    }

    if let Err(e) = FsOps::remove_dir_all(&person_dir).await {
        app_log!(error, "Failed to delete person directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to delete person".to_string(),
            "DELETE_ERROR".to_string(),
            vec!["Try again or contact support".to_string()],
            conversation_id,
        )));
    }

    app_log!(info, "Successfully deleted person: {}", normalized_person);

    Ok(Json(ActionResponse::success(
        format!("Person '{}' deleted successfully", request.data.person),
        "deleted".to_string(),
        conversation_id,
    )))
}

pub async fn upload_picture_handler(
    upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let normalized_person = FsOps::normalize_person_name(&upload.person);

    app_log!(
        info,
        "User {} (tenant: {}) uploading picture for {}",
        user.email,
        tenant.tenant_name,
        upload.person
    );

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let person_dir = tenant_data_dir.join(&normalized_person);

    if !person_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Person '{}' not found", upload.person),
            "NOT_FOUND".to_string(),
            vec!["Create the person first".to_string()],
            None,
        )));
    }

    // Handle Option<&Path> from TempFile::path()
    let file_path = match upload.file.path() {
        Some(path) => path,
        None => {
            app_log!(error, "Uploaded file has no path");
            return Err(Json(StandardErrorResponse::new(
                "Invalid uploaded file".to_string(),
                "UPLOAD_ERROR".to_string(),
                vec!["Please try uploading again".to_string()],
                None,
            )));
        }
    };

    let file_bytes = match tokio::fs::read(file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            app_log!(error, "Failed to read uploaded file: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Failed to process uploaded file".to_string(),
                "UPLOAD_ERROR".to_string(),
                vec!["Please try uploading again".to_string()],
                None,
            )));
        }
    };

    let profile_path = person_dir.join("profile.png");

    // Write file using tokio fs
    match tokio::fs::write(&profile_path, &file_bytes).await {
        Ok(_) => {
            // Validate the uploaded image
            if let Err(e) = FsOps::validate_image(&profile_path).await {
                app_log!(error, "Invalid image file: {}", e);
                // Remove invalid file
                let _ = tokio::fs::remove_file(&profile_path).await;
                return Err(Json(StandardErrorResponse::new(
                    format!("Invalid image file: {}", e),
                    "INVALID_IMAGE".to_string(),
                    vec!["Please upload a valid PNG or JPEG image".to_string()],
                    None,
                )));
            }

            app_log!(
                info,
                "Successfully uploaded profile picture for person: {}",
                normalized_person
            );

            Ok(Json(ActionResponse::success(
                format!(
                    "Profile picture uploaded successfully for {}",
                    upload.person
                ),
                "uploaded".to_string(),
                None,
            )))
        }
        Err(e) => {
            app_log!(error, "Failed to save uploaded file: {}", e);
            Err(Json(StandardErrorResponse::new(
                "Failed to save uploaded file".to_string(),
                "SAVE_ERROR".to_string(),
                vec!["Please try again".to_string()],
                None,
            )))
        }
    }
}

pub async fn get_picture_handler(
    person: String,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<NamedFile, Json<StandardErrorResponse>> {
    let normalized_person = FsOps::normalize_person_name(&person);

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let profile_path = tenant_data_dir.join(&normalized_person).join("profile.png");

    if !profile_path.exists() {
        return Err(Json(StandardErrorResponse::new(
            "Profile picture not found".to_string(),
            "NOT_FOUND".to_string(),
            vec!["Upload a profile picture first".to_string()],
            None,
        )));
    }

    match NamedFile::open(&profile_path).await {
        Ok(file) => Ok(file),
        Err(e) => {
            app_log!(error, "Failed to serve profile picture: {}", e);
            Err(Json(StandardErrorResponse::new(
                "Failed to serve profile picture".to_string(),
                "FILE_ERROR".to_string(),
                vec!["Try again or contact support".to_string()],
                None,
            )))
        }
    }
}
