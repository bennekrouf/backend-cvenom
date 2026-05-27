// Delegates email delivery to the api0 store's internal email endpoint.
// Cvenom never touches SMTP directly — api0 owns the sending infrastructure.
use anyhow::{Context, Result};
use super::templates::EmailKind;

pub async fn deliver(to: &str, kind: &EmailKind, lang: &str) -> Result<()> {
    let store_url = std::env::var("API0_STORE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:5007".into());
    let internal_secret = std::env::var("API0_INTERNAL_SECRET")
        .context("API0_INTERNAL_SECRET not set")?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/internal/email/send", store_url))
        .header("X-Internal-Secret", &internal_secret)
        .json(&serde_json::json!({
            "to":        to,
            "subject":   kind.subject(lang),
            "html_body": kind.html_body(lang),
        }))
        .send()
        .await
        .context("HTTP request to api0 email endpoint failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("api0 email endpoint returned {}: {}", status, body);
    }

    Ok(())
}
