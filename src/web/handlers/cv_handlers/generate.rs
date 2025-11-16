// src/web/handlers/cv_handlers/generate.rs
//! CV PDF generation handler
use crate::auth::AuthenticatedUser;
use crate::core::database::{get_tenant_folder_path, DatabaseConfig};
use crate::core::{FsOps, TemplateEngine};
use crate::image_validator::ImageValidator;
use crate::utils::{normalize_language, normalize_profile_name};
use crate::web::types::WithConversationId;
use crate::web::types::{
    GenerateRequest, PdfResponse, ServerConfig, StandardErrorResponse, StandardRequest,
};
use crate::{CvConfig, CvGenerator};
use graflog::{app_log, app_span};
use rocket::serde::json::Json;
use rocket::State;

use super::helpers::normalize_template;

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
        profile = %request.data.profile,
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
    let normalized_profile = normalize_profile_name(&request.data.profile);

    app_log!(
        info,
        "Parameters normalized, profile: {}, template: {}, lang: {}",
        normalized_profile,
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

    let profile_dir = tenant_data_dir.join(&normalized_profile);
    app_log!(
        debug,
        "Profile directory, path: {}, exists: {}",
        profile_dir.display(),
        profile_dir.exists()
    );

    // Check if profile directory exists
    if !profile_dir.exists() {
        app_log!(
            warn,
            "Profile directory does not exist: {}",
            profile_dir.display()
        );
        return Err(Json(StandardErrorResponse::new(
            format!(
                "Profile '{}' not found in your account",
                request.data.profile
            ),
            "PROFILE_NOT_FOUND".to_string(),
            vec![
                format!(
                    "Create profile '{}' first using the create endpoint",
                    request.data.profile
                ),
                "Check the profile name spelling".to_string(),
            ],
            conversation_id,
        )));
    }

    let profile_image_path = profile_dir.join("profile.png");
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

    app_log!(info, "Creating CV configuration, profile: {}, lang: {}, template: {}, data_dir: {}, output_dir: {}, templates_dir: {}",
        normalized_profile, lang, template_id, tenant_data_dir.display(), config.output_dir.display(), config.templates_dir.display()
    );

    let cv_config = CvConfig::new(&normalized_profile, &lang)
        .with_template(template_id.to_string())
        .with_data_dir(tenant_data_dir)
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());

    let pdf_gen_span = app_span!("pdf_generation", profile = %normalized_profile);
    let _pdf_enter = pdf_gen_span.enter();

    match CvGenerator::new(cv_config) {
        Ok(generator) => {
            app_log!(info, "CV generator created successfully");
            match generator.generate_pdf_data().await {
                Ok((pdf_data, filename)) => {
                    app_log!(
                        info,
                        "CV generation completed successfully, profile: {}, pdf_size: {}, filename: {}",
                        normalized_profile,
                        pdf_data.len(),
                        filename
                    );
                    Ok(PdfResponse::with_filename(pdf_data, filename))
                }
                Err(e) => {
                    app_log!(
                        error,
                        "CV generation failed, profile: {}, error: {}, error_debug: {:?}",
                        normalized_profile,
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
                    "Verify the profile exists".to_string(),
                ],
                conversation_id,
            )))
        }
    }
}
