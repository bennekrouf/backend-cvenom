// src/web/cv_handlers.rs
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
use tracing::{error, info};

pub async fn generate_cv_handler(
    request: Json<GenerateRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<PdfResponse, Json<ErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();

    let template_manager = match TemplateManager::new(config.templates_dir.clone()) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize template manager: {}", e);
            return Err(Json(ErrorResponse {
                success: false,
                error: "Template system initialization failed".to_string(),
                error_code: "TEMPLATE_INIT_ERROR".to_string(),
                suggestions: vec![
                    "Check if templates directory exists".to_string(),
                    "Contact system administrator".to_string(),
                ],
            }));
        }
    };

    let lang = normalize_language(request.lang.as_deref());
    let template_id = normalize_template(request.template.as_deref(), &template_manager);

    info!(
        "User {} (tenant: {}) generating CV for {}",
        user.email, tenant.tenant_name, request.person
    );

    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Json(ErrorResponse {
                success: false,
                error: "Database connection failed".to_string(),
                error_code: "DATABASE_ERROR".to_string(),
                suggestions: vec!["Try again in a few moments".to_string()],
            }));
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
            return Err(Json(ErrorResponse {
                success: false,
                error: "Failed to access tenant data directory".to_string(),
                error_code: "TENANT_DIR_ERROR".to_string(),
                suggestions: vec!["Contact system administrator".to_string()],
            }));
        }
    };

    let normalized_person = normalize_person_name(&request.person);
    let person_dir = tenant_data_dir.join(&normalized_person);

    let profile_image_path = person_dir.join("profile.png");
    if let Err(validation_error) = ImageValidator::validate_profile_image(&profile_image_path).await
    {
        error!(
            "Image validation failed for {} (tenant: {}): {}",
            request.person, tenant.tenant_name, validation_error.message
        );

        return Err(Json(ErrorResponse {
            success: false,
            error: validation_error.message,
            error_code: validation_error.error_type.code().to_string(),
            suggestions: vec![
                validation_error.suggestion,
                "You can also generate CV without a profile picture".to_string(),
                "Use the upload endpoint to replace the corrupted image".to_string(),
            ],
        }));
    }

    let cv_config = CvConfig::new(&normalized_person, &lang)
        .with_template(template_id.to_string())
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    match CvGenerator::new(cv_config) {
        Ok(generator) => match generator.generate_pdf_data() {
            Ok(pdf_data) => {
                info!(
                    "Successfully generated CV for {} by {} (tenant: {})",
                    request.person, user.email, tenant.tenant_name
                );
                Ok(PdfResponse(pdf_data))
            }
            Err(e) => {
                error!(
                    "Generation error for {} (tenant: {}): {}",
                    request.person, tenant.tenant_name, e
                );

                let error_msg = e.to_string();
                if error_msg.contains("Person directory not found") {
                    Err(Json(ErrorResponse {
                        success: false,
                        error: format!("Person '{}' not found in your account", request.person),
                        error_code: "PERSON_NOT_FOUND".to_string(),
                        suggestions: vec![
                            format!(
                                "Create person '{}' first using the create endpoint",
                                request.person
                            ),
                            "Check the person name spelling".to_string(),
                        ],
                    }))
                } else {
                    Err(Json(ErrorResponse {
                        success: false,
                        error: "CV generation failed".to_string(),
                        error_code: "GENERATION_ERROR".to_string(),
                        suggestions: vec![
                            "Try again in a few moments".to_string(),
                            "Check your CV data for any issues".to_string(),
                        ],
                    }))
                }
            }
        },
        Err(e) => {
            error!(
                "Config error for {} (tenant: {}): {}",
                request.person, tenant.tenant_name, e
            );
            Err(Json(ErrorResponse {
                success: false,
                error: "Invalid CV configuration".to_string(),
                error_code: "CONFIG_ERROR".to_string(),
                suggestions: vec![
                    "Check your request parameters".to_string(),
                    "Verify the person exists".to_string(),
                ],
            }))
        }
    }
}

pub async fn upload_and_convert_cv_handler(
    mut upload: Form<CvUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CvConvertResponse>, Json<ErrorResponse>> {
    let user = auth.user();
    let tenant = auth.tenant();

    info!(
        "User {} (tenant: {}) uploading CV for conversion",
        user.email, tenant.tenant_name
    );

    // Extract all file information BEFORE calling persist_to()
    let content_type = upload.cv_file.content_type();
    let file_size = upload.cv_file.len();
    // let ucf = upload.cv_file;

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

        return Err(Json(ErrorResponse {
            success: false,
            error: format!(
                "Only PDF and Word documents (.docx) are supported. Received content type: {}",
                received_type
            ),
            error_code: "INVALID_FORMAT".to_string(),
            suggestions: vec![
                "Upload a PDF file (.pdf)".to_string(),
                "Upload a Word document (.docx)".to_string(),
            ],
        }));
    }

    info!(
        "Processing file: {} (content-type: {:?})",
        original_filename, content_type
    );

    // Check file size (10MB limit)
    const MAX_SIZE: u64 = 10 * 1024 * 1024;
    if file_size > MAX_SIZE {
        return Err(Json(ErrorResponse {
            success: false,
            error: "File size exceeds 10MB limit".to_string(),
            error_code: "FILE_TOO_LARGE".to_string(),
            suggestions: vec![
                "Compress your CV file".to_string(),
                "Use a smaller file size (max 10MB)".to_string(),
            ],
        }));
    }

    // Database setup...
    let pool = match db_config.pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("Database connection failed: {}", e);
            return Err(Json(ErrorResponse {
                success: false,
                error: "Database connection failed".to_string(),
                error_code: "DATABASE_ERROR".to_string(),
                suggestions: vec!["Try again in a few moments".to_string()],
            }));
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
            return Err(Json(ErrorResponse {
                success: false,
                error: "Failed to access tenant data directory".to_string(),
                error_code: "TENANT_DIR_ERROR".to_string(),
                suggestions: vec!["Contact system administrator".to_string()],
            }));
        }
    };

    let temp_path = std::env::temp_dir().join(format!("cv_upload_{}", uuid::Uuid::new_v4()));

    // NOW call persist_to() after extracting all needed info
    if let Err(e) = upload.cv_file.persist_to(&temp_path).await {
        error!("Failed to save uploaded file: {}", e);
        return Err(Json(ErrorResponse {
            success: false,
            error: "Failed to process uploaded file".to_string(),
            error_code: "FILE_SAVE_ERROR".to_string(),
            suggestions: vec!["Try uploading the file again".to_string()],
        }));
    }

    let conversion_service = CvConversionService::new();
    let typst_content = match conversion_service
        .convert(&temp_path, &filename_with_extension)
        .await
    {
        Ok(content) => content,
        Err(error_msg) => {
            let _ = tokio::fs::remove_file(&temp_path).await;

            error!("CV conversion failed: {}", error_msg);
            return Err(Json(ErrorResponse {
                success: false,
                error: format!("CV conversion failed: {}", error_msg),
                error_code: "PARSE_ERROR".to_string(),
                suggestions: vec![
                    "Ensure your CV has clear, readable text".to_string(),
                    "Try uploading a different file format".to_string(),
                    "Check if the file is not corrupted".to_string(),
                ],
            }));
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
            let person_dir = tenant_data_dir.join(&normalized_person);
            info!(
                "CV converted and person created: {} by {} (tenant: {})",
                normalized_person, user.email, tenant.tenant_name
            );

            Ok(Json(CvConvertResponse {
                success: true,
                message: format!(
                    "CV successfully converted and person '{}' created",
                    person_name
                ),
                person_name: normalized_person.clone(),
                tenant: tenant.tenant_name.clone(),
                person_dir: person_dir.to_string_lossy().to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to create person from converted CV: {}", e);
            Err(Json(ErrorResponse {
                success: false,
                error: "Failed to create person directory".to_string(),
                error_code: "PERSON_CREATE_ERROR".to_string(),
                suggestions: vec![
                    "Try again in a few moments".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
            }))
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
