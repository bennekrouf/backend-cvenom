// src/font_validator.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use graflog::app_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRequirement {
    pub name: String,
    pub display_name: String,
    pub required: bool,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontValidationConfig {
    pub fonts: Vec<FontRequirement>,
    pub validation_enabled: bool,
    pub strict_mode: bool, // If true, fail on any missing required font
}

impl Default for FontValidationConfig {
    fn default() -> Self {
        Self {
            fonts: vec![
                FontRequirement {
                    name: "Font Awesome 7 Brands".to_string(),
                    display_name: "FontAwesome Brands".to_string(),
                    required: true,
                    alternatives: vec!["Font Awesome 5 Brands".to_string()],
                },
                FontRequirement {
                    name: "Font Awesome 7 Free".to_string(),
                    display_name: "FontAwesome Solid".to_string(),
                    required: true,
                    alternatives: vec!["Font Awesome 5 Free Solid".to_string()],
                },
                FontRequirement {
                    name: "Carlito".to_string(),
                    display_name: "Carlito (body font)".to_string(),
                    required: true,
                    alternatives: vec!["Arial".to_string(), "Helvetica".to_string()],
                },
            ],
            validation_enabled: true,
            strict_mode: false,
        }
    }
}

pub struct FontValidator {
    config: FontValidationConfig,
    available_fonts: Vec<String>,
}

#[derive(Debug)]
pub struct FontValidationResult {
    pub valid: bool,
    pub missing_fonts: Vec<String>,
    pub available_alternatives: HashMap<String, Vec<String>>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl FontValidator {
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self> {
        let config = if let Some(path) = config_path {
            Self::load_config(&path).await?
        } else {
            // Use same default location pattern as config.yaml
            let default_path = PathBuf::from("font_validation.yaml");
            if default_path.exists() {
                Self::load_config(&default_path).await?
            } else {
                FontValidationConfig::default()
            }
        };

        let available_fonts = Self::get_system_fonts().await?;

        Ok(Self {
            config,
            available_fonts,
        })
    }

    async fn load_config(path: &PathBuf) -> Result<FontValidationConfig> {
        if !path.exists() {
            app_log!(info, 
                "Font validation config not found at {}, using defaults",
                path.display()
            );
            return Ok(FontValidationConfig::default());
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read font validation config")?;

        let config: FontValidationConfig =
            serde_yaml::from_str(&content).context("Failed to parse font validation config")?;

        app_log!(info, "Loaded font validation config from {}", path.display());
        Ok(config)
    }

    async fn get_system_fonts() -> Result<Vec<String>> {
        app_log!(info, "Detecting system fonts...");

        // Try different methods based on OS
        if cfg!(target_os = "macos") {
            Self::get_macos_fonts().await
        } else if cfg!(target_os = "linux") {
            Self::get_linux_fonts().await
        } else if cfg!(target_os = "windows") {
            Self::get_windows_fonts().await
        } else {
            app_log!(warn, "Unsupported OS for font detection");
            Ok(vec![])
        }
    }

    async fn get_macos_fonts() -> Result<Vec<String>> {
        let output = Command::new("fc-list")
            .arg("--format=%{family}\n")
            .output()
            .or_else(|_| {
                // Fallback to system_profiler if fc-list not available
                Command::new("system_profiler")
                    .args(&["SPFontsDataType", "-json"])
                    .output()
            })?;

        if output.status.success() {
            let fonts_str = String::from_utf8_lossy(&output.stdout);
            let fonts: Vec<String> = fonts_str
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            app_log!(info, "Detected {} system fonts", fonts.len());
            Ok(fonts)
        } else {
            app_log!(warn, "Failed to detect fonts via command line tools");
            Ok(vec![])
        }
    }

    async fn get_linux_fonts() -> Result<Vec<String>> {
        let output = Command::new("fc-list")
            .arg("--format=%{family}\n")
            .output()?;

        if output.status.success() {
            let fonts_str = String::from_utf8_lossy(&output.stdout);
            let fonts: Vec<String> = fonts_str
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            app_log!(info, "Detected {} system fonts", fonts.len());
            Ok(fonts)
        } else {
            app_log!(warn, "Failed to run fc-list for font detection");
            Ok(vec![])
        }
    }

    async fn get_windows_fonts() -> Result<Vec<String>> {
        // Windows font detection via PowerShell
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                "Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts' | ForEach-Object { $_.PSObject.Properties | ForEach-Object { $_.Name } }"
            ])
            .output()?;

        if output.status.success() {
            let fonts_str = String::from_utf8_lossy(&output.stdout);
            let fonts: Vec<String> = fonts_str
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            app_log!(info, "Detected {} system fonts", fonts.len());
            Ok(fonts)
        } else {
            app_log!(warn, "Failed to detect Windows fonts");
            Ok(vec![])
        }
    }

    pub async fn validate(&self) -> Result<FontValidationResult> {
        if !self.config.validation_enabled {
            app_log!(info, "Font validation disabled");
            return Ok(FontValidationResult {
                valid: true,
                missing_fonts: vec![],
                available_alternatives: HashMap::new(),
                warnings: vec!["Font validation is disabled".to_string()],
                errors: vec![],
            });
        }

        app_log!(info, "Validating font requirements...");

        let mut result = FontValidationResult {
            valid: true,
            missing_fonts: vec![],
            available_alternatives: HashMap::new(),
            warnings: vec![],
            errors: vec![],
        };

        for font_req in &self.config.fonts {
            let font_available = self.is_font_available(&font_req.name);

            if !font_available {
                result.missing_fonts.push(font_req.name.clone());

                // Check alternatives
                let available_alts: Vec<String> = font_req
                    .alternatives
                    .iter()
                    .filter(|alt| self.is_font_available(alt))
                    .cloned()
                    .collect();

                if !available_alts.is_empty() {
                    result
                        .available_alternatives
                        .insert(font_req.name.clone(), available_alts.clone());

                    let alt_list = available_alts.join(", ");
                    if font_req.required {
                        result.warnings.push(format!(
                            "Required font '{}' not found, but alternatives available: {}",
                            font_req.display_name, alt_list
                        ));
                    } else {
                        result.warnings.push(format!(
                            "Optional font '{}' not found, alternatives available: {}",
                            font_req.display_name, alt_list
                        ));
                    }
                } else {
                    let message = format!(
                        "Font '{}' and all alternatives not found: {}",
                        font_req.display_name,
                        font_req.alternatives.join(", ")
                    );

                    if font_req.required {
                        result.errors.push(message);
                        if self.config.strict_mode {
                            result.valid = false;
                        }
                    } else {
                        result.warnings.push(message);
                    }
                }
            } else {
                app_log!(info, "✓ Font available: {}", font_req.display_name);
            }
        }

        Ok(result)
    }

    fn is_font_available(&self, font_name: &str) -> bool {
        self.available_fonts.iter().any(|available| {
            available.to_lowercase().contains(&font_name.to_lowercase())
                || font_name.to_lowercase().contains(&available.to_lowercase())
        })
    }

    pub fn print_validation_report(&self, result: &FontValidationResult) {
        app_log!(info, "=== Font Validation Report ===");

        if result.valid {
            app_log!(info, "✅ Font validation passed");
        } else {
            app_log!(info, "❌ Font validation failed");
        }

        if !result.warnings.is_empty() {
            app_log!(info, "\n⚠️  Warnings:");
            for warning in &result.warnings {
                app_log!(info, "  • {}", warning);
            }
        }

        if !result.errors.is_empty() {
            app_log!(info, "\n❌ Errors:");
            for error in &result.errors {
                app_log!(info, "  • {}", error);
            }
        }

        if !result.available_alternatives.is_empty() {
            app_log!(info, "\n💡 Available alternatives:");
            for (missing, alternatives) in &result.available_alternatives {
                app_log!(info, "  • {} → {}", missing, alternatives.join(", "));
            }
        }

        app_log!(info, "\n📝 Font installation help:");
        app_log!(info, "  macOS: ./install_font_mac.sh");
        app_log!(info, "  Ubuntu: ./install_font_ubuntu.sh");
        app_log!(info, "  Or disable font validation in config.yaml");
        app_log!(info, );
    }
}

pub async fn validate_fonts_or_exit(config_path: Option<PathBuf>) -> Result<()> {
    let font_config_path = config_path.unwrap_or_else(|| {
        // Use same pattern as config.yaml - look in current directory
        PathBuf::from("font_validation.yaml")
    });

    let validator = FontValidator::new(Some(font_config_path)).await?;
    let result = validator.validate().await?;

    validator.print_validation_report(&result);

    if !result.valid {
        app_log!(error, "Font validation failed - server cannot start");
        std::process::exit(1);
    }

    if !result.warnings.is_empty() {
        app_log!(warn, "Font validation completed with warnings - server will continue");
    } else {
        app_log!(info, "All font requirements satisfied");
    }

    Ok(())
}
