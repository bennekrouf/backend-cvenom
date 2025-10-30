// src/core/template_engine.rs
//! Unified template processing engine - consolidates template_system and template_processor
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::app_log;
use crate::core::FsOps;

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
}

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
                        Ok(template) => self.templates.push(template),
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

    /// Process template variables in content
    pub fn process_variables(content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in variables {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Prepare template workspace by copying template files
    pub async fn prepare_template_workspace(
        &self,
        template_id: &str,
        workspace_dir: &Path,
    ) -> Result<()> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        FsOps::ensure_dir_exists(workspace_dir).await?;

        // Copy all template files to workspace
        let mut entries = tokio::fs::read_dir(&template.path).await.with_context(|| {
            format!(
                "Failed to read template directory: {}",
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
            info,
            "Prepared template workspace: {} -> {}",
            template_id,
            workspace_dir.display()
        );
        Ok(())
    }

    /// Create person from templates
    pub async fn create_person_from_templates(
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

    /// Create person with typst content
    pub async fn create_person_with_typst_content(
        &self,
        person_name: &str,
        typst_content: &str,
        data_dir: &Path,
    ) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        FsOps::ensure_dir_exists(&person_dir).await?;

        // Create cv_params.toml using template
        self.create_cv_params(&person_dir, person_name, Some(person_name))
            .await?;

        // Create experiences_en.typ with the converted Typst content
        FsOps::write_file_safe(&person_dir.join("experiences_en.typ"), typst_content).await?;

        // Create experiences_fr.typ with default template
        self.create_experiences_fr(&person_dir).await?;

        // Create README
        self.create_readme(&person_dir, person_name).await?;

        app_log!(
            info,
            "Successfully created person with typst content: {}",
            person_name
        );
        Ok(())
    }

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
        let default_experiences_fr = r#"#import "../templates/default/template.typ": *

// Expériences en français
#let get_work_experience() = {
  // Ajoutez vos expériences professionnelles ici
  [
    #experience(
      title: "Votre poste",
      date: "2023 - Présent",
      description: "Nom de l'entreprise",
      details: [
        - Vos responsabilités et réalisations
        - Ajoutez plus de points selon vos besoins
      ]
    )
  ]
}

#let get_key_insights() = {
  [
    - Point clé ou réalisation #1
    - Point clé ou réalisation #2
    - Point clé ou réalisation #3
  ]
}

#let get_skills() = {
  (
    "Langages": ("Rust", "Python", "JavaScript"),
    "Frameworks": ("React", "Vue.js", "FastAPI"),
    "Outils": ("Git", "Docker", "CI/CD"),
  )
}

#let get_education() = {
  [
    #experience(
      title: "Votre diplôme",
      date: "2019 - 2023",
      description: "Université/École",
      details: [
        - Spécialisation ou mention
        - Projets remarquables
      ]
    )
  ]
}

#let get_languages() = {
  (
    "Français": "Langue maternelle",
    "Anglais": "Courant",
    "Allemand": "Notions",
  )
}
"#;
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
            "# CV Data for {}\n\n\
This directory contains CV data for {}.\n\n\
## Files\n\n\
- `cv_params.toml` - Personal information and configuration\n\
- `experiences_en.typ` - Work experience and content in English\n\
- `experiences_fr.typ` - Work experience and content in French\n\
- `profile.png` - Profile image (add your own)\n\n\
## Usage\n\n\
Generate CV in English:\n\
```bash\n\
cargo run -- generate {} en\n\
```\n\n\
Generate CV in French:\n\
```bash\n\
cargo run -- generate {} fr\n\
```\n\n\
## Customization\n\n\
1. Edit `cv_params.toml` to update personal information\n\
2. Modify `experiences_*.typ` files to add your experience\n\
3. Replace `profile.png` with your profile image\n",
            person_name, person_name, person_name, person_name
        );

        FsOps::write_file_safe(&person_dir.join("README.md"), &readme_content).await?;
        Ok(())
    }

    /// Get default experiences content
    fn get_default_experiences_content(&self) -> String {
        r#"#import "../templates/default/template.typ": *

// English experiences
#let get_work_experience() = {
  // Add your work experiences here
  [
    #experience(
      title: "Your Job Title",
      date: "2023 - Present",
      description: "Company Name",
      details: [
        - Your responsibilities and achievements
        - Add more bullet points as needed
      ]
    )
  ]
}

#let get_key_insights() = {
  [
    - Key insight or achievement #1
    - Key insight or achievement #2
    - Key insight or achievement #3
  ]
}

#let get_skills() = {
  (
    "Languages": ("Rust", "Python", "JavaScript"),
    "Frameworks": ("React", "Vue.js", "FastAPI"),
    "Tools": ("Git", "Docker", "CI/CD"),
  )
}

#let get_education() = {
  [
    #experience(
      title: "Your Degree",
      date: "2019 - 2023",
      description: "University/School",
      details: [
        - Specialization or honors
        - Notable projects
      ]
    )
  ]
}

#let get_languages() = {
  (
    "English": "Native",
    "French": "Fluent",
    "German": "Basic",
  )
}
"#
        .to_string()
    }

    /// Reload templates (useful for development)
    pub fn reload(&mut self) -> Result<()> {
        self.discover_templates()
    }
}
