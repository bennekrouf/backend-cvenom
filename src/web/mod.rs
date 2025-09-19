// src/web/mod.rs - REPLACE with this clean version (no v1 compatibility needed)

pub mod file_handlers;
pub mod handlers;
pub mod services;
pub mod types;

pub use handlers::*;
pub use types::*;

use crate::auth::{AuthConfig, AuthenticatedUser, OptionalAuth};
use crate::database::DatabaseConfig;
use anyhow::Result;
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

// Standard API Routes (clean, no v1/v2 confusion)

#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit(
    request: Json<StandardRequest<crate::linkedin_analysis::JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<DataResponse<JobAnalysisData>>, Json<StandardErrorResponse>> {
    handlers::analyze_job_fit_handler(request, auth, config, db_config).await
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
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::create_person_handler(request, auth, config, db_config).await
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
pub async fn options() -> Status {
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

    tokio::fs::create_dir_all(&data_dir).await?;

    let mut db_config = DatabaseConfig::new(database_path);

    if let Err(e) = db_config.init_pool().await {
        error!("Failed to initialize database: {}", e);
        return Err(e);
    }

    if let Err(e) = db_config.migrate().await {
        error!("Failed to run database migrations: {}", e);
        return Err(e);
    }

    let mut auth_config = AuthConfig::new("semantic-27923".to_string());

    if let Err(e) = auth_config.update_firebase_keys().await {
        error!("Failed to fetch Firebase keys: {}", e);
        return Err(e);
    }

    info!("Starting CVenom Multi-tenant API server");
    info!("Database: {}", db_config.database_path.display());
    info!("All endpoints use standard response format with conversation_id support");

    let _rocket = rocket::build()
        .attach(Cors)
        .manage(server_config)
        .manage(auth_config)
        .manage(db_config)
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
                options,
            ],
        )
        .launch()
        .await;

    Ok(())
}
