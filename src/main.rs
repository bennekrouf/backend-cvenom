// src/main.rs
use anyhow::Result;
use cv_generator::{core::ConfigManager, start_web_server};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::try_init().ok();

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
    info!("Server: http://0.0.0.0:4002");

    start_web_server(
        config.environment.tenant_data_path,
        config.environment.output_path,
        config.environment.templates_path,
        config.environment.database_path,
    )
    .await
}

