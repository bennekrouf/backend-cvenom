// src/core/ai_region.rs
//! The api0 AI regions cvenom's model calls run in, as GitAgent offers them.
//! Every AI action runs in a region, on the model api0 has selected for it:
//! cvenom picks no model and has no provider of its own. cv-import holds the
//! api0 licence and says which regions are on offer (`GET /regions`); a
//! user's pick is stored on their tenant, and every call to cv-import names
//! it in `X-AI-Region` (service_client.rs). A user who has not picked one
//! runs in `DEFAULT_REGION`.
//!
//! An AI action costs its base price times the region's api0 multiplier —
//! rounded up, at least 1, as GitAgent counts a step.

use anyhow::{Context, Result};

/// Every region cv-import knows, in the order the dashboard shows them.
pub const REGIONS: &[&str] = &["europe", "china", "usa"];

/// Where a user who has not picked a region runs.
pub const DEFAULT_REGION: &str = "europe";

/// One of the actions that call the AI, and its base price in credits (at a
/// multiplier of 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiAction {
    pub id: &'static str,
    pub base: i64,
}

pub const CV_IMPORT: AiAction = AiAction { id: "cv_import", base: 4 };
pub const OPTIMIZE: AiAction = AiAction { id: "optimize", base: 5 };
pub const TRANSLATE: AiAction = AiAction { id: "translate", base: 5 };
pub const COVER_LETTER: AiAction = AiAction { id: "cover_letter", base: 20 };
pub const PORTFOLIO: AiAction = AiAction { id: "portfolio", base: 20 };
pub const SKILLS_MATRIX: AiAction = AiAction { id: "skills_matrix", base: 2 };

/// Every AI action, in the order the dashboard lists their prices.
pub const AI_ACTIONS: &[AiAction] = &[CV_IMPORT, OPTIMIZE, TRANSLATE, COVER_LETTER, PORTFOLIO, SKILLS_MATRIX];

/// `base` credits at `multiplier` times the plain rate: rounded up, at least 1.
pub fn scaled(base: i64, multiplier: f64) -> i64 {
    // The epsilon keeps 5 × 1.2 at 6 rather than float noise rounding it to 7.
    ((base as f64 * multiplier.max(0.0) - 1e-9).ceil() as i64).max(1)
}

/// Each AI action's price at `multiplier`, as `{ action: credits }`.
pub fn prices(multiplier: f64) -> serde_json::Value {
    AI_ACTIONS
        .iter()
        .map(|a| (a.id.to_string(), serde_json::json!(scaled(a.base, multiplier))))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

/// What `action` costs in the region its call runs in (`resolve(region)`)
/// right now: its base price scaled by api0's multiplier there. A region that
/// is not on offer is an error, so nothing is charged for a call that would fail.
pub async fn cost(cv_service_url: &str, region: Option<&str>, action: AiAction) -> Result<i64> {
    let region = resolve(region);
    let offered = on_offer(cv_service_url).await?;
    let multiplier = offered
        .iter()
        .find(|r| r["id"].as_str() == Some(region))
        .map(|r| r["multiplier"].as_f64().unwrap_or(1.0))
        .ok_or_else(|| anyhow::anyhow!("The AI region '{region}' is not on offer right now"))?;
    Ok(scaled(action.base, multiplier))
}

/// The region a call runs in: the user's pick, or `DEFAULT_REGION`.
pub fn resolve(picked: Option<&str>) -> &'static str {
    picked.and_then(find).unwrap_or(DEFAULT_REGION)
}

/// The region `id` names, if it is one.
pub fn find(id: &str) -> Option<&'static str> {
    REGIONS.iter().copied().find(|r| *r == id.trim())
}

/// Whether a listed region can be used: only an explicit false says it can't.
pub fn is_available(region: &serde_json::Value) -> bool {
    region["available"] != false
}

/// The regions that can be used now.
pub async fn on_offer(cv_service_url: &str) -> Result<Vec<serde_json::Value>> {
    let mut regions = listed(cv_service_url).await?;
    regions.retain(is_available);
    Ok(regions)
}

/// Every region cv-import lists: `[{ id, flag, label, available, multiplier }]`,
/// no multiplier on one not open yet; empty when it has no api0 licence.
pub async fn listed(cv_service_url: &str) -> Result<Vec<serde_json::Value>> {
    let v: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/regions", cv_service_url.trim_end_matches('/')))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .context("cv-import is unreachable")?
        .json()
        .await
        .context("Bad reply from cv-import")?;
    if let Some(status) = v["status"].as_str().filter(|s| *s != "success") {
        anyhow::bail!("{status}");
    }
    Ok(v["regions"].as_array().cloned().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_knows_only_the_three_regions() {
        assert_eq!(find(" europe "), Some("europe"));
        assert_eq!(find("usa"), Some("usa"));
        assert_eq!(find("default"), None);
        assert_eq!(find("Europe"), None);
    }

    #[test]
    fn only_an_explicit_false_makes_a_region_unavailable() {
        assert!(!is_available(&serde_json::json!({ "id": "usa", "available": false })));
        assert!(is_available(&serde_json::json!({ "id": "china", "available": true })));
        assert!(is_available(&serde_json::json!({ "id": "china" })), "as an older cv-import lists it");
    }

    #[test]
    fn scaled_rounds_up_and_never_drops_to_zero() {
        assert_eq!(scaled(5, 1.0), 5);
        assert_eq!(scaled(5, 1.2), 6);
        assert_eq!(scaled(5, 1.25), 7);
        assert_eq!(scaled(20, 0.5), 10);
        assert_eq!(scaled(4, 0.01), 1);
        assert_eq!(scaled(4, -1.0), 1);
    }

    #[test]
    fn prices_cover_every_ai_action() {
        let p = prices(2.0);
        assert_eq!(p["cv_import"], 8);
        assert_eq!(p["cover_letter"], 40);
        assert_eq!(p.as_object().unwrap().len(), AI_ACTIONS.len());
    }

    #[test]
    fn a_user_without_a_pick_runs_in_the_default_region() {
        assert_eq!(resolve(None), DEFAULT_REGION);
        assert_eq!(resolve(Some("default")), DEFAULT_REGION);
        assert_eq!(resolve(Some(" china ")), "china");
    }

    #[tokio::test]
    async fn every_price_comes_from_a_region_on_offer() {
        // Nothing listens here: with no region list, nothing is priced or charged.
        assert!(cost("http://127.0.0.1:9", None, COVER_LETTER).await.is_err());
        assert!(cost("http://127.0.0.1:9", Some("china"), OPTIMIZE).await.is_err());
    }
}
