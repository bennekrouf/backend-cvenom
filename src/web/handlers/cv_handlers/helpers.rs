// src/web/handlers/cv_handlers/helpers.rs
//! Shared utility functions for CV handlers

use crate::{
    core::{FsOps, TemplateEngine},
    types::cv_data::{CvConverter, CvJson},
};
use anyhow::Context;
use graflog::app_log;
use std::path::Path;

/// Create person directory structure from CvJson data
pub async fn create_person_from_cv_data(
    person_dir: &Path,
    cv_data: &CvJson,
    person_name: &str,
) -> anyhow::Result<()> {
    // Ensure person directory exists
    FsOps::ensure_dir_exists(person_dir)
        .await
        .context("Failed to create person directory")?;

    // Convert CvJson to TOML
    let toml_content =
        CvConverter::to_toml(cv_data).context("Failed to convert CV data to TOML")?;

    let toml_path = person_dir.join("cv_params.toml");
    FsOps::write_file_safe(&toml_path, &toml_content)
        .await
        .context("Failed to write cv_params.toml")?;

    // Convert CvJson to Typst for both languages
    let en_typst = CvConverter::to_typst(cv_data, "en")
        .context("Failed to convert CV data to English Typst")?;

    let en_path = person_dir.join("experiences_en.typ");
    FsOps::write_file_safe(&en_path, &en_typst)
        .await
        .context("Failed to write experiences_en.typ")?;

    let fr_typst = CvConverter::to_typst(cv_data, "fr")
        .context("Failed to convert CV data to French Typst")?;

    let fr_path = person_dir.join("experiences_fr.typ");
    FsOps::write_file_safe(&fr_path, &fr_typst)
        .await
        .context("Failed to write experiences_fr.typ")?;

    // Create README
    let readme_content = format!(
        "# {} CV Data\n\n\
        Add your profile image as `profile.png` in this directory.\n\
        Add your company logo as `company_logo.png` (optional).\n\n\
        Edit the following files:\n\
        - `cv_params.toml` - Personal information, skills, and key insights\n\
        - `experiences_*.typ` - Work experience for each language (en/fr)\n",
        person_name
    );

    let readme_path = person_dir.join("README.md");
    FsOps::write_file_safe(&readme_path, &readme_content)
        .await
        .context("Failed to write README.md")?;

    app_log!(info, "Created person files from CV data: {}", person_name);
    Ok(())
}

/// Load person CV data as CvJson (for job matching, etc.)
pub async fn load_person_cv_data(
    person_name: &str,
    tenant_data_dir: &Path,
) -> anyhow::Result<CvJson> {
    let person_dir = tenant_data_dir.join(person_name);
    let toml_path = person_dir.join("cv_params.toml");
    let typst_path = person_dir.join("experiences_en.typ"); // Default to English

    if !toml_path.exists() || !typst_path.exists() {
        anyhow::bail!("CV files not found for person: {}", person_name);
    }

    CvConverter::from_files(&toml_path, &typst_path)
        .with_context(|| format!("Failed to load CV data for person: {}", person_name))
}

/// Normalize template name against available templates
pub fn normalize_template(template: Option<&str>, template_manager: &TemplateEngine) -> String {
    let requested = template.unwrap_or("default").to_lowercase();

    for available_template in template_manager.list_templates() {
        if available_template.to_lowercase() == requested {
            return available_template.to_lowercase();
        }
    }

    "default".to_string()
}

/// Save CvJson data to person directory as TOML and Typst files
pub async fn save_person_cv_data(
    person_name: &str,
    tenant_data_dir: &Path,
    cv_data: &CvJson,
    language: &str,
) -> anyhow::Result<()> {
    let person_dir = tenant_data_dir.join(person_name);
    FsOps::ensure_dir_exists(&person_dir).await?;

    // Convert and save TOML
    let toml_content = CvConverter::to_toml(cv_data)?;
    let toml_path = person_dir.join("cv_params.toml");
    FsOps::write_file_safe(&toml_path, &toml_content).await?;

    // Convert and save Typst
    let typst_content = CvConverter::to_typst(cv_data, language)?;
    let typst_path = person_dir.join(&format!("experiences_{}.typ", language));
    FsOps::write_file_safe(&typst_path, &typst_content).await?;

    app_log!(
        trace,
        "Saved CV data for person: {} in language: {}",
        person_name,
        language
    );
    Ok(())
}

/// Validate CV data structure
pub fn validate_cv_data(cv_data: &CvJson) -> anyhow::Result<()> {
    if cv_data.personal_info.name.trim().is_empty() {
        anyhow::bail!("CV data must include a person name");
    }

    if cv_data.work_experience.is_empty() {
        anyhow::bail!("CV data must include at least one work experience");
    }

    Ok(())
}

/// Extract person name from filename
pub fn extract_person_name_from_filename(filename: &str) -> String {
    filename.split('.').next().unwrap_or(filename).to_string()
}
