// src/web/mod.rs
pub mod file_handlers;
pub mod handlers;
pub mod services;
pub mod types;

pub use handlers::*;
use rocket::config::{Config, LogLevel};
pub use types::*;

use crate::auth::{AuthConfig, AuthenticatedUser, OptionalAuth};
use crate::database::DatabaseConfig;
use anyhow::Result;
use graflog::app_log;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{catchers, get, options, post, routes, Request, Response, State};
use std::path::PathBuf;

// CORS Fairing
pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        app_log!(
            info,
            "CORS: Request method: {:?}, Origin: {:?}",
            request.method(),
            request.headers().get_one("Origin")
        );
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request
            .headers()
            .get_one("Origin")
            .unwrap_or("https://studio.cvenom.com");

        app_log!(info, "CORS: Processing response for origin: {}", origin);

        let allowed_origins = [
            "https://studio.cvenom.com",
            "http://localhost:4001",
            "http://127.0.0.1:4001",
        ];

        if allowed_origins.contains(&origin) {
            response.set_header(Header::new("Access-Control-Allow-Origin", origin));
        } else {
            response.set_header(Header::new(
                "Access-Control-Allow-Origin",
                "https://studio.cvenom.com",
            ));
        }

        response.set_header(Header::new(
            "Access-Control-Allow-Headers", 
            "authorization, content-type, referer, sec-ch-ua, sec-ch-ua-mobile, sec-ch-ua-platform, user-agent"
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS, DELETE, PUT",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_header(Header::new("Access-Control-Max-Age", "86400"));
    }
}

// Standard API Routes (clean, no v1/v2 confusion)
#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit(
    request: Json<StandardRequest<crate::linkedin_analysis::JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<TextResponse>, Json<StandardErrorResponse>> {
    // Changed return type
    handlers::analyze_job_fit_handler(request, auth, config, db_config).await
}

#[rocket::put("/collaborators/<old_name>/rename", data = "<request>")]
pub async fn rename_collaborator_handler(
    old_name: String,
    request: Json<StandardRequest<RenameCollaboratorRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::rename_collaborator_handler(old_name, request, auth, config).await
}

#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<StandardRequest<GenerateRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<PdfResponse, Json<StandardErrorResponse>> {
    handlers::generate_cv_handler(request, auth, config, db_config).await
}

#[post("/create", data = "<request>")]
pub async fn create_person(
    request: Json<StandardRequest<CreatePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::create_person_handler(request, auth, config).await
}

#[post("/delete-person", data = "<request>")]
pub async fn delete_person(
    request: Json<StandardRequest<DeletePersonRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::delete_person_handler(request, auth, config, db_config).await
}

#[post("/upload-picture", data = "<upload>")]
pub async fn upload_picture(
    upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::upload_picture_handler(upload, auth, config, db_config).await
}

#[post("/cv/upload", data = "<upload>")]
pub async fn upload_and_convert_cv(
    upload: Form<CvUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::upload_and_convert_cv_handler(upload, auth, config, db_config).await
}

#[get("/templates")]
pub async fn get_templates(config: &State<ServerConfig>) -> Json<DataResponse<Vec<TemplateInfo>>> {
    handlers::get_templates_handler(config).await
}

#[get("/me")]
pub async fn get_current_user(auth: AuthenticatedUser) -> Json<DataResponse<UserInfo>> {
    handlers::get_current_user_handler(auth).await
}

#[get("/me", rank = 2)]
pub async fn get_current_user_error() -> Json<StandardErrorResponse> {
    handlers::get_current_user_error_handler().await
}

#[get("/health")]
pub async fn health(auth: OptionalAuth) -> Json<TextResponse> {
    handlers::health_handler(auth).await
}

#[get("/files/content?<path>")]
pub async fn get_tenant_file_content(
    path: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<String, Status> {
    file_handlers::get_tenant_file_content_handler(path, auth, config, db_config).await
}

#[post("/files/save", data = "<request>")]
pub async fn save_tenant_file_content(
    request: Json<StandardRequest<SaveFileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    file_handlers::save_tenant_file_content_handler(request, auth, config, db_config).await
}

#[get("/files/tree")]
pub async fn get_tenant_files(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    // Changed return type
    file_handlers::get_tenant_files_handler(auth, config, db_config).await
}

#[options("/<_..>")]
pub async fn options_handler() -> Status {
    Status::Ok
}

// Error catchers
#[rocket::catch(400)]
pub fn bad_request() -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        "Invalid request format".to_string(),
        "BAD_REQUEST".to_string(),
        vec![
            "Check your request JSON format".to_string(),
            "Verify all required fields are present".to_string(),
        ],
        None,
    ))
}

#[rocket::catch(500)]
pub fn internal_error() -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        "Internal server error".to_string(),
        "INTERNAL_ERROR".to_string(),
        vec![
            "Try again in a few moments".to_string(),
            "Contact support if the problem persists".to_string(),
        ],
        None,
    ))
}

pub async fn start_web_server(
    data_dir: PathBuf,
    output_dir: PathBuf,
    templates_dir: PathBuf,
    database_path: PathBuf,
    port: u16,
    cv_service_url: String,
) -> Result<()> {
    let server_config = ServerConfig {
        data_dir: data_dir.clone(),
        output_dir,
        templates_dir,
    };

    tokio::fs::create_dir_all(&data_dir).await?;

    let mut db_config = DatabaseConfig::new(database_path);

    if let Err(e) = db_config.init_pool().await {
        app_log!(error, "Failed to initialize database: {}", e);
        return Err(e);
    }

    if let Err(e) = db_config.migrate().await {
        app_log!(error, "Failed to run database migrations: {}", e);
        return Err(e);
    }

    let mut auth_config = AuthConfig::new("semantic-27923".to_string());

    if let Err(e) = auth_config.update_firebase_keys().await {
        app_log!(error, "Failed to fetch Firebase keys: {}", e);
        return Err(e);
    }

    app_log!(info, "Starting CVenom Multi-tenant API server");
    app_log!(info, "Database: {}", db_config.database_path.display());
    app_log!(
        info,
        "All endpoints use standard response format with conversation_id support"
    );
    app_log!(info, "Attempting to bind to port: {}", port);

    let config = Config {
        port,
        log_level: LogLevel::Off,
        ..Config::default()
    };

    let _rocket = rocket::custom(config)
        .configure(rocket::Config::figment().merge(("port", port)))
        .attach(Cors)
        .manage(server_config)
        .manage(auth_config)
        .manage(db_config)
        .manage(cv_service_url)
        .register("/api", catchers![bad_request, internal_error])
        .mount(
            "/api",
            routes![
                analyze_job_fit,
                generate_cv,
                create_person,
                delete_person,
                upload_picture,
                upload_and_convert_cv,
                get_templates,
                get_current_user,
                get_current_user_error,
                health,
                get_tenant_files,
                get_tenant_file_content,
                save_tenant_file_content,
                options_handler,
                rename_collaborator_handler,
            ],
        )
        .launch()
        .await;

    app_log!(
        info,
        "Server successfully started and bound to port: {}",
        port
    );
    Ok(())
}
