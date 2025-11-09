use std::env;

use anyhow::Result;
use cv_generator::{core::ConfigManager, start_web_server};
use graflog::app_log;
use graflog::init_logging;
use graflog::LogOption;

#[tokio::main]
async fn main() -> Result<()> {
    // if env::var("LOG_PATH_CVENOM").is_err() {
    //     eprintln!("Error: LOG_PATH_CVENOM environment variable is required");
    //     std::process::exit(1);
    // }

    let log_path =
        env::var("LOG_PATH_CVENOM").unwrap_or_else(|_| "/var/log/cvenom.log".to_string());
    init_logging!(&log_path, "cvenom", "backend", &[
        LogOption::Debug,
        LogOption::Custom("cvenom=debug".to_string()),
        LogOption::RocketOff
    ]);

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
