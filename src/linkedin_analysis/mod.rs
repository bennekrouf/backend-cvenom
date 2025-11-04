// src/linkedin_analysis/mod.rs
use serde::{Deserialize, Serialize};

pub mod job_analyzer;
pub mod types;

pub use job_analyzer::JobAnalyzer;
// pub use types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAnalysisRequest {
    pub job_url: String,
    pub person_name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContent {
    pub title: String,
    pub company: String,
    pub location: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMatchApiResponse {
    pub analysis: String,
}
