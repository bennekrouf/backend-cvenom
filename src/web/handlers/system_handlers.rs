// src/web/handlers/system_handlers.rs
use crate::auth::{AuthenticatedUser, OptionalAuth};
use crate::core::database::{DatabaseConfig, TenantRepository, get_tenant_folder_path};
use crate::core::{FsOps, TemplateEngine};
use crate::web::types::{
    ActionResponse, DataResponse, StandardErrorResponse, TemplateInfo, TextResponse, UserInfo,
};
use crate::web::{ResponseType, ServerConfig};
use graflog::app_log;
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

/// DELETE /me — permanently delete the caller's account.
/// Removes all files from disk and hard-deletes the tenant DB record.
pub async fn delete_account_handler(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let email = auth.user().email.clone();
    app_log!(info, "Account deletion requested for: {}", email);

    // 1. Delete all files on disk
    let tenant_data_dir = get_tenant_folder_path(&email, &config.data_dir);
    if tenant_data_dir.exists() {
        if let Err(e) = FsOps::remove_dir_all(&tenant_data_dir).await {
            app_log!(error, "Failed to delete tenant directory for {}: {}", email, e);
            // Proceed anyway — DB record must still be removed
        }
    }

    // 2. Hard-delete tenant record from DB
    let pool = match db_config.pool() {
        Ok(p) => p,
        Err(e) => {
            app_log!(error, "DB unavailable during account deletion: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Database error during account deletion".to_string(),
                "DB_ERROR".to_string(),
                vec!["Contact support if this persists".to_string()],
                None,
            )));
        }
    };
    let repo = TenantRepository::new(pool);
    if let Err(e) = repo.delete_by_email(&email).await {
        app_log!(error, "Failed to delete tenant DB record for {}: {}", email, e);
        return Err(Json(StandardErrorResponse::new(
            "Failed to delete account record".to_string(),
            "DB_DELETE_ERROR".to_string(),
            vec!["Contact support if this persists".to_string()],
            None,
        )));
    }

    app_log!(info, "Account fully deleted for: {}", email);
    Ok(Json(ActionResponse::success(
        "Account and all associated data deleted".to_string(),
        "account_deleted".to_string(),
        None,
    )))
}

pub async fn health_handler(auth: OptionalAuth) -> Json<TextResponse> {
    let message = if auth.user.is_some() {
        "System is healthy (authenticated user)".to_string()
    } else {
        "System is healthy".to_string()
    };

    Json(TextResponse::success(message, None))
}
