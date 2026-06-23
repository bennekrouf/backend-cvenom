//! Branding resolver — turns the user-facing knobs in `[styling]` into a flat
//! map of typst `sys.inputs` keys.
//!
//! Regression-safety contract:
//!   - We forward ONLY keys the user explicitly set (or a vibe preset filled
//!     in). Keys we never forward fall through to each template's literal
//!     `sys.inputs.at("k", default: <historical-value>)` fallback. That
//!     guarantees legacy profiles render byte-identically.
//!   - Vibe presets only apply when the user explicitly chose one.

use std::collections::BTreeMap;

use crate::web::handlers::cv_handlers::cv_data::StylingData;

/// Stable, sorted map of `sys.inputs` keys to forward to typst.
pub type TypstInputs = BTreeMap<&'static str, String>;

/// Build the typst input set for this styling. Empty map = forward nothing
/// new (i.e. legacy behavior).
pub fn resolve(styling: &StylingData) -> TypstInputs {
    let mut out: TypstInputs = BTreeMap::new();

    // 1. Vibe preset, if the user picked one.
    if let Some(preset) = vibe_preset(&styling.vibe) {
        for (k, v) in preset {
            out.insert(k, v.to_string());
        }
        out.insert("vibe", styling.vibe.clone());
    }

    // 2. Explicit user overrides (highest precedence).
    set_if_present(&mut out, "primary_color",    &styling.primary_color);
    set_if_present(&mut out, "secondary_color",  &styling.secondary_color);
    set_if_present(&mut out, "accent_color",     &styling.accent_color);
    set_if_present(&mut out, "neutral_color",    &styling.neutral_color);
    set_if_present(&mut out, "background_tone",  &styling.background_tone);
    set_if_present(&mut out, "font_personality", &styling.font_personality);
    set_if_present(&mut out, "density",          &styling.density);
    set_if_present(&mut out, "layout",           &styling.layout);
    set_if_present(&mut out, "divider",          &styling.divider);
    set_if_present(&mut out, "header_style",     &styling.header_style);
    set_if_present(&mut out, "photo_shape",      &styling.photo_shape);
    set_if_present(&mut out, "icon_style",       &styling.icon_style);
    set_if_present(&mut out, "skill_style",      &styling.skill_style);
    set_if_present(&mut out, "date_style",       &styling.date_style);
    set_if_present(&mut out, "lang_style",       &styling.lang_style);
    set_if_present(&mut out, "label_tone",       &styling.label_tone);
    set_if_present(&mut out, "paper",            &styling.paper);

    out
}

fn set_if_present(out: &mut TypstInputs, key: &'static str, value: &str) {
    if !value.is_empty() {
        out.insert(key, value.to_string());
    }
}

fn vibe_preset(vibe: &str) -> Option<Vec<(&'static str, &'static str)>> {
    let preset: Vec<(&str, &str)> = match vibe {
        "corporate" => vec![
            ("primary_color",    "#E11937"),
            ("accent_color",     "#1A1A1A"),
            ("font_personality", "modern_sans"),
            ("layout",           "sidebar_left"),
            ("divider",          "hairline"),
        ],
        "consulting" => vec![
            ("primary_color",    "#14365C"),
            ("accent_color",     "#C9A24B"),
            ("font_personality", "classic_serif"),
            ("layout",           "header_banner"),
            ("divider",          "bold"),
        ],
        "creative" => vec![
            ("primary_color",    "#FF4F64"),
            ("accent_color",     "#2D2D2D"),
            ("font_personality", "geometric"),
            ("layout",           "header_banner"),
            ("divider",          "none"),
            ("density",          "generous"),
        ],
        "academic" => vec![
            ("primary_color",    "#1F3A5F"),
            ("accent_color",     "#7A5C2E"),
            ("font_personality", "classic_serif"),
            ("density",          "compact"),
        ],
        "legal" => vec![
            ("primary_color",    "#0B2545"),
            ("accent_color",     "#8B7355"),
            ("font_personality", "classic_serif"),
            ("divider",          "bold"),
            ("density",          "compact"),
        ],
        "tech" => vec![
            ("primary_color",    "#6E40C9"),
            ("accent_color",     "#14A4E6"),
            ("font_personality", "geometric"),
            ("layout",           "sidebar_left"),
        ],
        "minimal" => vec![
            ("primary_color",    "#000000"),
            ("accent_color",     "#888888"),
            ("font_personality", "humanist"),
            ("divider",          "none"),
            ("density",          "generous"),
        ],
        _ => return None,
    };
    Some(preset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_styling() -> StylingData {
        let mut s = StylingData::default();
        s.primary_color   = "#14A4E6".into();
        s.secondary_color = "#757575".into();
        s
    }

    #[test]
    fn legacy_input_forwards_only_legacy_keys() {
        // The only keys forwarded should be the two the old code forwarded.
        let inputs = resolve(&legacy_styling());
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs.get("primary_color"),   Some(&"#14A4E6".to_string()));
        assert_eq!(inputs.get("secondary_color"), Some(&"#757575".to_string()));
    }

    #[test]
    fn empty_styling_forwards_nothing() {
        // A profile with no styling at all forwards no inputs — templates fall
        // back to their literal defaults.
        let inputs = resolve(&StylingData::default());
        assert!(inputs.is_empty());
    }

    #[test]
    fn vibe_preset_fills_in_keys_then_user_overrides_win() {
        let mut s = legacy_styling();
        s.vibe = "corporate".into();
        s.primary_color = "#123456".into();
        let inputs = resolve(&s);
        assert_eq!(inputs.get("primary_color"), Some(&"#123456".to_string())); // user > preset
        assert_eq!(inputs.get("accent_color"),  Some(&"#1A1A1A".to_string())); // preset
        assert_eq!(inputs.get("vibe"),          Some(&"corporate".to_string()));
    }

    #[test]
    fn unknown_vibe_is_ignored() {
        let mut s = legacy_styling();
        s.vibe = "garbage".into();
        let inputs = resolve(&s);
        assert!(inputs.get("vibe").is_none());
    }
}
