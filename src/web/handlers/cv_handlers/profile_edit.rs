// src/web/handlers/cv_handlers/profile_edit.rs
//! Small, targeted edits to an existing profile — for the MCP tools, where a
//! whole-CV rewrite is the wrong tool for "set my GitHub" or "add one bullet".
//!
//!   POST /profiles/details           → set personal fields and links in cv_params.toml
//!   POST /profiles/experience-bullet → append one bullet to one experience
//!
//! Only the targeted lines change: every other key, comment and section of
//! cv_params.toml and experiences_<lang>.typ stays as it was (the form editor's
//! PUT /cv-data, by contrast, regenerates both files from its own model). The
//! previous files are snapshotted first, so restore profile version undoes it.

use super::cv_data::{escape_typ, extract_first_quoted};
use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::core::profile_store;
use crate::utils::normalize_profile_name;
use crate::web::types::WithConversationId;
use crate::web::types::{ActionResponse, ServerConfig, StandardErrorResponse, StandardRequest};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::State;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct UpdateProfileDetailsRequest {
    pub profile: String,
    pub name: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub summary: Option<String>,
    pub github: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AddExperienceBulletRequest {
    pub profile: String,
    /// Company of the experience, as in get cv (case-insensitive, may be partial).
    pub company: String,
    pub bullet: String,
    /// Job title, to pick one experience when several match the company.
    pub title: Option<String>,
    pub lang: Option<String>,
}

fn err(message: String, code: &str, conversation_id: Option<String>) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(message, code.to_string(), vec![], conversation_id))
}

/// An argument that was actually given: MCP clients may send "" for an unset one.
fn given(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

/// The profile's directory inside the user's tenant, which must already exist.
fn existing_profile_dir(profile: &str, email: &str, data_dir: &Path) -> Result<(PathBuf, String), String> {
    let profile = normalize_profile_name(profile);
    if profile.is_empty() || profile.chars().all(|c| c == '_') {
        return Err("profile must not be empty".into());
    }
    let tenant_dir = get_tenant_folder_path(email, &data_dir.to_path_buf());
    if !tenant_dir.join(&profile).is_dir() {
        return Err(format!("Profile '{}' not found. Use list profiles for the exact name.", profile));
    }
    Ok((tenant_dir, profile))
}

async fn snapshot(tenant_dir: &Path, profile: &str) {
    if let Err(e) = profile_store::snapshot(tenant_dir, profile).await {
        graflog::app_log!(warn, "Could not snapshot profile before edit: {}", e);
    }
}

// ── Details (cv_params.toml) ──────────────────────────────────────────────────

/// Set the given fields in a cv_params.toml, keeping everything else intact.
/// Returns the new content and the names of the fields set.
fn apply_details(content: &str, req: &UpdateProfileDetailsRequest) -> Result<(String, Vec<&'static str>), String> {
    use toml_edit::{value, DocumentMut};

    let mut doc: DocumentMut = content
        .parse()
        .map_err(|e| format!("cv_params.toml is not valid TOML, fix it in the editor first: {}", e))?;
    let mut changed = Vec::new();

    // Templates read personal fields at the top level (`details.email`, …).
    for (field, key, v) in [
        ("name", "name", &req.name),
        ("email", "email", &req.email),
        ("phone", "phonenumber", &req.phone),
        ("address", "address", &req.address),
        ("summary", "summary", &req.summary),
    ] {
        if let Some(v) = given(v) {
            doc[key] = value(v);
            changed.push(field);
        }
    }
    if let Some(v) = given(&req.title) {
        doc["title"] = value(v);
        // Templates prefer `job_title` over `title`: a stale one would win.
        if doc.contains_key("job_title") {
            doc["job_title"] = value(v);
        }
        changed.push("title");
    }

    for (field, key, v) in [
        ("github", "github", &req.github),
        ("linkedin", "linkedin", &req.linkedin),
        ("website", "personal_info", &req.website),
    ] {
        if let Some(v) = given(v) {
            doc["links"][key] = value(v);
            changed.push(field);
        }
    }

    Ok((doc.to_string(), changed))
}

pub async fn update_profile_details_handler(
    request: Json<StandardRequest<UpdateProfileDetailsRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let req = &request.data;
    let (tenant_dir, profile) = existing_profile_dir(&req.profile, auth.email(), &config.data_dir)
        .map_err(|e| err(e, "PROFILE_NOT_FOUND", conversation_id.clone()))?;

    let path = tenant_dir.join(&profile).join("cv_params.toml");
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let (updated, changed) = apply_details(&content, req).map_err(|e| err(e, "INVALID_TOML", conversation_id.clone()))?;
    if changed.is_empty() {
        return Err(err(
            "Nothing to update: give at least one of name, title, email, phone, address, summary, github, linkedin, website".into(),
            "INVALID_INPUT",
            conversation_id,
        ));
    }

    snapshot(&tenant_dir, &profile).await;
    tokio::fs::write(&path, updated)
        .await
        .map_err(|e| err(format!("Failed to save cv_params.toml: {}", e), "WRITE_ERROR", conversation_id.clone()))?;

    Ok(Json(ActionResponse::success(
        format!("Updated {} on profile '{}'", changed.join(", "), profile),
        "profile_updated".to_string(),
        conversation_id,
    )))
}

// ── Experience bullet (experiences_<lang>.typ) ────────────────────────────────

/// One `== Company` / `#dated_experience(...)` entry, by byte offsets.
struct Experience {
    company: String,
    title: String,
    /// Offset of the `]` closing its `content: [ ... ]`.
    content_close: Option<usize>,
}

/// Offset of the bracket matching the opener at `open`, skipping string
/// literals (a `)` or `]` inside a bullet's text is not a closer).
fn matching_close(text: &str, open: usize, opener: u8, closer: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            // Jump to the closing quote; the `i += 1` below steps past it.
            b'"' => i += closing_quote(&text[i + 1..])? + 1,
            b if b == opener => depth += 1,
            b if b == closer => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Offset of the closing `"` of a string whose body starts at `s`.
fn closing_quote(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        match c {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return Some(i),
            _ => escaped = false,
        }
    }
    None
}

fn find_experiences(text: &str) -> Vec<Experience> {
    let mut headings = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if let Some(company) = line.trim().strip_prefix("== ") {
            headings.push((offset, company.trim().to_string()));
        }
        offset += line.len();
    }

    headings
        .iter()
        .enumerate()
        .map(|(n, (start, company))| {
            let end = headings.get(n + 1).map_or(text.len(), |h| h.0);
            let section = &text[*start..end];
            let mut exp = Experience { company: company.clone(), title: String::new(), content_close: None };
            if let Some(d) = section.find("#dated_experience(") {
                let call_open = start + d + "#dated_experience".len();
                if let Some(call_close) = matching_close(text, call_open, b'(', b')') {
                    let call = &text[call_open..call_close];
                    exp.title = extract_first_quoted(call).unwrap_or_default();
                    if let Some(c) = call.find("content:") {
                        if let Some(b) = call[c..].find('[') {
                            exp.content_close = matching_close(text, call_open + c + b, b'[', b']');
                        }
                    }
                }
            }
            exp
        })
        .collect()
}

fn describe(exp: &Experience) -> String {
    if exp.title.is_empty() { exp.company.clone() } else { format!("{} — {}", exp.company, exp.title) }
}

/// Append `bullet` to the experience matching `company` (and `title`, when
/// several match). Returns the new content and the experience's description.
fn add_bullet(text: &str, company: &str, title: Option<&str>, bullet: &str) -> Result<(String, String), String> {
    let experiences = find_experiences(text);
    if experiences.is_empty() {
        return Err("This profile has no experiences yet in that language.".into());
    }

    let wanted = company.trim().to_lowercase();
    let exact: Vec<&Experience> = experiences.iter().filter(|e| e.company.to_lowercase() == wanted).collect();
    let mut matches = if exact.is_empty() {
        experiences.iter().filter(|e| e.company.to_lowercase().contains(&wanted)).collect()
    } else {
        exact
    };
    if let Some(title) = title.map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()) {
        matches.retain(|e| e.title.to_lowercase().contains(&title));
    }

    let list = |exps: &[&Experience]| exps.iter().map(|e| describe(e)).collect::<Vec<_>>().join("; ");
    let exp = match matches.as_slice() {
        [one] => *one,
        [] => {
            let all: Vec<&Experience> = experiences.iter().collect();
            return Err(format!("No experience matches '{}'. Experiences: {}", company.trim(), list(&all)));
        }
        several => {
            return Err(format!("Several experiences match '{}', give the title to pick one: {}", company.trim(), list(several)));
        }
    };

    let close = exp
        .content_close
        .ok_or_else(|| format!("Could not find the bullet list of '{}' — edit it in the studio instead.", describe(exp)))?;

    let call = format!("#experience_details(\"{}\")", escape_typ(bullet.trim()));
    let line_start = text[..close].rfind('\n').map_or(0, |i| i + 1);
    let mut out = String::with_capacity(text.len() + call.len() + 16);
    if text[line_start..close].trim().is_empty() {
        // `]` on its own line: add the bullet on a line above it, indented one
        // level deeper (two spaces, as the generators write).
        let indent = &text[line_start..close];
        out.push_str(&text[..line_start]);
        out.push_str(&format!("{}  {}\n", indent, call));
        out.push_str(&text[line_start..]);
    } else {
        out.push_str(&text[..close]);
        out.push(' ');
        out.push_str(&call);
        out.push_str(&text[close..]);
    }
    Ok((out, describe(exp)))
}

pub async fn add_experience_bullet_handler(
    request: Json<StandardRequest<AddExperienceBulletRequest>>,
    auth: AuthenticatedUser,
    config: &State<ServerConfig>,
) -> Result<Json<ActionResponse>, Json<StandardErrorResponse>> {
    let conversation_id = request.conversation_id();
    let req = &request.data;
    if req.bullet.trim().is_empty() {
        return Err(err("bullet must not be empty".into(), "INVALID_INPUT", conversation_id));
    }
    if req.company.trim().is_empty() {
        return Err(err("company must not be empty".into(), "INVALID_INPUT", conversation_id));
    }
    let lang = given(&req.lang).unwrap_or("en").to_lowercase();
    if lang.len() != 2 || !lang.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(err(format!("Invalid lang '{}': use a two-letter code like en", lang), "INVALID_INPUT", conversation_id));
    }

    let (tenant_dir, profile) = existing_profile_dir(&req.profile, auth.email(), &config.data_dir)
        .map_err(|e| err(e, "PROFILE_NOT_FOUND", conversation_id.clone()))?;
    let path = tenant_dir.join(&profile).join(format!("experiences_{}.typ", lang));
    let content = tokio::fs::read_to_string(&path).await.map_err(|_| {
        err(
            format!("Profile '{}' has no '{}' CV (experiences_{}.typ). Use get cv to see its languages.", profile, lang, lang),
            "NOT_FOUND",
            conversation_id.clone(),
        )
    })?;

    let (updated, experience) = add_bullet(&content, &req.company, given(&req.title), &req.bullet)
        .map_err(|e| err(e, "EXPERIENCE_NOT_FOUND", conversation_id.clone()))?;

    snapshot(&tenant_dir, &profile).await;
    tokio::fs::write(&path, updated)
        .await
        .map_err(|e| err(format!("Failed to save experiences_{}.typ: {}", lang, e), "WRITE_ERROR", conversation_id.clone()))?;

    Ok(Json(ActionResponse::success(
        format!("Added a bullet to {} in profile '{}' ({})", experience, profile, lang),
        "profile_updated".to_string(),
        conversation_id,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML: &str = r#"# Profile Information
name = "Jane Doe"
job_title = "Developer"
manager_name = "Someone"

[links]
github = ""
linkedin = "https://linkedin.com/in/jane"

[[projects]]
title = "Thing"
"#;

    #[test]
    fn details_set_only_the_given_fields_and_keep_the_rest() {
        let req = UpdateProfileDetailsRequest {
            github: Some("https://github.com/jane".into()),
            title: Some("Tech Lead".into()),
            email: Some("  ".into()),
            ..Default::default()
        };
        let (out, changed) = apply_details(TOML, &req).unwrap();
        assert_eq!(changed, vec!["title", "github"]);
        assert!(out.contains("github = \"https://github.com/jane\""));
        assert!(out.contains("job_title = \"Tech Lead\""), "stale job_title would win in templates");
        assert!(out.contains("title = \"Tech Lead\""));
        assert!(out.contains("# Profile Information"), "comments kept");
        assert!(out.contains("manager_name = \"Someone\""), "unknown keys kept");
        assert!(out.contains("[[projects]]"), "other sections kept");
        assert!(out.contains("linkedin = \"https://linkedin.com/in/jane\""));
        assert!(!out.contains("email ="), "a blank argument is not an update");
    }

    #[test]
    fn details_create_links_when_missing() {
        let req = UpdateProfileDetailsRequest { website: Some("https://jane.dev".into()), ..Default::default() };
        let (out, _) = apply_details("name = \"Jane\"\n", &req).unwrap();
        let parsed: toml::Value = toml::from_str(&out).unwrap();
        assert_eq!(parsed["links"]["personal_info"].as_str(), Some("https://jane.dev"));
        assert_eq!(parsed["name"].as_str(), Some("Jane"));
    }

    const TYP: &str = r#"#import "template.typ": *

#let get_key_insights() = {
  ("Insight (one)",)
}

#let get_work_experience() = [
  == Acme Corp
  #dated_experience(
    "Lead Engineer",
    date: "2022 - Today",
    content: [
      #experience_details("Built the [core] platform (v2)")
    ]
  )

  == Acme Labs
  #dated_experience("Intern", date: "2019", content: [#experience_details("Tests")])

  == Globex
  #dated_experience(
    "Developer",
    date: "2020",
    content: [
    ]
  )
]
"#;

    fn bullets_of(text: &str, company: &str) -> Vec<String> {
        super::super::cv_data::parse_experiences_typ(text)
            .into_iter()
            .find(|e| e.company == company)
            .unwrap()
            .responsibilities
    }

    #[test]
    fn bullet_is_appended_to_the_exact_company_and_nothing_else_changes() {
        let (out, exp) = add_bullet(TYP, "acme corp", None, "Cut costs by 30% \"fast\"").unwrap();
        assert_eq!(exp, "Acme Corp — Lead Engineer");
        assert_eq!(
            bullets_of(&out, "Acme Corp"),
            vec!["Built the [core] platform (v2)".to_string(), "Cut costs by 30% \"fast\"".to_string()]
        );
        assert!(out.contains("      #experience_details(\"Cut costs by 30% \\\"fast\\\"\")\n    ]"));
        assert!(out.contains("get_key_insights"), "other sections kept");
        assert_eq!(out.replace("      #experience_details(\"Cut costs by 30% \\\"fast\\\"\")\n", ""), TYP);
    }

    #[test]
    fn bullet_goes_into_an_inline_or_empty_list() {
        let (out, _) = add_bullet(TYP, "Acme Labs", None, "Wrote docs").unwrap();
        assert_eq!(bullets_of(&out, "Acme Labs"), vec!["Tests".to_string(), "Wrote docs".to_string()]);

        let (out, _) = add_bullet(TYP, "globex", None, "Shipped it").unwrap();
        assert_eq!(bullets_of(&out, "Globex"), vec!["Shipped it".to_string()]);
    }

    #[test]
    fn ambiguous_or_unknown_company_lists_the_experiences() {
        // "acme" is a partial match for both Acme entries.
        let e = add_bullet(TYP, "acme", None, "x").unwrap_err();
        assert!(e.contains("Several") && e.contains("Acme Corp — Lead Engineer") && e.contains("Acme Labs — Intern"), "{}", e);
        // The title picks one.
        let (_, exp) = add_bullet(TYP, "acme", Some("intern"), "x").unwrap();
        assert_eq!(exp, "Acme Labs — Intern");

        let e = add_bullet(TYP, "Initech", None, "x").unwrap_err();
        assert!(e.contains("No experience matches") && e.contains("Globex — Developer"), "{}", e);
    }
}
