// src/core/template_engine.rs
//! Unified template processing engine - consolidates all template functionality

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::FsOps;
use crate::types::response::ConversionResponse;
use graflog::app_log;

// ===== Template Models =====

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub id: String,
    pub path: PathBuf,
    pub manifest: TemplateManifest,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TemplateManifest {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub main_file: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
}

// ===== Main Template Engine =====

pub struct TemplateEngine {
    templates_dir: PathBuf,
    templates: Vec<TemplateInfo>,
}

impl TemplateEngine {
    /// Create new template engine with automatic discovery
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        let mut engine = Self {
            templates_dir,
            templates: Vec::new(),
        };
        engine.discover_templates()?;
        Ok(engine)
    }

    /// Discover and load all available templates
    fn discover_templates(&mut self) -> Result<()> {
        self.templates.clear();

        if !self.templates_dir.exists() {
            app_log!(
                warn,
                "Templates directory does not exist: {}",
                self.templates_dir.display()
            );
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.templates_dir).with_context(|| {
            format!(
                "Failed to read templates directory: {}",
                self.templates_dir.display()
            )
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(template_name) = path.file_name().and_then(|n| n.to_str()) {
                    match self.load_template_info(template_name, &path) {
                        Ok(template) => {
                            app_log!(
                                trace,
                                "Loaded template: {} from {}",
                                template.id,
                                template.path.display()
                            );
                            self.templates.push(template);
                        }
                        Err(e) => {
                            app_log!(warn, "Failed to load template {}: {}", template_name, e)
                        }
                    }
                }
            }
        }

        app_log!(info, "Discovered {} templates", self.templates.len());
        Ok(())
    }

    /// Load template information from directory
    fn load_template_info(&self, template_id: &str, template_path: &Path) -> Result<TemplateInfo> {
        let manifest_path = template_path.join("manifest.toml");

        let manifest = if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)
                .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse manifest: {}", manifest_path.display()))?
        } else {
            TemplateManifest {
                name: template_id.to_string(),
                description: None,
                author: None,
                version: None,
                main_file: None,
                dependencies: None,
                features: None,
                languages: None,
            }
        };

        Ok(TemplateInfo {
            id: template_id.to_string(),
            path: template_path.to_path_buf(),
            manifest,
        })
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.iter().map(|t| t.id.clone()).collect()
    }

    /// Get template info by ID
    pub fn get_template(&self, template_id: &str) -> Option<&TemplateInfo> {
        self.templates.iter().find(|t| t.id == template_id)
    }

    /// Check if template exists
    pub fn template_exists(&self, template_id: &str) -> bool {
        self.get_template(template_id).is_some()
    }

    /// Get templates directory
    pub fn templates_dir(&self) -> &PathBuf {
        &self.templates_dir
    }

    // ===== Variable Processing =====

    /// Process template variables in content (supports both {{var}} and ${var} syntax)
    pub fn process_variables(content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in variables {
            // Support both syntaxes for backward compatibility
            let placeholder_mustache = format!("{{{{{}}}}}", key);
            let placeholder_shell = format!("${{{}}}", key);
            result = result.replace(&placeholder_mustache, value);
            result = result.replace(&placeholder_shell, value);
        }
        result
    }

    /// Process a template string with variables
    pub fn process_template(
        &self,
        template_content: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        Self::process_variables(template_content, variables)
    }

    // ===== Template Workspace Management =====

    /// Prepare template workspace by copying template files
    pub async fn prepare_template_workspace(
        &self,
        template_id: &str,
        workspace_dir: &Path,
    ) -> Result<()> {
        app_log!(trace, "Looking for template: '{}'", template_id);
        app_log!(
            trace,
            "Templates directory: {}",
            self.templates_dir.display()
        );
        app_log!(trace, "Available templates: {:?}", self.list_templates());

        let template = self.get_template(template_id).ok_or_else(|| {
            anyhow::anyhow!(
                "Template '{}' not found. Available templates: {:?}. Templates directory: {}",
                template_id,
                self.list_templates(),
                self.templates_dir.display()
            )
        })?;

        FsOps::ensure_dir_exists(workspace_dir).await?;

        // Copy all template files to workspace
        app_log!(
            trace,
            "Reading template files from: {}",
            template.path.display()
        );

        let mut entries = tokio::fs::read_dir(&template.path).await.with_context(|| {
            format!(
                "Failed to read template directory: {}. Check if directory exists and has proper permissions.",
                template.path.display()
            )
        })?;

        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let file_name = src_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name in template"))?;
            let dest_path = workspace_dir.join(file_name);

            if src_path.is_file() {
                FsOps::copy_file(&src_path, &dest_path).await?;
            }
        }

        app_log!(
            trace,
            "Prepared template workspace: {} -> {}",
            template_id,
            workspace_dir.display()
        );
        Ok(())
    }

    // ===== Person Creation Functions =====

    /// Create person from templates (legacy compatibility)
    pub fn create_person_from_templates(
        &self,
        person_name: &str,
        data_dir: &PathBuf,
        display_name: Option<&str>,
    ) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.create_person_from_templates_async(person_name, data_dir, display_name))
    }

    /// Create person from templates (async version)
    pub async fn create_person_from_templates_async(
        &self,
        person_name: &str,
        data_dir: &Path,
        display_name: Option<&str>,
    ) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        FsOps::ensure_dir_exists(&person_dir).await?;

        // Create cv_params.toml using template
        self.create_cv_params(&person_dir, person_name, display_name)
            .await?;

        // Create experiences files
        self.create_experiences_files(&person_dir).await?;

        // Create README
        self.create_readme(&person_dir, person_name).await?;

        app_log!(
            info,
            "Successfully created person from templates: {}",
            person_name
        );
        Ok(())
    }

    /// Create person with typst content and toml config from CV import service
    pub async fn create_person_with_typst_content(
        &self,
        person_name: &str,
        conversion_response: &str,
        data_dir: &Path,
    ) -> Result<()> {
        app_log!(
            info,
            "Creating person '{}' with CV import data",
            person_name
        );

        // Parse the JSON response
        let response: ConversionResponse =
            serde_json::from_str(conversion_response).with_context(|| {
                format!(
                    "Failed to parse conversion response JSON for person '{}'",
                    person_name
                )
            })?;

        app_log!(
            info,
            "Successfully parsed CV import response for '{}'",
            person_name
        );

        let person_dir = data_dir.join(person_name);
        FsOps::ensure_dir_exists(&person_dir)
            .await
            .with_context(|| {
                format!(
                    "Failed to create person directory: {}",
                    person_dir.display()
                )
            })?;

        app_log!(trace, "Created directory: {}", person_dir.display());

        // Save the TOML config as cv_params.toml
        FsOps::write_file_safe(&person_dir.join("cv_params.toml"), &response.toml_content)
            .await
            .with_context(|| {
                format!(
                    "Failed to write cv_params.toml for person '{}'",
                    person_name
                )
            })?;

        // Save the English Typst content as experiences_en.typ
        FsOps::write_file_safe(
            &person_dir.join("experiences_en.typ"),
            &response.typst_content,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to write experiences_en.typ for person '{}'",
                person_name
            )
        })?;

        // Save the French Typst content as experiences_fr.typ
        FsOps::write_file_safe(
            &person_dir.join("experiences_fr.typ"),
            &response.typst_content,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to write experiences_fr.typ for person '{}'",
                person_name
            )
        })?;

        // Create README
        self.create_readme(&person_dir, person_name)
            .await
            .with_context(|| format!("Failed to create README for person '{}'", person_name))?;

        app_log!(
            info,
            "Successfully created person with CV import data: {}",
            person_name
        );
        Ok(())
    }

    // ===== Private Helper Methods =====

    /// Create cv_params.toml
    async fn create_cv_params(
        &self,
        person_dir: &Path,
        person_name: &str,
        display_name: Option<&str>,
    ) -> Result<()> {
        let person_template_path = self.templates_dir.join("person_template.toml");

        if person_template_path.exists() {
            let template_content = FsOps::read_file_safe(&person_template_path).await?;
            let mut vars = HashMap::new();
            vars.insert(
                "name".to_string(),
                display_name.unwrap_or(person_name).to_string(),
            );

            let processed_content = Self::process_variables(&template_content, &vars);
            FsOps::write_file_safe(&person_dir.join("cv_params.toml"), &processed_content).await?;
        } else {
            app_log!(
                warn,
                "Person template not found, creating basic cv_params.toml"
            );
            let basic_config = format!(
                "[personal]\n\
name = \"{}\"\n\
title = \"\"\n\
summary = \"\"\n\
email = \"\"\n\
phone = \"\"\n\
linkedin = \"\"\n\
website = \"\"\n\
\n\
[styling]\n\
primary_color = \"#14A4E6\"\n\
secondary_color = \"#757575\"\n\
\n\
[content]\n\
show_picture = true\n\
show_contact = true\n",
                display_name.unwrap_or(person_name)
            );
            FsOps::write_file_safe(&person_dir.join("cv_params.toml"), &basic_config).await?;
        }
        Ok(())
    }

    /// Create experiences files (English and French)
    async fn create_experiences_files(&self, person_dir: &Path) -> Result<()> {
        // Create experiences_en.typ
        let experiences_template_path = self.templates_dir.join("experiences_template.typ");
        if experiences_template_path.exists() {
            let template_content = FsOps::read_file_safe(&experiences_template_path).await?;
            FsOps::write_file_safe(&person_dir.join("experiences_en.typ"), &template_content)
                .await?;
        } else {
            let default_experiences = self.get_default_experiences_content();
            FsOps::write_file_safe(&person_dir.join("experiences_en.typ"), &default_experiences)
                .await?;
        }

        // Create experiences_fr.typ
        self.create_experiences_fr(person_dir).await?;
        Ok(())
    }

    /// Create French experiences file
    async fn create_experiences_fr(&self, person_dir: &Path) -> Result<()> {
        let default_experiences_fr = r#"#import "template.typ": *

#let get_work_experience() = [
  = Expérience Professionnelle
  
  == Entreprise Actuelle
  #dated_experience(
    "Titre du Poste",
    date: "Date de début - Présent",
    description: "Brève description de l'entreprise et du secteur d'activité",
    content: [
      #experience_details(
        "Responsabilité clé ou réalisation avec des métriques spécifiques si possible"
      )
      #experience_details(
        "Autre responsabilité axée sur le leadership technique ou la livraison"
      )
      #experience_details(
        "Responsabilité supplémentaire mettant en avant l'impact ou la résolution de problèmes"
      )
    ]
  )
  
  == Entreprise Précédente
  #dated_experience(
    "Titre du Poste Précédent",
    date: "Date de début - Date de fin", 
    description: "Brève description de l'entreprise précédente",
    content: [
      #experience_details(
        "Responsabilité clé du rôle précédent"
      )
      #experience_details(
        "Autre responsabilité de l'expérience précédente"
      )
    ]
  )
]"#;

        FsOps::write_file_safe(
            &person_dir.join("experiences_fr.typ"),
            default_experiences_fr,
        )
        .await?;
        Ok(())
    }

    /// Create README file
    async fn create_readme(&self, person_dir: &Path, person_name: &str) -> Result<()> {
        let readme_content = format!(
            "# {} CV Data\n\n\
            Add your profile image as `profile.png` in this directory.\n\
            Add your company logo as `company_logo.png` (optional).\n\n\
            Edit the following files:\n\
            - `cv_params.toml` - Personal information, skills, and key insights\n\
            - `experiences_*.typ` - Work experience for each language (en/fr)\n\n\
            ## Available Languages\n\
            - English: experiences_en.typ\n\
            - French: experiences_fr.typ\n",
            person_name
        );

        FsOps::write_file_safe(&person_dir.join("README.md"), &readme_content).await?;
        Ok(())
    }

    /// Get default experiences content for English
    fn get_default_experiences_content(&self) -> String {
        r#"#import "template.typ": *

#let get_work_experience() = [
  = Work Experience
  
  == Current Company
  #dated_experience(
    "Job Title",
    date: "Start Date - Present",
    description: "Brief description of the company and industry",
    content: [
      #experience_details(
        "Key responsibility or achievement with specific metrics if possible"
      )
      #experience_details(
        "Another responsibility focusing on technical leadership or delivery"
      )
      #experience_details(
        "Additional responsibility highlighting impact or problem-solving"
      )
    ]
  )
  
  == Previous Company
  #dated_experience(
    "Previous Job Title",
    date: "Start Date - End Date", 
    description: "Brief description of previous company",
    content: [
      #experience_details(
        "Previous role key responsibility"
      )
      #experience_details(
        "Another responsibility from previous experience"
      )
    ]
  )
]"#
        .to_string()
    }
}

// ===== Legacy Compatibility =====

/// Legacy TemplateProcessor for backward compatibility
pub struct TemplateProcessor {
    engine: TemplateEngine,
}

impl TemplateProcessor {
    pub fn new(templates_dir: PathBuf) -> Self {
        let engine = TemplateEngine::new(templates_dir).expect("Failed to create template engine");
        Self { engine }
    }

    pub fn process_variables(content: &str, vars: &HashMap<String, String>) -> String {
        TemplateEngine::process_variables(content, vars)
    }

    pub fn process_template(
        &self,
        template_content: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        self.engine.process_template(template_content, variables)
    }

    pub fn create_person_from_templates(
        &self,
        person_name: &str,
        data_dir: &PathBuf,
        display_name: Option<&str>,
    ) -> Result<()> {
        self.engine
            .create_person_from_templates(person_name, data_dir, display_name)
    }
}
