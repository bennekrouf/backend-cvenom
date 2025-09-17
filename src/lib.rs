// src/lib.rs
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub use web::start_web_server;
pub mod auth;
pub mod config;
pub mod database;
pub mod font_validator;
pub mod generator;
pub mod image_validator;
pub mod template_processor;
pub mod template_system;
pub mod utils;
pub mod web;
pub mod workspace;

pub mod linkedin_analysis;
// Re-export main types
pub use config::CvConfig;
pub use generator::CvGenerator;
pub use template_processor::TemplateProcessor;

/// List all available persons
pub fn list_persons(data_dir: &PathBuf) -> Result<Vec<String>> {
    let mut persons = Vec::new();

    if !data_dir.exists() {
        return Ok(persons);
    }

    let entries = fs::read_dir(data_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let config_path = path.join("cv_params.toml");
                if config_path.exists() {
                    persons.push(name.to_string());
                }
            }
        }
    }

    persons.sort();
    Ok(persons)
}

/// List all available templates  
pub fn list_templates(templates_dir: &PathBuf) -> Result<Vec<String>> {
    let template_manager = template_system::TemplateManager::new(templates_dir.clone())?;
    let templates = template_manager
        .list_templates()
        .iter()
        .map(|t| t.id.clone())
        .collect();
    Ok(templates)
}

/// Convenience function for quick CV generation
pub fn generate_cv(
    person_name: &str,
    lang: &str,
    template: Option<&str>,
    output_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let mut config = CvConfig::new(person_name, lang);

    if let Some(template_str) = template {
        config = config.with_template(template_str.to_string());
    }

    if let Some(dir) = output_dir {
        config = config.with_output_dir(dir);
    }

    let generator = CvGenerator::new(config)?;
    generator.generate()
}
