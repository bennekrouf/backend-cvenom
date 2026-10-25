// src/web/handlers/payment_handlers.rs
//
// Stripe payment handlers for cvenom.
//
// Flow:
//   1. Frontend calls POST /payment/intent { amount_dollars } → backend creates a Stripe
//      PaymentIntent and returns the client_secret to the browser.
//   2. Browser confirms the payment with Stripe.js (no redirect).
//   3. Frontend calls POST /payment/confirm { payment_intent_id } → backend verifies the
//      PaymentIntent with Stripe, then calls api0 Store to top-up the USER's credit balance.
//      credits_added = amount_dollars * 100.
//
// Environment variables (read at call time so they can be hot-reloaded in dev):
//   STRIPE_SECRET_KEY       – cvenom's own Stripe secret key (sk_live_… / sk_test_…)
//   STRIPE_PUBLISHABLE_KEY  – cvenom's own Stripe publishable key (pk_live_… / pk_test_…)
//   API0_STORE_URL          – base URL of the api0 Store service  (e.g. http://localhost:5007)
//   API0_STORE_SERVICE_KEY  – cvenom's api0 Store key (scopes credits.read, credits.write,
//                             email.send); never the platform-wide API0_INTERNAL_SECRET

use graflog::app_log;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::core::database::{DatabaseConfig, TenantRepository};
use crate::web::types::StandardErrorResponse;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateIntentRequest {
    /// Amount in whole units of the chosen currency (minimum 1).
    /// The field name stays `amount_dollars` for backwards-compatibility with
    /// existing clients that don't send a `currency`.
    pub amount_dollars: u32,
    /// ISO 4217 lowercase currency code (e.g. "usd", "eur", "chf", "gbp").
    /// Defaults to "usd" when absent.
    pub currency: Option<String>,
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
    /// When the buyer ticked "start delivery now and waive the 14-day withdrawal right"
    /// (ISO-8601, client clock). Recorded on the PaymentIntent as proof of consent.
    #[serde(default)]
    pub withdrawal_waiver_accepted_at: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ConfirmPaymentResponse {
    pub success: bool,
    pub message: String,
    pub credits_added: i64,
    pub new_balance: i64,
}

/// Where mayorana sells its plans; `from=cvenom` brings the buyer back here.
pub const MAYORANA_PRICING_URL: &str = "https://mayorana.ch/pricing?from=cvenom";

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

/// Validate and normalise an ISO 4217 currency code.
///
/// Accepts lowercase codes for the currencies we support.
/// Returns `"usd"` for any unrecognised or empty value so the form degrades
/// gracefully instead of hard-failing.
fn normalise_currency(raw: Option<&str>) -> &'static str {
    match raw.unwrap_or("").to_ascii_lowercase().as_str() {
        "eur" => "eur",
        "gbp" => "gbp",
        "chf" => "chf",
        "cad" => "cad",
        "aud" => "aud",
        _     => "usd",
    }
}

/// Call Stripe API to create a PaymentIntent.
/// Returns the PaymentIntent ID and client_secret.
async fn stripe_create_payment_intent(
    secret_key: &str,
    amount_cents: u32,
    user_email: &str,
    currency: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();

    let params = [
        ("amount", amount_cents.to_string()),
        ("currency", currency.to_string()),
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

/// Record the buyer's withdrawal-right waiver on the PaymentIntent's metadata, so the
/// proof of consent lives next to the charge in Stripe.
async fn stripe_record_withdrawal_waiver(
    secret_key: &str,
    payment_intent_id: &str,
    accepted_at: &str,
) -> Result<(), String> {
    // Client-supplied value: keep it short and printable before storing it.
    let accepted_at: String = accepted_at.chars().filter(|c| c.is_ascii_graphic()).take(40).collect();
    let params = [
        ("metadata[withdrawal_waiver]", "accepted".to_string()),
        ("metadata[withdrawal_waiver_accepted_at]", accepted_at),
        ("metadata[withdrawal_waiver_recorded_at]", chrono::Utc::now().to_rfc3339()),
    ];
    let res = reqwest::Client::new()
        .post(format!("https://api.stripe.com/v1/payment_intents/{payment_intent_id}"))
        .basic_auth(secret_key, Some(""))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Stripe request failed: {e}"))?;
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Stripe error: {body}"));
    }
    Ok(())
}

// ── api0 Store helpers ────────────────────────────────────────────────────────

fn api0_store_url() -> Result<String, String> {
    std::env::var("API0_STORE_URL")
        .map_err(|_| "API0_STORE_URL environment variable not set".to_string())
}

fn api0_service_key() -> Result<String, String> {
    std::env::var("API0_STORE_SERVICE_KEY")
        .map_err(|_| "API0_STORE_SERVICE_KEY environment variable not set".to_string())
}

/// Top up a user's credit balance in api0 Store.
///
/// Calls: POST {API0_STORE_URL}/api/user/credits
/// Body:  { "email": "<user email>", "amount": <credits> }
pub async fn api0_topup_credits(user_email: &str, credits_to_add: i64, action_type: &str, description: Option<&str>) -> Result<i64, String> {
    let store_url = api0_store_url()?;
    let service_key = api0_service_key()?;
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "email": user_email,
        "amount": credits_to_add,
        "action_type": action_type,
        "description": description,
    });

    let res = client
        .post(format!("{store_url}/api/user/credits"))
        .header("Content-Type", "application/json")
        .header("X-Service-Key", &service_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("api0 store request failed: {e}"))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("api0 store error: {text}"));
    }

    let json: serde_json::Value = res.json().await
        .map_err(|e| format!("api0 store JSON parse error: {e}"))?;

    json["balance"].as_i64()
        .ok_or_else(|| "api0 store response missing 'balance'".to_string())
}

/// Fetch a user's current credit balance from api0 Store.
///
/// Calls: GET {API0_STORE_URL}/api/user/credits/{email}
pub async fn api0_get_balance(user_email: &str) -> Result<i64, String> {
    let store_url = api0_store_url()?;
    let service_key = api0_service_key()?;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{store_url}/api/user/credits/{}", utf8_percent_encode(user_email, NON_ALPHANUMERIC)))
        .header("X-Service-Key", &service_key)
        .send()
        .await
        .map_err(|e| format!("api0 store request failed: {e}"))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("api0 store error: {text}"));
    }

    let json: serde_json::Value = res.json().await
        .map_err(|e| format!("api0 store JSON parse error: {e}"))?;

    json["balance"].as_i64()
        .ok_or_else(|| "api0 store response missing 'balance'".to_string())
}

/// What the user can spend, and where it comes from: their mayorana plan's
/// credits left this month, and their own CVenom balance (top-ups, welcome,
/// referrals, refunds). The plan is spent first.
pub struct CreditBreakdown {
    /// Plan credits left plus the CVenom balance: what a charge can take.
    pub available: i64,
    pub plan_credits_left: i64,
    /// The live mayorana plan, as the store describes it; None without one.
    pub plan: Option<PlanSummary>,
}

/// The user's live mayorana plan, for showing next to the balance.
#[derive(Serialize, Deserialize, Clone)]
pub struct PlanSummary {
    pub name: String,
    pub credits_granted: i64,
    pub credits_left: i64,
    /// When this month's credits lapse, and the plan renews or ends (RFC 3339).
    pub period_end: String,
    #[serde(default)]
    pub cancel_at_period_end: bool,
    #[serde(default)]
    pub is_free: bool,
}

/// Calls: GET {API0_STORE_URL}/api/user/credits/{email}, reading `available`
/// `plan_credits_left` and `plan` (a store from before plans answers `balance` only).
pub async fn api0_get_credits(user_email: &str) -> Result<CreditBreakdown, String> {
    let store_url = api0_store_url()?;
    let service_key = api0_service_key()?;
    let res = reqwest::Client::new()
        .get(format!("{store_url}/api/user/credits/{}", utf8_percent_encode(user_email, NON_ALPHANUMERIC)))
        .header("X-Service-Key", &service_key)
        .send()
        .await
        .map_err(|e| format!("api0 store request failed: {e}"))?;
    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("api0 store error: {text}"));
    }
    let json: serde_json::Value = res.json().await.map_err(|e| format!("api0 store JSON parse error: {e}"))?;
    let available = json["available"]
        .as_i64()
        .or_else(|| json["balance"].as_i64())
        .ok_or_else(|| "api0 store response missing 'available'".to_string())?;
    Ok(CreditBreakdown {
        available,
        plan_credits_left: json["plan_credits_left"].as_i64().unwrap_or(0),
        plan: serde_json::from_value(json["plan"].clone()).ok(),
    })
}

/// What the user can spend: their mayorana plan's credits left this month,
/// plus their CVenom balance.
pub async fn api0_get_available(user_email: &str) -> Result<i64, String> {
    api0_get_credits(user_email).await.map(|c| c.available)
}

/// Charges `cost` credits for an operation: the user's mayorana plan pays
/// first, then their CVenom balance. The store checks and charges in one call,
/// so nothing is taken when the two together fall short.
///
/// Calls: POST {API0_STORE_URL}/api/user/credits/spend
pub async fn check_and_deduct_credits(
    user_email: &str,
    cost: i64,
    conversation_id: Option<String>,
    action_type: &str,
) -> Result<(), Json<StandardErrorResponse>> {
    let failed = |e: String| {
        Json(StandardErrorResponse::new(
            format!("Failed to deduct credits: {}", e),
            "CREDIT_DEDUCT_FAILED".to_string(),
            vec!["Contact support if this persists".to_string()],
            conversation_id.clone(),
        ))
    };
    let (store_url, service_key) = match (api0_store_url(), api0_service_key()) {
        (Ok(u), Ok(k)) => (u, k),
        (Err(e), _) | (_, Err(e)) => return Err(failed(e)),
    };
    let res = reqwest::Client::new()
        .post(format!("{store_url}/api/user/credits/spend"))
        .header("X-Service-Key", &service_key)
        .json(&serde_json::json!({ "email": user_email, "amount": cost, "action_type": action_type }))
        .send()
        .await
        .map_err(|e| failed(format!("api0 store request failed: {e}")))?;
    let status = res.status();
    let json: serde_json::Value = res.json().await.unwrap_or_default();

    if status.as_u16() == 402 {
        let available = json["available"].as_i64().unwrap_or(0);
        return Err(Json(StandardErrorResponse::new(
            format!("Not enough credits: you have {} but this costs {}", available, cost),
            "INSUFFICIENT_CREDITS".to_string(),
            vec![
                format!("Subscribe to a mayorana plan for monthly credits (CVenom plus the other apps): {MAYORANA_PRICING_URL}"),
                "Or buy a one-off top-up at https://studio.cvenom.com/credits".to_string(),
                format!("Sign in on mayorana.ch with {user_email} so the plan's credits count here."),
            ],
            conversation_id,
        )));
    }
    if !status.is_success() {
        return Err(failed(format!("api0 store error: {}", json["message"].as_str().unwrap_or("unknown"))));
    }

    // The low-credit warning is about what is left to spend, plan included.
    let new_available = json["available"].as_i64().unwrap_or(0);
    const LOW_CREDIT_THRESHOLD: i64 = 10;
    if new_available <= LOW_CREDIT_THRESHOLD && new_available + cost > LOW_CREDIT_THRESHOLD {
        crate::email::send_email(
            user_email,
            crate::email::EmailKind::LowCredits { balance: new_available },
            "en", // no lang context in credit check
        );
    }

    Ok(())
}

/// Give back credits taken by `check_and_deduct_credits` when the paid operation
/// then failed. Best effort: a failed refund is logged, never surfaced. It goes
/// to the CVenom balance even when the plan paid: the charge does not say which.
pub async fn refund_credits(user_email: &str, cost: i64, action_type: &str) {
    let refund_type = format!("{}_refund", action_type);
    if let Err(e) = api0_topup_credits(user_email, cost, &refund_type, Some("Refund: operation failed")).await {
        app_log!(error, "Failed to refund {} credits to {} ({}): {}", cost, user_email, refund_type, e);
    }
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
    let currency = normalise_currency(request.currency.as_deref());

    if amount_dollars < 1 {
        return Err(Json(StandardErrorResponse::new(
            format!("Minimum amount is 1 {}", currency.to_uppercase()),
            "INVALID_AMOUNT".to_string(),
            vec!["Provide an amount of at least 1".to_string()],
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
        amount = amount_dollars,
        currency = currency,
        "Creating Stripe PaymentIntent"
    );

    match stripe_create_payment_intent(&secret_key, amount_cents, user_email, currency).await {
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
    db_config: &rocket::State<DatabaseConfig>,
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

    // Proof of the withdrawal-right waiver. Best-effort: the charge already succeeded,
    // so a failure here must not stop the credits from being added.
    match request.withdrawal_waiver_accepted_at.as_deref() {
        Some(accepted_at) => {
            if let Err(e) = stripe_record_withdrawal_waiver(&secret_key, payment_intent_id, accepted_at).await {
                app_log!(error, user = %user_email, intent = %payment_intent_id, error = %e,
                    "Failed to record withdrawal waiver on PaymentIntent");
            }
        }
        None => app_log!(warn, user = %user_email, intent = %payment_intent_id,
            "Payment confirmed without a withdrawal waiver (outdated client?)"),
    }

    // 2. Compute credits to add (1 dollar = 4 credits; $5 = 20 credits = 20 CV generations)
    let amount_dollars = (amount_cents / 100) as i64;
    let credits_to_add = amount_dollars * 4;

    app_log!(
        info,
        user = %user_email,
        intent = %payment_intent_id,
        amount_dollars = amount_dollars,
        credits_to_add = credits_to_add,
        "Payment verified – topping up api0 credit balance"
    );

    // 3. Top up this user's credit balance in api0
    match api0_topup_credits(user_email, credits_to_add, "topup", None).await {
        Ok(new_balance) => {
            app_log!(
                info,
                user = %user_email,
                credits_added = credits_to_add,
                new_balance = new_balance,
                "Credit balance topped up successfully"
            );

            // 4. Commission ledger — fire-and-forget (never block the payment response)
            if let Ok(pool) = db_config.pool() {
                let repo = TenantRepository::new(pool);
                if let Ok(Some(tenant)) = repo.find_by_email_or_domain(user_email).await {
                    if let Some(code) = tenant.referred_by_code {
                        // Look up the BD's commission rate
                        let rate: f64 = sqlx::query_scalar(
                            "SELECT commission_rate FROM business_developers \
                             WHERE referral_code = ?",
                        )
                        .bind(&code)
                        .fetch_optional(pool)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or(0.30);

                        let commission = amount_dollars as f64 * rate;

                        let _ = sqlx::query(
                            "INSERT OR IGNORE INTO bd_commissions \
                             (referral_code, customer_email, stripe_payment_intent, \
                              amount_dollars, commission_dollars) \
                             VALUES (?, ?, ?, ?, ?)",
                        )
                        .bind(&code)
                        .bind(user_email)
                        .bind(payment_intent_id)
                        .bind(amount_dollars as f64)
                        .bind(commission)
                        .execute(pool)
                        .await;

                        app_log!(
                            info,
                            bd_code = %code,
                            customer = %user_email,
                            amount_dollars = amount_dollars,
                            commission = commission,
                            "BD commission recorded"
                        );

                        if let Ok(Some(bd_email)) = sqlx::query_scalar::<_, String>(
                            "SELECT email FROM business_developers WHERE referral_code = ?",
                        )
                        .bind(&code)
                        .fetch_optional(pool)
                        .await
                        {
                            crate::email::send_email(
                                &bd_email,
                                crate::email::EmailKind::CommissionEarned {
                                    customer_email: user_email.to_string(),
                                    amount_dollars: amount_dollars as f64,
                                    commission_dollars: commission,
                                },
                                "en", // BD emails in English
                            );
                        }
                    }
                }
            }

            crate::email::send_email(
                user_email,
                crate::email::EmailKind::PaymentReceipt {
                    amount_cents: amount_dollars * 100,
                    credits_added: credits_to_add,
                    new_balance,
                },
                "en", // webhook — no user lang context
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

// ── GET /payment/balance ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GetBalanceResponse {
    pub success: bool,
    /// Everything the user can spend: plan credits left plus their own balance.
    pub balance: i64,
    /// The part of `balance` that is their mayorana plan's, lapsing at its renewal.
    pub plan_credits_left: i64,
    /// Their live mayorana plan, or null.
    pub plan: Option<PlanSummary>,
}

pub async fn api0_get_transactions(
    user_email: &str,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, String> {
    let store_url = std::env::var("API0_STORE_URL")
        .map_err(|_| "API0_STORE_URL not set".to_string())?;
    let client = reqwest::Client::new();
    let mut request = client.get(format!(
        "{}/api/user/credit-transactions/{}",
        store_url,
        utf8_percent_encode(user_email, NON_ALPHANUMERIC)
    ));
    if let Some(limit) = limit {
        request = request.query(&[("limit", limit)]);
    }
    let response = request
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let body = response.text().await.map_err(|e| e.to_string())?;

    app_log!(info, "api0 credit-transactions status={} body={}", status, crate::utils::truncate_str(&body, 300));

    if !status.is_success() {
        // If the endpoint doesn't exist yet (404) return empty list gracefully
        if status.as_u16() == 404 {
            app_log!(warn, "api0 store has no /credit-transactions endpoint — store binary needs update");
            return Ok(vec![]);
        }
        return Err(format!("Store returned {}: {}", status, body));
    }

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("JSON parse error: {} — body: {}", e, crate::utils::truncate_str(&body, 200)))?;
    Ok(json["transactions"].as_array().cloned().unwrap_or_default())
}

/// GET /payment/balance
///
/// Returns the authenticated user's current credit balance.
pub async fn get_balance_handler(
    auth: AuthenticatedUser,
) -> Result<Json<GetBalanceResponse>, Json<StandardErrorResponse>> {
    let user_email = auth.email();

    // The plan's credits are the user's to spend here too.
    match api0_get_credits(user_email).await {
        Ok(c) => Ok(Json(GetBalanceResponse {
            success: true,
            balance: c.available,
            plan_credits_left: c.plan_credits_left,
            plan: c.plan,
        })),
        Err(e) => {
            app_log!(error, user = %user_email, error = %e, "Failed to get credit balance");
            Err(Json(StandardErrorResponse::new(
                format!("Failed to retrieve credit balance: {}", e),
                "BALANCE_ERROR".to_string(),
                vec!["Contact support if this persists".to_string()],
                None,
            )))
        }
    }
}

// ── GET /payment/plans ────────────────────────────────────────────────────────
//
// The mayorana plans on sale, read from the api0 gateway's public catalogue
// (what mayorana.ch/pricing shows), so the studio never keeps its own prices.
// Asked server-side: the gateway's CORS is for mayorana's origin, not ours.
//
//   API0_GATEWAY_URL  – default https://gateway.api0.ai

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MayoranaPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub price_cents: i64,
    pub currency: String,
    pub monthly_credits: i64,
    /// The apps it unlocks, by product id; "*" is every app.
    #[serde(default)]
    pub products: Vec<String>,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PlansResponse {
    pub success: bool,
    pub plans: Vec<MayoranaPlan>,
    pub pricing_url: &'static str,
}

pub async fn get_plans_handler() -> Result<Json<PlansResponse>, Json<StandardErrorResponse>> {
    let gateway = std::env::var("API0_GATEWAY_URL").unwrap_or_else(|_| "https://gateway.api0.ai".to_string());
    let fetched = async {
        let res = reqwest::Client::new()
            .get(format!("{}/api/plans", gateway.trim_end_matches('/')))
            .timeout(std::time::Duration::from_secs(8))
            .send()
            .await
            .map_err(|e| format!("gateway request failed: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("gateway returned {}", res.status()));
        }
        let json: serde_json::Value = res.json().await.map_err(|e| format!("gateway JSON parse error: {e}"))?;
        let plans: Vec<MayoranaPlan> = serde_json::from_value(json["plans"].clone()).unwrap_or_default();
        Ok::<_, String>(plans)
    }
    .await;

    match fetched {
        Ok(mut plans) => {
            // A plan priced 0 is the apps' free allowance, never an offer.
            plans.retain(|p| p.price_cents > 0);
            plans.sort_by_key(|p| p.sort_order);
            Ok(Json(PlansResponse { success: true, plans, pricing_url: MAYORANA_PRICING_URL }))
        }
        Err(e) => {
            app_log!(error, error = %e, "Failed to read mayorana plans");
            Err(Json(StandardErrorResponse::new(
                format!("Failed to load plans: {e}"),
                "PLANS_FETCH_FAILED".to_string(),
                vec![format!("See the plans at {MAYORANA_PRICING_URL}")],
                None,
            )))
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TransactionsResponse {
    pub success: bool,
    pub transactions: Vec<serde_json::Value>,
}

pub async fn get_transactions_handler(
    auth: AuthenticatedUser,
    limit: Option<i64>,
) -> Result<Json<TransactionsResponse>, Json<StandardErrorResponse>> {
    let user_email = &auth.user().email;
    match api0_get_transactions(user_email, limit).await {
        Ok(transactions) => Ok(Json(TransactionsResponse { success: true, transactions })),
        Err(e) => {
            app_log!(error, "Failed to get transactions for {}: {}", user_email, e);
            Err(Json(StandardErrorResponse::new(
                format!("Failed to retrieve transactions: {}", e),
                "TRANSACTIONS_FETCH_FAILED".to_string(),
                vec![],
                None,
            )))
        }
    }
}

// ── POST /admin/credits ───────────────────────────────────────────────────────
//
// Admin-only endpoint to add or remove credits for a **cvenom tenant**.
//
// The target email must already exist in cvenom's tenant table (SQLite).
// This intentionally prevents accidentally topping up an api0-only user
// who has never used cvenom. Use the api0 store's /api/admin/credits
// endpoint for generic api0 credit adjustments.
//
// Auth:  Requires a valid Firebase JWT (AuthenticatedUser guard). The caller's
//        email must match ADMIN_EMAIL ("mohamed.bennekrouf@gmail.com").
// Body:  { "email": "user@example.com", "amount": 100, "description": "optional note" }
//        amount can be negative to deduct credits.
//
// Returns: { success, email, amount, new_balance, description }

const ADMIN_EMAIL: &str = "mohamed.bennekrouf@gmail.com";

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminCreditRequest {
    pub email: String,
    pub amount: i64,
    pub description: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminCreditResponse {
    pub success: bool,
    pub email: String,
    pub amount: i64,
    pub new_balance: i64,
    pub description: Option<String>,
}

pub async fn admin_add_credits_handler(
    request: Json<AdminCreditRequest>,
    caller_email: &str,
    db_config: &DatabaseConfig,
) -> Result<Json<AdminCreditResponse>, Json<StandardErrorResponse>> {
    // ── Authenticate ──────────────────────────────────────────────────────────
    if caller_email.to_lowercase() != ADMIN_EMAIL {
        app_log!(warn, caller = %caller_email, "Admin credits endpoint: unauthorized caller");
        return Err(Json(StandardErrorResponse::new(
            "Unauthorized".to_string(),
            "UNAUTHORIZED".to_string(),
            vec![],
            None,
        )));
    }

    // ── Validate input ────────────────────────────────────────────────────────
    let email = request.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(Json(StandardErrorResponse::new(
            "email is required".to_string(),
            "INVALID_INPUT".to_string(),
            vec![],
            None,
        )));
    }
    if request.amount == 0 {
        return Err(Json(StandardErrorResponse::new(
            "amount must be non-zero".to_string(),
            "INVALID_INPUT".to_string(),
            vec![],
            None,
        )));
    }

    // ── Verify this is a known cvenom tenant ──────────────────────────────────
    // Credits live in api0's database; this check ensures we only adjust
    // credits for users who actually have a cvenom account — not arbitrary
    // api0 users. Use api0's own /api/admin/credits for the latter.
    let pool = db_config.pool().map_err(|e| {
        app_log!(error, error = %e, "Admin credits: DB connection failed");
        Json(StandardErrorResponse::new(
            "Database error".to_string(),
            "DATABASE_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;
    let repo = TenantRepository::new(pool);
    match repo.find_by_email_or_domain(&email).await {
        Ok(Some(_)) => {} // tenant exists — proceed
        Ok(None) => {
            app_log!(warn, email = %email, "Admin credits: email is not a cvenom tenant");
            return Err(Json(StandardErrorResponse::new(
                format!("'{}' is not a cvenom tenant", email),
                "TENANT_NOT_FOUND".to_string(),
                vec!["Only cvenom users can be topped up here. Use api0's /api/admin/credits for other users.".to_string()],
                None,
            )));
        }
        Err(e) => {
            app_log!(error, email = %email, error = %e, "Admin credits: tenant lookup failed");
            return Err(Json(StandardErrorResponse::new(
                "Database error during tenant lookup".to_string(),
                "DATABASE_ERROR".to_string(),
                vec![],
                None,
            )));
        }
    }

    // ── Forward to api0 store ─────────────────────────────────────────────────
    let action_type = if request.amount > 0 { "admin_topup" } else { "admin_deduct" };
    let desc = request.description.as_deref();

    match api0_topup_credits(&email, request.amount, action_type, desc).await {
        Ok(new_balance) => {
            app_log!(
                info,
                email = %email,
                amount = request.amount,
                new_balance = new_balance,
                action = %action_type,
                "Admin credit adjustment applied"
            );
            crate::email::send_email(
                &email,
                crate::email::EmailKind::CreditAdjustment {
                    amount: request.amount,
                    reason: request.description.clone().unwrap_or_else(|| action_type.to_string()),
                    new_balance,
                },
                "en", // admin action — no user lang context
            );

            Ok(Json(AdminCreditResponse {
                success: true,
                email,
                amount: request.amount,
                new_balance,
                description: request.description.clone(),
            }))
        }
        Err(e) => {
            app_log!(error, email = %email, error = %e, "Admin credit adjustment failed");
            Err(Json(StandardErrorResponse::new(
                format!("Credit update failed: {}", e),
                "CREDIT_UPDATE_FAILED".to_string(),
                vec![],
                None,
            )))
        }
    }
}

// ── GET /admin/credits/users — all tenants with their api0 balances ────────────

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminUserCredit {
    pub email: String,
    pub tenant_name: String,
    pub balance: i64,
    pub joined_at: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminCreditUsersResponse {
    pub success: bool,
    pub total_users: usize,
    pub total_credits: i64,
    pub users: Vec<AdminUserCredit>,
}

pub async fn admin_credit_users_handler(
    auth: AuthenticatedUser,
    db_config: &rocket::State<crate::core::database::DatabaseConfig>,
) -> Result<Json<AdminCreditUsersResponse>, Json<StandardErrorResponse>> {
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        return Err(Json(StandardErrorResponse::new(
            "Admin access required".to_string(),
            "FORBIDDEN".to_string(),
            vec![],
            None,
        )));
    }

    let pool = db_config.pool().map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Database error: {}", e),
            "DB_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    // Fetch all active email-based tenants
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT email, tenant_name, created_at FROM tenants \
         WHERE is_active = TRUE AND email IS NOT NULL ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        Json(StandardErrorResponse::new(
            format!("Failed to list tenants: {}", e),
            "DB_ERROR".to_string(),
            vec![],
            None,
        ))
    })?;

    // Fan out to api0 Store in parallel (bounded to 10 concurrent requests)
    let mut users: Vec<AdminUserCredit> = Vec::with_capacity(rows.len());
    // Process in chunks to avoid overwhelming the api0 store
    for chunk in rows.chunks(10) {
        let futures: Vec<_> = chunk.iter().map(|(email, tenant_name, joined_at)| {
            let email = email.clone();
            let tenant_name = tenant_name.clone();
            let joined_at = joined_at.clone();
            async move {
                let balance = api0_get_balance(&email).await.unwrap_or(0);
                AdminUserCredit { email, tenant_name, balance, joined_at }
            }
        }).collect();
        let results = futures::future::join_all(futures).await;
        users.extend(results);
    }

    // Sort by balance descending
    users.sort_by(|a, b| b.balance.cmp(&a.balance));

    let total_credits: i64 = users.iter().map(|u| u.balance).sum();

    Ok(Json(AdminCreditUsersResponse {
        success: true,
        total_users: users.len(),
        total_credits,
        users,
    }))
}

// ── GET /admin/credits/transactions/<email> — one user's transaction log ───────

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminUserTransactionsResponse {
    pub success: bool,
    pub email: String,
    pub balance: i64,
    pub transactions: Vec<serde_json::Value>,
}

pub async fn admin_user_transactions_handler(
    email: String,
    auth: AuthenticatedUser,
) -> Result<Json<AdminUserTransactionsResponse>, Json<StandardErrorResponse>> {
    if auth.email().to_lowercase() != ADMIN_EMAIL {
        return Err(Json(StandardErrorResponse::new(
            "Admin access required".to_string(),
            "FORBIDDEN".to_string(),
            vec![],
            None,
        )));
    }

    let decoded = percent_encoding::percent_decode_str(&email)
        .decode_utf8()
        .map(|s| s.to_string())
        .unwrap_or(email);

    let (balance, transactions) = tokio::join!(
        api0_get_balance(&decoded),
        api0_get_transactions(&decoded, None),
    );

    Ok(Json(AdminUserTransactionsResponse {
        success: true,
        email: decoded,
        balance: balance.unwrap_or(0),
        transactions: transactions.unwrap_or_default(),
    }))
}
