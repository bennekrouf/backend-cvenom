// src/web/handlers/cv_handlers/cv_data.rs
//
// Endpoints for the unified CV form editor.
//
//   GET  /profiles/:name/cv-data  → parse cv_params.toml + experiences_en.typ
//                                    and return a single CvFormData JSON.
//   PUT  /profiles/:name/cv-data  → accept CvFormData JSON, write cv_params.toml
//                                    and regenerate experiences_en.typ (and
//                                    experiences_fr.typ if it already exists).
//
// Security: The profile name is path-traversal-checked to ensure it stays
// inside the authenticated user's tenant directory.

use crate::auth::AuthenticatedUser;
use crate::core::database::get_tenant_folder_path;
use crate::web::types::{StandardErrorResponse};
use graflog::app_log;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct PersonalData {
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct LinksData {
    pub github: String,
    pub linkedin: String,
    pub website: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct EducationEntry {
    pub title: String,
    pub date: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct LanguagesData {
    pub native: Vec<String>,
    pub fluent: Vec<String>,
    pub intermediate: Vec<String>,
    pub basic: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct WorkExperienceEntry {
    pub company: String,
    pub title: String,
    pub date: String,
    pub description: String,
    pub responsibilities: Vec<String>,
    pub technologies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct StylingData {
    pub primary_color: String,
    pub secondary_color: String,
    /// Whether to render the uploaded photo on the CV (default: false)
    #[serde(default)]
    pub show_photo: bool,

    // ── Branding knobs (all optional; absent = use vibe preset or hardcoded
    //    template defaults). Empty string is treated as "not set". ──
    #[serde(default)] pub vibe:             String,
    #[serde(default)] pub accent_color:     String,
    #[serde(default)] pub neutral_color:    String,
    #[serde(default)] pub background_tone:  String,
    #[serde(default)] pub font_personality: String,
    #[serde(default)] pub density:          String,
    #[serde(default)] pub layout:           String,
    #[serde(default)] pub divider:          String,
    #[serde(default)] pub header_style:     String,
    #[serde(default)] pub photo_shape:      String,
    #[serde(default)] pub icon_style:       String,
    #[serde(default)] pub skill_style:      String,
    #[serde(default)] pub date_style:       String,
    #[serde(default)] pub lang_style:       String,
    #[serde(default)] pub label_tone:       String,
    #[serde(default)] pub paper:            String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct CvFormData {
    pub personal: PersonalData,
    pub links: LinksData,
    /// skill category name → list of skills
    pub skills: HashMap<String, Vec<String>>,
    pub education: Vec<EducationEntry>,
    pub languages: LanguagesData,
    pub work_experience: Vec<WorkExperienceEntry>,
    pub styling: StylingData,
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Resolve the profile directory, rejecting path traversal attempts.
fn resolve_profile_dir(
    profile_name: &str,
    email: &str,
    data_dir: &PathBuf,
) -> Result<PathBuf, String> {
    // Basic sanitisation: reject names containing slashes or dots as components.
    if profile_name.is_empty()
        || profile_name.contains('/')
        || profile_name.contains('\\')
        || profile_name == ".."
    {
        return Err("Invalid profile name".to_string());
    }

    let tenant_dir = get_tenant_folder_path(email, data_dir);
    let profile_dir = tenant_dir.join(profile_name);

    // Canonicalise to prevent `..` escape — but the directory may not exist yet,
    // so we just verify the prefix.
    let canonical_tenant = tenant_dir
        .canonicalize()
        .unwrap_or_else(|_| tenant_dir.clone());
    let tentative = canonical_tenant.join(profile_name);

    if !tentative.starts_with(&canonical_tenant) {
        return Err("Path traversal detected".to_string());
    }

    Ok(profile_dir)
}

// ── TOML parser ───────────────────────────────────────────────────────────────

fn parse_toml_cv(content: &str) -> CvFormData {
    let value: toml::Value = toml::from_str(content).unwrap_or(toml::Value::Table(Default::default()));
    let table = match value {
        toml::Value::Table(t) => t,
        _ => Default::default(),
    };

    // ── personal ──
    // Support both [Personal]/[personal] section (form-editor format) and flat
    // top-level keys (original AI-generated format used by old profiles).
    // We collect the relevant personal fields once, preferring the section.
    let get_personal_str = |key: &str| -> String {
        // Try [Personal] section first, then [personal], then top-level key.
        let from_section = table.get("Personal")
            .or_else(|| table.get("personal"))
            .and_then(|v| v.as_table())
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !from_section.is_empty() {
            return from_section.to_string();
        }
        table.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let title_raw = get_personal_str("title");
    let personal = PersonalData {
        name:    get_personal_str("name"),
        // Fall back to "job_title" (used by some older keyteo profiles)
        title:   if title_raw.is_empty() { get_personal_str("job_title") } else { title_raw },
        email:   get_personal_str("email"),
        phone:   get_personal_str("phonenumber"),
        address: get_personal_str("address"),
        summary: get_personal_str("summary"),
    };

    // ── links ──
    let links_raw = table.get("links").and_then(|v| v.as_table());
    let links = LinksData {
        github:   links_raw.and_then(|t| t.get("github")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
        linkedin: links_raw.and_then(|t| t.get("linkedin")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
        website:  links_raw.and_then(|t| t.get("personal_info")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
    };

    // ── skills ──
    let mut skills: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(skills_table) = table.get("skills").and_then(|v| v.as_table()) {
        for (key, val) in skills_table {
            let items: Vec<String> = val
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            skills.insert(key.clone(), items);
        }
    }

    // ── education ──
    let education: Vec<EducationEntry> = table.get("education")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|e| {
                let t = e.as_table()?;
                Some(EducationEntry {
                    title:    t.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    date:     t.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    location: t.get("location").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                })
            }).collect()
        })
        .unwrap_or_default();

    // ── languages ──
    let lang_raw = table.get("languages").and_then(|v| v.as_table());
    fn parse_str_array(t: Option<&toml::map::Map<String, toml::Value>>, key: &str) -> Vec<String> {
        t.and_then(|t| t.get(key))
         .and_then(|v| v.as_array())
         .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
         .unwrap_or_default()
    }
    let languages = LanguagesData {
        native:       parse_str_array(lang_raw, "native"),
        fluent:       parse_str_array(lang_raw, "fluent"),
        intermediate: parse_str_array(lang_raw, "intermediate"),
        basic:        parse_str_array(lang_raw, "basic"),
    };

    // ── styling ──
    let styling_raw = table.get("styling").and_then(|v| v.as_table());
    let str_field = |k: &str| -> String {
        styling_raw
            .and_then(|t| t.get(k))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let styling = StylingData {
        primary_color:   styling_raw.and_then(|t| t.get("primary_color")).and_then(|v| v.as_str()).unwrap_or("#14A4E6").to_string(),
        secondary_color: styling_raw.and_then(|t| t.get("secondary_color")).and_then(|v| v.as_str()).unwrap_or("#757575").to_string(),
        show_photo:      styling_raw.and_then(|t| t.get("show_photo")).and_then(|v| v.as_bool()).unwrap_or(false),
        vibe:             str_field("vibe"),
        accent_color:     str_field("accent_color"),
        neutral_color:    str_field("neutral_color"),
        background_tone:  str_field("background_tone"),
        font_personality: str_field("font_personality"),
        density:          str_field("density"),
        layout:           str_field("layout"),
        divider:          str_field("divider"),
        header_style:     str_field("header_style"),
        photo_shape:      str_field("photo_shape"),
        icon_style:       str_field("icon_style"),
        skill_style:      str_field("skill_style"),
        date_style:       str_field("date_style"),
        lang_style:       str_field("lang_style"),
        label_tone:       str_field("label_tone"),
        paper:            str_field("paper"),
    };

    CvFormData { personal, links, skills, education, languages, work_experience: vec![], styling }
}

// ── TOML generator ────────────────────────────────────────────────────────────

fn generate_toml(data: &CvFormData) -> String {
    let mut out = String::new();

    // Personal fields at the top level (flat format) so Typst templates can
    // access them as `details.name`, `details.email`, etc. without a section wrapper.
    out.push_str(&format!("name = \"{}\"\n", escape_toml(&data.personal.name)));
    out.push_str(&format!("title = \"{}\"\n", escape_toml(&data.personal.title)));
    out.push_str(&format!("email = \"{}\"\n", escape_toml(&data.personal.email)));
    out.push_str(&format!("phonenumber = \"{}\"\n", escape_toml(&data.personal.phone)));
    out.push_str(&format!("address = \"{}\"\n", escape_toml(&data.personal.address)));
    out.push_str(&format!("summary = \"{}\"\n", escape_toml(&data.personal.summary)));
    out.push('\n');

    // skills — sorted keys for stability
    out.push_str("[skills]\n");
    let mut skill_keys: Vec<&String> = data.skills.keys().collect();
    skill_keys.sort();
    for key in skill_keys {
        let items = &data.skills[key];
        out.push_str(&format!("{} = [{}]\n", key,
            items.iter().map(|s| format!("\"{}\"", escape_toml(s))).collect::<Vec<_>>().join(", ")
        ));
    }
    out.push('\n');

    // education
    for edu in &data.education {
        out.push_str("[[education]]\n");
        out.push_str(&format!("title = \"{}\"\n", escape_toml(&edu.title)));
        out.push_str(&format!("date = \"{}\"\n", escape_toml(&edu.date)));
        out.push_str(&format!("location = \"{}\"\n", escape_toml(&edu.location)));
        out.push('\n');
    }

    // languages
    out.push_str("[languages]\n");
    out.push_str(&format!("native = [{}]\n",       str_array_toml(&data.languages.native)));
    out.push_str(&format!("fluent = [{}]\n",       str_array_toml(&data.languages.fluent)));
    out.push_str(&format!("intermediate = [{}]\n", str_array_toml(&data.languages.intermediate)));
    out.push_str(&format!("basic = [{}]\n",        str_array_toml(&data.languages.basic)));
    out.push('\n');

    // links
    out.push_str("[links]\n");
    out.push_str(&format!("github = \"{}\"\n",        escape_toml(&data.links.github)));
    out.push_str(&format!("linkedin = \"{}\"\n",      escape_toml(&data.links.linkedin)));
    out.push_str(&format!("personal_info = \"{}\"\n", escape_toml(&data.links.website)));
    out.push('\n');

    // styling
    out.push_str("[styling]\n");
    out.push_str(&format!("primary_color = \"{}\"\n",   escape_toml(&data.styling.primary_color)));
    out.push_str(&format!("secondary_color = \"{}\"\n", escape_toml(&data.styling.secondary_color)));
    out.push_str(&format!("show_photo = {}\n",          data.styling.show_photo));
    // Optional branding knobs — only written when set, to keep legacy TOML
    // byte-identical for profiles that don't use them.
    let mut write_opt = |k: &str, v: &str| {
        if !v.is_empty() {
            out.push_str(&format!("{} = \"{}\"\n", k, escape_toml(v)));
        }
    };
    write_opt("vibe",             &data.styling.vibe);
    write_opt("accent_color",     &data.styling.accent_color);
    write_opt("neutral_color",    &data.styling.neutral_color);
    write_opt("background_tone",  &data.styling.background_tone);
    write_opt("font_personality", &data.styling.font_personality);
    write_opt("density",          &data.styling.density);
    write_opt("layout",           &data.styling.layout);
    write_opt("divider",          &data.styling.divider);
    write_opt("header_style",     &data.styling.header_style);
    write_opt("photo_shape",      &data.styling.photo_shape);
    write_opt("icon_style",       &data.styling.icon_style);
    write_opt("skill_style",      &data.styling.skill_style);
    write_opt("date_style",       &data.styling.date_style);
    write_opt("lang_style",       &data.styling.lang_style);
    write_opt("label_tone",       &data.styling.label_tone);
    write_opt("paper",            &data.styling.paper);
    out.push('\n');

    out
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn str_array_toml(items: &[String]) -> String {
    items.iter().map(|s| format!("\"{}\"", escape_toml(s))).collect::<Vec<_>>().join(", ")
}

// ── Typst experience parser ───────────────────────────────────────────────────
//
// Parses the predictable pattern generated by the template / AI:
//
//   == COMPANY NAME
//   #dated_experience(
//     "TITLE",
//     date: "DATE",
//     description: "DESCRIPTION",   ← optional
//     content: [
//       #experience_details("RESPONSIBILITY")
//       …
//     ]
//   )

fn parse_experiences_typ(content: &str) -> Vec<WorkExperienceEntry> {
    let mut result = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Detect company heading
        if trimmed.starts_with("== ") {
            let company = trimmed[3..].trim().to_string();
            let mut entry = WorkExperienceEntry { company, ..Default::default() };
            i += 1;

            // Scan forward to find #dated_experience(
            while i < lines.len() && !lines[i].trim().starts_with("#dated_experience(") {
                i += 1;
            }
            if i >= lines.len() {
                result.push(entry);
                continue;
            }

            // Collect everything from #dated_experience( through its closing )
            // by tracking parenthesis depth.
            let mut block = String::new();
            let mut depth = 0i32;
            while i < lines.len() {
                let line = lines[i];
                for ch in line.chars() {
                    match ch {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                }
                block.push_str(line);
                block.push('\n');
                i += 1;
                if depth <= 0 {
                    break;
                }
            }

            // Extract title — first quoted string after the opening (
            if let Some(title) = extract_first_quoted(&block) {
                entry.title = title;
            }
            // Extract date:
            if let Some(date) = extract_named_arg(&block, "date") {
                entry.date = date;
            }
            // Extract description:
            if let Some(desc) = extract_named_arg(&block, "description") {
                entry.description = desc;
            }
            // Extract #experience_details(...) contents
            entry.responsibilities = extract_experience_details(&block);

            result.push(entry);
        } else {
            i += 1;
        }
    }

    result
}

/// Return the first "quoted string" found in text (handles escaped quotes).
fn extract_first_quoted(text: &str) -> Option<String> {
    // Find the first " after the opening paren
    let start = text.find("(\"")?.saturating_add(2);
    collect_quoted(&text[start..])
}

/// Return the value of a named argument like `date: "..."` or `description: "..."`.
fn extract_named_arg(text: &str, key: &str) -> Option<String> {
    let needle = format!("{}:", key);
    let pos = text.find(&needle)?;
    let after = text[pos + needle.len()..].trim_start();
    if after.starts_with('"') {
        collect_quoted(&after[1..])
    } else {
        None
    }
}

/// Collect characters from the cursor (after the opening `"`) until the closing `"`,
/// honouring `\"` escapes.
fn collect_quoted(s: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut escaped = false;
    loop {
        match chars.next()? {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return Some(result),
            c => {
                escaped = false;
                result.push(c);
            }
        }
    }
}

/// Extract all strings inside `#experience_details("...")` calls.
fn extract_experience_details(block: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = block;
    let needle = "#experience_details(";
    while let Some(pos) = remaining.find(needle) {
        remaining = &remaining[pos + needle.len()..];
        // Skip optional whitespace/newline then expect a quote
        let after = remaining.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\n');
        if after.starts_with('"') {
            if let Some(s) = collect_quoted(&after[1..]) {
                let text = s.trim().to_string();
                if !text.is_empty() {
                    result.push(text);
                }
            }
        }
    }
    result
}

// ── Typst experience generator ────────────────────────────────────────────────

fn generate_experiences_typ(experiences: &[WorkExperienceEntry]) -> String {
    let mut out = String::from("#import \"template.typ\": *\n\n");
    // Stub `get_key_insights` so `keyteo` / `keyteo_full`'s
    // `#import "experiences.typ": get_work_experience, get_key_insights`
    // resolves even when this generator (which doesn't model key insights)
    // produced the file. Returns an empty tuple — call sites in those
    // templates check `.len() > 0` before rendering.
    out.push_str("#let get_key_insights() = ()\n\n");
    // No section heading inside the function body — each template renders its
    // own (`= #get_text("work_experience")` in default, `#section(...)` in
    // keyteo/enterprise2, etc.). Emitting one here produced a duplicate
    // heading on every PDF. Matches the convention in
    // `types/cv_data.rs::generate_experiences_typst`.
    out.push_str("#let get_work_experience() = [\n");

    for exp in experiences {
        out.push_str(&format!("  == {}\n", exp.company));
        out.push_str("  #dated_experience(\n");
        out.push_str(&format!("    \"{}\",\n", escape_typ(&exp.title)));
        out.push_str(&format!("    date: \"{}\",\n", escape_typ(&exp.date)));
        // Skip description when it duplicates one of the responsibilities —
        // a common artifact of LLM-assisted imports that otherwise renders
        // the same text twice (description block + first bullet). Mirrors
        // the dedup at `types/cv_data.rs::generate_experiences_typst`.
        let desc_norm = exp.description.trim().to_lowercase();
        let duplicates_resp = !desc_norm.is_empty()
            && exp.responsibilities.iter().any(|r| r.trim().to_lowercase() == desc_norm);
        if !desc_norm.is_empty() && !duplicates_resp {
            out.push_str(&format!("    description: \"{}\",\n", escape_typ(&exp.description)));
        }
        out.push_str("    content: [\n");
        for resp in &exp.responsibilities {
            if !resp.is_empty() {
                out.push_str(&format!("      #experience_details(\"{}\")\n", escape_typ(resp)));
            }
        }
        out.push_str("    ]\n");
        out.push_str("  )\n\n");
    }

    out.push_str("]\n");
    out
}

fn escape_typ(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_cv_data_handler(
    profile_name: String,
    lang: Option<String>,
    auth: AuthenticatedUser,
    config: &State<crate::web::ServerConfig>,
) -> Result<Json<CvFormData>, Json<StandardErrorResponse>> {
    let email = auth.email();
    let lang = lang.as_deref().unwrap_or("en");

    let profile_dir = match resolve_profile_dir(&profile_name, email, &config.data_dir) {
        Ok(p) => p,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                e, "INVALID_PROFILE".to_string(), vec![], None,
            )));
        }
    };

    // Read cv_params.toml
    let toml_path = profile_dir.join("cv_params.toml");
    let toml_content = tokio::fs::read_to_string(&toml_path).await.unwrap_or_default();
    let mut cv_data = parse_toml_cv(&toml_content);

    // Read experiences_{lang}.typ (optional)
    let exp_path = profile_dir.join(format!("experiences_{}.typ", lang));
    if let Ok(exp_content) = tokio::fs::read_to_string(&exp_path).await {
        cv_data.work_experience = parse_experiences_typ(&exp_content);
    }

    app_log!(info, user = %email, profile = %profile_name, lang = %lang, "Loaded cv-data");
    Ok(Json(cv_data))
}

pub async fn put_cv_data_handler(
    profile_name: String,
    lang: Option<String>,
    request: Json<CvFormData>,
    auth: AuthenticatedUser,
    config: &State<crate::web::ServerConfig>,
) -> Result<Json<serde_json::Value>, Json<StandardErrorResponse>> {
    let email = auth.email();
    let lang = lang.as_deref().unwrap_or("en");
    let data = request.into_inner();

    let profile_dir = match resolve_profile_dir(&profile_name, email, &config.data_dir) {
        Ok(p) => p,
        Err(e) => {
            return Err(Json(StandardErrorResponse::new(
                e, "INVALID_PROFILE".to_string(), vec![], None,
            )));
        }
    };

    // Ensure profile dir exists
    if let Err(e) = tokio::fs::create_dir_all(&profile_dir).await {
        return Err(Json(StandardErrorResponse::new(
            format!("Cannot create profile directory: {}", e),
            "FS_ERROR".to_string(), vec![], None,
        )));
    }

    // Write cv_params.toml
    let toml_content = generate_toml(&data);
    let toml_path = profile_dir.join("cv_params.toml");
    if let Err(e) = tokio::fs::write(&toml_path, &toml_content).await {
        app_log!(error, "Failed to write cv_params.toml: {}", e);
        return Err(Json(StandardErrorResponse::new(
            format!("Failed to save CV data: {}", e),
            "WRITE_ERROR".to_string(), vec![], None,
        )));
    }

    // Generate experiences.typ and write only to the selected language variant
    let exp_typ = generate_experiences_typ(&data.work_experience);
    let exp_filename = format!("experiences_{}.typ", lang);
    let exp_path = profile_dir.join(&exp_filename);
    if let Err(e) = tokio::fs::write(&exp_path, &exp_typ).await {
        app_log!(error, "Failed to write {}: {}", exp_filename, e);
        return Err(Json(StandardErrorResponse::new(
            format!("Failed to save experiences file: {}", e),
            "WRITE_ERROR".to_string(), vec![], None,
        )));
    }

    app_log!(
        info,
        user = %email,
        profile = %profile_name,
        lang = %lang,
        "Saved cv-data ({} experiences, {} edu entries)",
        data.work_experience.len(),
        data.education.len(),
    );

    Ok(Json(serde_json::json!({ "success": true, "message": "CV data saved" })))
}
