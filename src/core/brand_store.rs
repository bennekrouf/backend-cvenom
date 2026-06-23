//! Tenant-scoped brand library.
//!
//! Brands live at `<tenant_data_dir>/brands/<slug>/brand.toml` with an optional
//! sibling `logo.png`. A brand is just a named [`StylingData`] payload — the
//! existing branding resolver consumes it unchanged. The implicit "Default"
//! brand is not stored; absence of any brand = today's behavior.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::web::handlers::cv_handlers::cv_data::StylingData;

const BRANDS_DIR: &str = "brands";
const BRAND_FILE: &str = "brand.toml";
const LOGO_PNG: &str = "logo.png";

/// On-disk shape of `brand.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Brand {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub styling: StylingData,
}

/// Summary shape returned by [`list_brands`] — keeps response payloads small
/// and avoids streaming every styling field for a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandSummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub has_logo: bool,
}

fn brands_root(tenant_dir: &Path) -> PathBuf {
    tenant_dir.join(BRANDS_DIR)
}

fn brand_dir(tenant_dir: &Path, slug: &str) -> PathBuf {
    brands_root(tenant_dir).join(slug)
}

/// URL/folder-safe slug. Lowercases, keeps `[a-z0-9-]`, collapses other chars
/// into `-`, trims leading/trailing dashes. Rejects empty result.
pub fn slugify(name: &str) -> Result<String> {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = true;
    for ch in name.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        anyhow::bail!("brand name produces empty slug");
    }
    Ok(trimmed)
}

pub fn list_brands(tenant_dir: &Path) -> Result<Vec<BrandSummary>> {
    let root = brands_root(tenant_dir);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).with_context(|| format!("reading {:?}", root))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let slug = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let brand_path = entry.path().join(BRAND_FILE);
        if !brand_path.exists() {
            continue;
        }
        match load_brand(tenant_dir, &slug) {
            Ok(b) => out.push(BrandSummary {
                slug: slug.clone(),
                name: b.name,
                description: b.description,
                has_logo: entry.path().join(LOGO_PNG).exists(),
            }),
            Err(_) => continue, // skip malformed brand dirs rather than 500
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn load_brand(tenant_dir: &Path, slug: &str) -> Result<Brand> {
    let path = brand_dir(tenant_dir, slug).join(BRAND_FILE);
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
    let brand: Brand =
        toml::from_str(&raw).with_context(|| format!("parsing brand.toml at {:?}", path))?;
    Ok(brand)
}

pub fn save_brand(tenant_dir: &Path, slug: &str, brand: &Brand) -> Result<()> {
    let dir = brand_dir(tenant_dir, slug);
    fs::create_dir_all(&dir).with_context(|| format!("creating {:?}", dir))?;
    let toml_str = toml::to_string_pretty(brand).context("serializing brand to TOML")?;
    fs::write(dir.join(BRAND_FILE), toml_str).context("writing brand.toml")?;
    Ok(())
}

pub fn delete_brand(tenant_dir: &Path, slug: &str) -> Result<()> {
    let dir = brand_dir(tenant_dir, slug);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("removing {:?}", dir))?;
    }
    Ok(())
}

pub fn logo_path(tenant_dir: &Path, slug: &str) -> Option<PathBuf> {
    let p = brand_dir(tenant_dir, slug).join(LOGO_PNG);
    if p.exists() { Some(p) } else { None }
}

/// Write the brand's logo. Caller is expected to have validated the bytes are
/// a real image — we just persist them. Brand directory must already exist
/// (the brand was created via [`save_brand`] first).
pub fn write_logo(tenant_dir: &Path, slug: &str, bytes: &[u8]) -> Result<PathBuf> {
    let dir = brand_dir(tenant_dir, slug);
    if !dir.exists() {
        anyhow::bail!(
            "brand '{}' does not exist — create it before uploading a logo",
            slug
        );
    }
    let path = dir.join(LOGO_PNG);
    fs::write(&path, bytes).with_context(|| format!("writing {:?}", path))?;
    Ok(path)
}

pub fn delete_logo(tenant_dir: &Path, slug: &str) -> Result<()> {
    let path = brand_dir(tenant_dir, slug).join(LOGO_PNG);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("removing {:?}", path))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_brand(name: &str) -> Brand {
        let mut styling = StylingData::default();
        styling.primary_color = "#E11937".into();
        styling.vibe = "corporate".into();
        Brand {
            name: name.into(),
            description: "Corp red".into(),
            styling,
        }
    }

    #[test]
    fn slugify_basics() {
        assert_eq!(slugify("CGI").unwrap(), "cgi");
        assert_eq!(slugify("CGI IT Co.").unwrap(), "cgi-it-co");
        assert_eq!(slugify("  weird---name!! ").unwrap(), "weird-name");
        assert!(slugify("***").is_err());
    }

    #[test]
    fn empty_tenant_lists_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(list_brands(tmp.path()).unwrap().is_empty());
    }

    #[test]
    fn save_load_list_roundtrip() {
        let tmp = TempDir::new().unwrap();
        save_brand(tmp.path(), "cgi", &sample_brand("CGI")).unwrap();
        save_brand(tmp.path(), "acme", &sample_brand("ACME")).unwrap();

        let listed = list_brands(tmp.path()).unwrap();
        assert_eq!(listed.len(), 2);
        // sorted by name (case-insensitive)
        assert_eq!(listed[0].slug, "acme");
        assert_eq!(listed[1].slug, "cgi");
        assert!(!listed[0].has_logo);

        let loaded = load_brand(tmp.path(), "cgi").unwrap();
        assert_eq!(loaded.name, "CGI");
        assert_eq!(loaded.styling.primary_color, "#E11937");
        assert_eq!(loaded.styling.vibe, "corporate");
    }

    #[test]
    fn delete_removes_dir() {
        let tmp = TempDir::new().unwrap();
        save_brand(tmp.path(), "cgi", &sample_brand("CGI")).unwrap();
        assert_eq!(list_brands(tmp.path()).unwrap().len(), 1);
        delete_brand(tmp.path(), "cgi").unwrap();
        assert_eq!(list_brands(tmp.path()).unwrap().len(), 0);
    }
}
