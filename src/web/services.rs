// src/web/services.rs
use rocket::serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CvServiceResponse {
    pub typst_content: String,
    // Make other fields optional since the external service might not return them
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
        // TODO: Make this configurable via environment variables
        Self {
            service_url: "http://127.0.0.1:6666/api/v1/upload-cv".to_string(),
            timeout_seconds: 30,
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

    pub async fn convert(&self, file_path: &Path, file_name: &str) -> Result<String, String> {
        let content_type = if file_name.to_lowercase().ends_with(".pdf") {
            "application/pdf"
        } else if file_name.to_lowercase().ends_with(".docx") {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        } else {
            return Err("Unsupported file format".to_string());
        };

        let file_content = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let form = reqwest::multipart::Form::new().part(
            "cv_file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.to_string())
                .mime_str(content_type)
                .map_err(|e| format!("Failed to create multipart: {}", e))?,
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        info!("Calling CV conversion service: {}", self.service_url);

        let response = client
            .post(&self.service_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        info!("Response status: {}", status);

        // Get the response text first for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        info!("Response body: {}", response_text);

        if status.is_success() {
            // Try to parse the JSON response
            let response_body: CvServiceResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    format!(
                        "Failed to parse JSON response: {}. Response was: {}",
                        e, response_text
                    )
                })?;

            // Since the external service only returns typst_content, check if it exists
            if !response_body.typst_content.is_empty() {
                Ok(response_body.typst_content)
            } else {
                let error_msg = response_body
                    .error
                    .unwrap_or_else(|| "Empty typst_content in response".to_string());
                Err(error_msg)
            }
        } else {
            Err(format!(
                "Service returned error status {}: {}",
                status, response_text
            ))
        }
    }

    pub async fn create_person_with_typst_content(
        &self,
        person_name: &str,
        typst_content: &str,
        tenant_data_dir: &PathBuf,
        templates_dir: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let person_dir = tenant_data_dir.join(person_name);
        tokio::fs::create_dir_all(&person_dir).await?;

        // Create cv_params.toml using template
        let person_template_path = templates_dir.join("person_template.toml");
        if person_template_path.exists() {
            let template_content = tokio::fs::read_to_string(&person_template_path).await?;

            let mut variables = HashMap::new();
            variables.insert("name".to_string(), person_name.to_string());

            // Simple template replacement
            let processed_content = template_content.replace("{{name}}", person_name);
            tokio::fs::write(person_dir.join("cv_params.toml"), processed_content).await?;
        }

        // Create experiences_en.typ with the converted Typst content
        tokio::fs::write(person_dir.join("experiences_en.typ"), typst_content).await?;

        // Create experiences_fr.typ with default template
        let experience_template_path = templates_dir.join("experiences_template.typ");
        if experience_template_path.exists() {
            let template_content = tokio::fs::read_to_string(&experience_template_path).await?;
            tokio::fs::write(person_dir.join("experiences_fr.typ"), template_content).await?;
        }

        // Create README
        let readme_content = format!(
            "# {} CV Data\n\nCV generated from uploaded file.\n\nAdd your profile image as `profile.png` in this directory.\n\nEdit the following files:\n- `cv_params.toml` - Personal information, skills, and key insights\n- `experiences_en.typ` - Work experience (converted from uploaded CV)\n- `experiences_fr.typ` - Work experience in French (template)\n",
            person_name
        );
        tokio::fs::write(person_dir.join("README.md"), readme_content).await?;

        info!("Created person directory with CV content: {}", person_name);
        Ok(())
    }
}
