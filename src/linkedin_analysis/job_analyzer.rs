// src/linkedin_analysis/job_analyzer.rs
use super::{
    job_scraper::JobScraper,
    semantic_client::SemanticClient,
    types::{JobAnalysisRequest, JobAnalysisResponse},
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};

pub struct JobAnalyzer {
    scraper: JobScraper,
    semantic_client: SemanticClient,
}

impl JobAnalyzer {
    pub fn new() -> Result<Self> {
        let scraper = JobScraper::new();
        let semantic_client = SemanticClient::new().context(
            "Failed to initialize Semantic API client. Check SEMANTIC_API_KEY environment variable",
        )?;

        Ok(Self {
            scraper,
            semantic_client,
        })
    }

    pub async fn analyze_job_fit(
        &self,
        request: JobAnalysisRequest,
        tenant_data_dir: &PathBuf,
    ) -> JobAnalysisResponse {
        match self.perform_analysis(&request, tenant_data_dir).await {
            Ok(response) => response,
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
    ) -> Result<JobAnalysisResponse> {
        info!("Starting job analysis for person: {}", request.person_name);

        // Step 1: Scrape job content
        let job_content = self
            .scraper
            .extract_job_content(&request.job_url)
            .await
            .context("Failed to scrape job posting")?;

        let raw_job_content = job_content.to_llm_prompt();

        // Step 2: Load person's experiences
        let person_experiences = self
            .load_person_experiences(&request.person_name, tenant_data_dir)
            .await
            .context("Failed to load person's experiences")?;

        // Step 3: Analyze fit using Semantic API
        let fit_analysis = self
            .semantic_client
            .analyze_job_fit(&raw_job_content, &person_experiences)
            .await
            .context("Failed to analyze job fit")?;

        info!(
            "Job analysis completed successfully for {}",
            request.person_name
        );

        Ok(JobAnalysisResponse {
            success: true,
            job_content: Some(job_content),
            person_experiences: Some(person_experiences),
            fit_analysis: Some(fit_analysis),
            raw_job_content: Some(raw_job_content),
            error: None,
        })
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
}
