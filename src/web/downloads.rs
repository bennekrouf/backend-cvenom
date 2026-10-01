// src/web/downloads.rs
//
// Download links for generated documents.
//
// Generated files are named `<profile>_<template>_<lang>.pdf` in a directory
// shared by every tenant, so serving them by that name let anyone fetch anyone's
// CV by guessing a profile. A link is now a copy under a random directory —
// `outputs/dl/<uuid v4>/<filename>` — and that is the only shape `/outputs`
// serves, for as long as the emails promise: one hour.
//
// The URL itself is the credential, on purpose: the studio and email recipients
// open it with no auth header, and api0 hands it to a model to pass on.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How long a link works. The CV-ready emails say "expires in 1 hour".
pub const LINK_TTL: Duration = Duration::from_secs(60 * 60);

const LINKS_DIR: &str = "dl";

/// Publish `filename` (a file directly in `output_dir`) under a fresh random
/// directory and return its public URL. Expired links are swept on the way.
pub fn publish(output_dir: &Path, filename: &str, base_url: &str) -> std::io::Result<String> {
    let name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name"))?;

    let links = output_dir.join(LINKS_DIR);
    purge_expired(&links);

    let token = uuid::Uuid::new_v4().to_string();
    let dir = links.join(&token);
    std::fs::create_dir_all(&dir)?;
    std::fs::copy(output_dir.join(name), dir.join(name))?;

    Ok(format!(
        "{}/outputs/{}/{}/{}",
        base_url.trim_end_matches('/'),
        LINKS_DIR,
        token,
        encode_segment(name)
    ))
}

/// The file a request path names, if it is a live link: exactly
/// `dl/<uuid v4>/<name>`, created less than [`LINK_TTL`] ago. Anything else —
/// the old guessable names included — is `None`, i.e. a 404.
pub fn resolve(output_dir: &Path, requested: &Path) -> Option<PathBuf> {
    let parts: Vec<&str> = requested.iter().map(|p| p.to_str()).collect::<Option<_>>()?;
    let [links, token, name] = parts.as_slice() else { return None };

    if *links != LINKS_DIR || !is_v4_uuid(token) || name.is_empty() || name.contains('/') {
        return None;
    }

    let dir = output_dir.join(LINKS_DIR).join(token);
    if is_expired(&dir) {
        return None;
    }
    Some(dir.join(name))
}

/// Delete link directories older than [`LINK_TTL`]. Best effort.
pub fn purge_expired(links: &Path) {
    let Ok(entries) = std::fs::read_dir(links) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && is_expired(&path) {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}

fn is_expired(dir: &Path) -> bool {
    let created = std::fs::metadata(dir).and_then(|m| m.modified());
    match created {
        Ok(t) => SystemTime::now().duration_since(t).map_or(false, |age| age > LINK_TTL),
        // Missing or unreadable: treat as gone.
        Err(_) => true,
    }
}

/// Percent-encode one URL path segment (names are slugs, but be safe).
fn encode_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' => (b as char).to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

fn is_v4_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok_and(|u| u.get_version_num() == 4 && u.hyphenated().to_string() == s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cvenom-dl-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_published_file_is_served_only_through_its_random_link() {
        let out = scratch();
        std::fs::write(out.join("john-doe_modern_en.pdf"), b"%PDF").unwrap();

        let url = publish(&out, "john-doe_modern_en.pdf", "https://api.cvenom.com/").unwrap();
        let rel = url.strip_prefix("https://api.cvenom.com/outputs/").unwrap();
        assert!(rel.starts_with("dl/"));

        let served = resolve(&out, Path::new(rel)).expect("live link resolves");
        assert_eq!(std::fs::read(served).unwrap(), b"%PDF");

        // The guessable name no longer resolves
        assert!(resolve(&out, Path::new("john-doe_modern_en.pdf")).is_none());
        std::fs::remove_dir_all(out).ok();
    }

    #[test]
    fn anything_but_dl_uuid_name_is_refused() {
        let out = scratch();
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::create_dir_all(out.join("dl").join(&token)).unwrap();

        assert!(resolve(&out, Path::new(&format!("dl/{}/cv.pdf", token))).is_some());
        assert!(resolve(&out, Path::new("dl/not-a-uuid/cv.pdf")).is_none());
        assert!(resolve(&out, Path::new(&format!("dl/{}", token))).is_none());
        assert!(resolve(&out, Path::new(&format!("other/{}/cv.pdf", token))).is_none());
        assert!(resolve(&out, Path::new(&format!("dl/{}/a/b.pdf", token))).is_none());
        // A uuid that is not v4 (time-based, guessable) is refused too
        assert!(resolve(&out, Path::new("dl/6ba7b810-9dad-11d1-80b4-00c04fd430c8/cv.pdf")).is_none());
        std::fs::remove_dir_all(out).ok();
    }

    #[test]
    fn a_link_with_no_directory_is_gone() {
        let out = scratch();
        let token = uuid::Uuid::new_v4().to_string();
        assert!(resolve(&out, Path::new(&format!("dl/{}/cv.pdf", token))).is_none());
        std::fs::remove_dir_all(out).ok();
    }
}
