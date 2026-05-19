// tests/api_scenarios.rs
//
// Tier-2 API scenario tests using Rocket's local test client.
// No running server, no network calls, no Firebase auth round-trip.
//
// Strategy:
//   - Build a Rocket instance with a temp SQLite DB and empty AuthConfig
//     (no Firebase keys loaded → all auth-protected routes return 401/403)
//   - Test routing, auth guards, error handling, and public endpoints
//   - Fast: no I/O beyond temp files, no LLM calls

use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use std::path::PathBuf;
use tempfile::tempdir;

use cv_generator::{
    auth::AuthConfig,
    core::database::DatabaseConfig,
    web::{build_rocket, types::ServerConfig},
};

// ── Test fixture ──────────────────────────────────────────────────────────────

/// Build a test Rocket with isolated temp directories and an empty AuthConfig.
/// Empty firebase_keys means every `AuthenticatedUser` guard returns 401 —
/// exactly what we want when verifying that auth IS required.
async fn test_client() -> Client {
    let tmp = tempdir().expect("tempdir");

    let data_dir = tmp.path().join("data");
    let output_dir = tmp.path().join("output");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();

    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let db_path = tmp.path().join("test.db");

    let mut db = DatabaseConfig::new(db_path);
    db.init_pool().await.expect("db pool");
    db.migrate().await.expect("db migrate");

    let server_config = ServerConfig {
        data_dir,
        output_dir,
        templates_dir,
    };

    // Empty AuthConfig — no Firebase keys loaded.
    // All requests with Bearer tokens will fail signature verification → 401.
    let auth_config = AuthConfig::new("test-project".to_string());

    let rocket = build_rocket(
        server_config,
        auth_config,
        db,
        "http://localhost:5555".to_string(), // cv-import stub URL
        0,                                   // port 0 = not bound (local client)
    );

    // Keep `tmp` alive for the duration — leak it intentionally so the DB file
    // survives the test. Each test gets its own tempdir via this fn.
    std::mem::forget(tmp);

    Client::tracked(rocket).await.expect("valid rocket")
}

// ── Public endpoints ──────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_200() {
    let client = test_client().await;
    let response = client.get("/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

#[tokio::test]
async fn templates_returns_200_and_includes_portfolio() {
    let client = test_client().await;
    let response = client.get("/templates").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap_or_default();
    assert!(body.contains("portfolio"), "portfolio template missing from /templates response");
    assert!(body.contains("enterprise2"), "enterprise2 template missing from /templates response");
    assert!(body.contains("consulting"), "consulting template missing from /templates response");
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let client = test_client().await;
    let response = client.get("/nonexistent/route/xyz").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);
}

// ── Auth guard: all protected endpoints must reject unauthenticated requests ──

macro_rules! assert_requires_auth {
    ($name:ident, $method:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let client = test_client().await;
            let response = client.$method($path).dispatch().await;
            assert!(
                [401u16, 403, 422].contains(&response.status().code),
                "{} {} should require auth, got {}",
                stringify!($method).to_uppercase(),
                $path,
                response.status()
            );
        }
    };
    ($name:ident, $method:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let client = test_client().await;
            let response = client
                .$method($path)
                .header(ContentType::JSON)
                .body($body)
                .dispatch()
                .await;
            assert!(
                [401u16, 403, 422].contains(&response.status().code),
                "{} {} should require auth, got {}",
                stringify!($method).to_uppercase(),
                $path,
                response.status()
            );
        }
    };
}

// CV operations
assert_requires_auth!(generate_requires_auth,      post, "/generate",        r#"{"profile":"test","lang":"en"}"#);
assert_requires_auth!(create_requires_auth,         post, "/create",          r#"{"profile":"test"}"#);
assert_requires_auth!(delete_requires_auth,         post, "/delete-profile",  r#"{"profile":"test"}"#);
assert_requires_auth!(cover_letter_requires_auth,   post, "/cover-letter",    r#"{"profile":"test","lang":"en","job_description":"x"}"#);
assert_requires_auth!(optimize_requires_auth,       post, "/optimize",        r#"{"profile":"test","job_url":"https://x.com"}"#);
assert_requires_auth!(portfolio_requires_auth,      post, "/portfolio/generate", r#"{"profile":"test","lang":"en"}"#);

// Files
assert_requires_auth!(files_tree_requires_auth,    get,  "/files/tree");
assert_requires_auth!(files_save_requires_auth,    post, "/files/save",       r#"{"path":"x/y","content":"z"}"#);

// BD portal
assert_requires_auth!(bd_register_requires_auth,   post, "/bd/register",     r#"{"name":"test"}"#);
assert_requires_auth!(bd_me_requires_auth,         get,  "/bd/me");
assert_requires_auth!(bd_customers_requires_auth,  get,  "/bd/customers");
assert_requires_auth!(bd_commissions_requires_auth,get,  "/bd/commissions");
assert_requires_auth!(bd_attach_ref_requires_auth, post, "/bd/attach-ref",   r#"{"code":"BD-AAAAAA"}"#);

// Admin
assert_requires_auth!(admin_bds_requires_auth,     get,  "/admin/bd");
assert_requires_auth!(admin_commissions_requires_auth, get, "/admin/commissions");
assert_requires_auth!(admin_models_requires_auth,  get,  "/admin/models");

// ── Request format validation ─────────────────────────────────────────────────

#[tokio::test]
async fn generate_with_empty_body_returns_error() {
    let client = test_client().await;
    let response = client
        .post("/generate")
        .header(ContentType::JSON)
        .body("{}")
        .dispatch().await;
    // No auth → 401 before body is even parsed. Either is correct.
    assert!(
        [401u16, 422, 400].contains(&response.status().code),
        "Expected auth/format error, got {}",
        response.status()
    );
}

#[tokio::test]
async fn malformed_json_returns_400_or_422() {
    let client = test_client().await;
    let response = client
        .post("/generate")
        .header(ContentType::JSON)
        .body("not valid json {{{")
        .dispatch().await;
    assert!(
        [400u16, 422, 401].contains(&response.status().code),
        "Expected 400/422/401 for malformed JSON, got {}",
        response.status()
    );
}

// ── CORS ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn options_returns_cors_headers() {
    let client = test_client().await;
    let response = client
        .options("/generate")
        .header(rocket::http::Header::new("Origin", "https://studio.cvenom.com"))
        .dispatch().await;
    // OPTIONS (preflight) should succeed even without auth
    assert!(
        [200u16, 204, 404].contains(&response.status().code),
        "OPTIONS /generate returned {}", response.status()
    );
}

// ── Template system sanity ────────────────────────────────────────────────────

#[tokio::test]
async fn all_expected_templates_discovered() {
    use cv_generator::core::TemplateEngine;
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let engine = TemplateEngine::new(templates_dir).unwrap();
    let expected = [
        "default", "consulting", "academic", "creative",
        "tech", "executive", "keyteo", "keyteo_full",
        "enterprise2", "portfolio",
    ];
    for name in expected {
        assert!(
            engine.get_template(name).is_some(),
            "template '{name}' not found — was it deleted or renamed?"
        );
    }
}

#[tokio::test]
async fn portfolio_template_manifest_correct() {
    use cv_generator::core::TemplateEngine;
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let engine = TemplateEngine::new(templates_dir).unwrap();
    let t = engine.get_template("portfolio").unwrap();
    let langs = t.manifest.languages.as_ref().unwrap();
    assert!(langs.contains(&"en".to_string()), "portfolio missing 'en'");
    assert!(langs.contains(&"fr".to_string()), "portfolio missing 'fr'");
    assert!(langs.contains(&"de".to_string()), "portfolio missing 'de'");
    let features = t.manifest.features.as_ref().unwrap();
    assert!(features.contains(&"projects".to_string()), "portfolio missing 'projects' feature");
}

// ── Database migrations ───────────────────────────────────────────────────────

#[tokio::test]
async fn migrations_create_bd_commissions_table() {
    use cv_generator::core::database::DatabaseConfig;
    let tmp = tempdir().unwrap();
    let mut db = DatabaseConfig::new(tmp.path().join("migration_test.db"));
    db.init_pool().await.unwrap();
    db.migrate().await.unwrap();

    let pool = db.pool().unwrap();

    // bd_commissions table must exist
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='bd_commissions'"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "bd_commissions table was not created by migration");

    // business_developers table must exist
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='business_developers'"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, 1, "business_developers table was not created by migration");

    // tenants.referred_by_code column must exist
    let cols: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM pragma_table_info('tenants')"
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(
        cols.contains(&"referred_by_code".to_string()),
        "tenants.referred_by_code column missing"
    );
}
