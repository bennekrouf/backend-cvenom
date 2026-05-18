use graflog::app_log;

mod sender;
mod templates;

pub use templates::EmailKind;

/// Fire-and-forget: spawn email delivery on a background task.
pub fn send_email(to: &str, kind: EmailKind) {
    let to = to.to_string();
    tokio::spawn(async move {
        if let Err(e) = sender::deliver(&to, &kind).await {
            app_log!(error, "Failed to send {} email to {}: {}", kind.name(), to, e);
        } else {
            app_log!(info, "Sent {} email to {}", kind.name(), to);
        }
    });
}
