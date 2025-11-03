// src/workspace.rs
use graflog::app_log;
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
        app_log!(info, "Preparing workspace in tmp_workspace/...");

        let original_dir = std::env::current_dir().context("Failed to get current directory")?;

        let workspace_result = || -> Result<()> {
            std::env::set_current_dir("tmp_workspace")
                .context("Failed to change to temporary workspace")?;

            self.copy_person_files()?;
            self.copy_logo_files()?;

            // ADD THESE 5 LINES:
            let font_config_source = self.config.templates_dir.join("font_config.typ");
            if font_config_source.exists() {
                let font_config_dest = PathBuf::from("font_config.typ");
                fs::copy(&font_config_source, &font_config_dest)?;
            }

            self.prepare_template_files()?;

            Ok(())
        };

        match workspace_result() {
            Ok(_) => {
                app_log!(info, "Workspace preparation completed successfully");
                Ok(())
            }
            Err(e) => {
                app_log!(warn, "Workspace preparation failed: {}", e);
                self.restore_directory_and_cleanup(&original_dir)?;
                Err(e)
            }
        }
    }

    fn validate_image_sync(&self, image_path: &PathBuf) -> Result<(), String> {
        let metadata = fs::metadata(image_path).map_err(|_| "Cannot read image file")?;

        if metadata.len() == 0 {
            return Err("Image file is empty".to_string());
        }

        let header = fs::read(image_path).map_err(|_| "Cannot read image file")?;

        if header.len() < 8 {
            return Err("Image file too small or corrupted".to_string());
        }

        let file_name = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.ends_with(".png") {
            // Check PNG signature
            const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            if !header.starts_with(PNG_SIGNATURE) {
                if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    return Err("File is JPEG but has .png extension".to_string());
                }
                return Err("Invalid PNG file - corrupted or wrong format".to_string());
            }
        } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
            // Check JPEG signature
            if !header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Err("Invalid JPEG file - corrupted or wrong format".to_string());
            }
        } else {
            return Err("Unsupported image format - use PNG or JPEG only".to_string());
        }

        Ok(())
    }

    fn copy_person_files(&self) -> Result<()> {
        // Copy config (existing code)
        let config_source = self.config.person_config_path();
        let config_dest = PathBuf::from("cv_params.toml");

        app_log!(info, "DEBUG: config_source = {}", config_source.display());
        app_log!(
            info,
            "DEBUG: config_source exists = {}",
            config_source.exists()
        );

        fs::copy(&config_source, &config_dest).context("Failed to copy person config")?;

        // Copy experiences (existing code)
        let exp_source = self.config.person_experiences_path();
        let exp_dest = PathBuf::from("experiences.typ");
        fs::copy(&exp_source, &exp_dest).context("Failed to copy person experiences")?;

        // Copy profile image with validation
        let person_image_png = self.config.person_image_path();

        app_log!(
            info,
            "DEBUG: Looking for image at: {}",
            person_image_png.display()
        );
        app_log!(info, "DEBUG: Image exists: {}", person_image_png.exists());

        if person_image_png.exists() {
            // Validate the image before copying
            match self.validate_image_sync(&person_image_png) {
                Ok(_) => {
                    let profile_dest = PathBuf::from("profile.png");
                    fs::copy(&person_image_png, &profile_dest)?;
                    app_log!(info, "✅ Copied valid profile image");
                }
                Err(error_msg) => {
                    app_log!(info, "❌ Skipping corrupted image: {}", error_msg);
                    // Don't copy the corrupted file - let CV generate without photo
                }
            }
        } else {
            app_log!(
                info,
                "No profile image found - CV will generate without photo"
            );
        }

        Ok(())
    }

    fn copy_logo_files(&self) -> Result<()> {
        let tenant_logo_source = self.config.data_dir_absolute().join("company_logo.png");
        let person_logo_source = self.config.person_data_dir().join("company_logo.png");
        let logo_dest = PathBuf::from("company_logo.png");

        if person_logo_source.exists() {
            fs::copy(&person_logo_source, &logo_dest)?;
            app_log!(info, "Person logo copied successfully");
        } else if tenant_logo_source.exists() {
            fs::copy(&tenant_logo_source, &logo_dest)?;
            app_log!(info, "Tenant logo copied successfully");
        }

        Ok(())
    }

    fn prepare_template_files(&self) -> Result<()> {
        self.template_manager
            .prepare_template_workspace(&self.config.template, &PathBuf::from("."))
            .context("Failed to prepare template workspace")?;

        app_log!(
            info,
            "Workspace prepared with template: {}",
            self.config.template
        );
        Ok(())
    }

    fn restore_directory_and_cleanup(&self, original_dir: &PathBuf) -> Result<()> {
        if let Err(restore_err) = std::env::set_current_dir(original_dir) {
            app_log!(
                warn,
                "Critical: Failed to restore directory after error: {}",
                restore_err
            );
        }

        if PathBuf::from("tmp_workspace").exists() {
            if let Err(cleanup_err) = fs::remove_dir_all("tmp_workspace") {
                app_log!(
                    warn,
                    "Warning: Failed to clean up workspace: {}",
                    cleanup_err
                );
            }
        }

        Ok(())
    }

    pub fn cleanup_workspace(&self) -> Result<()> {
        if let Err(e) = std::env::set_current_dir("..") {
            app_log!(
                warn,
                "Warning: Failed to change back to root directory: {}",
                e
            );
        }

        if PathBuf::from("tmp_workspace").exists() {
            if let Err(e) = fs::remove_dir_all("tmp_workspace") {
                app_log!(warn, "Warning: Failed to remove workspace: {}", e);
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

        // ONLY add picture input if file exists AND is valid
        if PathBuf::from("profile.png").exists() {
            app_log!(
                info,
                "DEBUG: profile.png exists in workspace, checking validity..."
            );
            if let Ok(header) = fs::read("profile.png") {
                let is_valid = if header.len() >= 8 {
                    header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) || // PNG
                header.starts_with(&[0xFF, 0xD8, 0xFF]) // JPEG
                } else {
                    false
                };

                if is_valid {
                    cmd.arg("--input").arg("picture=profile.png");
                    app_log!(info, "✅ Added valid picture input to Typst command");
                } else {
                    app_log!(info, "❌ Skipping invalid picture file");
                }
            }
        } else {
            app_log!(
                info,
                "ℹ️  No profile.png in workspace - generating without photo"
            );
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
