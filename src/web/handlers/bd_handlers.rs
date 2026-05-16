// src/web/handlers/bd_handlers.rs — Business Developer portal endpoints

use graflog::app_log;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::core::database::DatabaseConfig;
use crate::web::types::StandardErrorResponse;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_error(msg: impl Into<String>, code: &str) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        msg.into(),
        code.to_string(),
        vec![],
        None,
    ))
}

fn pool_err(e: impl std::fmt::Display) -> Json<StandardErrorResponse> {
    make_error(format!("Database error: {}", e), "DB_ERROR")
}

fn generate_code() -> String {
    let suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(|c| (c as char).to_ascii_uppercase())
        .collect();
    format!("BD-{}", suffix)
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterBdRequest {
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct BdInfo {
    pub email: String,
    pub name: String,
    pub referral_code: String,
    pub commission_rate: f64,
    pub referral_url: String,
    pub customer_count: i64,
    pub estimated_revenue_usd: f64,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct BdResponse {
    pub success: bool,
    pub data: BdInfo,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CustomerRow {
    pub tenant_name: String,
    pub email: Option<String>,
    pub joined_at: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CustomersResponse {
    pub success: bool,
    pub customers: Vec<CustomerRow>,
}

// ── POST /bd/register ─────────────────────────────────────────────────────────

pub async fn register_bd_handler(
    body: Json<RegisterBdRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<BdResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(pool_err)?;
    let email = auth.email().to_string();

    // Idempotent: return existing record if already registered
    let existing: Option<(String, String, f64, String)> = sqlx::query_as(
        "SELECT referral_code, name, commission_rate, created_at \
         FROM business_developers WHERE email = ?",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| pool_err(e))?;

    let (referral_code, name, commission_rate, created_at) = if let Some(row) = existing {
        row
    } else {
        // Mint a unique code (retry on collision — extremely rare)
        let code = loop {
            let candidate = generate_code();
            let exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM business_developers WHERE referral_code = ?")
                    .bind(&candidate)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            if exists == 0 {
                break candidate;
            }
        };

        let display_name = if body.name.trim().is_empty() {
            email.split('@').next().unwrap_or("BD").to_string()
        } else {
            body.name.trim().to_string()
        };

        sqlx::query(
            "INSERT INTO business_developers (email, name, referral_code) VALUES (?, ?, ?)",
        )
        .bind(&email)
        .bind(&display_name)
        .bind(&code)
        .execute(pool)
        .await
        .map_err(|e| pool_err(e))?;

        app_log!(info, bd_email = %email, code = %code, "Business developer registered");

        let row: (String, String, f64, String) = sqlx::query_as(
            "SELECT referral_code, name, commission_rate, created_at \
             FROM business_developers WHERE email = ?",
        )
        .bind(&email)
        .fetch_one(pool)
        .await
        .map_err(|e| pool_err(e))?;

        row
    };

    let customer_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE referred_by_code = ?")
            .bind(&referral_code)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    Ok(Json(BdResponse {
        success: true,
        data: build_bd_info(
            email,
            name,
            referral_code,
            commission_rate,
            created_at,
            customer_count,
        ),
    }))
}

// ── GET /bd/me ────────────────────────────────────────────────────────────────

pub async fn get_bd_me_handler(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<BdResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(pool_err)?;
    let email = auth.email().to_string();

    let row: Option<(String, String, f64, String)> = sqlx::query_as(
        "SELECT referral_code, name, commission_rate, created_at \
         FROM business_developers WHERE email = ?",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|e| pool_err(e))?;

    let (referral_code, name, commission_rate, created_at) =
        row.ok_or_else(|| make_error("Not registered as a business developer", "BD_NOT_FOUND"))?;

    let customer_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE referred_by_code = ?")
            .bind(&referral_code)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    Ok(Json(BdResponse {
        success: true,
        data: build_bd_info(
            email,
            name,
            referral_code,
            commission_rate,
            created_at,
            customer_count,
        ),
    }))
}

// ── GET /bd/customers ─────────────────────────────────────────────────────────

pub async fn get_bd_customers_handler(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CustomersResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(pool_err)?;
    let email = auth.email();

    let code: Option<String> =
        sqlx::query_scalar("SELECT referral_code FROM business_developers WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
            .map_err(|e| pool_err(e))?;

    let referral_code =
        code.ok_or_else(|| make_error("Not registered as a business developer", "BD_NOT_FOUND"))?;

    let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT tenant_name, email, created_at \
         FROM tenants WHERE referred_by_code = ? ORDER BY created_at DESC",
    )
    .bind(&referral_code)
    .fetch_all(pool)
    .await
    .map_err(|e| pool_err(e))?;

    let customers = rows
        .into_iter()
        .map(|(tenant_name, email, joined_at)| CustomerRow {
            tenant_name,
            email,
            joined_at,
        })
        .collect();

    Ok(Json(CustomersResponse {
        success: true,
        customers,
    }))
}

// ── POST /bd/attach-ref ───────────────────────────────────────────────────────
// Called after login when ?ref=CODE was present in the URL.

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AttachRefRequest {
    pub code: String,
}

pub async fn attach_ref_handler(
    body: Json<AttachRefRequest>,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(pool_err)?;
    let tenant = auth.tenant();

    // Verify the code exists
    let code_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM business_developers WHERE referral_code = ?")
            .bind(&body.code)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    if code_exists == 0 {
        return Err(make_error("Invalid referral code", "BD_CODE_INVALID"));
    }

    // Only set if not already attributed
    sqlx::query(
        "UPDATE tenants SET referred_by_code = ? \
         WHERE id = ? AND referred_by_code IS NULL",
    )
    .bind(&body.code)
    .bind(tenant.id)
    .execute(pool)
    .await
    .map_err(|e| pool_err(e))?;

    app_log!(
        info,
        tenant_id = %tenant.id,
        code = %body.code,
        "Tenant attributed to BD code"
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

// ── Admin endpoints ───────────────────────────────────────────────────────────

const ADMIN_EMAIL: &str = "mohamed.bennekrouf@gmail.com";

fn admin_only(auth: &AuthenticatedUser) -> Result<(), Json<StandardErrorResponse>> {
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        Err(make_error("Admin access required", "FORBIDDEN"))
    } else {
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminBdRow {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub referral_code: String,
    pub commission_rate: f64,
    pub customer_count: i64,
    pub estimated_revenue_usd: f64,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminBdListResponse {
    pub success: bool,
    pub total_bds: usize,
    pub total_customers: i64,
    pub total_estimated_revenue_usd: f64,
    pub business_developers: Vec<AdminBdRow>,
}

/// GET /admin/bd — list all BDs with their stats (admin only)
pub async fn admin_list_bd_handler(
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<AdminBdListResponse>, Json<StandardErrorResponse>> {
    admin_only(&auth)?;
    let pool = db_config.pool().map_err(pool_err)?;

    let rows: Vec<(i64, String, String, String, f64, String)> = sqlx::query_as(
        "SELECT id, email, name, referral_code, commission_rate, created_at \
         FROM business_developers ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| pool_err(e))?;

    let mut bds: Vec<AdminBdRow> = Vec::with_capacity(rows.len());
    for (id, email, name, referral_code, commission_rate, created_at) in rows {
        let customer_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE referred_by_code = ?")
                .bind(&referral_code)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        let info = build_bd_info(
            email.clone(),
            name.clone(),
            referral_code.clone(),
            commission_rate,
            created_at.clone(),
            customer_count,
        );

        bds.push(AdminBdRow {
            id,
            email,
            name,
            referral_code,
            commission_rate,
            customer_count,
            estimated_revenue_usd: info.estimated_revenue_usd,
            created_at,
        });
    }

    let total_customers: i64 = bds.iter().map(|b| b.customer_count).sum();
    let total_revenue: f64 = bds.iter().map(|b| b.estimated_revenue_usd).sum();

    Ok(Json(AdminBdListResponse {
        success: true,
        total_bds: bds.len(),
        total_customers,
        total_estimated_revenue_usd: total_revenue,
        business_developers: bds,
    }))
}

/// GET /admin/bd/:code/customers — customers of a specific BD (admin only)
pub async fn admin_bd_customers_handler(
    code: String,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<CustomersResponse>, Json<StandardErrorResponse>> {
    admin_only(&auth)?;
    let pool = db_config.pool().map_err(pool_err)?;

    let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT tenant_name, email, created_at \
         FROM tenants WHERE referred_by_code = ? ORDER BY created_at DESC",
    )
    .bind(&code)
    .fetch_all(pool)
    .await
    .map_err(|e| pool_err(e))?;

    let customers = rows
        .into_iter()
        .map(|(tenant_name, email, joined_at)| CustomerRow { tenant_name, email, joined_at })
        .collect();

    Ok(Json(CustomersResponse { success: true, customers }))
}

/// DELETE /admin/bd/:email — remove a BD registration (admin only)
pub async fn admin_delete_bd_handler(
    email: String,
    auth: AuthenticatedUser,
    db_config: &State<DatabaseConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    admin_only(&auth)?;
    let pool = db_config.pool().map_err(pool_err)?;

    let rows = sqlx::query("DELETE FROM business_developers WHERE email = ?")
        .bind(&email)
        .execute(pool)
        .await
        .map_err(|e| pool_err(e))?
        .rows_affected();

    if rows == 0 {
        return Err(make_error("BD not found", "BD_NOT_FOUND"));
    }

    app_log!(info, admin = %auth.email(), deleted_bd = %email, "BD registration removed by admin");
    Ok(Json(serde_json::json!({ "success": true })))
}

// ── Private helper ────────────────────────────────────────────────────────────

fn build_bd_info(
    email: String,
    name: String,
    referral_code: String,
    commission_rate: f64,
    created_at: String,
    customer_count: i64,
) -> BdInfo {
    // Estimated revenue: each customer generates ~20 credits/month on average.
    // 1 credit = $0.25. BD commission = commission_rate.
    let credits_per_customer_month = 20.0_f64;
    let usd_per_credit = 0.25_f64;
    let estimated_revenue_usd =
        customer_count as f64 * credits_per_customer_month * usd_per_credit * commission_rate;

    let referral_url = format!("https://app.cvenom.com?ref={}", referral_code);

    BdInfo {
        email,
        name,
        referral_code,
        commission_rate,
        referral_url,
        customer_count,
        estimated_revenue_usd,
        created_at,
    }
}
