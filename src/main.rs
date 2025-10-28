use anyhow::Result;
use cv_generator::{core::ConfigManager, start_web_server};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::try_init().ok();

    let port = std::env::var("ROCKET_PORT")
        .map_err(|_| anyhow::anyhow!("ROCKET_PORT environment variable not set"))?
        .parse::<u16>()
        .map_err(|_| anyhow::anyhow!("ROCKET_PORT must be a valid port number"))?;

    let cv_service_url = std::env::var("CV_SERVICE_URL")
        .map_err(|_| anyhow::anyhow!("CV_SERVICE_URL environment variable not set"))?;

    info!("Parsed port: {}", port);
    info!("CV Service URL: {}", cv_service_url);

    // Load configuration using unified ConfigManager
    let config = ConfigManager::load()?;
    config.ensure_directories().await?;

    info!("Starting Multi-tenant CV Generator API Server");
    info!(
        "Environment: {}",
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string())
    );
    info!(
        "Tenant Data: {}",
        config.environment.tenant_data_path.display()
    );
    info!("Database: {}", config.environment.database_path.display());
    info!("Server: http://0.0.0.0:{}", port);
    info!("CV Service: {}", cv_service_url);

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

