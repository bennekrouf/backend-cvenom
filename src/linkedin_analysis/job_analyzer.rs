use super::{types::JobMatchApiRequest, JobAnalysisRequest, JobAnalysisResponse};
use crate::linkedin_analysis::JobContent;
use crate::linkedin_analysis::JobMatchApiResponse;
use anyhow::{Context, Result};
use graflog::app_log;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;

pub struct JobAnalyzer {
    client: Client,
    job_matching_url: String,
    timeout_seconds: u64,
}

impl JobAnalyzer {
    /// Create new JobAnalyzer with configuration from environment variables only
    pub fn new() -> Result<Self> {
        let job_matching_url = std::env::var("JOB_MATCHING_API_URL")
            .context("JOB_MATCHING_API_URL environment variable is required")?;

        let timeout_seconds = std::env::var("SERVICE_TIMEOUT")
            .context("SERVICE_TIMEOUT environment variable is required")?
            .parse::<u64>()
            .context("SERVICE_TIMEOUT must be a valid number")?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        app_log!(
            info,
            "JobAnalyzer initialized with URL: {}",
            job_matching_url
        );
        app_log!(info, "Timeout: {} seconds", timeout_seconds);

        Ok(Self {
            client,
            job_matching_url,
            timeout_seconds,
        })
    }

    /// Analyze job fit for a person
    pub async fn analyze_job_fit(
        &self,
        request: JobAnalysisRequest,
        tenant_data_dir: &PathBuf,
    ) -> JobAnalysisResponse {
        app_log!(
            info,
            "Starting job analysis for person: {}",
            request.person_name
        );

        // Check if person directory exists
        let person_dir = tenant_data_dir.join(&request.person_name);
        if !person_dir.exists() {
            return JobAnalysisResponse {
                success: false,
                error: Some(format!(
                    "Person directory not found: {}",
                    request.person_name
                )),
                job_content: None,
                person_experiences: None,
                fit_analysis: None,
                raw_job_content: None,
            };
        }

        // Extract job content from LinkedIn URL
        let job_content = match self.extract_job_content(&request.job_url).await {
            Ok(content) => content,
            Err(e) => {
                app_log!(error, "Failed to extract job content: {}", e);
                return JobAnalysisResponse {
                    success: false,
                    error: Some(format!("Failed to extract job content: {}", e)),
                    job_content: None,
                    person_experiences: None,
                    fit_analysis: None,
                    raw_job_content: None,
                };
            }
        };

        // Read person's experiences
        let person_experiences = match self.read_person_experiences(&person_dir).await {
            Ok(exp) => exp,
            Err(e) => {
                app_log!(error, "Failed to read person experiences: {}", e);
                return JobAnalysisResponse {
                    success: false,
                    error: Some(format!("Failed to read person experiences: {}", e)),
                    job_content: Some(job_content),
                    person_experiences: None,
                    fit_analysis: None,
                    raw_job_content: None,
                };
            }
        };

        // Create JSON representation of CV data
        let cv_json = match self.create_cv_json(&person_dir, &person_experiences).await {
            Ok(json) => json,
            Err(e) => {
                app_log!(error, "Failed to create CV JSON: {}", e);
                return JobAnalysisResponse {
                    success: false,
                    error: Some(format!("Failed to process CV data: {}", e)),
                    job_content: Some(job_content.clone()),
                    person_experiences: Some(person_experiences),
                    fit_analysis: None,
                    raw_job_content: Some(job_content.description),
                };
            }
        };

        // Call job matching API
        match self.call_job_matching_api(cv_json, &request.job_url).await {
            Ok(fit_analysis) => JobAnalysisResponse {
                success: true,
                error: None,
                job_content: Some(job_content.clone()),
                person_experiences: Some(person_experiences),
                fit_analysis: Some(fit_analysis),
                raw_job_content: Some(job_content.description),
            },
            Err(e) => {
                app_log!(error, "Job matching API failed: {}", e);
                JobAnalysisResponse {
                    success: false,
                    error: Some(format!("Job matching analysis failed: {}", e)),
                    job_content: Some(job_content.clone()),
                    person_experiences: Some(person_experiences),
                    fit_analysis: None,
                    raw_job_content: Some(job_content.description),
                }
            }
        }
    }

    /// Extract job content from LinkedIn URL
    async fn extract_job_content(&self, job_url: &str) -> Result<JobContent> {
        app_log!(info, "Extracting job content from URL: {}", job_url);

        // Create a placeholder job content since we're not scraping anymore
        Ok(JobContent {
            title: "Job Position".to_string(),
            company: "Company Name".to_string(),
            description: format!("Job description from {}", job_url),
            location: "Location".to_string(),
        })
    }

    /// Read person's experiences from files
    async fn read_person_experiences(&self, person_dir: &PathBuf) -> Result<String> {
        let experiences_en = person_dir.join("experiences_en.typ");
        let experiences_fr = person_dir.join("experiences_fr.typ");

        let experiences = if experiences_en.exists() {
            fs::read_to_string(&experiences_en).await?
        } else if experiences_fr.exists() {
            fs::read_to_string(&experiences_fr).await?
        } else {
            return Err(anyhow::anyhow!("No experience files found"));
        };

        Ok(experiences)
    }

    /// Create JSON representation of CV data
    async fn create_cv_json(&self, person_dir: &PathBuf, experiences: &str) -> Result<String> {
        let cv_params_path = person_dir.join("cv_params.toml");
        let cv_params = if cv_params_path.exists() {
            fs::read_to_string(&cv_params_path).await?
        } else {
            "# No CV params found".to_string()
        };

        let cv_data = serde_json::json!({
            "cv_params": cv_params,
            "experiences": experiences,
            "person_dir": person_dir.display().to_string()
        });

        Ok(cv_data.to_string())
    }

    /// Call the job matching API
    async fn call_job_matching_api(&self, cv_json: String, job_url: &str) -> Result<String> {
        let api_request = JobMatchApiRequest {
            cv_json,
            job_url: job_url.to_string(),
        };

        let response = self
            .client
            .post(&self.job_matching_url)
            .json(&api_request)
            .send()
            .await
            .context("Failed to send request to job matching API")?;

        if response.status().is_success() {
            let api_response: JobMatchApiResponse = response
                .json()
                .await
                .context("Failed to parse job matching API response")?;

            Ok(api_response.analysis)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Job matching API error: {}", error_text))
        }
    }
}
