// src/environment.rs
use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::info;

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
    pub fn load() -> Self {
        Self {
            job_matching_url: std::env::var("JOB_MATCHING_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5555".to_string()),
            timeout_seconds: std::env::var("SERVICE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

// Add to EnvironmentConfig
impl EnvironmentConfig {
    pub fn service_config(&self) -> ServiceConfig {
        ServiceConfig::load()
    }
}

impl EnvironmentConfig {
    pub fn load() -> Result<Self> {
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());

        info!("Loading environment configuration for: {}", environment);

        let config = match environment.as_str() {
            "production" => Self::production_config()?,
            _ => Self::local_config()?,
        };

        info!("Configuration loaded successfully");
        Ok(config)
    }

    fn production_config() -> Result<Self> {
        Ok(Self {
            tenant_data_path: Self::get_env_path("CVENOM_TENANT_DATA_PATH", "/app/data/tenants")?,
            output_path: Self::get_env_path("CVENOM_OUTPUT_PATH", "/app/data/output")?,
            templates_path: Self::get_env_path("CVENOM_TEMPLATES_PATH", "/app/templates")?,
            database_path: Self::get_env_path("CVENOM_DATABASE_PATH", "/app/data/database.db")?,
        })
    }

    fn local_config() -> Result<Self> {
        Ok(Self {
            tenant_data_path: Self::get_env_path("CVENOM_TENANT_DATA_PATH", "./data/tenants")?,
            output_path: Self::get_env_path("CVENOM_OUTPUT_PATH", "./out")?,
            templates_path: Self::get_env_path("CVENOM_TEMPLATES_PATH", "./templates")?,
            database_path: Self::get_env_path("CVENOM_DATABASE_PATH", "./data/database.db")?,
        })
    }

    fn get_env_path(env_var: &str, default: &str) -> Result<PathBuf> {
        let path_str = std::env::var(env_var).unwrap_or_else(|_| default.to_string());
        Ok(PathBuf::from(path_str))
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
            info!("Created directory: {}", path.display());
        }
        Ok(())
    }
}
