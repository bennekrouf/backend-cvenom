// src/lib.rs
use anyhow::{Context, Result};
use std::path::PathBuf;

pub use web::start_web_server;

pub mod auth;
pub mod config;
pub mod core; // Unified core module
pub mod environment;
pub mod font_validator;
pub mod generator;
pub mod image_validator;
pub mod linkedin_analysis;
pub mod types;
pub mod utils;
pub mod web;
pub mod workspace;

// Re-export main types for API compatibility
pub use config::CvConfig;
pub use core::{ConfigManager, Database, FsOps, TemplateEngine};
pub use environment::EnvironmentConfig;
pub use generator::CvGenerator;

/// List all available profiles - now uses core FsOps
pub fn list_profiles(data_dir: &PathBuf) -> Result<Vec<String>> {
    // Use async runtime for the async core function
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(FsOps::list_profiles(data_dir))
}

/// List all available templates - now uses core TemplateEngine
pub fn list_templates(templates_dir: &PathBuf) -> Result<Vec<String>> {
    let template_engine =
        TemplateEngine::new(templates_dir.clone()).context("Failed to create template engine")?;
    Ok(template_engine.list_templates())
}

/// Convenience function for quick CV generation - API unchanged
pub async fn generate_cv(
    profile_name: &str,
    lang: &str,
    template: Option<&str>,
    output_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let mut config = CvConfig::new(profile_name, lang);

    if let Some(template_str) = template {
        config = config.with_template(template_str.to_string());
    }

    if let Some(dir) = output_dir {
        config = config.with_output_dir(dir);
    }

    let generator = CvGenerator::new(config).context("Failed to create CV generator")?;
    generator.generate().await
}
