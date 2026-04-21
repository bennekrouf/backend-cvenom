// src/web/mod.rs
pub mod file_handlers;
pub mod handlers;
pub mod types;
use crate::auth::{AuthConfig, AuthenticatedUser, OptionalAuth};
use crate::core::database::DatabaseConfig;
use crate::linkedin_analysis::JobAnalysisRequest;
use crate::types::response::{OptimizeResponse, TranslateResponse};
use crate::web::handlers::cover_letter::CoverLetterRequest;
use crate::web::handlers::cover_letter::CoverLetterResult;
use crate::web::handlers::translate::TranslateCvRequest;
use crate::web::handlers::{
    cover_letter_handler,
    cover_letter_export_handler,
    delete_account_handler,
    get_cv_data_handler, put_cv_data_handler,
    optimize_and_generate_handler, optimize_cv_handler, save_optimized_handler, translate_cv_handler,
    upload_and_convert_cv_handler, import_text_cv_handler,
};
use crate::web::handlers::cv_handlers::ImportTextRequest;
use crate::web::handlers::cv_handlers::CoverLetterExportRequest;
use crate::core::database::{get_tenant_folder_path, TenantRepository};
use crate::core::FsOps;
use crate::web::handlers::cv_data::CvFormData;
use crate::web::handlers::payment_handlers::{ConfirmPaymentRequest, CreateIntentRequest, GetBalanceResponse, TransactionsResponse, get_transactions_handler, AdminCreditRequest, admin_add_credits_handler};
use crate::web::handlers::referral_handlers::{get_referral_link_handler, ReferralLinkResponse};
use anyhow::Result;
use graflog::app_log;

use rocket::config::{Config, LogLevel};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{catchers, delete, get, post, put, routes, Request, Response, State};
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
            "authorization, content-type, accept, origin, x-requested-with, x-referral-code",
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

/// POST /cv/import-text
/// Accept raw CV text (extracted by an LLM / Claude from a user-attached file) and create a profile.
/// Request body: { "cv_text": "...", "profile_name": "optional-name" }
#[post("/cv/import-text", data = "<request>")]
pub async fn import_cv_from_text(
    request: Json<StandardRequest<ImportTextRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    import_text_cv_handler(request, auth, config, cv_service_url).await
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

/// GET /profiles/:name/cv-data?lang=en
/// Returns a unified CvFormData JSON parsed from cv_params.toml + experiences_{lang}.typ.
#[get("/profiles/<name>/cv-data?<lang>")]
pub async fn get_cv_data(
    name: String,
    lang: Option<String>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<CvFormData>, Json<StandardErrorResponse>> {
    get_cv_data_handler(name, lang, auth, config).await
}

/// PUT /profiles/:name/cv-data?lang=en
/// Accepts CvFormData JSON, regenerates cv_params.toml and experiences_{lang}.typ.
#[put("/profiles/<name>/cv-data?<lang>", data = "<request>")]
pub async fn put_cv_data(
    name: String,
    lang: Option<String>,
    request: Json<CvFormData>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    put_cv_data_handler(name, lang, request, auth, config).await
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

/// Save an optimized CV under a new profile name.
/// Accepts `{ profile_name, cv_json, lang }` where `cv_json` is the value
/// returned by `/optimize` in `data.optimized_cv_json`.
#[post("/save-optimized", data = "<request>")]
pub async fn save_optimized_cv(
    request: Json<StandardRequest<SaveOptimizedRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    save_optimized_handler(request, auth, config).await
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

/// POST /cover-letter — generate a cover letter from CV data + job description.
/// Costs 20 credits (same as CV generation).
#[post("/cover-letter", data = "<request>")]
pub async fn generate_cover_letter(
    request: Json<StandardRequest<CoverLetterRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<CoverLetterResult>>, Json<StandardErrorResponse>> {
    cover_letter_handler(request, auth, config, cv_service_url).await
}

/// POST /cover-letter/export — export a cover letter text as .docx (no credit cost)
#[post("/cover-letter/export", data = "<request>")]
pub async fn export_cover_letter(
    request: Json<CoverLetterExportRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<DocxResponse, Json<StandardErrorResponse>> {
    cover_letter_export_handler(request, auth, config).await
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

/// DELETE /me — permanently delete caller's account and all associated data.
#[delete("/me")]
pub async fn delete_me(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    delete_account_handler(auth, config, db_config).await
}

/// GET /payment/balance — return the authenticated user's current credit balance
#[get("/payment/balance")]
pub async fn payment_balance(
    auth: AuthenticatedUser,
) -> Result<Json<GetBalanceResponse>, Json<StandardErrorResponse>> {
    crate::web::handlers::payment_handlers::get_balance_handler(auth).await
}

/// GET /payment/transactions — authenticated user's credit transaction history
#[get("/payment/transactions")]
pub async fn payment_transactions(
    auth: AuthenticatedUser,
) -> Result<Json<TransactionsResponse>, Json<StandardErrorResponse>> {
    get_transactions_handler(auth).await
}

/// POST /admin/credits — manually add or remove credits for a cvenom tenant (admin only).
/// Auth: valid Firebase JWT whose email is "mohamed.bennekrouf@gmail.com".
/// The target email must belong to an existing cvenom tenant; rejects otherwise.
/// Body: { "email": "...", "amount": 100, "description": "optional" }
#[post("/admin/credits", data = "<request>")]
pub async fn admin_credits(
    request: Json<AdminCreditRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<crate::web::handlers::payment_handlers::AdminCreditResponse>, Json<StandardErrorResponse>> {
    admin_add_credits_handler(request, auth.email(), db_config).await
}

/// GET /referral/my-link — return the authenticated user's referral link and stats
#[get("/referral/my-link")]
pub async fn get_my_referral_link(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ReferralLinkResponse>, Json<StandardErrorResponse>> {
    get_referral_link_handler(auth, db_config).await
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

    let google_project_id = std::env::var("CVENOM_GOOGLE_PROJECT_ID")
        .expect("CVENOM_GOOGLE_PROJECT_ID env var is required");
    let auth_config = AuthConfig::new(google_project_id);

    if let Err(e) = auth_config.update_firebase_keys().await {
        app_log!(error, "Failed to fetch Firebase keys: {}", e);
        return Err(e);
    }

    // Pre-warm the OIDC JWK cache when CVENOM_OIDC_AUDIENCE is configured.
    // Non-fatal: keys will be fetched on the first OIDC request if this fails.
    if auth_config.oidc_audience.is_some() {
        if let Err(e) = auth_config.update_oidc_jwks().await {
            app_log!(warn, "Failed to pre-fetch OIDC JWKs (will retry on first request): {}", e);
        }
    }

    // ── Data-retention background task ────────────────────────────────────────
    // Runs once per day. Deletes email-based tenants inactive for DATA_RETENTION_DAYS
    // (default 365). Domain tenants are never auto-deleted.
    if let Ok(cleanup_pool) = db_config.pool().map(|p| p.clone()) {
        let cleanup_data_dir = data_dir.clone();
        let retention_days = std::env::var("DATA_RETENTION_DAYS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(365);

        tokio::spawn(async move {
            // Wait 1 hour after startup before first run so it doesn't slow boot.
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
            loop {
                interval.tick().await;
                let repo = TenantRepository::new(&cleanup_pool);
                match repo.find_stale_email_tenants(retention_days).await {
                    Ok(stale) if !stale.is_empty() => {
                        app_log!(info, "[retention] Found {} stale tenant(s) to delete (inactive > {} days)", stale.len(), retention_days);
                        for tenant in &stale {
                            if let Some(email) = &tenant.email {
                                // Delete files
                                let dir = get_tenant_folder_path(email, &cleanup_data_dir);
                                if dir.exists() {
                                    if let Err(e) = FsOps::remove_dir_all(&dir).await {
                                        app_log!(error, "[retention] Failed to delete files for {}: {}", email, e);
                                    }
                                }
                                // Hard-delete DB record
                                if let Err(e) = repo.delete_by_email(email).await {
                                    app_log!(error, "[retention] Failed to delete DB record for {}: {}", email, e);
                                } else {
                                    app_log!(info, "[retention] Deleted account: {}", email);
                                }
                            }
                        }
                    }
                    Ok(_) => app_log!(info, "[retention] No stale accounts found."),
                    Err(e) => app_log!(error, "[retention] Cleanup query failed: {}", e),
                }
            }
        });
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
                import_cv_from_text,
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
                save_optimized_cv,
                translate_cv,
                generate_cover_letter,
                export_cover_letter,
                payment_intent,
                payment_confirm,
                payment_balance,
                payment_transactions,
                get_cv_data,
                put_cv_data,
                delete_me,
                get_my_referral_link,
                admin_credits,
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
