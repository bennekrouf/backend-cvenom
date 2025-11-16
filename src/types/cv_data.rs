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
    pub projects: Option<Vec<Project>>,
    pub certifications: Option<Vec<Certification>>,
    pub metadata: CvMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub name: String,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub summary: Option<String>,
    pub links: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub company: String,
    pub title: String,
    pub start_date: String,
    pub end_date: Option<String>, // None means current
    pub description: Option<String>,
    pub responsibilities: Vec<String>,
    pub achievements: Option<Vec<String>>,
    pub technologies: Option<Vec<String>>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub institution: String,
    pub degree: String,
    pub field: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub gpa: Option<String>,
    pub honors: Option<Vec<String>>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub technical: Option<Vec<String>>,
    pub programming_languages: Option<Vec<String>>,
    pub frameworks: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub soft_skills: Option<Vec<String>>,
    pub other: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Languages {
    pub native: Option<Vec<String>>,
    pub fluent: Option<Vec<String>>,
    pub intermediate: Option<Vec<String>>,
    pub basic: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub technologies: Option<Vec<String>>,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    pub issuer: String,
    pub date: String,
    pub expiry: Option<String>,
    pub credential_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvMetadata {
    pub language: String, // "en", "fr", etc.
    pub template: Option<String>,
    pub last_updated: Option<String>,
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

        // Skills section
        toml_content.push_str("\n[skills]\n");

        if let Some(technical) = &cv_data.skills.technical {
            toml_content.push_str(&format!("technical = {:?}\n", technical));
        }

        if let Some(prog_langs) = &cv_data.skills.programming_languages {
            toml_content.push_str(&format!("programming_languages = {:?}\n", prog_langs));
        }

        if let Some(frameworks) = &cv_data.skills.frameworks {
            toml_content.push_str(&format!("frameworks = {:?}\n", frameworks));
        }

        if let Some(tools) = &cv_data.skills.tools {
            toml_content.push_str(&format!("tools = {:?}\n", tools));
        }

        // Education section
        if !cv_data.education.is_empty() {
            toml_content.push_str("\n[[education]]\n");
            // Education section - FIXED
            for edu in &cv_data.education {
                toml_content.push_str("[[education]]\n");
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

        // Work experience function
        typst_content.push_str("#let get_work_experience() = [\n");

        // Section title based on language
        let section_title = match language {
            "fr" => "= Expérience Professionnelle",
            _ => "= Work Experience",
        };
        typst_content.push_str(&format!("  {}\n\n", section_title));

        // Process experiences
        for exp in &cv_data.work_experience {
            let date_range = if let Some(end) = &exp.end_date {
                format!("{} - {}", exp.start_date, end)
            } else {
                match language {
                    "fr" => format!("{} - Présent", exp.start_date),
                    _ => format!("{} - Present", exp.start_date),
                }
            };

            typst_content.push_str(&format!("  == {}\n", exp.company));
            typst_content.push_str("  #dated_experience(\n");
            typst_content.push_str(&format!("    \"{}\",\n", exp.title));
            typst_content.push_str(&format!("    date: \"{}\",\n", date_range));

            if let Some(desc) = &exp.description {
                typst_content.push_str(&format!("    description: \"{}\",\n", desc));
            }

            typst_content.push_str("    content: [\n");

            // Add responsibilities
            for responsibility in &exp.responsibilities {
                typst_content.push_str("      #experience_details(\n");
                typst_content.push_str(&format!("        \"{}\"\n", responsibility));
                typst_content.push_str("      )\n");
            }

            // Add achievements if present
            if let Some(achievements) = &exp.achievements {
                for achievement in achievements {
                    typst_content.push_str("      #experience_details(\n");
                    typst_content.push_str(&format!("        \"{}\"\n", achievement));
                    typst_content.push_str("      )\n");
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
        _typst_path: &std::path::Path,
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

        Ok(CvJson {
            personal_info,
            work_experience: Vec::new(), // TODO: Parse from Typst file
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
