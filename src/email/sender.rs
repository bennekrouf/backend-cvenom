use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::templates::EmailKind;

pub async fn deliver(to: &str, kind: &EmailKind) -> Result<()> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp-relay.brevo.com".into());
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);
    let smtp_user = std::env::var("SMTP_USER").context("SMTP_USER not set")?;
    let smtp_pass = std::env::var("SMTP_PASSWORD").context("SMTP_PASSWORD not set")?;
    let from_address = std::env::var("EMAIL_FROM").unwrap_or_else(|_| smtp_user.clone());

    let email = Message::builder()
        .from(format!("CVenom <{}>", from_address).parse()?)
        .to(to.parse()?)
        .subject(kind.subject())
        .header(ContentType::TEXT_HTML)
        .body(kind.html_body())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
        .credentials(creds)
        .port(smtp_port)
        .build();

    transport.send(email).await?;
    Ok(())
}
