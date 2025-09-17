// src/linkedin_analysis/semantic_client.rs
use super::types::{SemanticMessage, SemanticRequest, SemanticResponse};
use anyhow::{Context, Result};
use reqwest::Client;
use std::env;
use tracing::{error, info};

pub struct SemanticClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl SemanticClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("SEMANTIC_API_KEY")
            .context("SEMANTIC_API_KEY environment variable not set")?;

        let base_url =
            env::var("SEMANTIC_API_URL").unwrap_or_else(|_| "https://api0.ai".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }

    pub async fn analyze_job_fit(&self, job_content: &str, experiences: &str) -> Result<String> {
        let prompt = format!(
            r#"You are a career advisor analyzing job fit. 

JOB POSTING:
{}

CANDIDATE EXPERIENCES:
{}

TASK: Analyze how well the candidate's experiences align with this job posting. Provide:

1. **Key Strengths** - What aspects of the candidate's background directly match the job requirements
2. **Relevant Experience** - Specific experiences that demonstrate capability for this role  
3. **Potential Gaps** - Areas where additional development might be beneficial
4. **Overall Fit Score** - Rate 1-10 with brief justification
5. **Interview Tips** - 3 specific talking points to emphasize during interviews

Keep the analysis concise, actionable, and professional."#,
            job_content, experiences
        );

        self.send_completion("Job Fit Analysis", &prompt).await
    }

    pub async fn send_completion(&self, context: &str, content: &str) -> Result<String> {
        let request = SemanticRequest {
            messages: vec![SemanticMessage {
                context: context.to_string(),
                content: content.to_string(),
            }],
        };

        info!("Sending request to Semantic API: {}", context);

        let response = self
            .client
            .post(&format!("{}/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Semantic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Semantic API error {}: {}", status, error_text);
            anyhow::bail!("Semantic API returned error {}: {}", status, error_text);
        }

        let semantic_response: SemanticResponse = response
            .json()
            .await
            .context("Failed to parse Semantic API response")?;

        info!("Successfully received response from Semantic API");
        Ok(semantic_response.message)
    }
}
