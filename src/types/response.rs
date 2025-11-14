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

#[derive(Debug, Serialize, Deserialize)]
pub struct CvOptimizationResponse {
    pub optimized_cv: CvJson,
    pub job_title: String,
    pub company_name: String,
    pub optimizations: Option<Vec<String>>,
    pub status: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct OptimizeResponse {
    pub optimized_typst: String,
    pub job_title: String,
    pub company_name: String,
    pub status: String,
}

#[derive(serde::Deserialize, Serialize)]
pub struct TranslateResponse {
    pub translated_content: String,
    pub status: String,
}
