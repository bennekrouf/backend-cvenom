use serde::{Deserialize, Serialize};

use crate::types::cv_data::CvJson;

#[derive(serde::Deserialize)]
pub struct ConversionResponse {
    pub typst_content: String, // English Typst content
    pub toml_content: String,  // TOML configuration
    pub status: String,
}

// ===== Service Response Types =====

#[derive(Debug, Serialize, Deserialize)]
pub struct CvConversionResponse {
    pub cv_data: CvJson,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobMatchResponse {
    pub analysis: String,
    pub score: Option<f64>,
    pub recommendations: Option<Vec<String>>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CvTranslationResponse {
    pub translated_cv: CvJson,
    pub target_language: String,
    pub status: String,
}

/// Keyword gap analysis returned by the cv-import service alongside the optimized CV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordAnalysis {
    pub job_title: String,
    pub company: String,
    pub required_skills: Vec<String>,
    pub preferred_skills: Vec<String>,
    pub keywords: Vec<String>,
    pub experience_level: String,
    pub key_responsibilities: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub missing_keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CvOptimizationResponse {
    pub optimized_cv: CvJson,
    pub job_title: String,
    pub company_name: String,
    pub optimizations: Option<Vec<String>>,
    /// Keyword analysis from the two-pass LLM pipeline (may be absent on older service versions)
    pub keyword_analysis: Option<KeywordAnalysis>,
    pub status: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct OptimizeResponse {
    pub optimized_typst: String,
    pub job_title: String,
    pub company_name: String,
    pub optimizations: Option<Vec<String>>,
    pub keyword_analysis: Option<KeywordAnalysis>,
    /// Whether the optimized profile was saved back to disk
    pub saved: bool,
    pub status: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct TranslateResponse {
    pub translated_content: String,
    pub status: String,
}
