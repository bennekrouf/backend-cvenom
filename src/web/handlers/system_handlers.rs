// src/web/handlers/system_handlers.rs
use crate::auth::{AuthenticatedUser, OptionalAuth};
use crate::template_system::TemplateManager;
use crate::web::types::{
    DataResponse, StandardErrorResponse, TemplateInfo, TextResponse, UserInfo,
};
use rocket::serde::json::Json;
use rocket::State;
use crate::app_log;

pub async fn get_templates_handler(
    config: &State<crate::web::types::ServerConfig>,
) -> Json<DataResponse<Vec<TemplateInfo>>> {
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

            Json(DataResponse::success(
                "Templates retrieved successfully".to_string(),
                template_infos,
                None,
            ))
        }
        Err(e) => {
            app_log!(error, "Failed to initialize template manager: {}", e);
            let default_templates = vec![TemplateInfo {
                name: "default".to_string(),
                description: "Standard CV layout".to_string(),
            }];

            Json(DataResponse::success(
                "Templates retrieved (default only)".to_string(),
                default_templates,
                None,
            ))
        }
    }
}

pub async fn get_current_user_handler(auth: AuthenticatedUser) -> Json<DataResponse<UserInfo>> {
    let user = auth.user();
    let tenant = auth.tenant();

    let user_info = UserInfo {
        uid: user.uid.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        picture: user.picture.clone(),
        tenant_name: tenant.tenant_name.clone(),
    };

    Json(DataResponse::success(
        format!("User authenticated for tenant: {}", tenant.tenant_name),
        user_info,
        None,
    ))
}

pub async fn get_current_user_error_handler() -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        "Authentication required or user not authorized for any tenant".to_string(),
        "AUTHORIZATION_ERROR".to_string(),
        vec![
            "Login is required".to_string(),
            "Contact administrator for tenant access".to_string(),
        ],
        None,
    ))
}

pub async fn health_handler(auth: OptionalAuth) -> Json<TextResponse> {
    let message = if auth.user.is_some() {
        "System is healthy (authenticated user)".to_string()
    } else {
        "System is healthy".to_string()
    };

    Json(TextResponse::success(message, None))
}

