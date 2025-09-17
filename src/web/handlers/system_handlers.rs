// src/web/system_handlers.rs
use crate::auth::{AuthenticatedUser, OptionalAuth};
use crate::template_system::TemplateManager;
use crate::web::types::*;

use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

pub async fn get_templates_handler(config: &State<ServerConfig>) -> Json<TemplatesResponse> {
    match TemplateManager::new(config.templates_dir.clone()) {
        Ok(template_manager) => {
            let template_infos = template_manager
                .list_templates()
                .into_iter()
                .map(|template| TemplateInfo {
                    name: template.id.clone(),
                    description: template.manifest.description.clone(),
                })
                .collect();

            Json(TemplatesResponse {
                success: true,
                templates: template_infos,
            })
        }
        Err(e) => {
            error!("Failed to initialize template manager: {}", e);
            Json(TemplatesResponse {
                success: false,
                templates: vec![TemplateInfo {
                    name: "default".to_string(),
                    description: "Standard CV layout".to_string(),
                }],
            })
        }
    }
}

pub async fn get_current_user_handler(auth: AuthenticatedUser) -> Json<AuthResponse> {
    let user = auth.user();
    let tenant = auth.tenant();

    Json(AuthResponse {
        success: true,
        user: Some(UserInfo {
            uid: user.uid.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            picture: user.picture.clone(),
            tenant_name: tenant.tenant_name.clone(),
        }),
        message: format!(
            "User authenticated successfully for tenant: {}",
            tenant.tenant_name
        ),
    })
}

pub async fn get_current_user_error_handler() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        success: false,
        error: "Authentication required or user not authorized for any tenant".to_string(),
        error_code: "AUTHORIZATION_ERROR".to_string(),
        suggestions: vec![
            "Login is required".to_string(),
            "Contact administrator and ask authorization to this tenant".to_string(),
        ],
    })
}

pub async fn health_handler(auth: OptionalAuth) -> Json<&'static str> {
    if let Some(user) = auth.user {
        info!(
            "Health check by authenticated user: {} (tenant: {})",
            user.user().email,
            user.tenant().tenant_name
        );
    } else {
        info!("Health check by anonymous user");
    }
    Json("OK")
}
