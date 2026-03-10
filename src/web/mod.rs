// src/web/mod.rs
pub mod file_handlers;
pub mod handlers;
pub mod types;
use crate::auth::{AuthConfig, AuthenticatedUser, OptionalAuth};
use crate::core::database::DatabaseConfig;
use crate::linkedin_analysis::JobAnalysisRequest;
use crate::types::response::{OptimizeResponse, TranslateResponse};
use crate::web::handlers::translate::TranslateCvRequest;
use crate::web::handlers::{
    get_cv_data_handler, put_cv_data_handler,
    optimize_and_generate_handler, optimize_cv_handler, translate_cv_handler,
    upload_and_convert_cv_handler,
};
use crate::web::handlers::cv_data::CvFormData;
use crate::web::handlers::payment_handlers::{ConfirmPaymentRequest, CreateIntentRequest, GetBalanceResponse};
use anyhow::Result;
use graflog::app_log;

use rocket::config::{Config, LogLevel};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{catchers, get, post, put, routes, Request, Response, State};
use std::path::PathBuf;
pub use types::*;
mod cors_utils;
use cors_utils::universal_options_handler;

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
        let origin = request.headers().get_one("Origin");

        let allowed_origins = [
            "https://studio.cvenom.com",
            "http://localhost:4001",
            "http://127.0.0.1:4001",
        ];

        if let Some(origin) = origin {
            if allowed_origins.contains(&origin) {
                response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        } else {
            response.set_header(Header::new(
                "Access-Control-Allow-Origin",
                "https://studio.cvenom.com",
            ));
        }

        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "authorization, content-type, accept, origin, x-requested-with",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ));

        // Ensure OPTIONS requests always return 200
        if request.method() == Method::Options {
            response.set_status(Status::Ok);
        }
    }
}

#[post("/analyze-job-fit", data = "<request>")]
pub async fn analyze_job_fit(
    request: Json<StandardRequest<JobAnalysisRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<TextResponse>, Json<StandardErrorResponse>> {
    handlers::analyze_job_fit_handler(request, auth, config, cv_service_url, db_config).await
}

#[rocket::put("/profiles/<old_name>/rename", data = "<request>")]
pub async fn rename_profile_handler(
    old_name: String,
    request: Json<StandardRequest<RenameProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::rename_profile_handler(old_name, request, auth, config).await
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
pub async fn create_profile(
    request: Json<StandardRequest<CreateProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::create_profile_handler(request, auth, config).await
}

#[post("/delete-profile", data = "<request>")]
pub async fn delete_profile(
    request: Json<StandardRequest<DeleteProfileRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::delete_profile_handler(request, auth, config, db_config).await
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
    cv_service_url: &State<String>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    upload_and_convert_cv_handler(upload, auth, config, cv_service_url).await
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

// ── CV form-data routes ───────────────────────────────────────────────────────

/// GET /profiles/:name/cv-data
/// Returns a unified CvFormData JSON parsed from cv_params.toml + experiences.typ.
#[get("/profiles/<name>/cv-data")]
pub async fn get_cv_data(
    name: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<CvFormData>, Json<StandardErrorResponse>> {
    get_cv_data_handler(name, auth, config).await
}

/// PUT /profiles/:name/cv-data
/// Accepts CvFormData JSON, regenerates cv_params.toml and experiences_en.typ.
#[put("/profiles/<name>/cv-data", data = "<request>")]
pub async fn put_cv_data(
    name: String,
    request: Json<CvFormData>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    put_cv_data_handler(name, request, auth, config).await
}

#[get("/files/tree")]
pub async fn get_tenant_files(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Status> {
    // Changed return type
    file_handlers::get_tenant_files_handler(auth, config).await
}

#[post("/optimize", data = "<request>")]
pub async fn optimize_cv(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<OptimizeResponse>>, Json<StandardErrorResponse>> {
    optimize_cv_handler(request, auth, config, cv_service_url).await
}

/// Optimize the CV with ATS keyword injection **and** immediately compile + stream the PDF.
/// The optimized profile files are also persisted to disk for future use.
#[post("/optimize-and-generate", data = "<request>")]
pub async fn optimize_and_generate(
    request: Json<StandardRequest<OptimizeCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<PdfResponse, Json<StandardErrorResponse>> {
    optimize_and_generate_handler(request, auth, config, db_config, cv_service_url).await
}

#[post("/translate", data = "<request>")]
pub async fn translate_cv(
    request: Json<StandardRequest<TranslateCvRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<TranslateResponse>>, Json<StandardErrorResponse>> {
    translate_cv_handler(request, auth, config, cv_service_url).await
}

// ── Payment routes ────────────────────────────────────────────────────────────

/// POST /payment/intent — create a Stripe PaymentIntent
/// Returns { client_secret, publishable_key } to the frontend.
#[post("/payment/intent", data = "<request>")]
pub async fn payment_intent(
    request: Json<CreateIntentRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<crate::web::handlers::payment_handlers::CreateIntentResponse>, Json<StandardErrorResponse>> {
    crate::web::handlers::payment_handlers::create_payment_intent_handler(request, auth).await
}

/// POST /payment/confirm — verify Stripe payment + top-up api0 credits
#[post("/payment/confirm", data = "<request>")]
pub async fn payment_confirm(
    request: Json<ConfirmPaymentRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<crate::web::handlers::payment_handlers::ConfirmPaymentResponse>, Json<StandardErrorResponse>> {
    crate::web::handlers::payment_handlers::confirm_payment_handler(request, auth).await
}

/// GET /payment/balance — return the authenticated user's current credit balance
#[get("/payment/balance")]
pub async fn payment_balance(
    auth: AuthenticatedUser,
) -> Result<Json<GetBalanceResponse>, Json<StandardErrorResponse>> {
    crate::web::handlers::payment_handlers::get_balance_handler(auth).await
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
        .register("/", catchers![bad_request, internal_error])
        .mount(
            "/",
            routes![
                analyze_job_fit,
                generate_cv,
                create_profile,
                delete_profile,
                upload_picture,
                upload_and_convert_cv,
                get_templates,
                get_current_user,
                health,
                get_tenant_files,
                get_tenant_file_content,
                save_tenant_file_content,
                universal_options_handler,
                rename_profile_handler,
                optimize_cv,
                optimize_and_generate,
                translate_cv,
                payment_intent,
                payment_confirm,
                payment_balance,
                get_cv_data,
                put_cv_data,
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
