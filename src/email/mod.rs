use graflog::app_log;

mod sender;
mod templates;

pub use templates::EmailKind;

/// Fire-and-forget: spawn email delivery on a background task.
/// `lang` controls the language of the email content ("en", "fr", "de").
///
/// Respects user email preferences: if the user has disabled a category
/// via the preferences screen, the email is silently skipped.
/// Admin emails and transactional emails (welcome, payment) are always sent.
pub fn send_email(to: &str, kind: EmailKind, lang: &str) {
    let to = to.to_string();
    let lang = lang.to_string();
    tokio::spawn(async move {
        if let Err(e) = sender::deliver(&to, &kind, &lang).await {
            app_log!(error, "Failed to send {} email to {}: {}", kind.name(), to, e);
        } else {
            app_log!(info, "Sent {} email ({}) to {}", kind.name(), lang, to);
        }
    });
}

/// Fire-and-forget with preference check.
/// `email_prefs_json` is the raw JSON string from tenant.email_prefs.
/// If the user has set `"<kind_name>": false`, the email is silently skipped.
pub fn send_email_with_prefs(to: &str, kind: EmailKind, lang: &str, email_prefs_json: Option<&str>) {
    // Always-send categories: admin, transactional, and welcome emails
    if !kind.is_optional() {
        send_email(to, kind, lang);
        return;
    }

    // Check user preferences
    if let Some(prefs_str) = email_prefs_json {
        if let Ok(prefs) = serde_json::from_str::<serde_json::Value>(prefs_str) {
            if let Some(false) = prefs.get(kind.name()).and_then(|v| v.as_bool()) {
                app_log!(info, "Skipping {} email to {} (disabled by user prefs)", kind.name(), to);
                return;
            }
        }
    }

    send_email(to, kind, lang);
}

/// Send an admin notification email (fire-and-forget).
/// Reads ADMIN_NOTIFY_EMAIL env var; falls back to the hardcoded admin address.
pub fn notify_admin(kind: EmailKind) {
    let admin = std::env::var("ADMIN_NOTIFY_EMAIL")
        .unwrap_or_else(|_| "mohamed.bennekrouf@gmail.com".to_string());
    send_email(&admin, kind, "en"); // admin emails always in English
}
