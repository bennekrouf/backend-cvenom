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
    generate_portfolio_handler,
};
use crate::web::handlers::cv_handlers::GeneratePortfolioRequest;
use crate::web::handlers::cv_handlers::ImportTextRequest;
use crate::web::handlers::cv_handlers::CoverLetterExportRequest;
use crate::core::database::{get_tenant_folder_path, TenantRepository};
use crate::core::FsOps;
use crate::web::handlers::cv_data::CvFormData;
use crate::web::handlers::payment_handlers::{
    ConfirmPaymentRequest, CreateIntentRequest, GetBalanceResponse, TransactionsResponse,
    get_transactions_handler, AdminCreditRequest, admin_add_credits_handler,
    AdminCreditUsersResponse, AdminUserTransactionsResponse,
    admin_credit_users_handler, admin_user_transactions_handler,
};
use crate::web::handlers::referral_handlers::{get_referral_link_handler, ReferralLinkResponse};
use crate::web::handlers::feedback_handlers::{
    feedback_eligible_handler, submit_feedback_handler, admin_feedbacks_handler,
    SubmitFeedbackRequest, SubmitFeedbackResponse, FeedbackEligibleResponse,
    AdminFeedbackResponse,
};
use crate::web::handlers::model_handlers::{
    get_model_config_handler, update_model_config_handler,
    ModelConfigResponse, UpdateModelConfigResponse, UpdateModelConfigRequest,
};
use crate::web::handlers::bd_handlers::{
    register_bd_handler, get_bd_me_handler, get_bd_customers_handler, attach_ref_handler,
    get_bd_commissions_handler, admin_list_bd_handler, admin_bd_customers_handler,
    admin_delete_bd_handler, admin_list_commissions_handler, admin_mark_paid_handler,
    RegisterBdRequest, AttachRefRequest, MarkPaidRequest,
    BdResponse, CustomersResponse, AdminBdListResponse,
    BdCommissionsResponse, AdminCommissionsResponse, MarkPaidResponse,
};
use anyhow::Result;
use graflog::app_log;

use rocket::config::{Config, LogLevel};
use rocket::data::ByteUnit;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{catchers, delete, get, post, put, routes, Request, Response, State};
use rocket::fs::NamedFile;
use std::path::PathBuf;
pub use types::*;
mod cors_utils;
use cors_utils::universal_options_handler;

use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AnnounceTemplateRequest {
    pub template_name: String,
}

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
            "https://app.api0.ai",
            "http://localhost:4001",
            "http://localhost:3000",
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

#[get("/outputs/<file..>")]
pub async fn get_output_file(file: PathBuf, config: &State<ServerConfig>) -> Option<NamedFile> {
    NamedFile::open(config.output_dir.join(file)).await.ok()
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

#[rocket::put("/profiles/<profile_name>/change-language", data = "<request>")]
pub async fn change_profile_language_handler(
    profile_name: String,
    request: Json<StandardRequest<crate::web::types::ChangeLanguageRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    handlers::change_profile_language_handler(profile_name, request, auth, config).await
}

#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<StandardRequest<GenerateRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<GeneratePdfResponse>, Json<StandardErrorResponse>> {
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

#[get("/preferences")]
pub async fn get_preferences(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(|e| {
        Json(StandardErrorResponse::new(format!("DB error: {e}"), "INTERNAL_ERROR".into(), vec![], None))
    })?;
    let repo = crate::core::database::TenantRepository::new(pool);
    let prefs_json = repo.get_email_prefs(&auth.user().email).await.map_err(|e| {
        Json(StandardErrorResponse::new(format!("Failed to load preferences: {e}"), "PREFS_ERROR".into(), vec![], None))
    })?;
    let prefs: serde_json::Value = serde_json::from_str(&prefs_json).unwrap_or_default();
    let lang = auth.lang().to_string();
    Ok(Json(serde_json::json!({ "email_prefs": prefs, "preferred_lang": lang })))
}

#[put("/preferences", data = "<body>")]
pub async fn update_preferences(
    body: Json<serde_json::Value>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(|e| {
        Json(StandardErrorResponse::new(format!("DB error: {e}"), "INTERNAL_ERROR".into(), vec![], None))
    })?;
    let repo = crate::core::database::TenantRepository::new(pool);

    if let Some(email_prefs) = body.get("email_prefs") {
        let json_str = serde_json::to_string(email_prefs).unwrap_or_else(|_| "{}".into());
        repo.update_email_prefs(&auth.user().email, &json_str).await.map_err(|e| {
            Json(StandardErrorResponse::new(format!("Failed to save preferences: {e}"), "PREFS_ERROR".into(), vec![], None))
        })?;
    }
    if let Some(lang) = body.get("preferred_lang").and_then(|v| v.as_str()) {
        repo.update_preferred_lang(&auth.user().email, lang).await.map_err(|e| {
            Json(StandardErrorResponse::new(format!("Failed to save language: {e}"), "PREFS_ERROR".into(), vec![], None))
        })?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
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
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<OptimizeResponse>>, Json<StandardErrorResponse>> {
    optimize_cv_handler(request, auth, config, db_config, cv_service_url).await
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
) -> Result<Json<GeneratePdfResponse>, Json<StandardErrorResponse>> {
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
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<TranslateResponse>>, Json<StandardErrorResponse>> {
    translate_cv_handler(request, auth, config, db_config, cv_service_url).await
}

/// POST /cover-letter — generate a cover letter from CV data + job description.
/// Costs 20 credits (same as CV generation).
#[post("/cover-letter", data = "<request>")]
pub async fn generate_cover_letter(
    request: Json<StandardRequest<CoverLetterRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<DataResponse<CoverLetterResult>>, Json<StandardErrorResponse>> {
    cover_letter_handler(request, auth, config, db_config, cv_service_url).await
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

/// POST /payment/confirm — verify Stripe payment + top-up api0 credits + record BD commission
#[post("/payment/confirm", data = "<request>")]
pub async fn payment_confirm(
    request: Json<ConfirmPaymentRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<crate::web::handlers::payment_handlers::ConfirmPaymentResponse>, Json<StandardErrorResponse>> {
    crate::web::handlers::payment_handlers::confirm_payment_handler(request, auth, db_config).await
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

/// POST /admin/templates/announce — broadcast a "new template" email to all active users (admin only).
/// Body: { "template_name": "Modern Minimal" }
#[post("/admin/templates/announce", data = "<body>")]
pub async fn admin_announce_template(
    body: Json<AnnounceTemplateRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    const ADMIN_EMAIL: &str = "mohamed.bennekrouf@gmail.com";
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        return Err(Json(crate::web::types::StandardErrorResponse::new(
            "Unauthorized".to_string(),
            "UNAUTHORIZED".to_string(),
            vec![],
            None,
        )));
    }

    let pool = db_config.pool().map_err(|e| {
        Json(crate::web::types::StandardErrorResponse::new(
            format!("DB error: {e}"),
            "INTERNAL_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    let repo = TenantRepository::new(pool);
    let tenants = repo.list_active_email_tenants().await.map_err(|e| {
        Json(crate::web::types::StandardErrorResponse::new(
            format!("DB query failed: {e}"),
            "INTERNAL_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    let count = tenants.len();
    let template_name = body.template_name.clone();
    for (_id, email, _name) in tenants {
        crate::email::send_email(
            &email,
            crate::email::EmailKind::NewTemplate { template_name: template_name.clone() },
            "en",
        );
    }

    app_log!(info, "[admin] New-template broadcast sent to {} users: {}", count, template_name);
    Ok(Json(serde_json::json!({ "success": true, "sent_to": count, "template_name": template_name })))
}


// ── Business Developer routes ─────────────────────────────────────────────────

/// POST /bd/register — register as a BD (idempotent)
#[post("/bd/register", data = "<body>")]
pub async fn bd_register(
    body: Json<RegisterBdRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<BdResponse>, Json<StandardErrorResponse>> {
    register_bd_handler(body, auth, db_config).await
}

/// GET /bd/me — return BD profile + customer count + estimated revenue
#[get("/bd/me")]
pub async fn bd_me(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<BdResponse>, Json<StandardErrorResponse>> {
    get_bd_me_handler(auth, db_config).await
}

/// GET /bd/customers — list customers referred by this BD
#[get("/bd/customers")]
pub async fn bd_customers(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CustomersResponse>, Json<StandardErrorResponse>> {
    get_bd_customers_handler(auth, db_config).await
}

/// POST /bd/attach-ref — link the current tenant to a BD referral code
#[post("/bd/attach-ref", data = "<body>")]
pub async fn bd_attach_ref(
    body: Json<AttachRefRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    attach_ref_handler(body, auth, db_config).await
}

// ── Admin BD routes ───────────────────────────────────────────────────────────

/// GET /admin/bd — list all business developers with stats (admin only)
#[get("/admin/bd")]
pub async fn admin_list_bds(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<AdminBdListResponse>, Json<StandardErrorResponse>> {
    admin_list_bd_handler(auth, db_config).await
}

/// GET /admin/bd/<code>/customers — customers of one BD (admin only)
#[get("/admin/bd/<code>/customers")]
pub async fn admin_bd_customers(
    code: String,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CustomersResponse>, Json<StandardErrorResponse>> {
    admin_bd_customers_handler(code, auth, db_config).await
}

/// GET /bd/commissions — BD's own commission history (pending + paid)
#[get("/bd/commissions")]
pub async fn bd_commissions(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<BdCommissionsResponse>, Json<StandardErrorResponse>> {
    get_bd_commissions_handler(auth, db_config).await
}

/// GET /admin/commissions — all BDs with their pending/paid commission totals (admin only)
#[get("/admin/commissions")]
pub async fn admin_commissions(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<AdminCommissionsResponse>, Json<StandardErrorResponse>> {
    admin_list_commissions_handler(auth, db_config).await
}

/// POST /admin/commissions/pay — mark all pending commissions for a BD as paid (admin only)
#[post("/admin/commissions/pay", data = "<body>")]
pub async fn admin_commissions_pay(
    body: Json<MarkPaidRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<MarkPaidResponse>, Json<StandardErrorResponse>> {
    admin_mark_paid_handler(body, auth, db_config).await
}

/// DELETE /admin/bd/<email> — remove a BD (admin only)
#[delete("/admin/bd/<email>")]
pub async fn admin_delete_bd(
    email: String,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    admin_delete_bd_handler(email, auth, db_config).await
}

// ── Referral routes ───────────────────────────────────────────────────────────

/// GET /admin/models — read cv-import model config (admin only)
#[get("/admin/models")]
pub async fn admin_get_models(
    auth: AuthenticatedUser,
) -> Result<Json<ModelConfigResponse>, Json<StandardErrorResponse>> {
    get_model_config_handler(auth).await
}

/// POST /admin/models — update cv-import model config and restart (admin only)
#[post("/admin/models", data = "<body>")]
pub async fn admin_update_models(
    body: Json<UpdateModelConfigRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<UpdateModelConfigResponse>, Json<StandardErrorResponse>> {
    update_model_config_handler(body, auth).await
}

/// GET /admin/credits/users — all tenants with their api0 credit balances (admin only)
#[get("/admin/credits/users")]
pub async fn admin_credit_users(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<AdminCreditUsersResponse>, Json<StandardErrorResponse>> {
    admin_credit_users_handler(auth, db_config).await
}

/// GET /admin/credits/transactions/<email> — transaction log for one user (admin only)
#[get("/admin/credits/transactions/<email>")]
pub async fn admin_credit_user_transactions(
    email: String,
    auth: AuthenticatedUser,
) -> Result<Json<AdminUserTransactionsResponse>, Json<StandardErrorResponse>> {
    admin_user_transactions_handler(email, auth).await
}

/// POST /portfolio/generate — AI generates [[projects]] then compiles portfolio PDF
#[post("/portfolio/generate", data = "<request>")]
pub async fn generate_portfolio(
    request: Json<StandardRequest<GeneratePortfolioRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
    db_config: &State<DatabaseConfig>,
    cv_service_url: &State<String>,
) -> Result<Json<GeneratePdfResponse>, Json<StandardErrorResponse>> {
    generate_portfolio_handler(request, auth, config, db_config, cv_service_url).await
}

/// GET /referral/my-link — return the authenticated user's referral link and stats
#[get("/referral/my-link")]
pub async fn get_my_referral_link(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<ReferralLinkResponse>, Json<StandardErrorResponse>> {
    get_referral_link_handler(auth, db_config).await
}

/// GET /feedback/eligible — check if user can submit feedback today
#[get("/feedback/eligible")]
pub async fn feedback_eligible(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<FeedbackEligibleResponse>, Json<StandardErrorResponse>> {
    feedback_eligible_handler(auth, db_config).await
}

/// POST /feedback — submit feedback and optionally earn credits
#[post("/feedback", data = "<request>")]
pub async fn submit_feedback(
    request: Json<SubmitFeedbackRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<SubmitFeedbackResponse>, Json<StandardErrorResponse>> {
    submit_feedback_handler(request, auth, db_config).await
}

/// GET /admin/feedbacks — list all feedback (admin only)
#[get("/admin/feedbacks")]
pub async fn admin_feedbacks(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<AdminFeedbackResponse>, Json<StandardErrorResponse>> {
    admin_feedbacks_handler(auth, db_config).await
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

    // ── Tier-3 engagement email background task ───────────────────────────────
    // Runs once per day. Sends nudge emails (7 days, no CV) and win-back emails (30 days inactive).
    if let Ok(engage_pool) = db_config.pool().map(|p| p.clone()) {
        tokio::spawn(async move {
            // 10-minute initial delay so the server is fully up before the first run.
            tokio::time::sleep(std::time::Duration::from_secs(600)).await;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
            loop {
                interval.tick().await;
                let repo = TenantRepository::new(&engage_pool);

                // Nudge: signed up > 7 days ago, never generated a CV
                match repo.find_nudge_candidates().await {
                    Ok(candidates) => {
                        app_log!(info, "[engagement] Nudge candidates: {}", candidates.len());
                        for (_id, email, name) in candidates {
                            let credits = match crate::web::handlers::payment_handlers::api0_get_balance(&email).await {
                                Ok(b) => b,
                                Err(e) => {
                                    app_log!(warn, "[engagement] balance fetch failed for {}: {}", email, e);
                                    0
                                }
                            };
                            app_log!(info, "[engagement] Nudge {} credits={}", email, credits);
                            crate::email::send_email(
                                &email,
                                crate::email::EmailKind::Nudge { name, credits },
                                "en",
                            );
                            if let Err(e) = repo.mark_nudge_sent(&email).await {
                                app_log!(error, "[engagement] mark_nudge_sent failed for {}: {}", email, e);
                            }
                        }
                    }
                    Err(e) => app_log!(error, "[engagement] find_nudge_candidates failed: {}", e),
                }

                // Win-back: inactive for > 30 days, not yet emailed
                match repo.find_winback_candidates().await {
                    Ok(candidates) => {
                        app_log!(info, "[engagement] Win-back candidates: {}", candidates.len());
                        for (_id, email, name) in candidates {
                            crate::email::send_email(
                                &email,
                                crate::email::EmailKind::WinBack { name },
                                "en",
                            );
                            if let Err(e) = repo.mark_winback_sent(&email).await {
                                app_log!(error, "[engagement] mark_winback_sent failed for {}: {}", email, e);
                            }
                        }
                    }
                    Err(e) => app_log!(error, "[engagement] find_winback_candidates failed: {}", e),
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
        limits: rocket::data::Limits::default()
            .limit("file", ByteUnit::Megabyte(10))
            .limit("data-form", ByteUnit::Megabyte(10))
            .limit("form", ByteUnit::Megabyte(10)),
        ..Config::default()
    };

    let _rocket = build_rocket(server_config, auth_config, db_config, cv_service_url, port)
        .launch()
        .await;

    app_log!(info, "Server shutting down");
    Ok(())
}

/// Build the Rocket instance from already-initialised state.
/// Called by `start_web_server` in production and by tests with mocked state.
pub fn build_rocket(
    server_config: ServerConfig,
    auth_config: AuthConfig,
    db_config: DatabaseConfig,
    cv_service_url: String,
    port: u16,
) -> rocket::Rocket<rocket::Build> {
    let config = Config {
        port,
        log_level: LogLevel::Off,
        limits: rocket::data::Limits::default()
            .limit("file", ByteUnit::Megabyte(10))
            .limit("data-form", ByteUnit::Megabyte(10))
            .limit("form", ByteUnit::Megabyte(10)),
        ..Config::default()
    };

    rocket::custom(config)
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
                change_profile_language_handler,
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
                generate_portfolio,
                get_my_referral_link,
                bd_register,
                bd_me,
                bd_customers,
                bd_attach_ref,
                bd_commissions,
                admin_commissions,
                admin_commissions_pay,
                admin_get_models,
                admin_update_models,
                admin_list_bds,
                admin_bd_customers,
                admin_delete_bd,
                admin_credits,
                admin_credit_users,
                admin_credit_user_transactions,
                admin_announce_template,
                feedback_eligible,
                submit_feedback,
                admin_feedbacks,
                get_output_file,
                get_preferences,
                update_preferences,
            ],
        )
}

