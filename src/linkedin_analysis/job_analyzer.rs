use super::{
    types::{JobMatchApiError, JobMatchApiRequest},
    JobAnalysisRequest, JobAnalysisResponse,
};
use crate::linkedin_analysis::JobContent;
use crate::linkedin_analysis::JobMatchApiResponse;
use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};

pub struct JobAnalyzer {
    client: Client,
    api_base_url: String,
}

impl JobAnalyzer {
    pub fn new() -> Result<Self> {
        // Get API URL from environment or use default
        let api_base_url = std::env::var("JOB_MATCHING_API_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_base_url,
        })
    }

    pub async fn analyze_job_fit(
        &self,
        request: JobAnalysisRequest,
        tenant_data_dir: &PathBuf,
    ) -> JobAnalysisResponse {
        match self.perform_analysis(&request, tenant_data_dir).await {
            Ok((analysis, person_experiences, job_content)) => JobAnalysisResponse {
                success: true,
                job_content: Some(job_content),
                person_experiences: Some(person_experiences),
                fit_analysis: Some(analysis),
                raw_job_content: Some(format!("Job URL: {}", request.job_url)),
                error: None,
            },
            Err(e) => {
                error!("Job analysis failed: {}", e);
                JobAnalysisResponse {
                    success: false,
                    job_content: None,
                    person_experiences: None,
                    fit_analysis: None,
                    raw_job_content: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    async fn perform_analysis(
        &self,
        request: &JobAnalysisRequest,
        tenant_data_dir: &PathBuf,
    ) -> Result<(String, String, JobContent)> {
        info!("Starting job analysis for person: {}", request.person_name);

        // Load person's experiences text (for the response)
        let person_experiences = self
            .load_person_experiences(&request.person_name, tenant_data_dir)
            .await
            .context("Failed to load person's experiences")?;

        // Load person's CV data as JSON (for the API call)
        let cv_json = self
            .load_person_cv_json(&request.person_name, tenant_data_dir)
            .await
            .context("Failed to load person's CV data")?;

        // Call the job matching API
        let analysis = self
            .call_job_matching_api(&request.job_url, &cv_json)
            .await
            .context("Failed to analyze job fit")?;

        // Create a placeholder job content since we're not scraping anymore
        let job_content = JobContent {
            title: "Job Position".to_string(),
            company: "Company".to_string(),
            location: "Location".to_string(),
            description: format!("Job posting from: {}", request.job_url),
        };

        info!(
            "Job analysis completed successfully for {}",
            request.person_name
        );

        Ok((analysis, person_experiences, job_content))
    }

    async fn load_person_experiences(
        &self,
        person_name: &str,
        tenant_data_dir: &PathBuf,
    ) -> Result<String> {
        let normalized_person = crate::utils::normalize_person_name(person_name);
        let person_dir = tenant_data_dir.join(&normalized_person);

        if !person_dir.exists() {
            anyhow::bail!(
                "Person directory not found: {}. Create the person first using the create endpoint.",
                person_dir.display()
            );
        }

        // Try to load English experiences first, then French as fallback
        let experience_files = ["experiences_en.typ", "experiences_fr.typ"];

        for file_name in &experience_files {
            let exp_path = person_dir.join(file_name);
            if exp_path.exists() {
                match fs::read_to_string(&exp_path).await {
                    Ok(content) => {
                        if !content.trim().is_empty() {
                            info!("Loaded experiences from: {}", file_name);
                            return Ok(content);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read {}: {}", file_name, e);
                    }
                }
            }
        }

        anyhow::bail!(
            "No valid experience files found for person: {}. Expected files: {}",
            person_name,
            experience_files.join(", ")
        );
    }

    async fn load_person_cv_json(
        &self,
        person_name: &str,
        tenant_data_dir: &PathBuf,
    ) -> Result<String> {
        let normalized_person = crate::utils::normalize_person_name(person_name);
        let person_dir = tenant_data_dir.join(&normalized_person);

        // Load CV parameters
        let cv_params_path = person_dir.join("cv_params.toml");
        let cv_params = if cv_params_path.exists() {
            match fs::read_to_string(&cv_params_path).await {
                Ok(content) => match toml::from_str::<toml::Value>(&content) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        warn!("Failed to parse cv_params.toml: {}", e);
                        None
                    }
                },
                Err(e) => {
                    warn!("Failed to read cv_params.toml: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Load work experience from experiences file
        let experience_files = ["experiences_en.typ", "experiences_fr.typ"];
        let mut work_experience_text = None;

        for file_name in &experience_files {
            let exp_path = person_dir.join(file_name);
            if exp_path.exists() {
                match fs::read_to_string(&exp_path).await {
                    Ok(content) => {
                        if !content.trim().is_empty() {
                            work_experience_text = Some(content);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read {}: {}", file_name, e);
                    }
                }
            }
        }

        // Create a simplified CV JSON structure
        let mut cv_data = serde_json::Map::new();

        // Extract key insights from CV params
        if let Some(params) = cv_params {
            if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
                cv_data.insert(
                    "name".to_string(),
                    serde_json::Value::String(name.to_string()),
                );
            }

            if let Some(job_title) = params.get("job_title").and_then(|v| v.as_str()) {
                cv_data.insert(
                    "job_title".to_string(),
                    serde_json::Value::String(job_title.to_string()),
                );
            }

            // Extract skills
            if let Some(skills) = params.get("skills").and_then(|v| v.as_table()) {
                let skills_summary: Vec<String> = skills
                    .iter()
                    .flat_map(|(category, items)| {
                        if let Some(items_array) = items.as_array() {
                            items_array
                                .iter()
                                .filter_map(|item| {
                                    item.as_str().map(|s| format!("{}: {}", category, s))
                                })
                                .collect::<Vec<_>>()
                        } else if let Some(item_str) = items.as_str() {
                            vec![format!("{}: {}", category, item_str)]
                        } else {
                            vec![]
                        }
                    })
                    .collect();

                if !skills_summary.is_empty() {
                    cv_data.insert(
                        "technical_skills".to_string(),
                        serde_json::Value::Array(
                            skills_summary
                                .into_iter()
                                .map(serde_json::Value::String)
                                .collect(),
                        ),
                    );
                }
            }
        }

        // Add work experience summary
        if let Some(exp_text) = work_experience_text {
            // Extract basic info from the Typst experience text
            cv_data.insert(
                "work_experience_summary".to_string(),
                serde_json::Value::String(exp_text.chars().take(2000).collect()),
            );
        }

        // Default key insights
        let key_insights = vec![
            "Experienced technical professional with proven track record".to_string(),
            "Expert in modern development technologies and methodologies".to_string(),
            "Strong background in project delivery and team collaboration".to_string(),
        ];
        cv_data.insert(
            "key_insights".to_string(),
            serde_json::Value::Array(
                key_insights
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );

        let cv_json =
            serde_json::to_string(&cv_data).context("Failed to serialize CV data to JSON")?;

        Ok(cv_json)
    }

    async fn call_job_matching_api(&self, job_url: &str, cv_json: &str) -> Result<String> {
        let api_url = format!("{}/api/v1/analyze-job-match", self.api_base_url);

        let request_body = JobMatchApiRequest {
            cv_json: cv_json.to_string(),
            job_url: job_url.to_string(),
        };

        info!("Calling job matching API: {}", api_url);

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to job matching API")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            // Try to parse as success response
            match serde_json::from_str::<JobMatchApiResponse>(&response_text) {
                Ok(api_response) => {
                    info!("Successfully received analysis from job matching API");
                    Ok(api_response.analysis)
                }
                Err(_) => {
                    // If JSON parsing fails, try to extract analysis from raw response
                    warn!("Failed to parse API response as JSON, using raw response");
                    Ok(response_text)
                }
            }
        } else {
            // Try to parse as error response
            let error_message = match serde_json::from_str::<JobMatchApiError>(&response_text) {
                Ok(error_response) => error_response.error,
                Err(_) => format!("API returned error {}: {}", status, response_text),
            };

            error!("Job matching API error: {}", error_message);
            anyhow::bail!("Job matching API error: {}", error_message);
        }
    }
}
