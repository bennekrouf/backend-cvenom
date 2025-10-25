// src/core/template_engine.rs
//! Unified template processing engine - eliminates duplicate template logic

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::core::FsOps;
use crate::template_system::TemplateManager;

pub struct TemplateEngine {
    manager: TemplateManager,
    templates_dir: PathBuf,
}

impl TemplateEngine {
    /// Create new template engine
    pub fn new(templates_dir: PathBuf) -> Result<Self> {
        let manager = TemplateManager::new(templates_dir.clone())
            .context("Failed to create template manager")?;

        Ok(Self {
            manager,
            templates_dir,
        })
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.manager
            .list_templates()
            .iter()
            .map(|t| t.id.clone())
            .collect()
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

        info!(
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

        info!(
            "Successfully created person with typst content: {}",
            person_name
        );
        Ok(())
    }

    /// Prepare template workspace
    pub async fn prepare_template_workspace(
        &self,
        template_name: &str,
        workspace_dir: &Path,
    ) -> Result<()> {
        let workspace_pathbuf = workspace_dir.to_path_buf();
        self.manager
            .prepare_template_workspace(template_name, &workspace_pathbuf)
            .context("Failed to prepare template workspace")?;
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
            warn!("Person template not found, creating basic cv_params.toml");
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
            let default_experiences = r#"#import "../templates/default/template.typ": *

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
"#;
            FsOps::write_file_safe(&person_dir.join("experiences_en.typ"), default_experiences)
                .await?;
        }

        // Create experiences_fr.typ
        self.create_experiences_fr(person_dir).await?;
        Ok(())
    }

    /// Create French experiences file
    async fn create_experiences_fr(&self, person_dir: &Path) -> Result<()> {
        let default_experiences_fr = r#"#import "../templates/default/template.typ": *

// French experiences
#let get_work_experience() = {
  // Ajoutez vos expériences professionnelles ici
  [
    #experience(
      title: "Votre Titre de Poste",
      date: "2023 - Présent",
      description: "Nom de l'Entreprise",
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
            r#"# CV for {}

This directory contains the CV configuration and experiences for {}.

## Files

- `cv_params.toml` - Personal information and CV configuration
- `experiences_en.typ` - Work experiences in English
- `experiences_fr.typ` - Work experiences in French
- `profile.png` - Profile picture (optional)

## Usage

Generate CV using the web interface or CLI tools.
"#,
            person_name, person_name
        );
        FsOps::write_file_safe(&person_dir.join("README.md"), &readme_content).await?;
        Ok(())
    }
}
