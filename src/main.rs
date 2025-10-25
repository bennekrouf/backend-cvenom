// src/main.rs
use anyhow::Result;
use cv_generator::{start_web_server, EnvironmentConfig};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::try_init().ok();

    // Load environment configuration
    let env_config = EnvironmentConfig::load()?;
    env_config.ensure_directories().await?;

    info!("Starting Multi-tenant CV Generator API Server");
    info!(
        "Environment: {}",
        std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string())
    );
    info!("Tenant Data: {}", env_config.tenant_data_path.display());
    info!("Database: {}", env_config.database_path.display());
    info!("Server: http://0.0.0.0:4002");

    start_web_server(
        env_config.tenant_data_path,
        env_config.output_path,
        env_config.templates_path,
        env_config.database_path,
    )
    .await
}

