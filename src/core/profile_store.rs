// src/core/profile_store.rs
//! Profile versioning and copies.
//!
//! Before any write replaces a profile's `cv_params.toml` / `experiences_*.typ`,
//! the current files are snapshotted to `<tenant>/.history/<profile>/<version>/`.
//! History lives outside the profile folder so duplicating a profile does not
//! copy it, and the leading dot keeps it out of profile listings and the file tree.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

const HISTORY_DIR: &str = ".history";
/// Snapshots kept per profile; older ones are pruned.
const MAX_VERSIONS: usize = 30;

fn history_dir(tenant_dir: &Path, profile: &str) -> PathBuf {
    tenant_dir.join(HISTORY_DIR).join(profile)
}

/// The files that make up a profile's CV content (what writes replace).
fn is_content_file(name: &str) -> bool {
    name == "cv_params.toml" || (name.starts_with("experiences") && name.ends_with(".typ"))
}

/// Snapshot the profile's current content files. Returns the version id, or
/// `None` when the profile has nothing to snapshot yet (a new profile).
pub async fn snapshot(tenant_dir: &Path, profile: &str) -> Result<Option<String>> {
    let profile_dir = tenant_dir.join(profile);
    if !profile_dir.is_dir() {
        return Ok(None);
    }
    let mut files = Vec::new();
    let mut entries = fs::read_dir(&profile_dir).await?;
    while let Some(e) = entries.next_entry().await? {
        let name = e.file_name().to_string_lossy().to_string();
        if e.file_type().await?.is_file() && is_content_file(&name) {
            files.push(name);
        }
    }
    if files.is_empty() {
        return Ok(None);
    }

    let base = history_dir(tenant_dir, profile);
    // Millisecond precision plus a counter keeps ids unique and sortable.
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ").to_string();
    let mut version = stamp.clone();
    let mut n = 1;
    while base.join(&version).exists() {
        n += 1;
        version = format!("{}-{}", stamp, n);
    }
    let dest = base.join(&version);
    fs::create_dir_all(&dest).await?;
    for f in &files {
        fs::copy(profile_dir.join(f), dest.join(f))
            .await
            .with_context(|| format!("Failed to snapshot {}", f))?;
    }
    prune(&base).await?;
    Ok(Some(version))
}

async fn prune(base: &Path) -> Result<()> {
    let mut versions = list_dir_names(base).await?;
    versions.sort();
    let excess = versions.len().saturating_sub(MAX_VERSIONS);
    for v in versions.into_iter().take(excess) {
        fs::remove_dir_all(base.join(v)).await.ok();
    }
    Ok(())
}

async fn list_dir_names(dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut entries = fs::read_dir(dir).await?;
    while let Some(e) = entries.next_entry().await? {
        if e.file_type().await?.is_dir() {
            out.push(e.file_name().to_string_lossy().to_string());
        }
    }
    Ok(out)
}

/// Saved versions, newest first.
pub async fn list_versions(tenant_dir: &Path, profile: &str) -> Result<Vec<String>> {
    let mut v = list_dir_names(&history_dir(tenant_dir, profile)).await?;
    v.sort();
    v.reverse();
    Ok(v)
}

/// Restore a version (the newest when `version` is None). The current files are
/// snapshotted first, so a restore can itself be undone. Returns the restored id.
pub async fn restore(tenant_dir: &Path, profile: &str, version: Option<&str>) -> Result<String> {
    let versions = list_versions(tenant_dir, profile).await?;
    let chosen = match version {
        Some(v) if versions.iter().any(|x| x == v) => v.to_string(),
        Some(v) => bail!("Version '{}' not found for profile '{}'", v, profile),
        None => versions
            .first()
            .cloned()
            .with_context(|| format!("Profile '{}' has no saved versions", profile))?,
    };
    let src = history_dir(tenant_dir, profile).join(&chosen);
    let profile_dir = tenant_dir.join(profile);

    snapshot(tenant_dir, profile).await?;
    fs::create_dir_all(&profile_dir).await?;
    let mut entries = fs::read_dir(&src).await?;
    while let Some(e) = entries.next_entry().await? {
        fs::copy(e.path(), profile_dir.join(e.file_name())).await?;
    }
    Ok(chosen)
}

/// Copy a profile folder (CV files, pictures, logos) to a new profile name.
/// Fails if the destination already exists.
pub async fn duplicate(tenant_dir: &Path, source: &str, dest: &str) -> Result<()> {
    let src = tenant_dir.join(source);
    let dst = tenant_dir.join(dest);
    if !src.join("cv_params.toml").is_file() {
        bail!("Profile '{}' not found", source);
    }
    if dst.exists() {
        bail!("Profile '{}' already exists", dest);
    }
    copy_dir(&src, &dst).await.with_context(|| format!("Failed to copy '{}' to '{}'", source, dest))
}

fn copy_dir<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst).await?;
        let mut entries = fs::read_dir(src).await?;
        while let Some(e) = entries.next_entry().await? {
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            let ty = e.file_type().await?;
            if ty.is_dir() {
                copy_dir(&e.path(), &dst.join(&name)).await?;
            } else if ty.is_file() {
                fs::copy(e.path(), dst.join(&name)).await?;
            }
        }
        Ok(())
    })
}

/// Lowercase, alphanumerics joined by single dashes.
pub fn slug(s: &str) -> String {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Name for a job-tailored copy of `base`: `<base>-<company>-<title>`, capped
/// in length, with `-2`, `-3`… appended until it does not collide.
pub fn tailored_profile_name(tenant_dir: &Path, base: &str, company: &str, title: &str) -> String {
    let suffix: String = [slug(company), slug(title)]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let suffix = if suffix.is_empty() { "tailored".to_string() } else { suffix };
    let mut name = format!("{}-{}", base, suffix);
    if name.chars().count() > 80 {
        name = name.chars().take(80).collect::<String>().trim_end_matches('-').to_string();
    }
    unique_name(tenant_dir, &name)
}

pub fn unique_name(tenant_dir: &Path, name: &str) -> String {
    if !tenant_dir.join(name).exists() {
        return name.to_string();
    }
    (2..)
        .map(|n| format!("{}-{}", name, n))
        .find(|candidate| !tenant_dir.join(candidate).exists())
        .expect("unbounded range")
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn profile(tenant: &Path, name: &str, toml: &str) {
        let dir = tenant.join(name);
        fs::create_dir_all(&dir).await.unwrap();
        fs::write(dir.join("cv_params.toml"), toml).await.unwrap();
        fs::write(dir.join("experiences_en.typ"), "exp").await.unwrap();
        fs::write(dir.join("profile.png"), "img").await.unwrap();
    }

    #[tokio::test]
    async fn snapshot_then_restore_brings_back_overwritten_base() {
        let t = tempfile::tempdir().unwrap();
        profile(t.path(), "base", "name = \"original\"").await;

        let v = snapshot(t.path(), "base").await.unwrap().unwrap();
        fs::write(t.path().join("base/cv_params.toml"), "name = \"tailored\"").await.unwrap();

        assert_eq!(restore(t.path(), "base", None).await.unwrap(), v);
        let restored = fs::read_to_string(t.path().join("base/cv_params.toml")).await.unwrap();
        assert_eq!(restored, "name = \"original\"");
        // The restore snapshotted the tailored state, so it can be undone.
        assert_eq!(list_versions(t.path(), "base").await.unwrap().len(), 2);
        // Images are not versioned.
        assert!(!t.path().join(HISTORY_DIR).join("base").join(&v).join("profile.png").exists());
    }

    #[tokio::test]
    async fn new_profile_has_nothing_to_snapshot() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(snapshot(t.path(), "nope").await.unwrap(), None);
        assert!(restore(t.path(), "nope", None).await.is_err());
    }

    #[tokio::test]
    async fn duplicate_copies_files_and_refuses_to_overwrite() {
        let t = tempfile::tempdir().unwrap();
        profile(t.path(), "base", "name = \"a\"").await;
        profile(t.path(), "taken", "name = \"b\"").await;

        duplicate(t.path(), "base", "copy").await.unwrap();
        assert!(t.path().join("copy/profile.png").exists());
        assert!(t.path().join("copy/experiences_en.typ").exists());

        assert!(duplicate(t.path(), "base", "taken").await.is_err());
        assert!(duplicate(t.path(), "missing", "x").await.is_err());
    }

    #[test]
    fn tailored_name_is_derived_and_unique() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(
            tailored_profile_name(t.path(), "rust-dev", "Proton AG", "Senior Rust Engineer"),
            "rust-dev-proton-ag-senior-rust-engineer"
        );
        std::fs::create_dir(t.path().join("rust-dev-proton-ag-senior-rust-engineer")).unwrap();
        assert_eq!(
            tailored_profile_name(t.path(), "rust-dev", "Proton AG", "Senior Rust Engineer"),
            "rust-dev-proton-ag-senior-rust-engineer-2"
        );
        assert_eq!(tailored_profile_name(t.path(), "rust-dev", "", ""), "rust-dev-tailored");
    }
}
