use anyhow::Result;
use cv_generator::app_log;
use cv_generator::{core::ConfigManager, start_web_server};
use std::fs::OpenOptions;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Clear file on startup
        .open("/tmp/api0.log")
        .expect("Failed to open log file");

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .json()
                .with_writer(file)
                .with_current_span(false)
                .with_span_list(false),
        )
        .with(
            EnvFilter::from_default_env()
                .add_directive("trace".parse().expect("Invalid log directive")),
        )
        .init();

    let port = std::env::var("ROCKET_PORT")
        .map_err(|_| anyhow::anyhow!("ROCKET_PORT environment variable not set"))?
        .parse::<u16>()
        .map_err(|_| anyhow::anyhow!("ROCKET_PORT must be a valid port number"))?;

    let cv_service_url = std::env::var("CV_SERVICE_URL")
        .map_err(|_| anyhow::anyhow!("CV_SERVICE_URL environment variable not set"))?;

    app_log!(info, "Parsed port: {}", port);
    app_log!(info, "CV Service URL: {}", cv_service_url);

    // Load configuration using unified ConfigManager
    let config = ConfigManager::load()?;
    config.ensure_directories().await?;

    app_log!(info, "Starting Multi-tenant CV Generator API Server");
    app_log!(
        info,
        "Environment: {}",
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string())
    );
    app_log!(
        info,
        "Tenant Data: {}",
        config.environment.tenant_data_path.display()
    );
    app_log!(
        info,
        "Database: {}",
        config.environment.database_path.display()
    );
    app_log!(info, "Server: http://0.0.0.0:{}", port);
    app_log!(info, "CV Service: {}", cv_service_url);

    start_web_server(
        config.environment.tenant_data_path,
        config.environment.output_path,
        config.environment.templates_path,
        config.environment.database_path,
        port,
        cv_service_url,
    )
    .await
}
