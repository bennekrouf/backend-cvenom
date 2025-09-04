// src/template_system.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    pub name: String,
    pub description: String,
    pub main_file: String,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
    pub languages: Vec<String>,
    pub version: String,
}

impl Default for TemplateManifest {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Standard CV layout".to_string(),
            main_file: "main.typ".to_string(),
            dependencies: vec!["template.typ".to_string()],
            features: vec![],
            languages: vec!["en".to_string(), "fr".to_string()],
            version: "1.0.0".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub id: String,
    pub manifest: TemplateManifest,
    pub path: PathBuf,
}

impl Template {
    pub fn load_from_dir(template_dir: &PathBuf) -> Result<Self> {
        let id = template_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid template directory name"))?
            .to_string();

        // Load manifest.toml if exists, otherwise use defaults
        let manifest_path = template_dir.join("manifest.toml");
        let manifest = if manifest_path.exists() {
            let content =
                fs::read_to_string(&manifest_path).context("Failed to read template manifest")?;
            toml::from_str(&content).context("Failed to parse template manifest")?
        } else {
            // Generate default manifest based on directory name
            TemplateManifest {
                name: id.clone(),
                description: Self::generate_description(&id),
                ..Default::default()
            }
        };

        Ok(Self {
            id,
            manifest,
            path: template_dir.clone(),
        })
    }

    fn generate_description(id: &str) -> String {
        match id {
            "default" => "Standard CV layout".to_string(),
            name if name.contains("keyteo") => "CV with Keyteo branding".to_string(),
            name if name.contains("modern") => "Modern CV template".to_string(),
            name if name.contains("minimal") => "Minimal CV template".to_string(),
            _ => format!("{} CV template", id),
        }
    }

    pub fn main_template_file(&self) -> PathBuf {
        self.path.join(&self.manifest.main_file)
    }

    pub fn get_dependencies(&self) -> Vec<PathBuf> {
        self.manifest
            .dependencies
            .iter()
            .map(|dep| self.path.join(dep))
            .collect()
    }
}

pub struct TemplateManager {
    templates_dir: PathBuf,
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            templates_dir,
            templates: HashMap::new(),
        };
        manager.discover_templates()?;
        Ok(manager)
    }

    fn discover_templates(&mut self) -> Result<()> {
        info!("Discovering templates in: {}", self.templates_dir.display());

        if !self.templates_dir.exists() {
            warn!(
                "Templates directory does not exist: {}",
                self.templates_dir.display()
            );
            self.create_default_template()?;
            return Ok(());
        }

        let entries =
            fs::read_dir(&self.templates_dir).context("Failed to read templates directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                match Template::load_from_dir(&path) {
                    Ok(template) => {
                        info!("Discovered template: {} at {}", template.id, path.display());
                        self.templates.insert(template.id.clone(), template);
                    }
                    Err(e) => {
                        warn!("Failed to load template from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Always ensure we have a default template
        if !self.templates.contains_key("default") {
            self.create_default_template()?;
        }

        info!("Loaded {} templates", self.templates.len());
        Ok(())
    }

    fn create_default_template(&mut self) -> Result<()> {
        // Check if we have legacy cv.typ in templates root
        let legacy_cv = self.templates_dir.join("cv.typ");
        if legacy_cv.exists() {
            // Create virtual default template pointing to legacy file
            let template = Template {
                id: "default".to_string(),
                manifest: TemplateManifest::default(),
                path: self.templates_dir.clone(),
            };
            self.templates.insert("default".to_string(), template);
            info!("Created virtual default template from legacy cv.typ");
        } else {
            // Create minimal default template in memory
            let template = Template {
                id: "default".to_string(),
                manifest: TemplateManifest::default(),
                path: self.templates_dir.clone(),
            };
            self.templates.insert("default".to_string(), template);
            info!("Created fallback default template");
        }

        Ok(())
    }

    pub fn get_template(&self, template_id: &str) -> Option<&Template> {
        self.templates.get(template_id)
    }

    pub fn list_templates(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }

    pub fn template_exists(&self, template_id: &str) -> bool {
        self.templates.contains_key(template_id)
    }

    pub fn prepare_template_workspace(
        &self,
        template_id: &str,
        workspace_dir: &PathBuf,
    ) -> Result<PathBuf> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_id))?;

        info!("Preparing template workspace for: {}", template_id);
        info!("Template path: {}", template.path.display());
        info!("Workspace dir: {}", workspace_dir.display());

        // Adjust paths to account for being in tmp_workspace directory
        let main_template = PathBuf::from("..").join(&template.main_template_file());
        let main_dest = workspace_dir.join("main.typ");

        info!("Looking for main template at: {}", main_template.display());

        if main_template.exists() {
            fs::copy(&main_template, &main_dest).context("Failed to copy main template file")?;
            info!(
                "Copied main template: {} -> {}",
                main_template.display(),
                main_dest.display()
            );
        } else {
            anyhow::bail!("Template main file not found: {}", main_template.display());
        }

        // Copy dependencies with correct path resolution
        for dep_relative_path in &template.manifest.dependencies {
            let dep_source = PathBuf::from("..")
                .join(&template.path)
                .join(dep_relative_path);
            let dep_dest = workspace_dir.join(dep_relative_path);

            info!("Looking for dependency: {}", dep_source.display());

            if dep_source.exists() {
                fs::copy(&dep_source, &dep_dest).with_context(|| {
                    format!("Failed to copy dependency: {}", dep_source.display())
                })?;
                info!(
                    "Copied dependency: {} -> {}",
                    dep_source.display(),
                    dep_dest.display()
                );
            } else {
                warn!("Dependency not found: {}", dep_source.display());
            }
        }

        Ok(main_dest)
    }
}
