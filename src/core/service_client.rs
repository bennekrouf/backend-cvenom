// src/core/service_client.rs
//! Unified HTTP service client - uses JSON format for all cv-import interactions

use anyhow::{Context, Result};
use graflog::app_log;
use reqwest::multipart::{Form, Part};
use std::path::Path;

use crate::types::{
    cv_data::CvJson,
    response::{
        CvConversionResponse, CvOptimizationResponse, CvTranslationResponse, JobMatchResponse,
    },
};

const UPLOAD_CV_ENDPOINT: &str = "/upload-cv";
const JOBS_MATCH_ENDPOINT: &str = "/jobs-match";
const TRANSLATE_ENDPOINT: &str = "/translate";
const OPTIMIZE_ENDPOINT: &str = "/optimize";
const COVER_LETTER_ENDPOINT: &str = "/cover-letter";

const DEFAULT_TIMEOUT_SECS: u64 = 400;

pub struct ServiceClient {
    client: reqwest::Client,
    base_url: String,
}

impl ServiceClient {
    /// Create new service client with configuration
    pub fn new(base_url: String, _timeout_seconds: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, base_url })
    }

    /// 1. CV Upload/Conversion - sends file, receives CvJson
    pub async fn upload_cv(&self, file_path: &Path, file_name: &str) -> Result<CvJson> {
        let content_type = self.get_content_type(file_name)?;
        let url = format!("{}{}", self.base_url, UPLOAD_CV_ENDPOINT);

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
        app_log!(trace, "Response status: {}", status);

        if status.is_success() {
            let response_text = response
                .text()
                .await
                .context("Failed to read response text")?;

            app_log!(info, "Raw CV service response (first 500 chars): {}", &response_text[..response_text.len().min(500)]);

            // Parse as a generic JSON value first so we can check "status"
            // before attempting to deserialize the full cv_data structure.
            // This avoids failures when cv-import returns an error response
            // whose cv_data shape doesn't exactly match CvJson.
            let raw: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| format!("CV service returned non-JSON response: {}", response_text))?;

            let svc_status = raw.get("status").and_then(|v| v.as_str()).unwrap_or("error");

            if svc_status == "success" {
                // Only deserialize cv_data on success
                let cv_data: CvJson = serde_json::from_value(
                    raw.get("cv_data").cloned().unwrap_or(serde_json::Value::Null)
                ).with_context(|| format!("Failed to deserialize cv_data from response: {}", response_text))?;
                Ok(cv_data)
            } else {
                let detail = raw.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("CV conversion failed")
                    .to_string();
                app_log!(error, "CV service returned error: {}", detail);
                anyhow::bail!("{}", detail)
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            app_log!(error, "CV service HTTP error {}: {}", status, error_text);
            anyhow::bail!("CV service error (HTTP {}): {}", status, error_text)
        }
    }

    /// 2. Job Matching - sends CvJson + job_url, receives analysis
    pub async fn match_job(&self, cv_data: &CvJson, job_url: &str) -> Result<JobMatchResponse> {
        let url = format!("{}{}", self.base_url, JOBS_MATCH_ENDPOINT);

        let payload = serde_json::json!({
            "cv_data": cv_data,
            "job_url": job_url
        });

        app_log!(trace, "Calling job matching service: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call job matching service")?;

        let status = response.status();
        if status.is_success() {
            let match_response: JobMatchResponse = response
                .json()
                .await
                .context("Failed to parse job match response")?;
            Ok(match_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Job matching failed with status {}: {}", status, error_text)
        }
    }

    /// 3. CV Translation - sends CvJson, receives translated CvJson
    pub async fn translate_cv(&self, cv_data: &CvJson, target_lang: &str) -> Result<CvJson> {
        let url = format!("{}{}", self.base_url, TRANSLATE_ENDPOINT);

        let payload = serde_json::json!({
            "cv_data": cv_data,
            "target_language": target_lang
        });

        app_log!(trace, "Calling CV translation service: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call translation service")?;

        let status = response.status();
        if status.is_success() {
            let translation_response: CvTranslationResponse = response
                .json()
                .await
                .context("Failed to parse translation response")?;

            if translation_response.status == "success" {
                Ok(translation_response.translated_cv)
            } else {
                anyhow::bail!("Translation failed: {}", translation_response.status)
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Translation failed with status {}: {}", status, error_text)
        }
    }

    /// 4. CV Optimization - sends CvJson + job_url, receives optimized CvJson
    pub async fn optimize_cv(
        &self,
        cv_data: &CvJson,
        job_url: &str,
        job_description: Option<&str>,
    ) -> Result<CvOptimizationResponse> {
        let url = format!("{}{}", self.base_url, OPTIMIZE_ENDPOINT);

        let mut payload = serde_json::json!({
            "cv_data": cv_data,
            "job_url": job_url
        });
        if let Some(desc) = job_description {
            payload["job_description"] = serde_json::Value::String(desc.to_string());
        }

        app_log!(trace, "Calling CV optimization service: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call optimization service")?;

        let status = response.status();
        if status.is_success() {
            let optimization_response: CvOptimizationResponse = response
                .json()
                .await
                .context("Failed to parse optimization response")?;
            if optimization_response.status.starts_with("error:") {
                anyhow::bail!("{}", optimization_response.status.trim_start_matches("error: "));
            }
            Ok(optimization_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Optimization failed with status {}: {}", status, error_text)
        }
    }

    /// 5. Cover Letter Generation - sends CvJson + job_description + lang, receives cover letter text
    pub async fn generate_cover_letter(
        &self,
        cv_data: &CvJson,
        job_description: &str,
        lang: &str,
    ) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct CoverLetterServiceResponse {
            cover_letter: String,
            status: String,
        }

        let payload = serde_json::json!({
            "cv_data": cv_data,
            "job_description": job_description,
            "lang": lang
        });

        let url = format!("{}{}", self.base_url, COVER_LETTER_ENDPOINT);
        app_log!(trace, "Calling cover letter service: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to call cover letter service")?;

        let status = response.status();
        if status.is_success() {
            let resp: CoverLetterServiceResponse = response
                .json()
                .await
                .context("Failed to parse cover letter response")?;
            if resp.status.starts_with("error") {
                anyhow::bail!("{}", resp.status);
            }
            Ok(resp.cover_letter)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Cover letter generation failed with status {}: {}", status, error_text)
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
}

