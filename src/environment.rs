// src/environment.rs
use anyhow::{Context, Result};
use graflog::app_log;
use std::path::PathBuf;

#[derive(Clone)]
pub struct EnvironmentConfig {
    pub tenant_data_path: PathBuf,
    pub output_path: PathBuf,
    pub templates_path: PathBuf,
    pub database_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub job_matching_url: String,
    pub timeout_seconds: u64,
}

impl ServiceConfig {
    pub fn load() -> Result<Self> {
        let job_matching_url = std::env::var("JOB_MATCHING_API_URL")
            .context("JOB_MATCHING_API_URL environment variable is required")?;

        let timeout_seconds = std::env::var("SERVICE_TIMEOUT")
            .context("SERVICE_TIMEOUT environment variable is required")?
            .parse::<u64>()
            .context("SERVICE_TIMEOUT must be a valid number")?;

        Ok(Self {
            job_matching_url,
            timeout_seconds,
        })
    }
}

impl EnvironmentConfig {
    pub fn service_config(&self) -> Result<ServiceConfig> {
        ServiceConfig::load()
    }

    /// Load configuration from mandatory environment variables only
    pub fn load() -> Result<Self> {
        app_log!(
            info,
            "Loading environment configuration from environment variables"
        );

        let tenant_data_path = PathBuf::from(
            std::env::var("CVENOM_TENANT_DATA_PATH")
                .context("CVENOM_TENANT_DATA_PATH environment variable is required")?,
        );

        let output_path = PathBuf::from(
            std::env::var("CVENOM_OUTPUT_PATH")
                .context("CVENOM_OUTPUT_PATH environment variable is required")?,
        );

        let templates_path = PathBuf::from(
            std::env::var("CVENOM_TEMPLATES_PATH")
                .context("CVENOM_TEMPLATES_PATH environment variable is required")?,
        );

        let database_path = PathBuf::from(
            std::env::var("CVENOM_DATABASE_PATH")
                .context("CVENOM_DATABASE_PATH environment variable is required")?,
        );

        let config = Self {
            tenant_data_path,
            output_path,
            templates_path,
            database_path,
        };

        app_log!(info, "Configuration loaded successfully");
        app_log!(info, "Tenant data: {}", config.tenant_data_path.display());
        app_log!(info, "Output: {}", config.output_path.display());
        app_log!(info, "Templates: {}", config.templates_path.display());
        app_log!(info, "Database: {}", config.database_path.display());

        Ok(config)
    }

    pub async fn ensure_directories(&self) -> Result<()> {
        self.create_dir_if_not_exists(&self.tenant_data_path)
            .await
            .context("Failed to create tenant data directory")?;

        self.create_dir_if_not_exists(&self.output_path)
            .await
            .context("Failed to create output directory")?;

        self.create_dir_if_not_exists(&self.templates_path)
            .await
            .context("Failed to create templates directory")?;

        // Ensure database directory exists
        if let Some(db_dir) = self.database_path.parent() {
            self.create_dir_if_not_exists(&db_dir.to_path_buf())
                .await
                .context("Failed to create database directory")?;
        }

        Ok(())
    }

    async fn create_dir_if_not_exists(&self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
            app_log!(info, "Created directory: {}", path.display());
        }
        Ok(())
    }
}

