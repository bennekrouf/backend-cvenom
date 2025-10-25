// src/core/fs_ops.rs
//! Unified file system operations - eliminates duplicate file handling

use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tracing::info;

pub struct FsOps;

impl FsOps {
    /// Ensure directory exists - replaces all duplicate ensure_dir_exists functions
    pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
            info!("Created directory: {}", path.display());
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

        info!("Written file: {}", path.display());
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

        info!("Copied {} to {}", src.display(), dest.display());
        Ok(())
    }

    /// Remove directory recursively with error handling
    pub async fn remove_dir_all(path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)
                .await
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            info!("Removed directory: {}", path.display());
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
}
