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
    /// Convert CvJson to TOML configuration
    pub fn to_toml(cv_data: &CvJson) -> Result<String> {
        let mut toml_content = String::new();

        // Personal info - FLAT structure (no [personal] section)
        toml_content.push_str(&format!("name = \"{}\"\n", cv_data.personal_info.name));

        if let Some(title) = &cv_data.personal_info.title {
            toml_content.push_str(&format!("title = \"{}\"\n", title));
        } else {
            toml_content.push_str("title = \"\"\n");
        }

        if let Some(email) = &cv_data.personal_info.email {
            toml_content.push_str(&format!("email = \"{}\"\n", email));
        } else {
            toml_content.push_str("email = \"\"\n");
        }

        if let Some(phone) = &cv_data.personal_info.phone {
            toml_content.push_str(&format!("phonenumber = \"{}\"\n", phone));
        } else {
            toml_content.push_str("phonenumber = \"\"\n");
        }

        if let Some(address) = &cv_data.personal_info.address {
            toml_content.push_str(&format!("address = \"{}\"\n", address));
        } else {
            toml_content.push_str("address = \"\"\n");
        }

        if let Some(summary) = &cv_data.personal_info.summary {
            toml_content.push_str(&format!("summary = \"{}\"\n", summary));
        } else {
            toml_content.push_str("summary = \"\"\n");
        }

        // Links - FLAT structure (no [personal.links] section)
        if let Some(links) = &cv_data.personal_info.links {
            toml_content.push_str("\n[links]\n");
            for (key, value) in links {
                toml_content.push_str(&format!("{} = \"{}\"\n", key, value));
            }
        }

        // Skills section — only emit non-empty arrays so the template doesn't render
        // empty subsections (e.g. PROGRAMMING_LANGUAGES for a nurse).
        toml_content.push_str("\n[skills]\n");

        let write_skill = |buf: &mut String, key: &str, values: &Option<Vec<String>>| {
            if let Some(items) = values {
                let cleaned: Vec<&String> = items.iter().filter(|s| !s.trim().is_empty()).collect();
                if !cleaned.is_empty() {
                    buf.push_str(&format!("{} = {:?}\n", key, cleaned));
                }
            }
        };

        write_skill(&mut toml_content, "technical", &cv_data.skills.technical);
        write_skill(&mut toml_content, "programming_languages", &cv_data.skills.programming_languages);
        write_skill(&mut toml_content, "frameworks", &cv_data.skills.frameworks);
        write_skill(&mut toml_content, "tools", &cv_data.skills.tools);
        write_skill(&mut toml_content, "soft_skills", &cv_data.skills.soft_skills);

        // Education section
        if !cv_data.education.is_empty() {
            for edu in &cv_data.education {
                toml_content.push_str("\n[[education]]\n");
                toml_content.push_str(&format!(
                    "title = \"{} - {}\"\n",
                    edu.degree, edu.institution
                ));
                toml_content.push_str(&format!(
                    "date = \"{}\"\n",
                    if let Some(end) = &edu.end_date {
                        format!("{} - {}", edu.start_date, end)
                    } else {
                        format!("{} - Present", edu.start_date)
                    }
                ));
                if let Some(location) = &edu.location {
                    toml_content.push_str(&format!("location = \"{}\"\n", location));
                }
                toml_content.push_str("\n");
            }
        }

        // Languages section
        toml_content.push_str("[languages]\n");
        if let Some(native) = &cv_data.languages.native {
            toml_content.push_str(&format!("native = {:?}\n", native));
        }
        if let Some(fluent) = &cv_data.languages.fluent {
            toml_content.push_str(&format!("fluent = {:?}\n", fluent));
        }
        if let Some(intermediate) = &cv_data.languages.intermediate {
            toml_content.push_str(&format!("intermediate = {:?}\n", intermediate));
        }
        if let Some(basic) = &cv_data.languages.basic {
            toml_content.push_str(&format!("basic = {:?}\n", basic));
        }

        // Styling section
        toml_content.push_str("\n[styling]\n");
        toml_content.push_str("primary_color = \"#14A4E6\"\n");
        toml_content.push_str("secondary_color = \"#757575\"\n");

        Ok(toml_content)
    }

    /// Convert CvJson to Typst experiences content
    pub fn to_typst(cv_data: &CvJson, language: &str) -> Result<String> {
        let mut typst_content = String::new();

        // Import statement
        typst_content.push_str("#import \"template.typ\": *\n\n");

        // Stub `get_key_insights` so the `keyteo` / `keyteo_full` import
        // (`get_work_experience, get_key_insights`) resolves even though
        // this generator doesn't model key insights. Empty tuple — call
        // sites in those templates check `.len() > 0` before rendering.
        typst_content.push_str("#let get_key_insights() = ()\n\n");

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
                    Some(Education {
                        institution: "Unknown Institution".to_string(), // TODO: Parse from title
                        degree: table.get("title")?.as_str()?.to_string(),
                        field: None,
                        start_date: "Unknown".to_string(),
                        end_date: None,
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
    let start = text.find("(\"")?.saturating_add(2);
    typ_collect_quoted(&text[start..])
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
