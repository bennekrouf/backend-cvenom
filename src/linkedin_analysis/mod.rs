// src/linkedin_analysis/mod.rs
pub mod job_analyzer;
pub mod job_scraper;
pub mod semantic_client;
pub mod types;

pub use job_analyzer::JobAnalyzer;
pub use types::*;
