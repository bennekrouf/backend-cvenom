// src/linkedin_analysis/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContent {
    pub title: String,
    pub company: String,
    pub location: String,
    pub description: String,
}

impl JobContent {
    pub fn to_llm_prompt(&self) -> String {
        format!(
            "Job Title: {}\nCompany: {}\nLocation: {}\n\nJob Description:\n{}",
            self.title, self.company, self.location, self.description
        )
    }
}

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
pub struct SemanticRequest {
    pub messages: Vec<SemanticMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMessage {
    pub context: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResponse {
    pub message: String,
    pub usage: Option<SemanticUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticUsage {
    pub tokens: u32,
}
