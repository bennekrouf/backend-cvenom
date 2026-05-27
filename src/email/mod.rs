use graflog::app_log;

mod sender;
mod templates;

pub use templates::EmailKind;

/// Fire-and-forget: spawn email delivery on a background task.
/// `lang` controls the language of the email content ("en", "fr", "de").
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

/// Send an admin notification email (fire-and-forget).
/// Reads ADMIN_NOTIFY_EMAIL env var; falls back to the hardcoded admin address.
pub fn notify_admin(kind: EmailKind) {
    let admin = std::env::var("ADMIN_NOTIFY_EMAIL")
        .unwrap_or_else(|_| "mohamed.bennekrouf@gmail.com".to_string());
    send_email(&admin, kind, "en"); // admin emails always in English
}
