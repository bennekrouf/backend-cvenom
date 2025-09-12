// src/workspace.rs
use crate::config::CvConfig;
use crate::template_system::TemplateManager;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::{fs, process::Command};

pub struct WorkspaceManager<'a> {
    config: &'a CvConfig,
    template_manager: &'a TemplateManager,
}

impl<'a> WorkspaceManager<'a> {
    pub fn new(config: &'a CvConfig, template_manager: &'a TemplateManager) -> Self {
        Self {
            config,
            template_manager,
        }
    }

    pub fn prepare_workspace(&self) -> Result<()> {
        println!("Preparing workspace in tmp_workspace/...");

        let original_dir = std::env::current_dir().context("Failed to get current directory")?;

        let workspace_result = || -> Result<()> {
            std::env::set_current_dir("tmp_workspace")
                .context("Failed to change to temporary workspace")?;

            self.copy_person_files()?;
            self.copy_logo_files()?;
            self.prepare_template_files()?;

            Ok(())
        };

        match workspace_result() {
            Ok(_) => {
                println!("Workspace preparation completed successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("Workspace preparation failed: {}", e);
                self.restore_directory_and_cleanup(&original_dir)?;
                Err(e)
            }
        }
    }

    fn copy_person_files(&self) -> Result<()> {
        // Copy config
        let config_source = self.config.person_config_path();
        let config_dest = PathBuf::from("cv_params.toml");

        // Debug output
        println!("DEBUG: config_source = {}", config_source.display());
        println!("DEBUG: config_source exists = {}", config_source.exists());
        println!("DEBUG: current dir = {:?}", std::env::current_dir());

        fs::copy(&config_source, &config_dest).context("Failed to copy person config")?;

        // Copy experiences
        let exp_source = self.config.person_experiences_path();
        let exp_dest = PathBuf::from("experiences.typ");
        fs::copy(&exp_source, &exp_dest).context("Failed to copy person experiences")?;

        // Copy profile image if exists
        let person_image_png = self.config.person_image_path();
        if person_image_png.exists() {
            let profile_dest = PathBuf::from("profile.png");
            fs::copy(&person_image_png, &profile_dest)?;
            println!("Copied profile image");
        }

        Ok(())
    }

    fn copy_logo_files(&self) -> Result<()> {
        let tenant_logo_source = self.config.data_dir_absolute().join("company_logo.png");
        let person_logo_source = self.config.person_data_dir().join("company_logo.png");
        let logo_dest = PathBuf::from("company_logo.png");

        if person_logo_source.exists() {
            fs::copy(&person_logo_source, &logo_dest)?;
            println!("Person logo copied successfully");
        } else if tenant_logo_source.exists() {
            fs::copy(&tenant_logo_source, &logo_dest)?;
            println!("Tenant logo copied successfully");
        }

        Ok(())
    }

    fn prepare_template_files(&self) -> Result<()> {
        self.template_manager
            .prepare_template_workspace(&self.config.template, &PathBuf::from("."))
            .context("Failed to prepare template workspace")?;

        println!("Workspace prepared with template: {}", self.config.template);
        Ok(())
    }

    fn restore_directory_and_cleanup(&self, original_dir: &PathBuf) -> Result<()> {
        if let Err(restore_err) = std::env::set_current_dir(original_dir) {
            eprintln!(
                "Critical: Failed to restore directory after error: {}",
                restore_err
            );
        }

        if PathBuf::from("tmp_workspace").exists() {
            if let Err(cleanup_err) = fs::remove_dir_all("tmp_workspace") {
                eprintln!("Warning: Failed to clean up workspace: {}", cleanup_err);
            }
        }

        Ok(())
    }

    pub fn cleanup_workspace(&self) -> Result<()> {
        if let Err(e) = std::env::set_current_dir("..") {
            eprintln!("Warning: Failed to change back to root directory: {}", e);
        }

        if PathBuf::from("tmp_workspace").exists() {
            if let Err(e) = fs::remove_dir_all("tmp_workspace") {
                eprintln!("Warning: Failed to remove workspace: {}", e);
            }
        }

        Ok(())
    }

    pub fn compile_cv(&self) -> Result<PathBuf> {
        let output_path = PathBuf::from("..")
            .join(&self.config.output_dir)
            .join(format!(
                "{}_{}_{}.pdf",
                self.config.person_name,
                self.config.template.as_str(),
                self.config.lang
            ));

        let mut cmd = Command::new("typst");
        cmd.arg("compile").arg("main.typ").arg(&output_path);
        cmd.arg("--input").arg(format!("lang={}", self.config.lang));

        if PathBuf::from("company_logo.png").exists() {
            cmd.arg("--input").arg("company_logo.png=company_logo.png");
        }

        if PathBuf::from("profile.png").exists() {
            cmd.arg("--input").arg("picture=profile.png");
        }

        let output = cmd.output().context("Failed to execute typst command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Typst compilation failed: stderr={}, stdout={}",
                stderr,
                stdout
            );
        }

        Ok(output_path)
    }
}
