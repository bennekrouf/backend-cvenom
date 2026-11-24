// src/web/handlers/cv_handlers/skills_matrix.rs
//! POST /profiles/:name/skills-matrix/suggest — default values for
//! enterprise3's skills matrix.
//!
//! Years are computed from the missions (their dates, and whether they mention
//! the skill) and levels guessed from the years; with `ai: true` the AI then
//! refines the levels and fills the years nothing mentioned. Values the user
//! entered are never changed. Nothing is written: the editor shows the
//! suggestion and saves it with the rest of the form.

use super::cv_data::parse_experiences_typ;
use crate::auth::AuthenticatedUser;
use crate::core::ai_region;
use crate::core::database::get_tenant_folder_path;
use crate::core::skills_matrix::{self as sm, MatrixGroup, Mission, SRC_AI, SRC_USER};
use crate::core::ServiceClient;
use crate::utils::normalize_language;
use crate::web::handlers::payment_handlers::check_and_deduct_ai_credits;
use crate::web::types::StandardErrorResponse;
use graflog::app_log;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct SuggestSkillsMatrixRequest {
    /// The matrix as the editor has it. Empty: start from the stored one, or
    /// from the profile's skills, areas of expertise and sectors.
    #[serde(default)]
    pub groups: Vec<MatrixGroup>,
    /// Ask the AI too (costs credits).
    #[serde(default)]
    pub ai: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SuggestSkillsMatrixResponse {
    pub groups: Vec<MatrixGroup>,
    pub ai_used: bool,
    /// Set when the AI was asked but failed: the computed values still came back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_error: Option<String>,
}

fn err(message: String, code: &str) -> Json<StandardErrorResponse> {
    Json(StandardErrorResponse::new(message, code.to_string(), vec![], None))
}

/// Forget every value the user didn't type, so it is worked out again.
fn reset_suggested(groups: &mut [MatrixGroup]) {
    for it in groups.iter_mut().flat_map(|g| g.items.iter_mut()) {
        if it.years_source.as_deref().is_some_and(|s| s != SRC_USER) {
            it.years.clear();
            it.years_source = None;
        }
        if it.level_source.as_deref().is_some_and(|s| s != SRC_USER) {
            it.level = 0;
            it.level_source = None;
        }
    }
}

/// Take the AI's values for everything the user didn't set. Items are matched
/// by category and name; the AI can't add or remove rows.
fn merge_ai(groups: &mut [MatrixGroup], ai: &[MatrixGroup]) {
    for g in groups.iter_mut() {
        let ai_group = ai.iter().find(|a| a.category.trim().eq_ignore_ascii_case(g.category.trim()));
        for it in g.items.iter_mut() {
            let found = ai_group
                .into_iter()
                .chain(ai.iter())
                .flat_map(|a| a.items.iter())
                .find(|a| a.name.trim().eq_ignore_ascii_case(it.name.trim()));
            let Some(a) = found else { continue };
            // Computed years come from the dates and stay; the AI only fills gaps.
            if it.years.trim().is_empty() && !a.years.trim().is_empty() {
                it.years = a.years.trim().to_string();
                it.years_source = Some(SRC_AI.into());
            }
            if it.level_source.as_deref() != Some(SRC_USER) && (1..=4).contains(&a.level) {
                it.level = a.level;
                it.level_source = Some(SRC_AI.into());
            }
        }
    }
}

pub async fn suggest_skills_matrix_handler(
    profile_name: String,
    lang: Option<String>,
    request: SuggestSkillsMatrixRequest,
    auth: AuthenticatedUser,
    data_dir: &std::path::Path,
    cv_service_url: &str,
) -> Result<Json<SuggestSkillsMatrixResponse>, Json<StandardErrorResponse>> {
    let lang = normalize_language(lang.as_deref());
    // Same lookup as /cv-data, which the editor uses for this profile.
    let profile = profile_name.trim().to_string();
    if profile.is_empty() || profile.contains(['/', '\\']) || profile.contains("..") {
        return Err(err("Invalid profile name".into(), "INVALID_PROFILE"));
    }
    let profile_dir = get_tenant_folder_path(auth.email(), &data_dir.to_path_buf()).join(&profile);
    if !profile_dir.is_dir() {
        return Err(err(format!("Profile '{}' not found", profile_name), "PROFILE_NOT_FOUND"));
    }

    let toml_text = tokio::fs::read_to_string(profile_dir.join("cv_params.toml")).await.unwrap_or_default();
    let params: toml::Value = toml::from_str(&toml_text).unwrap_or(toml::Value::Table(Default::default()));

    let mut exp_text = String::new();
    for name in [format!("experiences_{lang}.typ"), "experiences_en.typ".into(), "experiences.typ".into()] {
        if let Ok(t) = tokio::fs::read_to_string(profile_dir.join(name)).await {
            exp_text = t;
            break;
        }
    }
    let missions: Vec<Mission> = parse_experiences_typ(&exp_text)
        .into_iter()
        .map(|e| Mission {
            date: e.date,
            text: [e.title, e.description, e.responsibilities.join("\n"), e.technologies.join(", ")].join("\n"),
        })
        .collect();

    let mut groups = if !request.groups.is_empty() {
        let mut g = request.groups;
        reset_suggested(&mut g);
        sm::fill_defaults(&mut g, &missions);
        g
    } else {
        let stored = sm::read(&params);
        if stored.is_empty() {
            sm::suggest_from_profile(&params, &missions, &lang)
        } else {
            let mut g = stored;
            sm::fill_defaults(&mut g, &missions);
            g
        }
    };

    let mut ai_used = false;
    let mut ai_error = None;
    if request.ai && groups.iter().any(|g| !g.items.is_empty()) {
        check_and_deduct_ai_credits(&auth, ai_region::SKILLS_MATRIX, cv_service_url, None, "skills_matrix").await?;
        let client = ServiceClient::new_in_region(cv_service_url.to_string(), 120, auth.ai_region())
            .map_err(|e| err(format!("Failed to create service client: {e}"), "SERVICE_CLIENT_ERROR"))?;
        let missions_json: Vec<_> = missions
            .iter()
            .map(|m| serde_json::json!({ "date": m.date, "text": m.text }))
            .collect();
        let payload = serde_json::json!({ "groups": groups, "missions": missions_json, "lang": lang });
        match client.post_json::<_, AiResponse>("/skills-matrix", &payload).await {
            Ok(r) if !r.status.starts_with("error") => {
                merge_ai(&mut groups, &r.groups);
                ai_used = true;
            }
            Ok(r) => ai_error = Some(r.status),
            Err(e) => ai_error = Some(e.to_string()),
        }
        if let Some(e) = &ai_error {
            app_log!(warn, profile = %profile, "Skills matrix AI pass failed: {}", e);
        }
    }

    app_log!(info, user = %auth.email(), profile = %profile, ai = ai_used, "Suggested skills matrix ({} groups)", groups.len());
    Ok(Json(SuggestSkillsMatrixResponse { groups, ai_used, ai_error }))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct AiResponse {
    #[serde(default)]
    groups: Vec<MatrixGroup>,
    #[serde(default)]
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::skills_matrix::{MatrixItem, SRC_COMPUTED, SRC_ESTIMATE};

    fn item(name: &str, years: &str, ys: Option<&str>, level: u8, ls: Option<&str>) -> MatrixItem {
        MatrixItem {
            name: name.into(),
            years: years.into(),
            level,
            years_source: ys.map(String::from),
            level_source: ls.map(String::from),
        }
    }

    #[test]
    fn ai_fills_gaps_and_levels_but_never_user_values() {
        let mut groups = vec![MatrixGroup {
            category: "Tech".into(),
            items: vec![
                item("Rust", "5", Some(SRC_COMPUTED), 3, Some(SRC_ESTIMATE)),
                item("Kafka", "", None, 0, None),
                item("Java", "15+", Some(SRC_USER), 4, Some(SRC_USER)),
            ],
        }];
        let ai = vec![MatrixGroup {
            category: "tech".into(),
            items: vec![item("rust", "7", None, 4, None), item("Kafka", "2", None, 2, None), item("Java", "3", None, 1, None)],
        }];
        merge_ai(&mut groups, &ai);
        let it = &groups[0].items;
        assert_eq!((it[0].years.as_str(), it[0].level), ("5", 4));
        assert_eq!((it[1].years.as_str(), it[1].level), ("2", 2));
        assert_eq!(it[1].years_source.as_deref(), Some(SRC_AI));
        assert_eq!((it[2].years.as_str(), it[2].level), ("15+", 4));
    }

    #[test]
    fn reset_keeps_only_user_values() {
        let mut groups = vec![MatrixGroup {
            category: "Tech".into(),
            items: vec![item("Rust", "5", Some(SRC_COMPUTED), 4, Some(SRC_USER))],
        }];
        reset_suggested(&mut groups);
        assert_eq!(groups[0].items[0].years, "");
        assert_eq!(groups[0].items[0].level, 4);
    }
}
