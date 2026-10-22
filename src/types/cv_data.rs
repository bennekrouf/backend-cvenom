// src/types/cv_data.rs
//! Unified CV data structures for cv-import service interactions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Unified CV JSON Structure =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvJson {
    pub personal_info: PersonalInfo,
    pub work_experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub skills: Skills,
    pub languages: Languages,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<Project>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_certifications")]
    pub certifications: Option<Vec<Certification>>,
    pub metadata: CvMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub company: String,
    pub title: String,
    pub start_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>, // None means current
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub responsibilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achievements: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technologies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub institution: String,
    pub degree: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub start_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programming_languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frameworks: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft_skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_skills_other")]
    pub other: Option<HashMap<String, Vec<String>>>,
}

/// Accept both `{"key": ["a","b"]}` and `{"key": "a"}` (or `null`) for `skills.other`.
/// AI models sometimes return a plain string instead of a list.
fn deserialize_skills_other<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, Vec<String>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrList {
        List(Vec<String>),
        One(String),
        Null,
    }

    let opt: Option<HashMap<String, StringOrList>> = Option::deserialize(deserializer)?;
    Ok(opt.map(|m| {
        m.into_iter()
            .map(|(k, v)| {
                let list = match v {
                    StringOrList::List(items) => items,
                    StringOrList::One(s) if s.is_empty() => Vec::new(),
                    StringOrList::One(s) => vec![s],
                    StringOrList::Null => Vec::new(),
                };
                (k, list)
            })
            .collect()
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Languages {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fluent: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermediate: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technologies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Deserializer that accepts both `"string"` and `{ name, issuer, date, … }` for certifications.
/// AI models sometimes return plain strings instead of structured objects.
fn deserialize_certifications<'de, D>(deserializer: D) -> Result<Option<Vec<Certification>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CertItem {
        Struct(Certification),
        Plain(String),
    }

    let opt: Option<Vec<CertItem>> = Option::deserialize(deserializer)?;
    Ok(opt.map(|items| {
        items
            .into_iter()
            .map(|item| match item {
                CertItem::Struct(c) => c,
                CertItem::Plain(s) => Certification {
                    name: s,
                    issuer: String::new(),
                    date: String::new(),
                    expiry: None,
                    credential_id: None,
                    url: None,
                },
            })
            .collect()
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvMetadata {
    pub language: String, // "en", "fr", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

// Helper function to get section case-insensitively
fn get_section_ci<'a>(
    toml_value: &'a toml::Value,
    section_name: &str,
) -> Option<&'a toml::value::Table> {
    if let Some(table) = toml_value.as_table() {
        for (key, value) in table {
            if key.to_lowercase() == section_name.to_lowercase() {
                // Handle both [section] (table) and [[section]] (array of tables - return first)
                if let Some(table) = value.as_table() {
                    return Some(table);
                } else if let Some(array) = value.as_array() {
                    // For [[section]], return the first table in the array
                    if let Some(first_item) = array.first() {
                        if let Some(first_table) = first_item.as_table() {
                            return Some(first_table);
                        }
                    }
                }
            }
        }
    }
    None
}

// ===== Local Conversion Logic =====

/// Escape a string for embedding inside Typst double-quoted literals.
/// Without this, AI-generated text containing `"` or `\` breaks the
/// experiences parser and causes experiences to disappear in the form editor.
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub struct CvConverter;

impl CvConverter {
    /// Convert CvJson to TOML configuration.
    ///
    /// Built as a `toml::Table` and serialized by the `toml` crate so every value
    /// is escaped correctly. The old hand-written `format!("summary = \"{}\"")`
    /// wrote raw newlines and quotes into basic strings, producing a
    /// cv_params.toml that Typst could not parse.
    pub fn to_toml(cv_data: &CvJson) -> Result<String> {
        use toml::{Table, Value};

        let str_val = |v: &Option<String>| Value::String(v.clone().unwrap_or_default());
        let str_list = |items: &[String]| {
            Value::Array(items.iter().map(|s| Value::String(s.clone())).collect())
        };

        let p = &cv_data.personal_info;
        let mut root = Table::new();
        // Personal info - FLAT structure (no [personal] section)
        root.insert("name".into(), Value::String(p.name.clone()));
        root.insert("title".into(), str_val(&p.title));
        root.insert("email".into(), str_val(&p.email));
        root.insert("phonenumber".into(), str_val(&p.phone));
        root.insert("address".into(), str_val(&p.address));
        root.insert("summary".into(), str_val(&p.summary));
        // from_files reads these back; dropping them lost them on every save.
        for (key, value) in [("linkedin", &p.linkedin), ("website", &p.website)] {
            if let Some(v) = value.as_ref().filter(|v| !v.trim().is_empty()) {
                root.insert(key.into(), Value::String(v.clone()));
            }
        }

        // Links - FLAT structure (no [personal.links] section)
        if let Some(links) = &p.links {
            let table: Table = links
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            root.insert("links".into(), Value::Table(table));
        }

        // Skills section — only emit non-empty arrays so the template doesn't render
        // empty subsections (e.g. PROGRAMMING_LANGUAGES for a nurse).
        let mut skills = Table::new();
        let sk = &cv_data.skills;
        for (key, values) in [
            ("technical", &sk.technical),
            ("programming_languages", &sk.programming_languages),
            ("frameworks", &sk.frameworks),
            ("tools", &sk.tools),
            ("soft_skills", &sk.soft_skills),
        ] {
            if let Some(items) = values {
                let cleaned: Vec<String> =
                    items.iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                if !cleaned.is_empty() {
                    skills.insert(key.into(), str_list(&cleaned));
                }
            }
        }
        root.insert("skills".into(), Value::Table(skills));

        // Education section
        if !cv_data.education.is_empty() {
            let entries = cv_data
                .education
                .iter()
                .map(|edu| {
                    let mut t = Table::new();
                    let title = if edu.institution.trim().is_empty() {
                        edu.degree.clone()
                    } else {
                        format!("{} - {}", edu.degree, edu.institution)
                    };
                    t.insert("title".into(), Value::String(title));
                    let start = edu.start_date.trim();
                    let end = edu.end_date.as_deref().map(str::trim).unwrap_or("");
                    // No dates known: leave empty rather than inventing "- Present".
                    let date = match (start.is_empty(), end.is_empty()) {
                        (true, true) => String::new(),
                        (true, false) => end.to_string(),
                        (false, true) => format!("{} - Present", start),
                        (false, false) => format!("{} - {}", start, end),
                    };
                    t.insert("date".into(), Value::String(date));
                    if let Some(location) = &edu.location {
                        t.insert("location".into(), Value::String(location.clone()));
                    }
                    Value::Table(t)
                })
                .collect();
            root.insert("education".into(), Value::Array(entries));
        }

        // Languages section
        let mut languages = Table::new();
        let l = &cv_data.languages;
        for (key, values) in [
            ("native", &l.native),
            ("fluent", &l.fluent),
            ("intermediate", &l.intermediate),
            ("basic", &l.basic),
        ] {
            if let Some(items) = values {
                languages.insert(key.into(), str_list(items));
            }
        }
        root.insert("languages".into(), Value::Table(languages));

        // Styling section
        let mut styling = Table::new();
        styling.insert("primary_color".into(), Value::String("#14A4E6".into()));
        styling.insert("secondary_color".into(), Value::String("#757575".into()));
        root.insert("styling".into(), Value::Table(styling));

        let out = toml::to_string(&root).context("Failed to serialize cv_params.toml")?;
        // Belt and braces: never hand back TOML we cannot read ourselves.
        toml::from_str::<toml::Table>(&out).context("Generated cv_params.toml does not parse")?;
        Ok(out)
    }

    /// Convert CvJson to Typst experiences content
    pub fn to_typst(cv_data: &CvJson, language: &str) -> Result<String> {
        let mut typst_content = String::new();

        // Import statement
        typst_content.push_str("#import \"template.typ\": *\n\n");

        // Work experience function — no section heading here.
        // Each template renders its own section title (via get_text or section())
        // so it can control style and avoid duplicate headings.
        typst_content.push_str("#let get_work_experience() = [\n");

        // Process experiences
        for exp in &cv_data.work_experience {
            let date_range = if let Some(end) = &exp.end_date {
                format!("{} - {}", exp.start_date, end)
            } else {
                match language {
                    "fr" => format!("{} - Présent", exp.start_date),
                    "de" => format!("{} - Heute", exp.start_date),
                    _ => format!("{} - Present", exp.start_date),
                }
            };

            typst_content.push_str(&format!("  == {}\n", exp.company));
            typst_content.push_str("  #dated_experience(\n");
            typst_content.push_str(&format!("    \"{}\",\n", escape_typst(&exp.title)));
            typst_content.push_str(&format!("    date: \"{}\",\n", escape_typst(&date_range)));

            // Only emit description when it adds new information — drop it if it
            // duplicates a responsibility (a common artifact of LLM-assisted imports),
            // otherwise the same text shows twice and pushes the job title out of view.
            if let Some(desc) = &exp.description {
                let norm = desc.trim().to_lowercase();
                let duplicates_resp = !norm.is_empty()
                    && exp
                        .responsibilities
                        .iter()
                        .any(|r| r.trim().to_lowercase() == norm);
                if !norm.is_empty() && !duplicates_resp {
                    typst_content
                        .push_str(&format!("    description: \"{}\",\n", escape_typst(desc)));
                }
            }

            typst_content.push_str("    content: [\n");

            // Add responsibilities
            for responsibility in &exp.responsibilities {
                typst_content.push_str(&format!(
                    "      #experience_details(\"{}\")\n",
                    escape_typst(responsibility)
                ));
            }

            // Add achievements if present
            if let Some(achievements) = &exp.achievements {
                for achievement in achievements {
                    typst_content.push_str(&format!(
                        "      #experience_details(\"{}\")\n",
                        escape_typst(achievement)
                    ));
                }
            }

            typst_content.push_str("    ]\n");
            typst_content.push_str("  )\n\n");
        }

        typst_content.push_str("]\n");
        Ok(typst_content)
    }

    /// Load CV data from existing TOML and Typst files
    pub fn from_files(
        toml_path: &std::path::Path,
        typst_path: &std::path::Path,
    ) -> Result<CvJson> {
        // Parse existing TOML file
        let toml_content =
            std::fs::read_to_string(toml_path).context("Failed to read TOML file")?;

        let toml_value: toml::Value =
            toml::from_str(&toml_content).context("Failed to parse TOML")?;

        // Helper function to get field from either root level or personal section
        let get_personal_field = |field_name: &str| -> String {
            // Try root level first (flat structure)
            if let Some(value) = toml_value.get(field_name).and_then(|v| v.as_str()) {
                return value.to_string();
            }
            // Try personal section (nested structure)
            if let Some(personal_section) = get_section_ci(&toml_value, "personal") {
                if let Some(value) = personal_section.get(field_name).and_then(|v| v.as_str()) {
                    return value.to_string();
                }
            }
            // Try personal_info section (alternative nested structure)
            if let Some(personal_section) = get_section_ci(&toml_value, "personal_info") {
                if let Some(value) = personal_section.get(field_name).and_then(|v| v.as_str()) {
                    return value.to_string();
                }
            }
            // Return empty string instead of None
            String::new()
        };

        let personal_info = PersonalInfo {
            name: {
                let name = get_personal_field("name");
                if name.is_empty() {
                    "Unknown".to_string()
                } else {
                    name
                }
            },
            title: Some(get_personal_field("title")),
            email: Some(get_personal_field("email")),
            phone: Some(get_personal_field("phonenumber")),
            address: Some(get_personal_field("address")),
            linkedin: Some(get_personal_field("linkedin")),
            website: Some(get_personal_field("website")),
            summary: Some(get_personal_field("summary")),
            links: None, // TODO: Parse links if needed
        };

        // Extract skills using case-insensitive lookup
        let skills = if let Some(skills_section) = get_section_ci(&toml_value, "skills") {
            Skills {
                technical: skills_section
                    .get("technical")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                programming_languages: skills_section
                    .get("programming_languages")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                frameworks: skills_section
                    .get("frameworks")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                tools: skills_section
                    .get("tools")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                soft_skills: None,
                other: None,
            }
        } else {
            Skills {
                technical: None,
                programming_languages: None,
                frameworks: None,
                tools: None,
                soft_skills: None,
                other: None,
            }
        };

        // Extract languages using case-insensitive lookup
        let languages = if let Some(lang_section) = get_section_ci(&toml_value, "languages") {
            Languages {
                native: lang_section
                    .get("native")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                fluent: lang_section
                    .get("fluent")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                intermediate: lang_section
                    .get("intermediate")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                basic: lang_section
                    .get("basic")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
            }
        } else {
            Languages {
                native: None,
                fluent: None,
                intermediate: None,
                basic: None,
            }
        };

        // Extract education using case-insensitive lookup
        let education = if let Some(edu_array) = get_section_ci(&toml_value, "education")
            .and_then(|_| toml_value.get("education"))
            .and_then(|v| v.as_array())
        {
            edu_array
                .iter()
                .filter_map(|edu| {
                    let table = edu.as_table()?;
                    let (degree, institution) =
                        split_education_title(table.get("title")?.as_str()?);
                    let (start_date, end_date) = split_education_date(
                        table.get("date").and_then(|v| v.as_str()).unwrap_or(""),
                    );
                    Some(Education {
                        institution,
                        degree,
                        field: None,
                        start_date,
                        end_date,
                        gpa: None,
                        honors: None,
                        location: table
                            .get("location")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        // Parse work experience from the Typst file
        let typst_content = std::fs::read_to_string(typst_path)
            .unwrap_or_default();
        let work_experience = parse_typst_experiences(&typst_content);

        Ok(CvJson {
            personal_info,
            work_experience,
            education,
            skills,
            languages,
            projects: None,
            certifications: None,
            metadata: CvMetadata {
                language: "en".to_string(),
                template: Some("default".to_string()),
                last_updated: None,
                version: None,
            },
        })
    }
}

// ── Education title/date parsing ───────────────────────────────────────────────

/// Placeholders written by older versions of `from_files`; stripped on read so
/// already-corrupted profiles heal on their next save.
fn is_placeholder(s: &str) -> bool {
    matches!(s.trim(), "" | "Unknown" | "Unknown Institution")
}

/// Inverse of `to_toml`'s `"{degree} - {institution}"`. Splits on the last
/// " - " so degrees containing a dash keep it.
fn split_education_title(title: &str) -> (String, String) {
    let mut title = title.trim();
    while let Some(rest) = title.strip_suffix(" - Unknown Institution") {
        title = rest.trim_end();
    }
    match title.rsplit_once(" - ") {
        Some((degree, institution)) if !degree.trim().is_empty() => {
            (degree.trim().to_string(), institution.trim().to_string())
        }
        _ => (title.to_string(), String::new()),
    }
}

/// Inverse of `to_toml`'s `"{start} - {end|Present}"`.
fn split_education_date(date: &str) -> (String, Option<String>) {
    let clean = |s: &str| if is_placeholder(s) { String::new() } else { s.trim().to_string() };
    match date.split_once(" - ") {
        Some((start, end)) => {
            let end = end.trim();
            let end = if end.eq_ignore_ascii_case("present") || is_placeholder(end) {
                None
            } else {
                Some(end.to_string())
            };
            (clean(start), end)
        }
        // A single value is a graduation date; keep it as the end so it isn't
        // rewritten as "{date} - Present".
        None => {
            let d = clean(date);
            (String::new(), if d.is_empty() { None } else { Some(d) })
        }
    }
}

// ── Typst experience parser ────────────────────────────────────────────────────

/// Parse a Typst experiences file (generated by `to_typst`) into `Experience` entries.
/// Handles the pattern:
///   == COMPANY
///   #dated_experience("TITLE", date: "START - END", description: "...", content: [
///     #experience_details("RESPONSIBILITY")
///   ])
fn parse_typst_experiences(content: &str) -> Vec<Experience> {
    let mut result = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("== ") {
            let company = trimmed[3..].trim().to_string();
            let mut exp = Experience {
                company,
                title: String::new(),
                start_date: String::new(),
                end_date: None,
                description: None,
                responsibilities: Vec::new(),
                achievements: None,
                technologies: None,
                location: None,
            };
            i += 1;

            // Scan forward to find #dated_experience(
            while i < lines.len() && !lines[i].trim().starts_with("#dated_experience(") {
                i += 1;
            }
            if i >= lines.len() {
                result.push(exp);
                continue;
            }

            // Collect the full block by tracking parenthesis depth
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

            // Extract title (first quoted string after the opening paren)
            if let Some(title) = typ_extract_first_quoted(&block) {
                exp.title = title;
            }
            // Extract date: "START - END"
            if let Some(date) = typ_extract_named_arg(&block, "date") {
                let parts: Vec<&str> = date.splitn(2, " - ").collect();
                exp.start_date = parts[0].trim().to_string();
                if parts.len() > 1 {
                    let end = parts[1].trim().to_string();
                    if end.is_empty() || end == "Present" || end == "Présent" {
                        exp.end_date = None; // current position
                    } else {
                        exp.end_date = Some(end);
                    }
                }
            }
            // Extract optional description
            if let Some(desc) = typ_extract_named_arg(&block, "description") {
                if !desc.is_empty() {
                    exp.description = Some(desc);
                }
            }
            // Extract responsibilities from #experience_details("...")
            exp.responsibilities = typ_extract_details(&block);

            result.push(exp);
        } else {
            i += 1;
        }
    }

    result
}

fn typ_extract_first_quoted(text: &str) -> Option<String> {
    // Tolerate whitespace/newlines between the opening `(` and the `"` — the
    // generator emits the title on its own line, so a naive `find("(\"")`
    // would silently miss the title and produce `title=""` on every round-trip.
    let paren = text.find('(')?;
    let after = text[paren + 1..].trim_start();
    if !after.starts_with('"') {
        return None;
    }
    typ_collect_quoted(&after[1..])
}

fn typ_extract_named_arg(text: &str, key: &str) -> Option<String> {
    let needle = format!("{}:", key);
    let pos = text.find(&needle)?;
    let after = text[pos + needle.len()..].trim_start();
    if after.starts_with('"') {
        typ_collect_quoted(&after[1..])
    } else {
        None
    }
}

/// Collect characters after the opening `"` until the matching `"`, honouring `\"` escapes.
fn typ_collect_quoted(s: &str) -> Option<String> {
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

fn typ_extract_details(block: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = block;
    let needle = "#experience_details(";
    while let Some(pos) = remaining.find(needle) {
        remaining = &remaining[pos + needle.len()..];
        let after = remaining.trim_start_matches(|c: char| c.is_ascii_whitespace());
        if after.starts_with('"') {
            if let Some(s) = typ_collect_quoted(&after[1..]) {
                result.push(s);
                remaining = &after[1..];
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cv_with_summary(summary: &str) -> CvJson {
        serde_json::from_value(serde_json::json!({
            "personal_info": {
                "name": "Jane \"JD\" Doe",
                "summary": summary,
                "linkedin": "https://linkedin.com/in/jane",
                "links": { "my site": "https://x.y" }
            },
            "work_experience": [],
            "education": [{ "institution": "EPFL", "degree": "MSc \\ CS", "start_date": "2010" }],
            "skills": { "technical": ["Rust", "C\"++"] },
            "languages": { "native": ["French"] },
            "metadata": { "language": "en" }
        }))
        .unwrap()
    }

    #[test]
    fn to_toml_escapes_newlines_quotes_and_emoji() {
        // Regression: a multi-line AI summary was written as raw newlines inside
        // "…", giving "failed to parse TOML (invalid basic string)" in Typst.
        let summary = "Staff Software Engineer and Founder.\n🔹 Building api0.ai: \"MCP\" gateway\n\tC:\\path";
        let out = CvConverter::to_toml(&cv_with_summary(summary)).unwrap();
        let parsed: toml::Table = toml::from_str(&out).unwrap();

        assert_eq!(parsed["summary"].as_str().unwrap(), summary);
        assert_eq!(parsed["name"].as_str().unwrap(), "Jane \"JD\" Doe");
        assert_eq!(parsed["linkedin"].as_str().unwrap(), "https://linkedin.com/in/jane");
        assert_eq!(parsed["links"]["my site"].as_str().unwrap(), "https://x.y");
        assert_eq!(parsed["skills"]["technical"][1].as_str().unwrap(), "C\"++");
        assert_eq!(parsed["education"][0]["title"].as_str().unwrap(), "MSc \\ CS - EPFL");
    }

    #[test]
    fn to_toml_round_trips_through_from_files() {
        let dir = tempfile::tempdir().unwrap();
        let summary = "Line one.\nLine two.";
        let cv = cv_with_summary(summary);
        let toml_path = dir.path().join("cv_params.toml");
        let typ_path = dir.path().join("experiences_en.typ");
        std::fs::write(&toml_path, CvConverter::to_toml(&cv).unwrap()).unwrap();
        std::fs::write(&typ_path, CvConverter::to_typst(&cv, "en").unwrap()).unwrap();

        let back = CvConverter::from_files(&toml_path, &typ_path).unwrap();
        assert_eq!(back.personal_info.summary.as_deref(), Some(summary));
        assert_eq!(back.personal_info.linkedin.as_deref(), Some("https://linkedin.com/in/jane"));
        assert_eq!(back.education[0].degree, "MSc \\ CS");
        assert_eq!(back.education[0].institution, "EPFL");
        assert_eq!(back.education[0].start_date, "2010");
        assert_eq!(back.education[0].end_date, None);
    }

    #[test]
    fn education_title_and_date_parse_back() {
        assert_eq!(
            split_education_title("Master - Centrale Lyon"),
            ("Master".into(), "Centrale Lyon".into())
        );
        // Heals profiles corrupted by the old "Unknown Institution" placeholder.
        assert_eq!(
            split_education_title("Diplôme d'ingénieur - INSA Lyon - Unknown Institution - Unknown Institution"),
            ("Diplôme d'ingénieur".into(), "INSA Lyon".into())
        );
        assert_eq!(split_education_title("PhD"), ("PhD".into(), String::new()));
        assert_eq!(split_education_date("Unknown - Present"), (String::new(), None));
        assert_eq!(split_education_date("2015 - 2018"), ("2015".into(), Some("2018".into())));
        assert_eq!(split_education_date("2018"), (String::new(), Some("2018".into())));
        assert_eq!(split_education_date(""), (String::new(), None));
    }

    #[test]
    fn certifications_as_strings() {
        let json = r#"{
            "personal_info": { "name": "Test" },
            "work_experience": [],
            "education": [],
            "skills": {},
            "languages": {},
            "certifications": [
                "AWS Solutions Architect",
                "Certifications en architectures logicielles"
            ],
            "metadata": { "language": "fr" }
        }"#;
        let cv: CvJson = serde_json::from_str(json).expect("should parse string certifications");
        let certs = cv.certifications.unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0].name, "AWS Solutions Architect");
        assert!(certs[0].issuer.is_empty());
    }

    #[test]
    fn certifications_as_structs() {
        let json = r#"{
            "personal_info": { "name": "Test" },
            "work_experience": [],
            "education": [],
            "skills": {},
            "languages": {},
            "certifications": [
                { "name": "AWS SAA", "issuer": "Amazon", "date": "2023" }
            ],
            "metadata": { "language": "en" }
        }"#;
        let cv: CvJson = serde_json::from_str(json).expect("should parse struct certifications");
        let certs = cv.certifications.unwrap();
        assert_eq!(certs[0].issuer, "Amazon");
    }

    #[test]
    fn skills_other_accepts_string_value() {
        // Real-world payload: cv-import sometimes returns a plain string instead of a list
        // for entries inside skills.other (e.g. "certifications": "AFGSU2 Obtenu en 2024").
        let json = r#"{
            "personal_info": { "name": "Test" },
            "work_experience": [],
            "education": [],
            "skills": {
                "technical": [],
                "other": { "certifications": "AFGSU2 Obtenu en 2024" }
            },
            "languages": {},
            "metadata": { "language": "fr" }
        }"#;
        let cv: CvJson = serde_json::from_str(json).expect("should parse string value in skills.other");
        let other = cv.skills.other.unwrap();
        assert_eq!(other.get("certifications").unwrap(), &vec!["AFGSU2 Obtenu en 2024".to_string()]);
    }

    #[test]
    fn certifications_mixed() {
        let json = r#"{
            "personal_info": { "name": "Test" },
            "work_experience": [],
            "education": [],
            "skills": {},
            "languages": {},
            "certifications": [
                "Plain cert",
                { "name": "Structured", "issuer": "Org", "date": "2024" }
            ],
            "metadata": { "language": "en" }
        }"#;
        let cv: CvJson = serde_json::from_str(json).expect("should parse mixed certifications");
        let certs = cv.certifications.unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0].name, "Plain cert");
        assert_eq!(certs[1].issuer, "Org");
    }
}
