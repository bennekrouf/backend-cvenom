// src/web/handlers/feedback_handlers.rs
//
// POST /feedback — store user feedback and reward +10 credits if the reason
//                  contains at least 10 words.
// GET  /feedback/eligible — check if the user can submit feedback today.

use crate::auth::AuthenticatedUser;
use crate::core::database::DatabaseConfig;
use crate::web::handlers::payment_handlers::api0_topup_credits;
use crate::web::types::StandardErrorResponse;
use graflog::app_log;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

const CREDIT_REWARD: i64 = 10;
const MIN_WORDS: usize = 10;
const ADMIN_EMAIL: &str = "mohamed.bennekrouf@gmail.com";

// ── Request / response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SubmitFeedbackRequest {
    /// 1 = very dissatisfied … 5 = very satisfied
    pub score: i32,
    /// Free-text reason (max 500 chars enforced by frontend)
    pub reason: String,
    /// User consents to being contacted about their feedback
    pub contact_ok: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SubmitFeedbackResponse {
    pub success: bool,
    pub message: String,
    pub credits_granted: i64,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FeedbackEligibleResponse {
    pub eligible: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FeedbackRow {
    pub id: i64,
    pub email: String,
    pub score: i32,
    pub reason: String,
    pub contact_ok: bool,
    pub credits_granted: bool,
    pub created_at: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminFeedbackResponse {
    pub success: bool,
    pub feedbacks: Vec<FeedbackRow>,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

fn make_err(msg: &str, code: &str) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        msg.to_string(),
        code.to_string(),
        vec![],
        None,
    ))
}

// ── GET /feedback/eligible ───────────────────────────────────────────────────

pub async fn feedback_eligible_handler(
    auth: AuthenticatedUser,
    db_config: &DatabaseConfig,
) -> Result<Json<FeedbackEligibleResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;
    let email = auth.email().to_lowercase();

    let today_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feedback WHERE email = ? AND date(created_at) = date('now')",
    )
    .bind(&email)
    .fetch_one(pool)
    .await
    .map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;

    Ok(Json(FeedbackEligibleResponse {
        eligible: today_count.0 == 0,
    }))
}

// ── POST /feedback ───────────────────────────────────────────────────────────

pub async fn submit_feedback_handler(
    request: Json<SubmitFeedbackRequest>,
    auth: AuthenticatedUser,
    db_config: &DatabaseConfig,
) -> Result<Json<SubmitFeedbackResponse>, Json<StandardErrorResponse>> {
    let pool = db_config.pool().map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;
    let email = auth.email().to_lowercase();

    // Validate score
    if !(1..=5).contains(&request.score) {
        return Err(make_err("Score must be between 1 and 5", "INVALID_INPUT"));
    }

    // Rate limit: max 1 feedback per day per user
    let today_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feedback WHERE email = ? AND date(created_at) = date('now')",
    )
    .bind(&email)
    .fetch_one(pool)
    .await
    .map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;

    if today_count.0 > 0 {
        return Err(make_err(
            "You have already submitted feedback today",
            "FEEDBACK_ALREADY_SUBMITTED",
        ));
    }

    let reason = request.reason.chars().take(500).collect::<String>();
    let words = word_count(&reason);
    let qualifies = words >= MIN_WORDS;

    // Insert feedback
    sqlx::query(
        "INSERT INTO feedback (email, score, reason, contact_ok, credits_granted) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&email)
    .bind(request.score)
    .bind(&reason)
    .bind(request.contact_ok)
    .bind(qualifies)
    .execute(pool)
    .await
    .map_err(|e| make_err(&format!("Failed to save feedback: {e}"), "DB_ERROR"))?;

    app_log!(
        info,
        email = %email,
        score = request.score,
        words = words,
        qualifies = qualifies,
        "Feedback submitted"
    );

    // Grant credits if the feedback is substantive
    let credits_granted = if qualifies {
        match api0_topup_credits(&email, CREDIT_REWARD, "feedback_reward", Some("Thank you for your feedback!")).await {
            Ok(_) => {
                app_log!(info, email = %email, "Feedback reward: +{} credits", CREDIT_REWARD);
                CREDIT_REWARD
            }
            Err(e) => {
                app_log!(error, email = %email, error = %e, "Failed to grant feedback credits");
                0
            }
        }
    } else {
        0
    };

    let message = if credits_granted > 0 {
        format!("Thank you! You earned {} credits for your feedback.", credits_granted)
    } else if !qualifies {
        "Thank you for your feedback! Write at least 10 words to earn credits.".to_string()
    } else {
        "Thank you for your feedback!".to_string()
    };

    Ok(Json(SubmitFeedbackResponse {
        success: true,
        message,
        credits_granted,
    }))
}

// ── GET /admin/feedbacks ────────────────────────────────────────────────────

pub async fn admin_feedbacks_handler(
    auth: AuthenticatedUser,
    db_config: &DatabaseConfig,
) -> Result<Json<AdminFeedbackResponse>, Json<StandardErrorResponse>> {
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        return Err(make_err("Access denied", "FORBIDDEN"));
    }

    let pool = db_config.pool().map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;

    let rows: Vec<(i64, String, i32, String, bool, bool, String)> = sqlx::query_as(
        "SELECT id, email, score, reason, contact_ok, credits_granted, created_at \
         FROM feedback ORDER BY created_at DESC LIMIT 500",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| make_err(&format!("DB error: {e}"), "DB_ERROR"))?;

    let feedbacks = rows
        .into_iter()
        .map(|(id, email, score, reason, contact_ok, credits_granted, created_at)| FeedbackRow {
            id,
            email,
            score,
            reason,
            contact_ok,
            credits_granted,
            created_at,
        })
        .collect();

    Ok(Json(AdminFeedbackResponse {
        success: true,
        feedbacks,
    }))
}
