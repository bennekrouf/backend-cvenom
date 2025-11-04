// src/core/config_manager.rs
//! Unified configuration management - pure environment variables, no config.yaml

use anyhow::{Context, Result};
use graflog::app_log;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub environment: EnvironmentConfig,
    pub service: ServiceConfig,
    pub cv: Option<CvConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentConfig {
    pub tenant_data_path: PathBuf,
    pub output_path: PathBuf,
    pub templates_path: PathBuf,
    pub database_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub job_matching_url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct CvConfig {
    pub person_name: String,
    pub lang: String,
    pub template: String,
    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl ConfigManager {
    /// Load all configurations from environment variables only
    pub fn load() -> Result<Self> {
        let environment = Self::load_environment()?;
        let service = Self::load_service()?;

        Ok(Self {
            environment,
            service,
            cv: None,
        })
    }

    /// Load environment configuration from mandatory environment variables
    fn load_environment() -> Result<EnvironmentConfig> {
        app_log!(info, "Loading environment configuration from env vars");

        // All paths are now mandatory environment variables
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

        app_log!(info, "Tenant data path: {}", tenant_data_path.display());
        app_log!(info, "Output path: {}", output_path.display());
        app_log!(info, "Templates path: {}", templates_path.display());
        app_log!(info, "Database path: {}", database_path.display());

        Ok(EnvironmentConfig {
            tenant_data_path,
            output_path,
            templates_path,
            database_path,
        })
    }

    /// Load service configuration from mandatory environment variables
    fn load_service() -> Result<ServiceConfig> {
        let job_matching_url = std::env::var("JOB_MATCHING_API_URL")
            .context("JOB_MATCHING_API_URL environment variable is required")?;

        let timeout_seconds = std::env::var("SERVICE_TIMEOUT")
            .context("SERVICE_TIMEOUT environment variable is required")?
            .parse::<u64>()
            .context("SERVICE_TIMEOUT must be a valid number")?;

        app_log!(info, "Job matching URL: {}", job_matching_url);
        app_log!(info, "Service timeout: {} seconds", timeout_seconds);

        Ok(ServiceConfig {
            job_matching_url,
            timeout_seconds,
        })
    }

    /// Create CV configuration
    pub fn create_cv_config(
        &self,
        person_name: String,
        lang: String,
        template: Option<String>,
        data_dir: Option<PathBuf>,
        output_dir: Option<PathBuf>,
    ) -> CvConfig {
        CvConfig {
            person_name,
            lang,
            template: template.unwrap_or_else(|| "default".to_string()),
            data_dir: data_dir.unwrap_or_else(|| self.environment.tenant_data_path.clone()),
            templates_dir: self.environment.templates_path.clone(),
            output_dir: output_dir.unwrap_or_else(|| self.environment.output_path.clone()),
        }
    }

    /// Ensure all required directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        use crate::core::FsOps;

        FsOps::ensure_dir_exists(&self.environment.tenant_data_path).await?;
        FsOps::ensure_dir_exists(&self.environment.output_path).await?;
        FsOps::ensure_dir_exists(&self.environment.templates_path).await?;

        // Ensure database directory exists
        if let Some(db_parent) = self.environment.database_path.parent() {
            FsOps::ensure_dir_exists(db_parent).await?;
        }

        Ok(())
    }
}

impl CvConfig {
    /// Get person configuration file path
    pub fn person_config_path(&self) -> PathBuf {
        self.data_dir.join(&self.person_name).join("cv_params.toml")
    }

    /// Get person experiences file path
    pub fn person_experiences_path(&self) -> PathBuf {
        self.data_dir
            .join(&self.person_name)
            .join(format!("experiences_{}.typ", self.lang))
    }

    /// Get person image path
    pub fn person_image_path(&self) -> PathBuf {
        self.data_dir.join(&self.person_name).join("profile.png")
    }

    /// Get person data directory
    pub fn person_data_dir(&self) -> PathBuf {
        self.data_dir.join(&self.person_name)
    }

    /// Get absolute data directory
    pub fn data_dir_absolute(&self) -> PathBuf {
        self.data_dir.clone()
    }
}

