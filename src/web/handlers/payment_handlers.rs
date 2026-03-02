// src/web/handlers/payment_handlers.rs
//
// Stripe payment handlers for cvenom.
//
// Flow:
//   1. Frontend calls POST /payment/intent { amount_dollars } → backend creates a Stripe
//      PaymentIntent and returns the client_secret to the browser.
//   2. Browser confirms the payment with Stripe.js (no redirect).
//   3. Frontend calls POST /payment/confirm { payment_intent_id } → backend verifies the
//      PaymentIntent with Stripe, then calls api0 Store to top-up cvenom's tenant credit
//      balance: credits_added = amount_dollars * 100.
//
// Environment variables (read at call time so they can be hot-reloaded in dev):
//   STRIPE_SECRET_KEY       – cvenom's own Stripe secret key (sk_live_… / sk_test_…)
//   STRIPE_PUBLISHABLE_KEY  – cvenom's own Stripe publishable key (pk_live_… / pk_test_…)
//   API0_STORE_URL          – base URL of the api0 Store service  (e.g. http://localhost:5007)
//   API0_ACCOUNT_EMAIL      – the email address of cvenom's api0 account (tenant email)
//   API0_INTERNAL_SECRET    – a shared secret accepted by api0 Store for internal credit top-up

use graflog::app_log;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::web::types::StandardErrorResponse;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateIntentRequest {
    /// Amount in whole dollars (minimum 1).
    pub amount_dollars: u32,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateIntentResponse {
    pub success: bool,
    pub client_secret: String,
    pub publishable_key: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ConfirmPaymentRequest {
    pub payment_intent_id: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ConfirmPaymentResponse {
    pub success: bool,
    pub message: String,
    pub credits_added: i64,
    pub new_balance: i64,
}

// ── Stripe API helpers ────────────────────────────────────────────────────────

/// Retrieve the Stripe secret key from the environment.
fn stripe_secret_key() -> Result<String, String> {
    std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| "STRIPE_SECRET_KEY environment variable not set".to_string())
}

/// Retrieve the Stripe publishable key from the environment.
fn stripe_publishable_key() -> Result<String, String> {
    std::env::var("STRIPE_PUBLISHABLE_KEY")
        .map_err(|_| "STRIPE_PUBLISHABLE_KEY environment variable not set".to_string())
}

/// Call Stripe API to create a PaymentIntent.
/// Returns the PaymentIntent ID and client_secret.
async fn stripe_create_payment_intent(
    secret_key: &str,
    amount_cents: u32,
    user_email: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();

    let params = [
        ("amount", amount_cents.to_string()),
        ("currency", "usd".to_string()),
        ("receipt_email", user_email.to_string()),
        // automatic_payment_methods = true lets Stripe.js render the right payment form
        ("automatic_payment_methods[enabled]", "true".to_string()),
    ];

    let res = client
        .post("https://api.stripe.com/v1/payment_intents")
        .basic_auth(secret_key, Some(""))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Stripe request failed: {e}"))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Stripe error: {body}"));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Stripe JSON parse error: {e}"))?;

    let id = json["id"]
        .as_str()
        .ok_or("Stripe response missing 'id'")?
        .to_string();

    let client_secret = json["client_secret"]
        .as_str()
        .ok_or("Stripe response missing 'client_secret'")?
        .to_string();

    Ok((id, client_secret))
}

/// Call Stripe API to retrieve a PaymentIntent and verify it has succeeded.
/// Returns the amount in cents on success.
async fn stripe_verify_payment_intent(
    secret_key: &str,
    payment_intent_id: &str,
) -> Result<u32, String> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://api.stripe.com/v1/payment_intents/{}",
        payment_intent_id
    );

    let res = client
        .get(&url)
        .basic_auth(secret_key, Some(""))
        .send()
        .await
        .map_err(|e| format!("Stripe request failed: {e}"))?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Stripe error: {body}"));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Stripe JSON parse error: {e}"))?;

    let status = json["status"].as_str().unwrap_or("unknown");
    if status != "succeeded" {
        return Err(format!(
            "PaymentIntent status is '{status}', expected 'succeeded'"
        ));
    }

    let amount_cents = json["amount"]
        .as_u64()
        .ok_or("Stripe response missing 'amount'")?;

    Ok(amount_cents as u32)
}

// ── api0 Store helper ─────────────────────────────────────────────────────────

/// Top up cvenom's credit balance in api0 Store.
///
/// Calls: POST {API0_STORE_URL}/api/user/credits
/// Body:  { "email": "<cvenom api0 account email>", "amount": <credits> }
///
/// Returns the new balance reported by the store, or an error string.
async fn api0_topup_credits(credits_to_add: i64) -> Result<i64, String> {
    let store_url = std::env::var("API0_STORE_URL")
        .map_err(|_| "API0_STORE_URL environment variable not set".to_string())?;

    let account_email = std::env::var("API0_ACCOUNT_EMAIL")
        .map_err(|_| "API0_ACCOUNT_EMAIL environment variable not set".to_string())?;

    let internal_secret = std::env::var("API0_INTERNAL_SECRET")
        .map_err(|_| "API0_INTERNAL_SECRET environment variable not set".to_string())?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "email": account_email,
        "amount": credits_to_add,
    });

    let res = client
        .post(format!("{store_url}/api/user/credits"))
        .header("Content-Type", "application/json")
        .header("X-Internal-Secret", &internal_secret)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("api0 store request failed: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("api0 store error {status}: {text}"));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("api0 store JSON parse error: {e}"))?;

    // The Store handler returns { "success": true, "balance": <new_balance>, ... }
    let new_balance = json["balance"]
        .as_i64()
        .ok_or_else(|| "api0 store response missing 'balance'".to_string())?;

    Ok(new_balance)
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// POST /payment/intent
///
/// Creates a Stripe PaymentIntent and returns the client_secret to the frontend.
/// Requires Firebase JWT authentication.
pub async fn create_payment_intent_handler(
    request: Json<CreateIntentRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<CreateIntentResponse>, Json<StandardErrorResponse>> {
    let amount_dollars = request.amount_dollars;

    if amount_dollars < 1 {
        return Err(Json(StandardErrorResponse::new(
            "Minimum amount is $1.00".to_string(),
            "INVALID_AMOUNT".to_string(),
            vec!["Provide an amount of at least 1 dollar".to_string()],
            None,
        )));
    }

    let secret_key = match stripe_secret_key() {
        Ok(k) => k,
        Err(e) => {
            app_log!(error, "Payment configuration error: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Payment service not configured".to_string(),
                "CONFIG_ERROR".to_string(),
                vec!["Contact support".to_string()],
                None,
            )));
        }
    };

    let publishable_key = match stripe_publishable_key() {
        Ok(k) => k,
        Err(e) => {
            app_log!(error, "Payment configuration error: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Payment service not configured".to_string(),
                "CONFIG_ERROR".to_string(),
                vec!["Contact support".to_string()],
                None,
            )));
        }
    };

    let amount_cents = amount_dollars * 100;
    let user_email = auth.email();

    app_log!(
        info,
        user = %user_email,
        amount_dollars = amount_dollars,
        "Creating Stripe PaymentIntent"
    );

    match stripe_create_payment_intent(&secret_key, amount_cents, user_email).await {
        Ok((intent_id, client_secret)) => {
            app_log!(
                info,
                user = %user_email,
                intent_id = %intent_id,
                "Stripe PaymentIntent created"
            );
            Ok(Json(CreateIntentResponse {
                success: true,
                client_secret,
                publishable_key,
            }))
        }
        Err(e) => {
            app_log!(error, user = %user_email, error = %e, "Failed to create PaymentIntent");
            Err(Json(StandardErrorResponse::new(
                "Failed to create payment".to_string(),
                "STRIPE_ERROR".to_string(),
                vec!["Try again or contact support".to_string()],
                None,
            )))
        }
    }
}

/// POST /payment/confirm
///
/// Verifies a Stripe PaymentIntent has succeeded, then tops up cvenom's api0 credit balance.
/// credits_added = amount_dollars * 100 (1 dollar = 100 credits).
/// Requires Firebase JWT authentication.
pub async fn confirm_payment_handler(
    request: Json<ConfirmPaymentRequest>,
    auth: AuthenticatedUser,
) -> Result<Json<ConfirmPaymentResponse>, Json<StandardErrorResponse>> {
    let payment_intent_id = &request.payment_intent_id;
    let user_email = auth.email();

    app_log!(
        info,
        user = %user_email,
        payment_intent_id = %payment_intent_id,
        "Confirming Stripe payment"
    );

    let secret_key = match stripe_secret_key() {
        Ok(k) => k,
        Err(e) => {
            app_log!(error, "Payment configuration error: {}", e);
            return Err(Json(StandardErrorResponse::new(
                "Payment service not configured".to_string(),
                "CONFIG_ERROR".to_string(),
                vec!["Contact support".to_string()],
                None,
            )));
        }
    };

    // 1. Verify payment with Stripe
    let amount_cents = match stripe_verify_payment_intent(&secret_key, payment_intent_id).await {
        Ok(cents) => cents,
        Err(e) => {
            app_log!(
                error,
                user = %user_email,
                intent = %payment_intent_id,
                error = %e,
                "Stripe payment verification failed"
            );
            return Err(Json(StandardErrorResponse::new(
                format!("Payment verification failed: {e}"),
                "VERIFICATION_FAILED".to_string(),
                vec![
                    "Contact support with your payment reference".to_string(),
                    format!("Payment ID: {payment_intent_id}"),
                ],
                None,
            )));
        }
    };

    // 2. Compute credits to add (1 dollar = 100 credits)
    let amount_dollars = (amount_cents / 100) as i64;
    let credits_to_add = amount_dollars * 100;

    app_log!(
        info,
        user = %user_email,
        intent = %payment_intent_id,
        amount_dollars = amount_dollars,
        credits_to_add = credits_to_add,
        "Payment verified – topping up api0 credit balance"
    );

    // 3. Top up cvenom's api0 tenant credit balance
    match api0_topup_credits(credits_to_add).await {
        Ok(new_balance) => {
            app_log!(
                info,
                user = %user_email,
                credits_added = credits_to_add,
                new_balance = new_balance,
                "Credit balance topped up successfully"
            );
            Ok(Json(ConfirmPaymentResponse {
                success: true,
                message: format!(
                    "Payment confirmed! ${amount_dollars} ({credits_to_add} credits) added."
                ),
                credits_added: credits_to_add,
                new_balance,
            }))
        }
        Err(e) => {
            // Payment went through Stripe but credit top-up failed.
            // Log with high severity — manual reconciliation needed.
            app_log!(
                error,
                user = %user_email,
                intent = %payment_intent_id,
                amount_dollars = amount_dollars,
                credits_to_add = credits_to_add,
                error = %e,
                "CRITICAL: Stripe payment succeeded but api0 credit top-up FAILED – manual reconciliation required"
            );
            Err(Json(StandardErrorResponse::new(
                "Payment received but credit update failed. Support has been notified.".to_string(),
                "CREDIT_UPDATE_FAILED".to_string(),
                vec![
                    format!("Payment ID: {payment_intent_id}"),
                    "Please contact support and provide your Payment ID.".to_string(),
                ],
                None,
            )))
        }
    }
}
