// src/web/handlers/profile_handlers.rs - Updated with new tenant structure
use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::FsOps;
use crate::web::types::{
    ActionResponse, CreateProfileRequest, DeleteProfileRequest, StandardErrorResponse,
    StandardRequest, UploadForm, WithConversationId,
};
use crate::web::RenameProfileRequest;
use crate::web::ServerConfig;
use graflog::app_log;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::State;

pub async fn create_profile_handler(
    request: Json<StandardRequest<CreateProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    // let normalized_profile = FsOps::normalize_profile_name(&request.data.profile);
    let profile_name = &request.data.profile;
    let conversation_id = request.conversation_id();

    app_log!(
        info,
        "Creating profile: {} for tenant: {} (user: {}) [{}]",
        profile_name,
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

    if let Err(e) = template_engine.create_profile_from_templates(
        &profile_name,
        &tenant_data_dir,
        Some(&request.data.profile),
    )
    // .await
    {
        app_log!(error, "Failed to create profile: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to create profile".to_string(),
            "CREATION_ERROR".to_string(),
            vec!["Try again or contact support".to_string()],
            conversation_id,
        )));
    }

    app_log!(info, "Successfully created profile: {}", profile_name);

    Ok(Json(ActionResponse::success(
        format!("Profile '{}' created successfully", request.data.profile),
        "created".to_string(),
        conversation_id,
    )))
}

pub async fn rename_profile_handler(
    old_name: String,
    request: Json<StandardRequest<RenameProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    // 1. Validate inputs
    if old_name.trim().is_empty() {
        return Err(Json(StandardErrorResponse::new(
            "Old profile name cannot be empty".to_string(),
            "INVALID_OLD_NAME".to_string(),
            vec!["Provide a valid profile name".to_string()],
            conversation_id,
        )));
    }

    if request.data.new_name.trim().is_empty() {
        return Err(Json(StandardErrorResponse::new(
            "New profile name cannot be empty".to_string(),
            "INVALID_NEW_NAME".to_string(),
            vec!["Provide a valid new profile name".to_string()],
            conversation_id,
        )));
    }

    // DON'T normalize the old_name - use it as-is from the URL
    let normalized_new_name = FsOps::normalize_profile_name(&request.data.new_name);

    if old_name == normalized_new_name {
        return Err(Json(StandardErrorResponse::new(
            "Old and new names are the same".to_string(),
            "NAMES_IDENTICAL".to_string(),
            vec!["Choose a different name".to_string()],
            conversation_id,
        )));
    }

    // 2. Check permissions
    let tenant_data_dir = get_tenant_folder_path(&user.email, &config.data_dir);

    if let Err(e) = FsOps::ensure_dir_exists(&tenant_data_dir).await {
        app_log!(error, "Failed to access tenant directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to access tenant data directory".to_string(),
            "TENANT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            conversation_id,
        )));
    }

    let old_profile_dir = tenant_data_dir.join(&old_name); // Use original old_name
    let new_profile_dir = tenant_data_dir.join(&normalized_new_name);

    if !old_profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' not found", old_name),
            "PROFILE_NOT_FOUND".to_string(),
            vec![
                "Check the profile name spelling".to_string(),
                "Use 'Show profiles' to see available profiles".to_string(),
            ],
            conversation_id,
        )));
    }

    // 3. Check if new name exists
    if new_profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' already exists", request.data.new_name),
            "PROFILE_ALREADY_EXISTS".to_string(),
            vec![
                "Choose a different name".to_string(),
                "Delete the existing profile first if needed".to_string(),
            ],
            conversation_id,
        )));
    }

    app_log!(
        info,
        "User {} (tenant: {}) renaming profile {} to {}",
        user.email,
        tenant.tenant_name,
        old_name,
        normalized_new_name
    );

    // Perform the rename operation
    if let Err(e) = tokio::fs::rename(&old_profile_dir, &new_profile_dir).await {
        app_log!(
            error,
            "Failed to rename directory from {} to {}: {}",
            old_profile_dir.display(),
            new_profile_dir.display(),
            e
        );
        return Err(Json(StandardErrorResponse::new(
            "Failed to rename profile directory".to_string(),
            "RENAME_ERROR".to_string(),
            vec!["Try again or contact support".to_string()],
            conversation_id,
        )));
    }

    app_log!(
        info,
        "Successfully renamed profile {} to {} for tenant: {}",
        old_name,
        request.data.new_name,
        tenant.tenant_name
    );

    Ok(Json(ActionResponse::success(
        format!(
            "Profile '{}' has been successfully renamed to '{}'",
            old_name, request.data.new_name
        ),
        "PROFILE_RENAMED".to_string(),
        conversation_id,
    )))
}

pub async fn list_profiles_handler(
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<Vec<String>>, Json<StandardErrorResponse>> {
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    match FsOps::list_profiles(&tenant_data_dir).await {
        Ok(profiles) => Ok(Json(profiles)),
        Err(e) => {
            app_log!(error, "Failed to list profiles: {}", e);
            Err(Json(StandardErrorResponse::new(
                "Failed to list profiles".to_string(),
                "LIST_ERROR".to_string(),
                vec!["Try again or contact support".to_string()],
                None,
            )))
        }
    }
}

pub async fn delete_profile_handler(
    request: Json<StandardRequest<DeleteProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let profile_name = &request.data.profile; // Use raw name for delete
    let conversation_id = request.conversation_id();

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let profile_dir = tenant_data_dir.join(profile_name); // Use raw name

    app_log!(
        info,
        "Attempting to delete profile at: {}",
        profile_dir.display()
    );

    if !profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' not found", request.data.profile),
            "NOT_FOUND".to_string(),
            vec!["Check the profile name and try again".to_string()],
            conversation_id,
        )));
    }

    if let Err(e) = FsOps::remove_dir_all(&profile_dir).await {
        app_log!(error, "Failed to delete profile directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to delete profile".to_string(),
            "DELETE_ERROR".to_string(),
            vec!["Try again or contact support".to_string()],
            conversation_id,
        )));
    }

    app_log!(info, "Successfully deleted profile: {}", profile_name);

    Ok(Json(ActionResponse::success(
        format!("Profile '{}' deleted successfully", request.data.profile),
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
    let normalized_profile = FsOps::normalize_profile_name(&upload.profile);

    app_log!(
        info,
        "User {} (tenant: {}) uploading picture for {}",
        user.email,
        tenant.tenant_name,
        upload.profile
    );

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let profile_dir = tenant_data_dir.join(&normalized_profile);

    if !profile_dir.exists() {
        return Err(Json(StandardErrorResponse::new(
            format!("Profile '{}' not found", upload.profile),
            "NOT_FOUND".to_string(),
            vec!["Create the profile first".to_string()],
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

    let profile_path = profile_dir.join("profile.png");

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
                "Successfully uploaded profile picture for profile: {}",
                normalized_profile
            );

            Ok(Json(ActionResponse::success(
                format!(
                    "Profile picture uploaded successfully for {}",
                    upload.profile
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
    profile: String,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<NamedFile, Json<StandardErrorResponse>> {
    let normalized_profile = FsOps::normalize_profile_name(&profile);

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    let profile_path = tenant_data_dir
        .join(&normalized_profile)
        .join("profile.png");

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

