// src/core/service_client.rs
//! Unified HTTP service client - eliminates duplicate HTTP client code

use anyhow::{Context, Result};
use graflog::app_log;
use reqwest::multipart::{Form, Part};
use std::path::Path;
use std::time::Duration;

pub struct ServiceClient {
    client: reqwest::Client,
    base_url: String,
    // timeout: Duration,
}

impl ServiceClient {
    /// Create new service client with configuration
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            // timeout: Duration::from_secs(timeout_seconds),
        })
    }

    /// Upload CV file for conversion
    pub async fn upload_cv(&self, file_path: &Path, file_name: &str) -> Result<String> {
        let content_type = self.get_content_type(file_name)?;
        let url = format!("{}/upload-cv", self.base_url);

        let file_content = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let form = Form::new().part(
            "cv_file",
            Part::bytes(file_content)
                .file_name(file_name.to_string())
                .mime_str(content_type)
                .context("Failed to create multipart")?,
        );

        app_log!(info, "Calling CV conversion service: {}", url);

        let response = self
            .client
            .post(&url)
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
            self.parse_cv_response(&response_text)
        } else {
            anyhow::bail!(
                "Service returned error status {}: {}",
                status,
                response_text
            )
        }
    }

    /// Generic POST request with JSON
    pub async fn post_json<T, R>(&self, endpoint: &str, payload: &T) -> Result<R>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .post(&url)
            .json(payload)
            .send()
            .await
            .with_context(|| format!("Failed to POST to {}", url))?;

        let status = response.status();
        if status.is_success() {
            response
                .json::<R>()
                .await
                .context("Failed to parse JSON response")
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("HTTP {} error: {}", status, error_text)
        }
    }

    /// Generic GET request
    pub async fn get<R>(&self, endpoint: &str) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to GET from {}", url))?;

        let status = response.status();
        if status.is_success() {
            response
                .json::<R>()
                .await
                .context("Failed to parse JSON response")
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("HTTP {} error: {}", status, error_text)
        }
    }

    /// Get content type for file
    fn get_content_type(&self, file_name: &str) -> Result<&'static str> {
        let lower_name = file_name.to_lowercase();
        if lower_name.ends_with(".pdf") {
            Ok("application/pdf")
        } else if lower_name.ends_with(".docx") {
            Ok("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        } else {
            anyhow::bail!("Unsupported file format: {}", file_name)
        }
    }

    /// Parse CV service response
    fn parse_cv_response(&self, response_text: &str) -> Result<String> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct CvServiceResponse {
            typst_content: String,
            // success: Option<bool>,
            error: Option<String>,
            // code: Option<String>,
        }

        let response_body: CvServiceResponse =
            serde_json::from_str(response_text).with_context(|| {
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
    }
}
