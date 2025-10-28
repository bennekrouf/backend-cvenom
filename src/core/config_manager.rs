// src/core/config_manager.rs
//! Unified configuration management - eliminates duplicate config loading

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

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
    /// Load all configurations
    pub fn load() -> Result<Self> {
        let environment = Self::load_environment()?;
        let service = Self::load_service()?;

        Ok(Self {
            environment,
            service,
            cv: None,
        })
    }

    /// Load environment configuration
    fn load_environment() -> Result<EnvironmentConfig> {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
        info!("Loading environment configuration for: {}", env);

        let base_dir = if env == "production" {
            PathBuf::from("/app")
        } else {
            std::env::current_dir().context("Failed to get current directory")?
        };

        Ok(EnvironmentConfig {
            tenant_data_path: base_dir.join("data"),
            output_path: base_dir.join("out"),
            templates_path: base_dir.join("templates"),
            database_path: base_dir.join("cv_generator.db"),
        })
    }

    /// Load service configuration
    fn load_service() -> Result<ServiceConfig> {
        let job_matching_url = std::env::var("JOB_MATCHING_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:5555".to_string());

        Ok(ServiceConfig {
            job_matching_url,
            timeout_seconds: 30,
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
