//! Brand library handlers — tenant-scoped CRUD over `<tenant>/brands/<slug>/`.
//!
//! Step 1 of the brand-library rollout: storage + REST endpoints. The frontend
//! picker and the workspace-side "apply this brand" wiring come in later steps.

use crate::auth::AuthenticatedUser;
use crate::core::brand_store::{self, Brand, BrandSummary};
use crate::core::database::get_tenant_folder_path;
use crate::web::types::{ServerConfig, StandardErrorResponse};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::State;

fn tenant_dir(auth: &AuthenticatedUser, config: &ServerConfig) -> std::path::PathBuf {
    get_tenant_folder_path(&auth.user().email, &config.data_dir)
}

fn err(status: &str, msg: impl Into<String>) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(
        msg.into(),
        status.to_string(),
        vec!["Try again or contact support".to_string()],
        None,
    ))
}

pub async fn list_brands_handler(
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<Vec<BrandSummary>>, Json<StandardErrorResponse>> {
    let dir = tenant_dir(&auth, config);
    match brand_store::list_brands(&dir) {
        Ok(list) => Ok(Json(list)),
        Err(e) => {
            app_log!(error, "list_brands failed: {}", e);
            Err(err("LIST_ERROR", "Failed to list brands"))
        }
    }
}

pub async fn get_brand_handler(
    slug: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<Brand>, Json<StandardErrorResponse>> {
    let dir = tenant_dir(&auth, config);
    match brand_store::load_brand(&dir, &slug) {
        Ok(b) => Ok(Json(b)),
        Err(e) => {
            app_log!(warn, "get_brand({}) failed: {}", slug, e);
            Err(err("NOT_FOUND", format!("Brand '{}' not found", slug)))
        }
    }
}

/// PUT body — clients send the desired display `name`, optional `description`,
/// and `styling`. The path slug is what URLs/folders use; we don't rederive it
/// from the name so renames are stable (a future "rename brand" can change the
/// stored name without moving the folder).
#[derive(Debug, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PutBrandRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub styling: crate::web::handlers::cv_handlers::cv_data::StylingData,
}

pub async fn put_brand_handler(
    slug: String,
    body: Json<PutBrandRequest>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<Brand>, Json<StandardErrorResponse>> {
    // Validate slug shape (clients shouldn't send anything we wouldn't generate).
    if brand_store::slugify(&slug).map(|s| s != slug).unwrap_or(true) {
        return Err(err(
            "INVALID_SLUG",
            "Slug must be lowercase letters, digits, and dashes",
        ));
    }
    if body.name.trim().is_empty() {
        return Err(err("INVALID_NAME", "Brand name is required"));
    }

    let brand = Brand {
        name: body.name.clone(),
        description: body.description.clone(),
        styling: body.styling.clone(),
    };
    let dir = tenant_dir(&auth, config);
    match brand_store::save_brand(&dir, &slug, &brand) {
        Ok(()) => Ok(Json(brand)),
        Err(e) => {
            app_log!(error, "save_brand({}) failed: {}", slug, e);
            Err(err("SAVE_ERROR", "Failed to save brand"))
        }
    }
}

pub async fn delete_brand_handler(
    slug: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let dir = tenant_dir(&auth, config);
    match brand_store::delete_brand(&dir, &slug) {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted": slug }))),
        Err(e) => {
            app_log!(error, "delete_brand({}) failed: {}", slug, e);
            Err(err("DELETE_ERROR", "Failed to delete brand"))
        }
    }
}

// ── Logo upload / download / delete ───────────────────────────────────────────

pub async fn upload_brand_logo_handler(
    slug: String,
    upload: rocket::form::Form<crate::web::types::BrandLogoUploadForm<'_>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let dir = tenant_dir(&auth, config);

    // The brand must exist before a logo can be attached.
    if brand_store::load_brand(&dir, &slug).is_err() {
        return Err(err("NOT_FOUND", format!("Brand '{}' not found", slug)));
    }

    let file_path = match upload.file.path() {
        Some(p) => p,
        None => return Err(err("UPLOAD_ERROR", "Uploaded file has no path")),
    };
    let bytes = match tokio::fs::read(file_path).await {
        Ok(b) => b,
        Err(e) => {
            app_log!(error, "reading uploaded brand logo failed: {}", e);
            return Err(err("UPLOAD_ERROR", "Failed to read uploaded file"));
        }
    };

    // Brand logos must be PNG: templates that show the logo all embed a literal
    // `company_logo.png` filename in `image()`, so typst picks the PNG decoder
    // by extension. A JPEG sneaking in under the .png name would fail with
    // "Invalid PNG signature" at compile time — reject it up-front instead.
    const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < PNG_SIGNATURE.len() || !bytes.starts_with(PNG_SIGNATURE) {
        return Err(err(
            "INVALID_IMAGE",
            "Brand logo must be a PNG image. Convert your file to PNG and try again.",
        ));
    }

    let written = match brand_store::write_logo(&dir, &slug, &bytes) {
        Ok(p) => p,
        Err(e) => {
            app_log!(error, "write_logo({}) failed: {}", slug, e);
            return Err(err("SAVE_ERROR", "Failed to save logo"));
        }
    };

    // Belt-and-suspenders: also run the shared image validator so corrupted
    // PNGs (truncated, fake header) are caught and cleaned up.
    if let Err(e) = crate::core::FsOps::validate_image(&written).await {
        let _ = tokio::fs::remove_file(&written).await;
        app_log!(warn, "uploaded brand logo failed validation: {}", e);
        return Err(err(
            "INVALID_IMAGE",
            "Uploaded file is not a valid PNG image",
        ));
    }

    Ok(Json(serde_json::json!({ "uploaded": slug })))
}

pub async fn get_brand_logo_handler(
    slug: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<rocket::fs::NamedFile, rocket::http::Status> {
    let dir = tenant_dir(&auth, config);
    match brand_store::logo_path(&dir, &slug) {
        Some(path) => rocket::fs::NamedFile::open(path)
            .await
            .map_err(|_| rocket::http::Status::NotFound),
        None => Err(rocket::http::Status::NotFound),
    }
}

pub async fn delete_brand_logo_handler(
    slug: String,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let dir = tenant_dir(&auth, config);
    match brand_store::delete_logo(&dir, &slug) {
        Ok(()) => Ok(Json(serde_json::json!({ "deleted_logo": slug }))),
        Err(e) => {
            app_log!(error, "delete_logo({}) failed: {}", slug, e);
            Err(err("DELETE_ERROR", "Failed to delete logo"))
        }
    }
}
