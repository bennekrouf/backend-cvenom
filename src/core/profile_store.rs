// src/core/profile_store.rs
//! Profile versioning and copies.
//!
//! Before any write replaces a profile's `cv_params.toml` / `experiences_*.typ`,
//! the current files are snapshotted to `<tenant>/.history/<profile>/<version>/`.
//! History lives outside the profile folder so duplicating a profile does not
//! copy it, and the leading dot keeps it out of profile listings and the file tree.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
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

/// File in a tailored profile's folder recording the job post it was made for.
pub const JOB_SOURCE_FILE: &str = "job.toml";

/// The job post a tailored profile was generated from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobSource {
    /// Original posting URL (LinkedIn, …); absent when the description was pasted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub company: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub created_at: String,
}

impl JobSource {
    /// `job_url` is kept only when it is a real http(s) URL (the UI sends
    /// "manual" for pasted descriptions).
    pub fn new(job_url: &str, company: &str, title: &str) -> Self {
        let url = job_url.trim();
        let is_url = url.starts_with("http://") || url.starts_with("https://");
        Self {
            url: is_url.then(|| url.to_string()),
            company: company.trim().to_string(),
            title: title.trim().to_string(),
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        }
    }
}

pub async fn write_job_source(tenant_dir: &Path, profile: &str, source: &JobSource) -> Result<()> {
    let path = tenant_dir.join(profile).join(JOB_SOURCE_FILE);
    let content = toml::to_string(source).context("Failed to serialize job source")?;
    fs::write(&path, content).await.with_context(|| format!("Failed to write {}", path.display()))
}

/// `None` when the profile was not made from a job post.
pub async fn read_job_source(tenant_dir: &Path, profile: &str) -> Result<Option<JobSource>> {
    let path = tenant_dir.join(profile).join(JOB_SOURCE_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).await?;
    let source = toml::from_str(&content).with_context(|| format!("Invalid {}", path.display()))?;
    Ok(Some(source))
}

/// Marker file in an archived profile's folder. The leading dot keeps it out of
/// the file tree and out of duplicates (a copy of an archived profile is active);
/// it moves with the folder on rename and goes with it on delete.
pub const ARCHIVED_MARKER: &str = ".archived";

pub fn is_archived(profile_dir: &Path) -> bool {
    profile_dir.join(ARCHIVED_MARKER).is_file()
}

/// Archive (`true`) or unarchive (`false`) a profile. Archived profiles keep
/// all their files and still work with every tool; they are only listed apart.
pub async fn set_archived(tenant_dir: &Path, profile: &str, archived: bool) -> Result<()> {
    let profile_dir = tenant_dir.join(profile);
    if !profile_dir.join("cv_params.toml").is_file() {
        bail!("Profile '{}' not found", profile);
    }
    let marker = profile_dir.join(ARCHIVED_MARKER);
    if archived {
        let stamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        fs::write(&marker, stamp).await.context("Failed to archive profile")?;
    } else if marker.exists() {
        fs::remove_file(&marker).await.context("Failed to unarchive profile")?;
    }
    Ok(())
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

    #[tokio::test]
    async fn job_source_round_trip_and_manual_has_no_url() {
        let tenant = tempfile::tempdir().unwrap();
        profile(tenant.path(), "p", "").await;
        assert_eq!(read_job_source(tenant.path(), "p").await.unwrap(), None);

        let src = JobSource::new(" https://www.linkedin.com/jobs/view/123 ", "Proton", "Rust Engineer");
        write_job_source(tenant.path(), "p", &src).await.unwrap();
        assert_eq!(read_job_source(tenant.path(), "p").await.unwrap(), Some(src));

        assert_eq!(JobSource::new("manual", "Proton", "").url, None);
    }

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

    #[tokio::test]
    async fn archive_round_trip_and_duplicate_is_active() {
        let t = tempfile::tempdir().unwrap();
        profile(t.path(), "base", "").await;
        let dir = t.path().join("base");
        assert!(!is_archived(&dir));

        set_archived(t.path(), "base", true).await.unwrap();
        assert!(is_archived(&dir));
        duplicate(t.path(), "base", "copy").await.unwrap();
        assert!(!is_archived(&t.path().join("copy")));

        set_archived(t.path(), "base", false).await.unwrap();
        assert!(!is_archived(&dir));
        // Unarchiving an active profile is a no-op.
        set_archived(t.path(), "base", false).await.unwrap();
        assert!(set_archived(t.path(), "missing", true).await.is_err());
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
