// src/linkedin_analysis/job_scraper.rs
use super::types::JobContent;
use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{info, warn};

pub struct JobScraper {
    client: Client,
}

impl JobScraper {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn extract_job_content(&self, url: &str) -> Result<JobContent> {
        info!("Fetching job post: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch job post")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let html = response
            .text()
            .await
            .context("Failed to read response body")?;
        let document = Html::parse_document(&html);

        let content = self
            .parse_linkedin_job(&document)
            .or_else(|| self.parse_generic_job(&document))
            .context("Failed to extract job content from page")?;

        info!(
            "Successfully extracted job: {} at {}",
            content.title, content.company
        );
        Ok(content)
    }

    fn parse_linkedin_job(&self, document: &Html) -> Option<JobContent> {
        let title_selectors = [
            "h1.top-card-layout__title",
            ".job-details-jobs-unified-top-card__job-title",
            "h1[data-test-id='job-title']",
            ".jobs-unified-top-card__job-title",
        ];

        let company_selectors = [
            ".job-details-jobs-unified-top-card__company-name",
            ".top-card-layout__card .top-card-layout__second-subline",
            "a[data-test-id='job-poster-name']",
            ".jobs-unified-top-card__company-name",
        ];

        let description_selectors = [
            ".jobs-box__html-content",
            ".jobs-description__container",
            ".jobs-description-content__text",
            "[data-test-id='job-description']",
        ];

        let location_selectors = [
            ".job-details-jobs-unified-top-card__bullet",
            ".top-card-layout__card .top-card-layout__first-subline",
            "[data-test-id='job-location']",
            ".jobs-unified-top-card__bullet",
        ];

        let title = self.find_text_by_selectors(document, &title_selectors)?;
        let company = self.find_text_by_selectors(document, &company_selectors)?;
        let description = self.find_text_by_selectors(document, &description_selectors)?;
        let location = self
            .find_text_by_selectors(document, &location_selectors)
            .unwrap_or_default();

        Some(JobContent {
            title,
            company,
            location,
            description,
        })
    }

    fn parse_generic_job(&self, document: &Html) -> Option<JobContent> {
        warn!("Falling back to generic job parsing");

        let title_selectors = [
            "h1",
            "[class*='title']",
            "[class*='job-title']",
            "[class*='position']",
        ];

        let company_selectors = [
            "[class*='company']",
            "[class*='employer']",
            "[class*='organization']",
        ];

        let description_selectors = [
            "[class*='description']",
            "[class*='content']",
            "[class*='details']",
            "main",
            "article",
        ];

        let title = self.find_text_by_selectors(document, &title_selectors)?;
        let company = self
            .find_text_by_selectors(document, &company_selectors)
            .unwrap_or_default();
        let description = self.find_text_by_selectors(document, &description_selectors)?;

        Some(JobContent {
            title,
            company,
            location: String::new(),
            description,
        })
    }

    fn find_text_by_selectors(&self, document: &Html, selectors: &[&str]) -> Option<String> {
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = Self::clean_text(&element.text().collect::<Vec<_>>().join(" "));
                    if !text.is_empty() && text.len() > 5 {
                        return Some(text);
                    }
                }
            }
        }
        None
    }

    fn clean_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
