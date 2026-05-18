use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::{OnceLock, RwLock};

use super::templates::EmailKind;

// ── In-memory SMTP config (loaded from DB at startup, writable by admin) ─────

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host:       String,
    pub port:       u16,
    pub user:       String,
    pub password:   String,
    pub from_addr:  String,
}

impl SmtpConfig {
    /// Build from env vars (fallback when DB has no config yet).
    pub fn from_env() -> Option<Self> {
        let user = std::env::var("SMTP_USER").ok()?;
        let password = std::env::var("SMTP_PASSWORD").ok()?;
        let host = std::env::var("SMTP_HOST")
            .unwrap_or_else(|_| "smtp-relay.brevo.com".into());
        let port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(587);
        let from_addr = std::env::var("EMAIL_FROM").unwrap_or_else(|_| user.clone());
        Some(Self { host, port, user, password, from_addr })
    }
}

static SMTP_CONFIG: OnceLock<RwLock<Option<SmtpConfig>>> = OnceLock::new();

fn config_lock() -> &'static RwLock<Option<SmtpConfig>> {
    SMTP_CONFIG.get_or_init(|| RwLock::new(SmtpConfig::from_env()))
}

/// Called at startup with the value loaded from the DB (overrides env vars).
pub fn init_smtp_config(cfg: SmtpConfig) {
    *config_lock().write().unwrap() = Some(cfg);
}

/// Called by the admin PUT endpoint after saving to DB.
pub fn reload_smtp_config(cfg: SmtpConfig) {
    *config_lock().write().unwrap() = Some(cfg);
}

/// Return a snapshot of the current config (None if completely unconfigured).
pub fn current_smtp_config() -> Option<SmtpConfig> {
    config_lock().read().unwrap().clone()
}

// ── Delivery ──────────────────────────────────────────────────────────────────

pub async fn deliver(to: &str, kind: &EmailKind) -> Result<()> {
    let cfg = current_smtp_config()
        .context("SMTP not configured — set SMTP_USER/SMTP_PASSWORD or use the admin panel")?;

    let email = Message::builder()
        .from(format!("CVenom <{}>", cfg.from_addr).parse()?)
        .to(to.parse()?)
        .subject(kind.subject())
        .header(ContentType::TEXT_HTML)
        .body(kind.html_body())?;

    let creds = Credentials::new(cfg.user, cfg.password);

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)?
        .credentials(creds)
        .port(cfg.port)
        .build();

    transport.send(email).await?;
    Ok(())
}
