// src/workspace.rs
use crate::config::CvConfig;
use crate::core::TemplateEngine;
use graflog::app_log;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::{fs, process::Command};

pub struct WorkspaceManager<'a> {
    config: &'a CvConfig,
    template_engine: &'a TemplateEngine,
}

impl<'a> WorkspaceManager<'a> {
    pub fn new(config: &'a CvConfig, template_engine: &'a TemplateEngine) -> Self {
        Self {
            config,
            template_engine,
        }
    }

    pub async fn prepare_workspace(&self) -> Result<()> {
        app_log!(info, "Preparing workspace in tmp_workspace/...");

        let original_dir = std::env::current_dir().context("Failed to get current directory")?;

        let workspace_result = async || -> Result<()> {
            std::env::set_current_dir("tmp_workspace")
                .context("Failed to change to temporary workspace")?;

            self.copy_profile_files()?;
            self.copy_logo_files()?;

            // Copy shared Typst utilities into the workspace
            for shared_file in &["font_config.typ", "common.typ"] {
                let source = self.config.templates_dir.join(shared_file);
                if source.exists() {
                    fs::copy(&source, PathBuf::from(shared_file))?;
                }
            }

            self.prepare_template_files().await?;

            Ok(())
        };

        match workspace_result().await {
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

        // Validate by magic bytes only — the stored file is always "profile.png"
        // regardless of the original upload extension, so checking the filename
        // extension would incorrectly reject valid JPEG uploads.
        const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        const JPEG_SIGNATURE: &[u8] = &[0xFF, 0xD8, 0xFF];

        if !header.starts_with(PNG_SIGNATURE) && !header.starts_with(JPEG_SIGNATURE) {
            return Err("Invalid image file — only JPEG and PNG formats are supported".to_string());
        }

        Ok(())
    }

    fn copy_profile_files(&self) -> Result<()> {
        // Copy config (existing code)
        let config_source = self.config.profile_config_path();
        let config_dest = PathBuf::from("cv_params.toml");

        app_log!(info, "DEBUG: config_source = {}", config_source.display());
        app_log!(
            info,
            "DEBUG: config_source exists = {}",
            config_source.exists()
        );

        fs::copy(&config_source, &config_dest).context("Failed to copy profile config")?;

        // Copy experiences — optional: some document types (e.g. portfolio) don't use it
        let exp_source = self.config.profile_experiences_path();
        let exp_dest = PathBuf::from("experiences.typ");
        if exp_source.exists() {
            fs::copy(&exp_source, &exp_dest).context("Failed to copy profile experiences")?;
        } else {
            app_log!(info, "No experiences file found at {} — skipping (not required for this document type)", exp_source.display());
        }

        // Copy profile image with validation
        let profile_image_png = self.config.profile_image_path();

        app_log!(
            info,
            "DEBUG: Looking for image at: {}",
            profile_image_png.display()
        );
        app_log!(info, "DEBUG: Image exists: {}", profile_image_png.exists());

        // Resolve photo: profile-specific first, then tenant-level default
        let resolved_image = if profile_image_png.exists() {
            Some(profile_image_png)
        } else {
            let default_photo = self.config.data_dir_absolute().join("default_photo.png");
            if default_photo.exists() {
                app_log!(info, "No profile photo — using tenant default photo");
                Some(default_photo)
            } else {
                None
            }
        };

        if let Some(image_path) = resolved_image {
            // Validate the image before copying
            match self.validate_image_sync(&image_path) {
                Ok(_) => {
                    // The stored file is always named "profile.png" on disk but may
                    // contain JPEG bytes (uploaded as .jpg then saved under .png name).
                    // Typst decodes by extension, so copy with the real extension so
                    // that image("profile.jpg") / image("profile.png") uses the right codec.
                    let header = fs::read(&image_path).unwrap_or_default();
                    const JPEG_SIG: &[u8] = &[0xFF, 0xD8, 0xFF];
                    let dest_name = if header.starts_with(JPEG_SIG) { "profile.jpg" } else { "profile.png" };
                    let profile_dest = PathBuf::from(dest_name);
                    fs::copy(&image_path, &profile_dest)?;
                    app_log!(info, "✅ Copied valid profile image as {}", dest_name);
                }
                Err(error_msg) => {
                    app_log!(info, "❌ Skipping corrupted image: {}", error_msg);
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
        let profile_logo_source = self.config.profile_data_dir().join("company_logo.png");
        let brand_logo_source = self
            .config
            .brand_dir
            .as_ref()
            .map(|p| p.join("logo.png"));
        let logo_dest = PathBuf::from("company_logo.png");

        // Sniff the PNG magic bytes so a corrupted or wrong-format logo never
        // takes the whole compilation down — templates pin the filename to
        // `company_logo.png`, so typst aborts hard on a bad PNG. If the brand
        // logo is broken, fall through to profile / tenant / no-logo instead.
        let is_valid_png = |p: &std::path::Path| -> bool {
            const PNG_SIG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            match std::fs::File::open(p) {
                Ok(mut f) => {
                    use std::io::Read;
                    let mut buf = [0u8; 8];
                    matches!(f.read_exact(&mut buf), Ok(())) && buf == PNG_SIG
                }
                Err(_) => false,
            }
        };

        // Precedence: brand > profile > tenant. A brand was explicitly chosen
        // for this generation, so its logo should win when valid.
        if let Some(brand_logo) = brand_logo_source.as_ref().filter(|p| p.exists()) {
            if is_valid_png(brand_logo) {
                fs::copy(brand_logo, &logo_dest)?;
                app_log!(info, "Brand logo copied successfully");
                return Ok(());
            } else {
                app_log!(
                    warn,
                    "Brand logo at {:?} is not a valid PNG — skipping and falling back to profile/tenant logo",
                    brand_logo
                );
            }
        }
        if profile_logo_source.exists() && is_valid_png(&profile_logo_source) {
            fs::copy(&profile_logo_source, &logo_dest)?;
            app_log!(info, "Profile logo copied successfully");
        } else if tenant_logo_source.exists() && is_valid_png(&tenant_logo_source) {
            fs::copy(&tenant_logo_source, &logo_dest)?;
            app_log!(info, "Tenant logo copied successfully");
        }

        Ok(())
    }

    async fn prepare_template_files(&self) -> Result<()> {
        self.template_engine
            .prepare_template_workspace(&self.config.template, &PathBuf::from("."))
            .await
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
                self.config.profile_name,
                self.config.template.as_str(),
                self.config.lang
            ));

        let mut cmd = Command::new("typst");
        cmd.arg("compile").arg("main.typ").arg(&output_path);
        cmd.arg("--input").arg(format!("lang={}", self.config.lang));

        if PathBuf::from("company_logo.png").exists() {
            cmd.arg("--input").arg("company_logo.png=company_logo.png");
        }

        // Add picture input only if a valid image was copied to the workspace.
        // copy_profile_files() writes "profile.jpg" for JPEG content and
        // "profile.png" for PNG content so Typst uses the correct decoder.
        let workspace_pic = if PathBuf::from("profile.jpg").exists() {
            Some("profile.jpg")
        } else if PathBuf::from("profile.png").exists() {
            Some("profile.png")
        } else {
            None
        };

        if let Some(pic_file) = workspace_pic {
            app_log!(info, "✅ Adding picture input to Typst command: {}", pic_file);
            cmd.arg("--input").arg(format!("picture={}", pic_file));
        } else {
            app_log!(info, "ℹ️  No profile image in workspace - generating without photo");
        }

        // Forward branding to Typst as `--input k=v` flags. The resolver emits
        // only explicit overrides (and vibe-preset values); keys it omits fall
        // through to each template's literal defaults, so legacy profiles that
        // only set primary/secondary render unchanged.
        //
        // Source precedence:
        //   1. Selected brand's styling (when self.config.brand is Some)
        //   2. The profile's [styling] block in cv_params.toml
        // A brand is only attached when the caller explicitly picked one, so
        // there's no risk of silently switching styling on legacy callers.
        if self.config.use_custom_colors {
            let styling: Option<crate::web::handlers::cv_handlers::cv_data::StylingData> =
                if let Some(brand) = &self.config.brand {
                    Some(brand.styling.clone())
                } else if let Ok(toml_content) = fs::read_to_string("cv_params.toml") {
                    if let Ok(toml::Value::Table(table)) =
                        toml::from_str::<toml::Value>(&toml_content)
                    {
                        if let Some(styling_tbl) = table.get("styling").and_then(|v| v.as_table()) {
                            let str_at = |k: &str| -> String {
                                styling_tbl
                                    .get(k)
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            };
                            Some(crate::web::handlers::cv_handlers::cv_data::StylingData {
                                primary_color:    str_at("primary_color"),
                                secondary_color: str_at("secondary_color"),
                                show_photo: styling_tbl
                                    .get("show_photo")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                vibe:             str_at("vibe"),
                                accent_color:     str_at("accent_color"),
                                neutral_color:    str_at("neutral_color"),
                                background_tone:  str_at("background_tone"),
                                font_personality: str_at("font_personality"),
                                density:          str_at("density"),
                                layout:           str_at("layout"),
                                divider:          str_at("divider"),
                                header_style:     str_at("header_style"),
                                photo_shape:      str_at("photo_shape"),
                                icon_style:       str_at("icon_style"),
                                skill_style:      str_at("skill_style"),
                                date_style:       str_at("date_style"),
                                lang_style:       str_at("lang_style"),
                                label_tone:       str_at("label_tone"),
                                paper:            str_at("paper"),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let Some(styling) = styling {
                for (k, v) in crate::core::branding::resolve(&styling) {
                    cmd.arg("--input").arg(format!("{}={}", k, v));
                }
            }
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
