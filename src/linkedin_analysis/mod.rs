// src/linkedin_analysis/mod.rs
pub mod job_analyzer;
pub mod types;

// src/linkedin_analysis/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAnalysisRequest {
    pub job_url: String,
    pub person_name: String,
}

// Simple job content placeholder since we're not scraping anymore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContent {
    pub title: String,
    pub company: String,
    pub location: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAnalysisResponse {
    pub success: bool,
    pub job_content: Option<JobContent>,
    pub person_experiences: Option<String>,
    pub fit_analysis: Option<String>,
    pub raw_job_content: Option<String>,
    pub error: Option<String>,
}

// Internal response format from the job matching API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobMatchApiResponse {
    pub analysis: String,
}
