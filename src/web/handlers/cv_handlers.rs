// src/web/handlers/cv_handlers.rs - REPLACE with this corrected version

use crate::auth::AuthenticatedUser;
use crate::database::{DatabaseConfig, TenantService};
use crate::image_validator::ImageValidator;
use crate::template_system::TemplateManager;
use crate::utils::{normalize_language, normalize_person_name};
use crate::web::services::CvConversionService;
use crate::web::types::*;
use crate::{CvConfig, CvGenerator};

use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::State;
use crate::app_log;

pub async fn generate_cv_handler(
    request: Json<StandardRequest<GenerateRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<PdfResponse, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();
    let conversation_id = request.conversation_id();

    let template_manager = match TemplateManager::new(config.templates_dir.clone()) {
        Ok(manager) => manager,
        Err(e) => {
            app_log!(error, "Failed to initialize template manager: {}", e);
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

    // ADD THIS LINE: Normalize the person name
    let normalized_person = normalize_person_name(&request.data.person);

    app_log!(info, 
        "User {} (tenant: {}) generating CV for {} (normalized: {})",
        user.email, tenant.tenant_name, request.data.person, normalized_person
    );

    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            app_log!(error, "Database connection failed: {}", e);
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
            app_log!(error, "Failed to ensure tenant data directory: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Failed to access tenant data directory".to_string(),
                "TENANT_DIR_ERROR".to_string(),
                vec!["Contact system administrator".to_string()],
                conversation_id,
            )));
        }
    };

    // CHANGE THIS LINE: Use normalized_person instead of request.data.person
    let person_dir = tenant_data_dir.join(&normalized_person);

    let profile_image_path = person_dir.join("profile.png");
    if let Err(validation_error) = ImageValidator::validate_profile_image(&profile_image_path).await
    {
        app_log!(error, 
            "Image validation failed for {} (tenant: {}): {}",
            request.data.person, tenant.tenant_name, validation_error.message
        );

        return Err(Json(StandardErrorResponse::new(
            validation_error.message,
            validation_error.error_type.code().to_string(),
            vec![
                validation_error.suggestion,
                "You can also generate CV without a profile picture".to_string(),
                "Use the upload endpoint to replace the corrupted image".to_string(),
            ],
            conversation_id,
        )));
    }

    // CHANGE THIS LINE: Use normalized_person instead of request.data.person
    let cv_config = CvConfig::new(&normalized_person, &lang)
        .with_template(template_id.to_string())
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    match CvGenerator::new(cv_config) {
        Ok(generator) => match generator.generate_pdf_data() {
            Ok(pdf_data) => {
                app_log!(info, 
                    "Successfully generated CV for {} by {} (tenant: {})",
                    request.data.person, user.email, tenant.tenant_name
                );
                Ok(PdfResponse(pdf_data))
            }
            Err(e) => {
                app_log!(error, 
                    "Generation error for {} (tenant: {}): {}",
                    request.data.person, tenant.tenant_name, e
                );

                let error_msg = e.to_string();
                if error_msg.contains("Person directory not found") {
                    Err(Json(StandardErrorResponse::new(
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
                    )))
                } else {
                    Err(Json(StandardErrorResponse::new(
                        "CV generation failed".to_string(),
                        "GENERATION_ERROR".to_string(),
                        vec![
                            "Try again in a few moments".to_string(),
                            "Check your CV data for any issues".to_string(),
                        ],
                        conversation_id,
                    )))
                }
            }
        },
        Err(e) => {
            app_log!(error, 
                "Config error for {} (tenant: {}): {}",
                request.data.person, tenant.tenant_name, e
            );
            Err(Json(StandardErrorResponse::new(
                "Invalid CV configuration".to_string(),
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
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();

    app_log!(info, 
        "User {} (tenant: {}) uploading CV for conversion",
        user.email, tenant.tenant_name
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

    // Database setup...
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            app_log!(error, "Database connection failed: {}", e);
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
            app_log!(error, "Failed to ensure tenant data directory: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Failed to access tenant data directory".to_string(),
                "TENANT_DIR_ERROR".to_string(),
                vec!["Contact system administrator".to_string()],
                None,
            )));
        }
    };

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

    let conversion_service = CvConversionService::new();
    let typst_content = match conversion_service
        .convert(&temp_path, &filename_with_extension)
        .await
    {
        Ok(content) => content,
        Err(error_msg) => {
            let _ = tokio::fs::remove_file(&temp_path).await;

            app_log!(error, "CV conversion failed: {}", error_msg);
            return Err(Json(StandardErrorResponse::new(
                format!("CV conversion failed: {}", error_msg),
                "PARSE_ERROR".to_string(),
                vec![
                    "Ensure your CV has clear, readable text".to_string(),
                    "Try uploading a different file format".to_string(),
                    "Check if the file is not corrupted".to_string(),
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
            app_log!(info, 
                "CV converted and person created: {} by {} (tenant: {})",
                normalized_person, user.email, tenant.tenant_name
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
                "Failed to create collaborator directory".to_string(),
                "PERSON_CREATE_ERROR".to_string(),
                vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                None,
            )))
        }
    }
}

fn normalize_template(template: Option<&str>, template_manager: &TemplateManager) -> String {
    let requested = template.unwrap_or("default").to_lowercase();

    for available_template in template_manager.list_templates() {
        if available_template.id.to_lowercase() == requested {
            return available_template.id.to_lowercase();
        }
    }

    "default".to_string()
}
