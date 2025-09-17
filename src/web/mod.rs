// src/web/mod.rs
pub mod file_handlers;
pub mod handlers;
pub mod services;
pub mod types;

pub use handlers::*;
pub use types::*;

use crate::auth::{AuthConfig, AuthenticatedUser, OptionalAuth};
use crate::database::DatabaseConfig;
use anyhow::Result;
// use linkedin_handlers::analyze_job_fit_handler;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{catchers, get, options, post, routes, Request, Response, State};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

// CORS Fairing
pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit(
    request: Json<crate::linkedin_analysis::JobAnalysisRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<crate::linkedin_analysis::JobAnalysisResponse>, Json<ErrorResponse>> {
    handlers::analyze_job_fit_handler(request, auth, config, db_config).await
}

// Route handlers - delegate to handler modules
#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<GenerateRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<PdfResponse, Json<ErrorResponse>> {
    handlers::generate_cv_handler(request, auth, config, db_config).await
}

#[post("/delete-person", data = "<request>")]
pub async fn delete_person(
    request: Json<DeletePersonRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<DeletePersonResponse>, Status> {
    handlers::delete_person_handler(request, auth, config, db_config).await
}

#[post("/create", data = "<request>")]
pub async fn create_person(
    request: Json<CreatePersonRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CreatePersonResponse>, Status> {
    handlers::create_person_handler(request, auth, config, db_config).await
}

#[post("/upload-picture", data = "<upload>")]
pub async fn upload_picture(
    upload: Form<UploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<UploadResponse>, Status> {
    handlers::upload_picture_handler(upload, auth, config, db_config).await
}

#[post("/cv/upload", data = "<upload>")]
pub async fn upload_and_convert_cv(
    upload: Form<CvUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CvConvertResponse>, Json<ErrorResponse>> {
    handlers::upload_and_convert_cv_handler(upload, auth, config, db_config).await
}

#[get("/templates")]
pub async fn get_templates(config: &State<ServerConfig>) -> Json<TemplatesResponse> {
    handlers::get_templates_handler(config).await
}

#[get("/me")]
pub async fn get_current_user(auth: AuthenticatedUser) -> Json<AuthResponse> {
    handlers::get_current_user_handler(auth).await
}

#[get("/me", rank = 2)]
pub async fn get_current_user_error() -> Json<ErrorResponse> {
    handlers::get_current_user_error_handler().await
}

#[get("/health")]
pub async fn health(auth: OptionalAuth) -> Json<&'static str> {
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
    request: Json<SaveFileRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    file_handlers::save_tenant_file_content_handler(request, auth, config, db_config).await
}

#[get("/files/tree")]
pub async fn get_tenant_files(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    file_handlers::get_tenant_files_handler(auth, config, db_config).await
}

#[options("/<_..>")]
pub async fn options() -> Status {
    Status::Ok
}

// Error catchers
#[rocket::catch(400)]
pub fn bad_request() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        success: false,
        error: "Invalid request format".to_string(),
        error_code: "BAD_REQUEST".to_string(),
        suggestions: vec![
            "Check your request JSON format".to_string(),
            "Verify all required fields are present".to_string(),
        ],
    })
}

#[rocket::catch(500)]
pub fn internal_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        success: false,
        error: "Internal server error".to_string(),
        error_code: "INTERNAL_ERROR".to_string(),
        suggestions: vec![
            "Try again in a few moments".to_string(),
            "Contact support if the problem persists".to_string(),
        ],
    })
}

// Main server start function
pub async fn start_web_server(
    data_dir: PathBuf,
    output_dir: PathBuf,
    templates_dir: PathBuf,
    database_path: PathBuf,
) -> Result<()> {
    Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new("cv_generator=INFO,rocket::server=OFF")),
        )
        .init();

    let server_config = ServerConfig {
        data_dir: data_dir.clone(),
        output_dir,
        templates_dir,
    };

    // Ensure data directory exists BEFORE creating database
    tokio::fs::create_dir_all(&data_dir).await?;

    // Use the passed database path directly
    let mut db_config = DatabaseConfig::new(database_path);

    // Initialize database pool and run migrations
    if let Err(e) = db_config.init_pool().await {
        error!("Failed to initialize database: {}", e);
        return Err(e);
    }

    if let Err(e) = db_config.migrate().await {
        error!("Failed to run database migrations: {}", e);
        return Err(e);
    }

    // Initialize auth config with your Firebase project ID
    let mut auth_config = AuthConfig::new("semantic-27923".to_string());

    // Fetch Firebase public keys
    if let Err(e) = auth_config.update_firebase_keys().await {
        error!("Failed to fetch Firebase keys: {}", e);
        return Err(e);
    }

    info!("Starting Multi-tenant CV Generator API server");
    info!("Database: {}", db_config.database_path.display());
    info!("Protected endpoints require Firebase Authentication + Tenant Authorization");

    let _rocket = rocket::build()
        .attach(Cors)
        .manage(server_config)
        .manage(auth_config)
        .manage(db_config)
        .register("/api", catchers![bad_request, internal_error])
        .mount(
            "/api",
            routes![
                generate_cv,
                delete_person,
                create_person,
                upload_picture,
                upload_and_convert_cv,
                get_templates,
                get_current_user,
                get_current_user_error,
                health,
                get_tenant_files,
                analyze_job_fit,
                get_tenant_file_content,
                save_tenant_file_content,
                options
            ],
        )
        .launch()
        .await;

    Ok(())
}

