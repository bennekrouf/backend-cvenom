// src/web/handlers/referral_handlers.rs
//
// Referral system for cvenom.
//
// Flow:
//   - Each user's referral code is their tenant_name.
//   - Referral URL: https://cvenom.com?ref=<tenant_name>
//   - The frontend captures ?ref= in localStorage and sends X-Referral-Code header on signup.
//   - auth.rs calls credit_referral() for brand-new users when the header is present.
//   - Referrer earns 50 credits; referred user earns 25 credits.

use graflog::app_log;
use rocket::serde::json::Json;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::auth::AuthenticatedUser;
use crate::web::types::StandardErrorResponse;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReferralLinkData {
    pub referral_code: String,
    pub referral_url: String,
    pub total_referrals: i64,
    pub credited_referrals: i64,
    pub credits_earned: i64,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReferralLinkResponse {
    pub success: bool,
    pub data: ReferralLinkData,
}

// ── GET /referral/my-link ─────────────────────────────────────────────────────

/// Return the authenticated user's referral link and stats.
pub async fn get_referral_link_handler(
    auth: AuthenticatedUser,
    db_config: &rocket::State<crate::core::database::DatabaseConfig>,
) -> Result<Json<ReferralLinkResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Database error: {}", e),
            "DB_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    let email = auth.email();
    let referral_code = auth.tenant_name().to_string();
    let referral_url = format!("https://cvenom.com?ref={}", referral_code);

    let total_referrals: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM referrals WHERE referrer_email = ?",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let credited_referrals: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM referrals WHERE referrer_email = ? AND status = 'credited'",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let credits_earned = credited_referrals * 50;

    app_log!(
        info,
        user = %email,
        referral_code = %referral_code,
        total = total_referrals,
        credited = credited_referrals,
        "Referral link requested"
    );

    Ok(Json(ReferralLinkResponse {
        success: true,
        data: ReferralLinkData {
            referral_code,
            referral_url,
            total_referrals,
            credited_referrals,
            credits_earned,
        },
    }))
}

// ── Internal: credit_referral ─────────────────────────────────────────────────

/// Called from auth.rs after a new user is created.
/// Looks up referrer by tenant_name = ref_code, inserts referral row,
/// credits both parties, and marks the row as 'credited'.
pub async fn credit_referral(
    referred_email: String,
    ref_code: String,
    pool: SqlitePool,
    store_url: String,
    secret: String,
) {
    // 1. Look up referrer by tenant_name
    let referrer_email: Option<String> = sqlx::query_scalar(
        "SELECT email FROM tenants WHERE tenant_name = ? AND email IS NOT NULL AND is_active = TRUE LIMIT 1",
    )
    .bind(&ref_code)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let referrer_email = match referrer_email {
        Some(e) => e,
        None => {
            app_log!(
                warn,
                ref_code = %ref_code,
                referred = %referred_email,
                "Referral: no active tenant found for ref_code"
            );
            return;
        }
    };

    // Prevent self-referral
    if referrer_email == referred_email {
        app_log!(
            warn,
            email = %referred_email,
            "Referral: self-referral attempt ignored"
        );
        return;
    }

    // 2. Insert referral row (IGNORE if referred_email already exists — UNIQUE constraint)
    let insert_result = sqlx::query(
        "INSERT OR IGNORE INTO referrals (referrer_email, referred_email, status) VALUES (?, ?, 'pending')",
    )
    .bind(&referrer_email)
    .bind(&referred_email)
    .execute(&pool)
    .await;

    let rows_affected = match insert_result {
        Ok(r) => r.rows_affected(),
        Err(e) => {
            app_log!(error, error = %e, referred = %referred_email, "Referral: DB insert failed");
            return;
        }
    };

    if rows_affected == 0 {
        // Already processed — idempotent, do nothing
        app_log!(
            info,
            referred = %referred_email,
            "Referral: row already exists, skipping credit"
        );
        return;
    }

    let client = reqwest::Client::new();

    // 3. Credit referrer +50
    let referrer_body = serde_json::json!({
        "email": referrer_email,
        "amount": 50_i64,
        "action_type": "referral_bonus",
    });
    match client
        .post(format!("{}/api/user/credits", store_url))
        .header("Content-Type", "application/json")
        .header("X-Internal-Secret", &secret)
        .json(&referrer_body)
        .send()
        .await
    {
        Ok(_) => app_log!(
            info,
            referrer = %referrer_email,
            "Referral: granted 50 credits to referrer"
        ),
        Err(e) => app_log!(
            error,
            referrer = %referrer_email,
            error = %e,
            "Referral: failed to credit referrer"
        ),
    }

    // 4. Credit referred user +25
    let referred_body = serde_json::json!({
        "email": referred_email,
        "amount": 25_i64,
        "action_type": "referral_welcome_bonus",
    });
    match client
        .post(format!("{}/api/user/credits", store_url))
        .header("Content-Type", "application/json")
        .header("X-Internal-Secret", &secret)
        .json(&referred_body)
        .send()
        .await
    {
        Ok(_) => app_log!(
            info,
            referred = %referred_email,
            "Referral: granted 25 credits to referred user"
        ),
        Err(e) => app_log!(
            error,
            referred = %referred_email,
            error = %e,
            "Referral: failed to credit referred user"
        ),
    }

    // 5. Mark referral as credited
    let _ = sqlx::query(
        "UPDATE referrals SET status = 'credited', credited_at = datetime('now') WHERE referred_email = ?",
    )
    .bind(&referred_email)
    .execute(&pool)
    .await;

    app_log!(
        info,
        referrer = %referrer_email,
        referred = %referred_email,
        "Referral: credited successfully"
    );
}
