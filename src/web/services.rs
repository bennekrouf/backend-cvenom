// src/web/services.rs
use crate::{environment::ServiceConfig, template_processor::TemplateProcessor, utils};
use anyhow::{Context, Result};
use rocket::serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::app_log;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CvServiceResponse {
    pub typst_content: String,
    pub success: Option<bool>,
    pub error: Option<String>,
    pub code: Option<String>,
}

pub struct CvConversionService {
    service_url: String,
    timeout_seconds: u64,
}

impl CvConversionService {
    pub fn new() -> Self {
        let config = ServiceConfig::load();
        Self {
            service_url: config.job_matching_url + "/api/v1/upload-cv",
            timeout_seconds: config.timeout_seconds,
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.service_url = url;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    fn get_content_type(file_name: &str) -> Result<&'static str> {
        let lower_name = file_name.to_lowercase();
        if lower_name.ends_with(".pdf") {
            Ok("application/pdf")
        } else if lower_name.ends_with(".docx") {
            Ok("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        } else {
            anyhow::bail!("Unsupported file format: {}", file_name)
        }
    }

    pub async fn convert(&self, file_path: &Path, file_name: &str) -> Result<String> {
        let content_type = Self::get_content_type(file_name)?;

        let file_content = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let form = reqwest::multipart::Form::new().part(
            "cv_file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.to_string())
                .mime_str(content_type)
                .context("Failed to create multipart")?,
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        app_log!(info, "Calling CV conversion service: {}", self.service_url);

        let response = client
            .post(&self.service_url)
            .multipart(form)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = response.status();
        app_log!(info, "Response status: {}", status);

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        app_log!(info, "Response body: {}", response_text);

        if status.is_success() {
            let response_body: CvServiceResponse = serde_json::from_str(&response_text)
                .with_context(|| {
                    format!(
                        "Failed to parse JSON response. Response was: {}",
                        response_text
                    )
                })?;

            if !response_body.typst_content.is_empty() {
                Ok(response_body.typst_content)
            } else {
                let error_msg = response_body
                    .error
                    .unwrap_or_else(|| "Empty typst_content in response".to_string());
                anyhow::bail!("{}", error_msg)
            }
        } else {
            anyhow::bail!(
                "Service returned error status {}: {}",
                status,
                response_text
            )
        }
    }

    pub async fn create_person_with_typst_content(
        &self,
        person_name: &str,
        typst_content: &str,
        tenant_data_dir: &PathBuf,
        templates_dir: &PathBuf,
    ) -> Result<()> {
        let person_dir = tenant_data_dir.join(person_name);
        utils::ensure_dir_exists(&person_dir).await?;

        // Create cv_params.toml using template
        self.create_cv_params(&person_dir, person_name, templates_dir)
            .await?;

        // Create experiences_en.typ with the converted Typst content
        utils::write_file_safe(&person_dir.join("experiences_en.typ"), typst_content).await?;

        // Create experiences_fr.typ with default template
        self.create_experiences_fr(&person_dir, templates_dir)
            .await?;

        // Create README
        self.create_readme(&person_dir, person_name).await?;

        app_log!(info, "Successfully created person: {}", person_name);
        Ok(())
    }

    async fn create_cv_params(
        &self,
        person_dir: &PathBuf,
        person_name: &str,
        templates_dir: &PathBuf,
    ) -> Result<()> {
        let person_template_path = templates_dir.join("person_template.toml");
        if person_template_path.exists() {
            let template_content = utils::read_file_safe(&person_template_path).await?;
            let mut vars = HashMap::new();
            vars.insert("name".to_string(), person_name.to_string());
            let processed_content = TemplateProcessor::process_variables(&template_content, &vars);
            utils::write_file_safe(&person_dir.join("cv_params.toml"), &processed_content).await?;
        } else {
            app_log!(warn, "Person template not found, creating basic cv_params.toml");
            let basic_config = format!(
                r#"[personal]
name = "{}"
title = ""
summary = ""
email = ""
phone = ""
linkedin = ""
github = ""
"#,
                person_name
            );
            utils::write_file_safe(&person_dir.join("cv_params.toml"), &basic_config).await?;
        }
        Ok(())
    }

    async fn create_experiences_fr(
        &self,
        person_dir: &PathBuf,
        templates_dir: &PathBuf,
    ) -> Result<()> {
        let experiences_template_path = templates_dir.join("experiences_template.typ");
        if experiences_template_path.exists() {
            let template_content = utils::read_file_safe(&experiences_template_path).await?;
            utils::write_file_safe(&person_dir.join("experiences_fr.typ"), &template_content)
                .await?;
        } else {
            app_log!(warn, "Experiences template not found, creating basic experiences_fr.typ");
            let basic_experiences = r#"// French experiences
#import "/template.typ": *

// Add your French experiences here
"#;
            utils::write_file_safe(&person_dir.join("experiences_fr.typ"), basic_experiences)
                .await?;
        }
        Ok(())
    }

    async fn create_readme(&self, person_dir: &PathBuf, person_name: &str) -> Result<()> {
        let readme_content = format!(
            r#"# CV for {}

This directory contains the CV configuration and experiences for {}.

## Files:
- `cv_params.toml`: Personal information and CV configuration
- `experiences_en.typ`: English version of professional experiences
- `experiences_fr.typ`: French version of professional experiences

## Usage:
Use the web interface or CLI to generate PDFs from these files.
"#,
            person_name, person_name
        );
        utils::write_file_safe(&person_dir.join("README.md"), &readme_content).await?;
        Ok(())
    }
}

