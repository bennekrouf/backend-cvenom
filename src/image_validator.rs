// src/image_validator.rs
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::error;

#[derive(Debug, Clone)]
pub struct ImageValidationError {
    pub path: PathBuf,
    pub error_type: ImageErrorType,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub enum ImageErrorType {
    FileNotFound,
    CorruptedFile,
    WrongFormat,
    EmptyFile,
    TooLarge,
    UnreadableFile,
}

impl ImageErrorType {
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound => "IMAGE_NOT_FOUND",
            Self::CorruptedFile => "IMAGE_CORRUPTED",
            Self::WrongFormat => "IMAGE_WRONG_FORMAT",
            Self::EmptyFile => "IMAGE_EMPTY",
            Self::TooLarge => "IMAGE_TOO_LARGE",
            Self::UnreadableFile => "IMAGE_UNREADABLE",
        }
    }
}

pub struct ImageValidator;

impl ImageValidator {
    /// Validate profile image for CV generation
    pub async fn validate_profile_image(image_path: &PathBuf) -> Result<(), ImageValidationError> {
        // Check if file exists
        if !image_path.exists() {
            return Ok(()); // No image is fine - CV can generate without photo
        }

        // Check file metadata
        let metadata = fs::metadata(image_path)
            .await
            .map_err(|_| ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::UnreadableFile,
                message: "Cannot read image file metadata".to_string(),
                suggestion: "Check file permissions or try re-uploading the image".to_string(),
            })?;

        // Check if file is empty
        if metadata.len() == 0 {
            return Err(ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::EmptyFile,
                message: "Profile image file is empty".to_string(),
                suggestion: "Please upload a valid image file".to_string(),
            });
        }

        // Check file size (max 10MB)
        const MAX_SIZE: u64 = 10 * 1024 * 1024;
        if metadata.len() > MAX_SIZE {
            return Err(ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::TooLarge,
                message: format!(
                    "Image file too large: {:.1}MB (max 10MB)",
                    metadata.len() as f64 / 1024.0 / 1024.0
                ),
                suggestion: "Please resize or compress your image and try again".to_string(),
            });
        }

        // Read file header to validate format
        let header = fs::read(image_path)
            .await
            .map_err(|e| ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::UnreadableFile,
                message: format!("Cannot read image file: {}", e),
                suggestion: "Check file permissions or try re-uploading the image".to_string(),
            })?;

        if header.len() < 8 {
            return Err(ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::CorruptedFile,
                message: "Image file too small or corrupted".to_string(),
                suggestion: "Please upload a valid image file".to_string(),
            });
        }

        // Validate format based on file extension and header
        let file_name = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name.ends_with(".png") {
            Self::validate_png_header(&header, image_path)?;
        } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
            Self::validate_jpeg_header(&header, image_path)?;
        } else {
            return Err(ImageValidationError {
                path: image_path.clone(),
                error_type: ImageErrorType::WrongFormat,
                message: "Unsupported image format".to_string(),
                suggestion: "Please use PNG or JPEG format only".to_string(),
            });
        }

        Ok(())
    }

    fn validate_png_header(header: &[u8], path: &PathBuf) -> Result<(), ImageValidationError> {
        const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        if !header.starts_with(PNG_SIGNATURE) {
            // Check if it's actually a JPEG with wrong extension
            if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Err(ImageValidationError {
                    path: path.clone(),
                    error_type: ImageErrorType::WrongFormat,
                    message: "File is JPEG but has .png extension".to_string(),
                    suggestion: "Please rename file to .jpg extension or convert to PNG format"
                        .to_string(),
                });
            }

            return Err(ImageValidationError {
                path: path.clone(),
                error_type: ImageErrorType::CorruptedFile,
                message: "Invalid PNG file - corrupted or wrong format".to_string(),
                suggestion: "Please upload a valid PNG image file".to_string(),
            });
        }
        Ok(())
    }

    fn validate_jpeg_header(header: &[u8], path: &PathBuf) -> Result<(), ImageValidationError> {
        if !header.starts_with(&[0xFF, 0xD8, 0xFF]) {
            // Check if it's actually a PNG with wrong extension
            if header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                return Err(ImageValidationError {
                    path: path.clone(),
                    error_type: ImageErrorType::WrongFormat,
                    message: "File is PNG but has .jpg/.jpeg extension".to_string(),
                    suggestion: "Please rename file to .png extension or convert to JPEG format"
                        .to_string(),
                });
            }

            return Err(ImageValidationError {
                path: path.clone(),
                error_type: ImageErrorType::CorruptedFile,
                message: "Invalid JPEG file - corrupted or wrong format".to_string(),
                suggestion: "Please upload a valid JPEG image file".to_string(),
            });
        }
        Ok(())
    }

    /// Validate and prepare image for workspace (returns true if image should be copied)
    pub async fn validate_and_prepare(source_path: &PathBuf) -> Result<bool, ImageValidationError> {
        match Self::validate_profile_image(source_path).await {
            Ok(_) => {
                if source_path.exists() {
                    tracing::info!("Profile image validation passed: {}", source_path.display());
                    Ok(true)
                } else {
                    tracing::info!("No profile image found - will generate CV without photo");
                    Ok(false)
                }
            }
            Err(validation_error) => {
                error!("Image validation failed: {}", validation_error.message);
                Err(validation_error)
            }
        }
    }
}
