// src/web/handlers/cv_handlers.rs

use crate::auth::AuthenticatedUser;
use crate::core::FsOps;
use crate::core::TemplateEngine;
use crate::database::{get_tenant_folder_path, DatabaseConfig};
use crate::image_validator::ImageValidator;
use crate::utils::{normalize_language, normalize_person_name};
use crate::web::services::CvConversionService;
use crate::web::types::*;
use crate::{CvConfig, CvGenerator};

use graflog::app_log;
use graflog::app_span;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::State;

pub async fn generate_cv_handler(
    request: Json<StandardRequest<GenerateRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    _db_config: &State<DatabaseConfig>,
) -> Result<PdfResponse, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    let generate_span = app_span!("cv_generation",
        user_email = %user.email,
        tenant = %tenant.tenant_name,
        person = %request.data.person,
        template = %request.data.template.as_deref().unwrap_or("default"),
        lang = %request.data.lang.as_deref().unwrap_or("en")
    );
    let _enter = generate_span.enter();

    let template_manager = match TemplateEngine::new(config.templates_dir.clone()) {
        Ok(manager) => {
            app_log!(
                info,
                "Template manager initialized successfully, templates_dir: {}",
                config.templates_dir.display()
            );
            manager
        }
        Err(e) => {
            app_log!(
                error,
                "Failed to initialize template manager, error: {}, templates_dir: {}",
                e,
                config.templates_dir.display()
            );
            return Err(Json(StandardErrorResponse::new(
                "Template system initialization failed".to_string(),
                "TEMPLATE_INIT_ERROR".to_string(),
                vec![
                    "Check if templates directory exists".to_string(),
                    "Contact system administrator".to_string(),
                ],
                conversation_id,
            )));
        }
    };

    let lang = normalize_language(request.data.lang.as_deref());
    let template_id = normalize_template(request.data.template.as_deref(), &template_manager);
    let normalized_person = normalize_person_name(&request.data.person);

    app_log!(
        info,
        "Parameters normalized, person: {}, template: {}, lang: {}",
        normalized_person,
        template_id,
        lang
    );

    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);
    app_log!(
        debug,
        "Using tenant data directory: {}",
        tenant_data_dir.display()
    );

    // Ensure directory exists
    if let Err(e) = FsOps::ensure_dir_exists(&tenant_data_dir).await {
        app_log!(
            error,
            "Failed to create tenant directory, error: {}, path: {}",
            e,
            tenant_data_dir.display()
        );
        return Err(Json(StandardErrorResponse::new(
            "Failed to access tenant data directory".to_string(),
            "TENANT_DIR_ERROR".to_string(),
            vec!["Contact system administrator".to_string()],
            conversation_id,
        )));
    }

    let person_dir = tenant_data_dir.join(&normalized_person);
    app_log!(
        debug,
        "Person directory, path: {}, exists: {}",
        person_dir.display(),
        person_dir.exists()
    );

    // Check if person directory exists
    if !person_dir.exists() {
        app_log!(
            warn,
            "Person directory does not exist: {}",
            person_dir.display()
        );
        return Err(Json(StandardErrorResponse::new(
            format!("Person '{}' not found in your account", request.data.person),
            "PERSON_NOT_FOUND".to_string(),
            vec![
                format!(
                    "Create person '{}' first using the create endpoint",
                    request.data.person
                ),
                "Check the person name spelling".to_string(),
            ],
            conversation_id,
        )));
    }

    let profile_image_path = person_dir.join("profile.png");
    app_log!(
        info,
        "Checking profile image, path: {}, exists: {}",
        profile_image_path.display(),
        profile_image_path.exists()
    );

    if let Err(validation_error) = ImageValidator::validate_profile_image(&profile_image_path).await
    {
        app_log!(
            warn,
            "Image validation failed: {}",
            validation_error.message
        );
    }

    app_log!(info, "Creating CV configuration, person: {}, lang: {}, template: {}, data_dir: {}, output_dir: {}, templates_dir: {}",
        normalized_person, lang, template_id, tenant_data_dir.display(), config.output_dir.display(), config.templates_dir.display()
    );

    let cv_config = CvConfig::new(&normalized_person, &lang)
        .with_template(template_id.to_string())
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    let pdf_gen_span = app_span!("pdf_generation", person = %normalized_person);
    let _pdf_enter = pdf_gen_span.enter();

    match CvGenerator::new(cv_config) {
        Ok(generator) => {
            app_log!(info, "CV generator created successfully");
            match generator.generate_pdf_data().await {
                Ok((pdf_data, filename)) => {
                    app_log!(
                        info,
                            "CV generation completed successfully, person: {}, pdf_size: {}, filename: {}",
                            normalized_person,
                            pdf_data.len(),
                            filename
                        );
                    Ok(PdfResponse::with_filename(pdf_data, filename))
                }
                Err(e) => {
                    app_log!(
                        error,
                        "CV generation failed, person: {}, error: {}, error_debug: {:?}",
                        normalized_person,
                        e,
                        e
                    );
                    Err(Json(StandardErrorResponse::new(
                        format!("CV generation failed: {}", e),
                        "GENERATION_ERROR".to_string(),
                        vec![
                            "Check the error details above".to_string(),
                            "Verify all required files exist".to_string(),
                        ],
                        conversation_id,
                    )))
                }
            }
        }
        Err(e) => {
            app_log!(
                error,
                "Failed to create CV generator, error: {}, error_debug: {:?}",
                e,
                e
            );
            Err(Json(StandardErrorResponse::new(
                format!("CV generator initialization failed: {}", e),
                "CONFIG_ERROR".to_string(),
                vec![
                    "Check your request parameters".to_string(),
                    "Verify the person exists".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}

pub async fn upload_and_convert_cv_handler(
    mut upload: Form<CvUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    _db_config: &State<DatabaseConfig>,
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

    // Extract all file information BEFORE calling persist_to()
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
        // Add extension based on content type
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
                "Only PDF and Word documents (.docx) are supported. Received content type: {}",
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

    // Use new tenant folder path
    let tenant_data_dir = get_tenant_folder_path(&auth.user().email, &config.data_dir);

    // Ensure directory exists
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

    // NOW call persist_to() after extracting all needed info
    if let Err(e) = upload.cv_file.persist_to(&temp_path).await {
        app_log!(error, "Failed to save uploaded file: {}", e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to process uploaded file".to_string(),
            "FILE_SAVE_ERROR".to_string(),
            vec!["Try uploading the file again".to_string()],
            None,
        )));
    }

    let conversion_service = match CvConversionService::new() {
        Ok(service) => service,
        Err(e) => {
            app_log!(error, "Failed to create conversion service: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "SERVICE_CONFIG_ERROR_MESSAGE".to_string(),
                "SERVICE_CONFIG_ERROR".to_string(),
                vec![
                    "CONTACT_SYSTEM_ADMIN".to_string(),
                    "SERVICE_TEMPORARILY_UNAVAILABLE".to_string(),
                ],
                None,
            )));
        }
    };

    let typst_content = match conversion_service
        .convert(&temp_path, &filename_with_extension)
        .await
    {
        Ok(content) => content,
        Err(error_msg) => {
            let _ = tokio::fs::remove_file(&temp_path).await;

            app_log!(error, "CV conversion failed: {}", error_msg);
            return Err(Json(StandardErrorResponse::new(
                "CV_CONVERSION_FAILED_MESSAGE".to_string(),
                "PARSE_ERROR".to_string(),
                vec![
                    "ENSURE_CV_READABLE_TEXT".to_string(),
                    "TRY_DIFFERENT_FILE_FORMAT".to_string(),
                    "CHECK_FILE_NOT_CORRUPTED".to_string(),
                ],
                None,
            )));
        }
    };

    let _ = tokio::fs::remove_file(&temp_path).await;

    let person_name = original_filename
        .split('.')
        .next()
        .unwrap_or(&original_filename);

    let normalized_person = normalize_person_name(person_name);

    match conversion_service
        .create_person_with_typst_content(
            &normalized_person,
            &typst_content,
            &tenant_data_dir,
            &config.templates_dir,
        )
        .await
    {
        Ok(_) => {
            app_log!(
                info,
                "CV converted and person created: {} by {} (tenant: {})",
                normalized_person,
                user.email,
                tenant.tenant_name
            );

            let next_actions = vec![
                format!("Upload profile picture for {}", person_name),
                format!("Edit CV parameters for {}", person_name),
                format!("Generate CV PDF for {}", person_name),
            ];

            let response = ActionResponse::success(
                format!(
                    "CV successfully converted and collaborator '{}' created",
                    person_name
                ),
                "created".to_string(),
                None,
            )
            .with_next_actions(next_actions);

            Ok(Json(response))
        }
        Err(e) => {
            app_log!(error, "Failed to create person from converted CV: {}", e);
            Err(Json(StandardErrorResponse::new(
                "FAILED_CREATE_COLLABORATOR_DIRECTORY".to_string(),
                "PERSON_CREATE_ERROR".to_string(),
                vec![
                    "TRY_AGAIN_MOMENTS".to_string(),
                    "CONTACT_SUPPORT_PERSISTS".to_string(),
                ],
                None,
            )))
        }
    }
}

fn normalize_template(template: Option<&str>, template_manager: &TemplateEngine) -> String {
    let requested = template.unwrap_or("default").to_lowercase();

    for available_template in template_manager.list_templates() {
        if available_template.to_lowercase() == requested {
            return available_template.to_lowercase();
        }
    }

    "default".to_string()
}
