// src/utils.rs
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Normalize profile name for file system usage
pub fn normalize_profile_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Normalize language code
pub fn normalize_language(lang: Option<&str>) -> String {
    match lang.map(|s| s.to_lowercase()).as_deref() {
        Some("fr") | Some("french") | Some("français") => "fr".to_string(),
        Some("en") | Some("english") | Some("anglais") => "en".to_string(),
        Some("es") | Some("spanish") | Some("español") => "es".to_string(),
        Some("de") | Some("german") | Some("deutsch") => "de".to_string(),
        _ => "en".to_string(), // Default to English for None or unknown languages
    }
}

/// Build tenant profile directory path
pub fn tenant_profile_path(base: &PathBuf, tenant: &str, profile: &str) -> PathBuf {
    base.join(tenant).join(profile)
}

/// Build output file path
pub fn output_file_path(base: &PathBuf, profile: &str, template: &str, lang: &str) -> PathBuf {
    base.join(format!(
        "{}_{}_{}_{}.pdf",
        profile,
        template,
        lang,
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ))
}

/// Ensure directory exists
pub async fn ensure_directory(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        tokio::fs::create_dir_all(path)
            .await
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Read file content as string with proper error context
pub async fn read_file_content(path: &PathBuf) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Write file content with proper error context
pub async fn write_file_content(path: &PathBuf, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(&parent.to_path_buf()).await?;
    }

    tokio::fs::write(path, content)
        .await
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

/// Check if file exists and is readable
pub async fn file_accessible(path: &PathBuf) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

/// Get file extension in lowercase
pub fn get_file_extension(filename: &str) -> Option<String> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Validate file extension against allowed types
pub fn validate_file_extension(filename: &str, allowed: &[&str]) -> Result<()> {
    let ext = get_file_extension(filename)
        .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", filename))?;

    if !allowed.contains(&ext.as_str()) {
        anyhow::bail!(
            "Unsupported file extension: {}. Allowed: {:?}",
            ext,
            allowed
        );
    }

    Ok(())
}

// File system utilities
pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
    tokio::fs::create_dir_all(path)
        .await
        .with_context(|| format!("Failed to create directory: {}", path.display()))
}

pub async fn write_file_safe(path: &Path, content: &str) -> Result<()> {
    tokio::fs::write(path, content)
        .await
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

pub async fn read_file_safe(path: &Path) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_profile_name() {
        assert_eq!(normalize_profile_name("John Doe"), "john_doe");
        assert_eq!(normalize_profile_name("jean-paul"), "jean-paul");
        assert_eq!(normalize_profile_name("Marie@Company"), "marie_company");
    }

    #[test]
    fn test_normalize_language() {
        assert_eq!(normalize_language(Some("fr")), "fr");
        assert_eq!(normalize_language(Some("French")), "fr");
        assert_eq!(normalize_language(Some("EN")), "en");
        assert_eq!(normalize_language(Some("unknown")), "en");
        assert_eq!(normalize_language(None), "en");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("test.pdf"), Some("pdf".to_string()));
        assert_eq!(
            get_file_extension("document.DOCX"),
            Some("docx".to_string())
        );
        assert_eq!(get_file_extension("noext"), None);
    }

    #[test]
    fn test_validate_file_extension() {
        assert!(validate_file_extension("test.pdf", &["pdf", "docx"]).is_ok());
        assert!(validate_file_extension("test.txt", &["pdf", "docx"]).is_err());
        assert!(validate_file_extension("noext", &["pdf"]).is_err());
    }
}

