// src/generator.rs
use crate::app_log;
use crate::config::CvConfig;
use crate::template_processor::TemplateProcessor;
use crate::template_system::TemplateManager;
use crate::workspace::WorkspaceManager;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::{fs, process::Command};

pub struct CvGenerator {
    pub config: CvConfig,
    template_manager: TemplateManager,
}

impl CvGenerator {
    pub fn new(mut config: CvConfig) -> Result<Self> {
        let template_manager = TemplateManager::new(config.templates_dir.clone())
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

    pub fn generate(&self) -> Result<PathBuf> {
        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace()?;

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

    pub fn generate_pdf_data(&self) -> Result<Vec<u8>> {
        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace()?;

        let output_path = workspace.compile_cv()?;
        let pdf_data = fs::read(&output_path).context("Failed to read generated PDF")?;

        workspace.cleanup_workspace()?;

        Ok(pdf_data)
    }

    pub fn watch(&self) -> Result<()> {
        self.setup_output_dir()?;

        let workspace = WorkspaceManager::new(&self.config, &self.template_manager);
        workspace.prepare_workspace()?;

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

fn normalize_template_for_generator(template: &str, template_manager: &TemplateManager) -> String {
    let requested = template.to_lowercase();
    for available_template in template_manager.list_templates() {
        if available_template.id.to_lowercase() == requested {
            return available_template.id.to_lowercase(); // Force lowercase return
        }
    }
    "default".to_string()
}
