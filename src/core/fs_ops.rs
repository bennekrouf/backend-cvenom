// src/core/fs_ops.rs
//! Enhanced unified file system operations

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use graflog::app_log;

pub struct FsOps;

impl FsOps {
    /// Ensure directory exists - replaces all duplicate ensure_dir_exists functions
    pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
            app_log!(info, "Created directory: {}", path.display());
        }
        Ok(())
    }

    /// Read file safely - replaces all duplicate read_file_safe functions
    pub async fn read_file_safe(path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))
    }

    /// Write file safely - replaces all duplicate write_file_safe functions
    pub async fn write_file_safe(path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            Self::ensure_dir_exists(parent).await?;
        }

        fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        app_log!(info, "Written file: {}", path.display());
        Ok(())
    }

    /// Copy file with error handling
    pub async fn copy_file(src: &Path, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            Self::ensure_dir_exists(parent).await?;
        }

        fs::copy(src, dest)
            .await
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

        app_log!(info, "Copied {} to {}", src.display(), dest.display());
        Ok(())
    }

    /// Remove directory recursively with error handling
    pub async fn remove_dir_all(path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)
                .await
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            app_log!(info, "Removed directory: {}", path.display());
        }
        Ok(())
    }

    /// Check if person directory is valid (has cv_params.toml)
    pub async fn is_valid_person_dir(path: &Path) -> bool {
        path.is_dir() && path.join("cv_params.toml").exists()
    }

    /// List valid person directories
    pub async fn list_persons(data_dir: &Path) -> Result<Vec<String>> {
        let mut persons = Vec::new();

        if !data_dir.exists() {
            return Ok(persons);
        }

        let mut entries = fs::read_dir(data_dir)
            .await
            .with_context(|| format!("Failed to read directory: {}", data_dir.display()))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if Self::is_valid_person_dir(&path).await {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    persons.push(name.to_string());
                }
            }
        }

        persons.sort();
        Ok(persons)
    }

    /// Normalize path - replaces scattered path normalization patterns
    pub fn normalize_path(base: &Path, relative: &Path) -> PathBuf {
        if relative.is_absolute() {
            relative.to_path_buf()
        } else {
            base.join(relative)
        }
    }

    /// Normalize person name - replaces utils::normalize_person_name
    pub fn normalize_person_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .chars()
            .map(|c| match c {
                ' ' | '_' | '.' => '-',
                c if c.is_alphanumeric() => c,
                _ => '-',
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Get file extension safely
    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Check if file exists with any of the given extensions
    pub async fn find_file_with_extensions(
        base_path: &Path,
        extensions: &[&str],
    ) -> Option<PathBuf> {
        for ext in extensions {
            let path_with_ext = base_path.with_extension(ext);
            if path_with_ext.exists() {
                return Some(path_with_ext);
            }
        }
        None
    }

    /// Validate image file format and integrity
    pub async fn validate_image(path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)
            .await
            .with_context(|| format!("Cannot read image file: {}", path.display()))?;

        if metadata.len() == 0 {
            anyhow::bail!("Image file is empty");
        }

        let header = fs::read(path)
            .await
            .with_context(|| format!("Cannot read image file: {}", path.display()))?;

        if header.len() < 8 {
            anyhow::bail!("Image file too small or corrupted");
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.ends_with(".png") {
            const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            if !header.starts_with(PNG_SIGNATURE) {
                if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    anyhow::bail!("File is JPEG but has .png extension");
                }
                anyhow::bail!("Invalid PNG file - corrupted or wrong format");
            }
        } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
            if !header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                anyhow::bail!("Invalid JPEG file - corrupted or wrong format");
            }
        } else {
            anyhow::bail!("Unsupported image format - use PNG or JPEG only");
        }

        Ok(())
    }

    /// Create backup of file with timestamp
    pub async fn backup_file(path: &Path) -> Result<PathBuf> {
        if !path.exists() {
            anyhow::bail!("File to backup does not exist: {}", path.display());
        }

        let backup_name = format!(
            "{}.backup.{}",
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("file"),
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );

        let backup_path = path.with_file_name(backup_name);
        Self::copy_file(path, &backup_path).await?;
        Ok(backup_path)
    }

    /// Clean up temporary files matching pattern
    pub async fn cleanup_temp_files(dir: &Path, pattern: &str) -> Result<usize> {
        let mut count = 0;
        if !dir.exists() {
            return Ok(count);
        }

        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(pattern) {
                    if path.is_file() {
                        fs::remove_file(&path).await?;
                        count += 1;
                        app_log!(info, "Cleaned up temp file: {}", path.display());
                    }
                }
            }
        }
        Ok(count)
    }
}

