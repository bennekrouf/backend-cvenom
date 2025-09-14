// src/environment.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub tenant_data_path: PathBuf,
    pub output_path: PathBuf,
    pub templates_path: PathBuf,
    pub database_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    local: EnvironmentConfig,
    production: EnvironmentConfig,
}

impl EnvironmentConfig {
    /// Load configuration based on environment
    pub fn load() -> Result<Self> {
        let environment = Self::get_environment();
        info!("Loading configuration for environment: {}", environment);

        // Remove the fallback - require config.yaml to exist
        Self::load_from_file(&environment)
    }

    fn get_environment() -> String {
        std::env::var("CVENOM_ENV")
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .or_else(|_| std::env::var("ENV"))
            .unwrap_or_else(|_| "local".to_string())
    }

    fn load_from_file(environment: &str) -> Result<Self> {
        let config_path = PathBuf::from("config.yaml");
        if !config_path.exists() {
            anyhow::bail!("config.yaml not found in current directory. Server cannot start without configuration.");
        }

        let config_content =
            std::fs::read_to_string(&config_path).context("Failed to read config.yaml")?;

        let config_file: ConfigFile =
            serde_yaml::from_str(&config_content).context("Failed to parse config.yaml")?;

        let env_config = match environment {
            "production" => config_file.production,
            _ => config_file.local,
        };

        // Make paths absolute
        Ok(Self {
            tenant_data_path: Self::resolve_path(&env_config.tenant_data_path)?,
            output_path: Self::resolve_path(&env_config.output_path)?,
            templates_path: Self::resolve_path(&env_config.templates_path)?,
            database_path: Self::resolve_path(&env_config.database_path)?,
        })
    }

    fn resolve_path(path: &PathBuf) -> Result<PathBuf> {
        if path.is_absolute() {
            Ok(path.clone())
        } else {
            // For relative paths, resolve from current working directory
            let current_dir = std::env::current_dir().context("Failed to get current directory")?;
            Ok(current_dir.join(path))
        }
    }

    /// Ensure all configured directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            &self.tenant_data_path,
            &self.output_path,
            &self.templates_path,
        ];

        for dir in dirs {
            if let Some(parent) = dir.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
            if !dir.exists() && dir != &self.database_path {
                // Don't create database file, just parent dir
                tokio::fs::create_dir_all(dir)
                    .await
                    .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            }
        }

        // Ensure database parent directory exists
        if let Some(db_parent) = self.database_path.parent() {
            tokio::fs::create_dir_all(db_parent)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create database directory: {}",
                        db_parent.display()
                    )
                })?;
        }

        info!("All configured directories ensured to exist");
        Ok(())
    }
}
