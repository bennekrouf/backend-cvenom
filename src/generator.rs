// src/generator.rs
use crate::config::CvConfig;
use chrono::Utc;

use crate::core::TemplateEngine;
use crate::template_processor::TemplateProcessor;
use crate::workspace::WorkspaceManager;
use anyhow::{Context, Result};
use graflog::app_log;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::path::PathBuf;
use std::{fs, process::Command};

fn sanitize_filename(input: &str) -> String {
    utf8_percent_encode(input, NON_ALPHANUMERIC)
        .to_string()
        .replace("%20", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub struct CvGenerator {
    pub config: CvConfig,
    template_manager: TemplateEngine,
}

impl CvGenerator {
    pub fn new(mut config: CvConfig) -> Result<Self> {
        let template_manager = TemplateEngine::new(config.templates_dir.clone())
            .context("Failed to initialize template manager")?;

        // Validate and normalize template
        config.template = normalize_template_for_generator(&config.template, &template_manager);

        // Validate person directory exists
        let person_dir = config.person_data_dir();
        if !person_dir.exists() {
            anyhow::bail!(
                "Person directory not found: {}. Create it with required files.",
                person_dir.display()
            );
        }

        // Validate experiences file exists
        let experiences_path = config.person_experiences_path();
        if !experiences_path.exists() {
            anyhow::bail!("Experiences file not found: {}", experiences_path.display());
        }

        Ok(Self {
            config,
            template_manager,
        })
    }

    pub async fn generate(&self) -> Result<PathBuf> {
        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace().await?;

        let output_path = workspace.compile_cv()?;
        workspace.cleanup_workspace()?;

        app_log!(
            info,
            "✅ Successfully compiled CV for {} ({} template, {} lang) to {}",
            self.config.person_name,
            self.config.template,
            self.config.lang,
            output_path.display()
        );

        Ok(output_path)
    }

    pub async fn generate_pdf_data(&self) -> Result<(Vec<u8>, String)> {
        // Generate filename using available data
        let filename = format!(
            "{}_CV_{}.pdf",
            sanitize_filename(&self.config.person_name),
            Utc::now().format("%Y")
        );

        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace().await?;

        let output_path = workspace.compile_cv()?;
        let pdf_data = fs::read(&output_path).context("Failed to read generated PDF")?;

        workspace.cleanup_workspace()?;

        Ok((pdf_data, filename))
    }

    pub async fn watch(&self) -> Result<()> {
        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace().await?;

        let output_path = self.config.output_dir.join(format!(
            "{}_{}_{}.pdf",
            self.config.person_name, self.config.template, self.config.lang
        ));

        let status = Command::new("typst")
            .arg("watch")
            .arg("main.typ")
            .arg(&output_path)
            .status()
            .context("Failed to execute typst watch command")?;

        if !status.success() {
            anyhow::bail!("Typst watch failed");
        }

        Ok(())
    }

    pub fn create_person_unchecked(&self) -> Result<()> {
        let template_processor = TemplateProcessor::new(self.config.templates_dir.clone());
        template_processor.create_person_from_templates(
            &self.config.person_name,
            &self.config.data_dir,
            Some(&self.config.person_name),
        )?;

        app_log!(
            info,
            "Created person directory structure for: {}",
            self.config.person_name
        );
        Ok(())
    }

    fn setup_output_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.config.output_dir).context("Failed to create output directory")?;
        fs::create_dir_all("tmp_workspace").context("Failed to create temporary workspace")?;
        Ok(())
    }
}

fn normalize_template_for_generator(template: &str, template_manager: &TemplateEngine) -> String {
    let requested = template.to_lowercase();
    for available_template in template_manager.list_templates() {
        if available_template.to_lowercase() == requested {
            return available_template.to_lowercase(); // Force lowercase return
        }
    }
    "default".to_string()
}
