use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::templates::EmailKind;

pub async fn deliver(to: &str, kind: &EmailKind) -> Result<()> {
    let smtp_user = std::env::var("EMAIL_FROM").unwrap_or_else(|_| "mb@mayorana.ch".into());
    let smtp_pass = std::env::var("GMAIL_APP_PASSWORD").context("GMAIL_APP_PASSWORD not set")?;

    let email = Message::builder()
        .from(format!("CVenom <{}>", smtp_user).parse()?)
        .to(to.parse()?)
        .subject(kind.subject())
        .header(ContentType::TEXT_HTML)
        .body(kind.html_body())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")?
        .credentials(creds)
        .port(587)
        .build();

    transport.send(email).await?;
    Ok(())
}
