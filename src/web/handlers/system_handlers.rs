// src/web/handlers/system_handlers.rs
use crate::auth::{AuthenticatedUser, OptionalAuth};
use crate::core::TemplateEngine;
use crate::web::types::{
    DataResponse, StandardErrorResponse, TemplateInfo, TextResponse, UserInfo,
};
use crate::web::ResponseType;
// use graflog::app_log;
use rocket::serde::json::Json;
use rocket::State;

pub async fn get_templates_handler(
    config: &State<crate::web::types::ServerConfig>,
) -> Json<DataResponse<Vec<TemplateInfo>>> {
    match TemplateEngine::new(config.templates_dir.clone()) {
        Ok(template_engine) => {
            let templates: Vec<TemplateInfo> = template_engine
                .list_templates()
                .into_iter()
                .map(|template_name| {
                    let template_info = template_engine.get_template(&template_name);
                    TemplateInfo {
                        // id: template_name,
                        name: template_info
                            .map(|t| t.manifest.name.clone())
                            .unwrap_or_default(),
                        description: template_info
                            .and_then(|t| t.manifest.description.clone())
                            .unwrap_or_else(|| "No description available".to_string()),
                    }
                })
                .collect();

            Json(DataResponse {
                success: true,
                data: templates,
                message: "Templates retrieved successfully".to_string(),
                conversation_id: None,
                display_format: None,
                response_type: ResponseType::Data,
            })
        }
        Err(e) => Json(DataResponse {
            success: false,
            data: Vec::new(),
            message: format!("Failed to load templates: {}", e),
            conversation_id: None,
            display_format: None,
            response_type: ResponseType::Error,
        }),
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
