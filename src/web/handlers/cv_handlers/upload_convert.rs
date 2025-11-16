// src/web/handlers/cv_handlers/upload_convert.rs
//! CV upload and conversion handler

use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::core::{FsOps, ServiceClient};
use crate::utils::normalize_profile_name;
use crate::web::types::{ActionResponse, CvUploadForm, StandardErrorResponse};
use graflog::{app_log, app_span};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::State;

use super::helpers::create_profile_from_cv_data;

pub async fn upload_and_convert_cv_handler(
    mut upload: Form<CvUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<crate::web::types::ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();

    let upload_span = app_span!("cv_upload_conversion",
        user_email = %user.email,
        tenant = %tenant.tenant_name
    );
    let _enter = upload_span.enter();

    app_log!(
        info,
        "User {} (tenant: {}) uploading CV for conversion",
        user.email,
        tenant.tenant_name
    );

    // Extract file information
    let content_type = upload.cv_file.content_type();
    let file_size = upload.cv_file.len();

    let original_filename = upload
        .cv_file
        .raw_name()
        .and_then(|n| n.as_str())
        .unwrap_or_else(|| {
            if content_type.map_or(false, |ct| ct.is_pdf()) {
                "uploaded_cv.pdf"
            } else {
                "uploaded_cv.docx"
            }
        })
        .to_string();

    let filename_with_extension = if original_filename.to_lowercase().ends_with(".pdf")
        || original_filename.to_lowercase().ends_with(".docx")
    {
        original_filename.to_string()
    } else {
        if content_type.map_or(false, |ct| ct.is_pdf()) {
            format!("{}.pdf", original_filename)
        } else {
            format!("{}.docx", original_filename)
        }
    };

    // Validate content type
    let is_pdf = content_type.map_or(false, |ct| ct.is_pdf());
    let is_docx = content_type.map_or(false, |ct| {
        ct.to_string()
            .contains("vnd.openxmlformats-officedocument.wordprocessingml.document")
    });

    if !is_pdf && !is_docx {
        let received_type = content_type
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        return Err(Json(StandardErrorResponse::new(
            format!(
                "Only PDF and Word documents are supported. Received: {}",
                received_type
            ),
            "INVALID_FORMAT".to_string(),
            vec![
                "Upload a PDF file (.pdf)".to_string(),
                "Upload a Word document (.docx)".to_string(),
            ],
            None,
        )));
    }

    // Check file size (10MB limit)
    const MAX_SIZE: u64 = 10 * 1024 * 1024;
    if file_size > MAX_SIZE {
        return Err(Json(StandardErrorResponse::new(
            "File size exceeds 10MB limit".to_string(),
            "FILE_TOO_LARGE".to_string(),
            vec![
                "Compress your CV file".to_string(),
                "Use a smaller file size (max 10MB)".to_string(),
            ],
            None,
        )));
    }

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    if let Err(e) = FsOps::ensure_dir_exists(&tenant_data_dir).await {
        app_log!(error, "Failed to create tenant directory: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to access tenant data directory".to_string(),
            "TENANT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            None,
        )));
    }

    let temp_path = std::env::temp_dir().join(format!("cv_upload_{}", uuid::Uuid::new_v4()));

    if let Err(e) = upload.cv_file.persist_to(&temp_path).await {
        app_log!(error, "Failed to save uploaded file: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to process uploaded file".to_string(),
            "FILE_SAVE_ERROR".to_string(),
            vec!["Try uploading the file again".to_string()],
            None,
        )));
    }

    // Initialize service client for cv-import
    let service_client = match ServiceClient::new(cv_service_url.inner().clone(), 400) {
        Ok(client) => client,
        Err(e) => {
            app_log!(error, "Failed to initialize service client: {}", e);
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(Json(StandardErrorResponse::new(
                "Service configuration error".to_string(),
                "SERVICE_CONFIG_ERROR".to_string(),
                vec![
                    "Ensure cv_import service is available".to_string(),
                    "Contact system administrator".to_string(),
                ],
                None,
            )));
        }
    };

    // Get CvJson from cv-import service
    let cv_data = match service_client
        .upload_cv(&temp_path, &filename_with_extension)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            app_log!(error, "CV conversion failed: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "CV conversion failed".to_string(),
                "CONVERSION_ERROR".to_string(),
                vec![
                    "Ensure CV has readable text".to_string(),
                    "Try a different file format".to_string(),
                    "Check file is not corrupted".to_string(),
                ],
                None,
            )));
        }
    };

    let _ = tokio::fs::remove_file(&temp_path).await;

    let profile_name = original_filename
        .split('.')
        .next()
        .unwrap_or(&original_filename);

    let normalized_profile = normalize_profile_name(profile_name);
    let profile_dir = tenant_data_dir.join(&normalized_profile);

    // Convert CvJson to local file structure
    match create_profile_from_cv_data(&profile_dir, &cv_data, &normalized_profile).await {
        Ok(_) => {
            app_log!(
                info,
                "CV converted and profile created: {} by {} (tenant: {})",
                normalized_profile,
                user.email,
                tenant.tenant_name
            );

            let next_actions = vec![
                format!("Upload profile picture for {}", profile_name),
                format!("Edit CV parameters for {}", profile_name),
                format!("Generate CV PDF for {}", profile_name),
            ];

            let response = ActionResponse::success(
                format!(
                    "CV successfully converted and profile '{}' created",
                    profile_name
                ),
                "created".to_string(),
                None,
            )
            .with_next_actions(next_actions);

            Ok(Json(response))
        }
        Err(e) => {
            app_log!(error, "Failed to create profile from converted CV: {}", e);
            Err(Json(StandardErrorResponse::new(
                "Failed to create profile directory".to_string(),
                "PROFILE_CREATE_ERROR".to_string(),
                vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                None,
            )))
        }
    }
}
