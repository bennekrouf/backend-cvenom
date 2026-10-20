// src/web/handlers/cv_handlers/helpers.rs
//! Shared utility functions for CV handlers

use crate::{
    core::{FsOps, TemplateEngine},
    types::cv_data::{CvConverter, CvJson},
};
use anyhow::Context;
use graflog::app_log;
use std::path::Path;

/// Create profile directory structure from CvJson data
pub async fn create_profile_from_cv_data(
    profile_dir: &Path,
    cv_data: &CvJson,
    profile_name: &str,
) -> anyhow::Result<()> {
    // Ensure profile directory exists
    FsOps::ensure_dir_exists(profile_dir)
        .await
        .context("Failed to create profile directory")?;

    // Convert CvJson to TOML
    let toml_content =
        CvConverter::to_toml(cv_data).context("Failed to convert CV data to TOML")?;

    let toml_path = profile_dir.join("cv_params.toml");
    FsOps::write_file_safe(&toml_path, &toml_content)
        .await
        .context("Failed to write cv_params.toml")?;

    // Convert CvJson to Typst for both languages
    let en_typst = CvConverter::to_typst(cv_data, "en")
        .context("Failed to convert CV data to English Typst")?;

    let en_path = profile_dir.join("experiences_en.typ");
    FsOps::write_file_safe(&en_path, &en_typst)
        .await
        .context("Failed to write experiences_en.typ")?;

    // Create README
    let readme_content = format!(
        "# {} CV Data\n\n\
        Add your profile image as `profile.png` in this directory.\n\
        Add your company logo as `company_logo.png` (optional).\n\n\
        Edit the following files:\n\
        - `cv_params.toml` - Personal information, skills, and key insights\n\
        - `experiences_*.typ` - Work experience for each language\n",
        profile_name
    );

    let readme_path = profile_dir.join("README.md");
    FsOps::write_file_safe(&readme_path, &readme_content)
        .await
        .context("Failed to write README.md")?;

    app_log!(info, "Created profile files from CV data: {}", profile_name);
    Ok(())
}

/// Load profile CV data as CvJson (for job matching, etc.)
pub async fn load_profile_cv_data(
    profile_name: &str,
    tenant_data_dir: &Path,
) -> anyhow::Result<CvJson> {
    let profile_dir = tenant_data_dir.join(profile_name);
    let toml_path = profile_dir.join("cv_params.toml");
    
    // Support both language-specific and legacy filenames
    let typst_path_en = profile_dir.join("experiences_en.typ");
    let typst_path_legacy = profile_dir.join("experiences.typ");
    
    let active_typst_path = if typst_path_en.exists() {
        Some(typst_path_en)
    } else if typst_path_legacy.exists() {
        Some(typst_path_legacy)
    } else {
        None
    };

    if !toml_path.exists() || active_typst_path.is_none() {
        anyhow::bail!("CV files not found for profile: {} (checked experiences_en.typ and experiences.typ)", profile_name);
    }

    CvConverter::from_files(&toml_path, &active_typst_path.unwrap())
        .with_context(|| format!("Failed to load CV data for profile: {}", profile_name))
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

/// Save CvJson data to profile directory as TOML and Typst files.
///
/// Both files are rendered and validated before anything is written, the
/// profile's current files are snapshotted first (see `profile_store`), and
/// settings CvJson does not model (custom `[styling]`, extra keys) are kept
/// from the existing cv_params.toml instead of being reset.
pub async fn save_profile_cv_data(
    profile_name: &str,
    tenant_data_dir: &Path,
    cv_data: &CvJson,
    language: &str,
) -> anyhow::Result<()> {
    let profile_dir = tenant_data_dir.join(profile_name);
    let toml_path = profile_dir.join("cv_params.toml");
    let typst_path = profile_dir.join(&format!("experiences_{}.typ", language));

    // ── Render everything first: a failure here leaves the profile untouched ──
    let generated = CvConverter::to_toml(cv_data)?;
    let existing = tokio::fs::read_to_string(&toml_path).await.ok();
    let toml_content = merge_unmodelled_toml(existing.as_deref(), &generated)?;
    let typst_content = CvConverter::to_typst(cv_data, language)?;

    crate::core::profile_store::snapshot(tenant_data_dir, profile_name)
        .await
        .context("Failed to snapshot profile before saving")?;

    FsOps::ensure_dir_exists(&profile_dir).await?;
    FsOps::write_file_safe(&toml_path, &toml_content).await?;
    FsOps::write_file_safe(&typst_path, &typst_content).await?;

    app_log!(
        trace,
        "Saved CV data for profile: {} in language: {}",
        profile_name,
        language
    );
    Ok(())
}

/// Overlay freshly generated cv_params.toml on the existing one: generated
/// keys win, except `[styling]`, which is the user's; keys only the existing
/// file has are kept. An unreadable existing file is ignored. The result is
/// parsed back before being returned.
pub fn merge_unmodelled_toml(existing: Option<&str>, generated: &str) -> anyhow::Result<String> {
    let mut merged: toml::Table = toml::from_str(generated).context("Generated TOML is invalid")?;
    if let Some(old) = existing.and_then(|s| toml::from_str::<toml::Table>(s).ok()) {
        for (key, value) in old {
            if key == "styling" || !merged.contains_key(&key) {
                merged.insert(key, value);
            }
        }
    }
    let out = toml::to_string(&merged).context("Failed to serialize cv_params.toml")?;
    toml::from_str::<toml::Table>(&out).context("Merged cv_params.toml does not parse")?;
    Ok(out)
}

/// Validate CV data structure
pub fn validate_cv_data(cv_data: &CvJson) -> anyhow::Result<()> {
    if cv_data.personal_info.name.trim().is_empty() {
        anyhow::bail!("CV data must include a profile name");
    }

    if cv_data.work_experience.is_empty() {
        anyhow::bail!("CV data must include at least one work experience");
    }

    Ok(())
}

/// Extract profile name from filename
pub fn extract_profile_name_from_filename(filename: &str) -> String {
    filename.split('.').next().unwrap_or(filename).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_user_styling_and_unmodelled_keys() {
        let existing = "name = \"Old\"\nkey_insights = [\"a\"]\n[styling]\nprimary_color = \"#FF0000\"\n";
        let generated = "name = \"New\"\nsummary = \"x\\ny\"\n[styling]\nprimary_color = \"#14A4E6\"\n";
        let merged: toml::Table = toml::from_str(&merge_unmodelled_toml(Some(existing), generated).unwrap()).unwrap();
        assert_eq!(merged["name"].as_str(), Some("New"));
        assert_eq!(merged["summary"].as_str(), Some("x\ny"));
        assert_eq!(merged["styling"]["primary_color"].as_str(), Some("#FF0000"));
        assert!(merged.contains_key("key_insights"));
    }

    #[test]
    fn merge_ignores_a_corrupt_existing_file() {
        // e.g. a cv_params.toml already broken by the old writer
        let broken = "summary = \"line one\nline two\"\n";
        let out = merge_unmodelled_toml(Some(broken), "summary = \"ok\"\n").unwrap();
        assert_eq!(toml::from_str::<toml::Table>(&out).unwrap()["summary"].as_str(), Some("ok"));
    }

    #[tokio::test]
    async fn save_snapshots_previous_files_first() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("base");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cv_params.toml"), "name = \"Original\"\n").unwrap();
        std::fs::write(dir.join("experiences_en.typ"), "old").unwrap();

        let cv: CvJson = serde_json::from_value(serde_json::json!({
            "personal_info": { "name": "Tailored", "summary": "multi\nline" },
            "work_experience": [], "education": [], "skills": {}, "languages": {},
            "metadata": { "language": "en" }
        }))
        .unwrap();
        save_profile_cv_data("base", t.path(), &cv, "en").await.unwrap();

        let versions = crate::core::profile_store::list_versions(t.path(), "base").await.unwrap();
        assert_eq!(versions.len(), 1);
        crate::core::profile_store::restore(t.path(), "base", None).await.unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("cv_params.toml")).unwrap(), "name = \"Original\"\n");
    }
}
