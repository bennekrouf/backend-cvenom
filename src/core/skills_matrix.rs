// src/core/skills_matrix.rs
//! The skills matrix ("Compétence | Années | Niveau") that enterprise3 draws on
//! its last page, stored in cv_params.toml as
//!
//! ```toml
//! [[skills_matrix]]
//! category = "Technologies, applications & outils"
//! [[skills_matrix.items]]
//! name = "Rust"
//! years = 5          # or "10+"
//! level = 4          # 1-4
//! ```
//!
//! Default values are computed from the missions: a skill's years are the
//! months, without double counting overlaps, of every mission that mentions
//! it, and its level follows from those years. The AI may refine both later;
//! a value the user typed is never overwritten.

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// Where a value came from, so the editor can show what still needs a look.
pub const SRC_USER: &str = "user";
pub const SRC_COMPUTED: &str = "computed";
pub const SRC_ESTIMATE: &str = "estimate";
pub const SRC_AI: &str = "ai";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct MatrixItem {
    pub name: String,
    /// "5", "10+", or "" when unknown.
    #[serde(default)]
    pub years: String,
    /// 1-4, 0 when unknown.
    #[serde(default)]
    pub level: u8,
    /// Not stored in the TOML: only tells the editor how each value was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub years_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct MatrixGroup {
    pub category: String,
    #[serde(default)]
    pub items: Vec<MatrixItem>,
}

/// One mission as far as the matrix cares: its dates and all of its text.
#[derive(Debug, Clone, Default)]
pub struct Mission {
    pub date: String,
    pub text: String,
}

// ── Periods ─────────────────────────────────────────────────────────────────

/// Months as `year * 12 + month0`, so a span is `end - start + 1` months.
type Month = i32;

const PRESENT_WORDS: &[&str] = &[
    "present", "current", "now", "today", "ongoing", "to date",
    "aujourd", "actuel", "ce jour", "en cours", "maintenant",
    "heute", "aktuell", "jetzt", "laufend",
];

const MONTHS: &[(&str, i32)] = &[
    ("january", 0), ("janvier", 0), ("januar", 0), ("janv", 0), ("jan", 0), ("jän", 0),
    ("february", 1), ("février", 1), ("fevrier", 1), ("februar", 1), ("févr", 1), ("fevr", 1), ("feb", 1), ("fév", 1), ("fev", 1),
    ("march", 2), ("mars", 2), ("märz", 2), ("marz", 2), ("mar", 2), ("mär", 2),
    ("april", 3), ("avril", 3), ("apr", 3), ("avr", 3),
    ("may", 4), ("mai", 4),
    ("june", 5), ("juin", 5), ("juni", 5), ("jun", 5),
    ("july", 6), ("juillet", 6), ("juli", 6), ("juil", 6), ("jul", 6),
    ("august", 7), ("août", 7), ("aout", 7), ("aug", 7),
    ("september", 8), ("septembre", 8), ("sept", 8), ("sep", 8),
    ("october", 9), ("octobre", 9), ("oktober", 9), ("oct", 9), ("okt", 9),
    ("november", 10), ("novembre", 10), ("nov", 10),
    ("december", 11), ("décembre", 11), ("decembre", 11), ("dezember", 11), ("déc", 11), ("dec", 11), ("dez", 11),
];

fn now_month() -> Month {
    let today = chrono::Local::now().date_naive();
    today.year() * 12 + today.month0() as i32
}

/// A point in a date string: its year, and its month when one is given.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    year: i32,
    month: Option<i32>,
}

/// Every year in `s` (1950-2099), with the month written next to it:
/// "03/2018", "2018-03", "Mar 2018", "mars 2018".
fn points(s: &str) -> Vec<Point> {
    let lower = s.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 4 <= chars.len() {
        let is_year = chars[i..i + 4].iter().all(|c| c.is_ascii_digit())
            && (i == 0 || !chars[i - 1].is_ascii_digit())
            && (i + 4 == chars.len() || !chars[i + 4].is_ascii_digit());
        if !is_year {
            i += 1;
            continue;
        }
        let year: i32 = chars[i..i + 4].iter().collect::<String>().parse().unwrap_or(0);
        if !(1950..=2099).contains(&year) {
            i += 4;
            continue;
        }
        // "03/2018", "03.2018"
        let before: String = chars[..i].iter().collect();
        let mut month = None;
        let trimmed = before.trim_end_matches([' ']);
        if let Some(rest) = trimmed.strip_suffix(['/', '.', '-']) {
            let digits: String = rest.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<Vec<_>>().into_iter().rev().collect();
            if let Ok(m) = digits.parse::<i32>() {
                if (1..=12).contains(&m) && digits.len() <= 2 {
                    month = Some(m - 1);
                }
            }
        }
        // "Mar 2018", "mars 2018"
        if month.is_none() {
            let word: String = trimmed
                .trim_end_matches(['.', ','])
                .chars()
                .rev()
                .take_while(|c| c.is_alphabetic())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if !word.is_empty() {
                month = MONTHS.iter().find(|(name, _)| *name == word).map(|(_, m)| *m);
            }
        }
        // "2018-03", "2018/03"
        if month.is_none() && i + 7 <= chars.len() && matches!(chars[i + 4], '-' | '/') {
            let digits: String = chars[i + 5..(i + 7)].iter().collect();
            if let Ok(m) = digits.parse::<i32>() {
                if (1..=12).contains(&m) && (i + 7 == chars.len() || !chars[i + 7].is_ascii_digit()) {
                    month = Some(m - 1);
                }
            }
        }
        out.push(Point { year, month });
        i += 4;
    }
    out
}

/// The months a mission's date covers, both ends included.
/// "Jan 2020 - Present", "2019 - 2021", "03/2018 – 12/2020", "Depuis 2021".
pub fn period(date: &str, now: Month) -> Option<(Month, Month)> {
    let pts = points(date);
    let lower = date.to_lowercase();
    let ongoing = PRESENT_WORDS.iter().any(|w| lower.contains(w))
        || lower.contains("since") || lower.contains("depuis") || lower.contains("seit")
        || lower.trim_end().ends_with(['-', '–', '—']);
    let start = pts.first()?;
    let start_m = start.year * 12 + start.month.unwrap_or(0);
    let end_m = if pts.len() >= 2 {
        let e = pts[pts.len() - 1];
        e.year * 12 + e.month.unwrap_or(11)
    } else if ongoing {
        now
    } else {
        // A single date: that month, or that whole year.
        match start.month {
            Some(_) => start_m,
            None => start.year * 12 + 11,
        }
    };
    let end_m = end_m.min(now.max(start_m));
    (end_m >= start_m).then_some((start_m, end_m))
}

/// Months covered by the union of `spans`.
fn union_months(mut spans: Vec<(Month, Month)>) -> i32 {
    spans.sort();
    let mut total = 0;
    let mut cur: Option<(Month, Month)> = None;
    for (s, e) in spans {
        match cur {
            Some((cs, ce)) if s <= ce + 1 => cur = Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                total += ce - cs + 1;
                cur = Some((s, e));
            }
            None => cur = Some((s, e)),
        }
    }
    if let Some((cs, ce)) = cur {
        total += ce - cs + 1;
    }
    total
}

// ── Mentions ────────────────────────────────────────────────────────────────

/// The names a skill goes by: "Java / Spring Boot" → Java, Spring Boot;
/// "Azure (CosmosDB, Functions)" → Azure, CosmosDB, Functions.
pub fn aliases(name: &str) -> Vec<String> {
    let mut out = vec![name.trim().to_string()];
    // Keep compound names like CI/CD whole: only split on " / ".
    let normalized = name
        .replace(" / ", "\u{1}")
        .replace(" & ", "\u{1}")
        .replace(['(', ')', ','], "\u{1}");
    for part in normalized.split('\u{1}') {
        let p = part.trim();
        if !p.is_empty() && !out.iter().any(|o| o.eq_ignore_ascii_case(p)) {
            out.push(p.to_string());
        }
    }
    out
}

fn is_word_char(c: Option<char>, next: Option<char>) -> bool {
    match c {
        Some(c) if c.is_alphanumeric() || c == '+' || c == '#' => true,
        // "Node.js": a dot followed by a letter belongs to the word.
        Some('.') => next.is_some_and(|n| n.is_alphanumeric()),
        _ => false,
    }
}

/// Whether `needle` appears in `haystack` as a whole word, ignoring case.
pub fn mentions(haystack: &str, needle: &str) -> bool {
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    let hay = haystack.to_lowercase();
    let hay_chars: Vec<(usize, char)> = hay.char_indices().collect();
    let mut from = 0;
    while let Some(pos) = hay[from..].find(&needle) {
        let start = from + pos;
        let end = start + needle.len();
        let idx = hay_chars.partition_point(|(i, _)| *i < start);
        let before = if idx == 0 { None } else { Some(hay_chars[idx - 1].1) };
        let before_prev = if idx >= 2 { Some(hay_chars[idx - 2].1) } else { None };
        let mut after_it = hay[end..].chars();
        let after = after_it.next();
        let after_next = after_it.next();
        // For the char before, "next" is the needle's first char.
        let before_is_word = match before {
            Some('.') => before_prev.is_some_and(|p| p.is_alphanumeric()),
            other => is_word_char(other, None),
        };
        if !before_is_word && !is_word_char(after, after_next) {
            return true;
        }
        from = start + needle.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    false
}

// ── Defaults ────────────────────────────────────────────────────────────────

/// `text` with every `other` name that contains `alias` blanked out, so
/// "Azure" isn't found in "Azure DevOps" when that is a row of its own.
fn without_longer(text: &str, alias: &str, others: &[String]) -> String {
    let mut t = text.to_lowercase();
    let a = alias.to_lowercase();
    for o in others {
        let o = o.trim().to_lowercase();
        if o.len() > a.len() && mentions(&o, &a) {
            t = t.replace(&o, &" ".repeat(o.len()));
        }
    }
    t
}

/// Months of experience with `skill` across `missions`, overlaps counted once.
/// `others` are the matrix's other rows, whose names don't count for this one.
pub fn months_for(skill: &str, missions: &[Mission], others: &[String], now: Month) -> i32 {
    let names = aliases(skill);
    let others: Vec<String> = others.iter().filter(|o| !o.trim().eq_ignore_ascii_case(skill.trim())).cloned().collect();
    let spans: Vec<_> = missions
        .iter()
        .filter(|m| names.iter().any(|n| mentions(&without_longer(&m.text, n, &others), n)))
        .filter_map(|m| period(&m.date, now))
        .collect();
    union_months(spans)
}

/// Whole years for display; anything under a year still shows as 1.
pub fn years_label(months: i32) -> String {
    if months <= 0 {
        String::new()
    } else {
        ((months as f32 / 12.0).round() as i32).max(1).to_string()
    }
}

/// A first guess at the level from the years alone.
pub fn level_from_years(years: &str) -> u8 {
    let n: i32 = years.trim().trim_end_matches('+').trim().parse().unwrap_or(0);
    match n {
        0 => 0,
        1..=2 => 2,
        3..=5 => 3,
        _ => 4,
    }
}

/// Fill what is still empty in `groups` from the missions. Values already
/// there are kept and marked as the user's.
pub fn fill_defaults(groups: &mut [MatrixGroup], missions: &[Mission]) {
    let now = now_month();
    let names = all_names(groups);
    for g in groups.iter_mut() {
        for it in g.items.iter_mut() {
            if !it.years.trim().is_empty() {
                it.years_source.get_or_insert_with(|| SRC_USER.into());
            } else {
                let y = years_label(months_for(&it.name, missions, &names, now));
                if !y.is_empty() {
                    it.years = y;
                    it.years_source = Some(SRC_COMPUTED.into());
                }
            }
            if it.level > 0 {
                it.level_source.get_or_insert_with(|| SRC_USER.into());
            } else {
                let l = level_from_years(&it.years);
                if l > 0 {
                    it.level = l;
                    it.level_source = Some(SRC_ESTIMATE.into());
                }
            }
        }
    }
}

/// Every row's name, for `months_for`'s `others`.
fn all_names(groups: &[MatrixGroup]) -> Vec<String> {
    groups.iter().flat_map(|g| g.items.iter().map(|i| i.name.clone())).collect()
}

/// Category names in the CV's language, as CGI's template words them.
pub fn category_label(kind: &str, lang: &str) -> &'static str {
    match (kind, lang) {
        ("tech", "fr") => "Technologies, applications & outils",
        ("tech", "de") => "Technologien, Anwendungen & Tools",
        ("tech", _) => "Technologies, applications & tools",
        ("domains", "fr") => "Domaines des TI",
        ("domains", "de") => "IT-Fachgebiete",
        ("domains", _) => "IT domains",
        ("sectors", "fr") => "Connaissances sectorielles",
        ("sectors", "de") => "Branchenkenntnisse",
        (_, _) => "Industry knowledge",
    }
}

fn push_unique(list: &mut Vec<String>, name: &str) {
    let n = name.trim();
    if !n.is_empty() && !list.iter().any(|x| x.eq_ignore_ascii_case(n)) {
        list.push(n.to_string());
    }
}

/// A first matrix built from the profile: its skills and key skills, its
/// areas of expertise and its sectors, each with computed defaults.
pub fn suggest_from_profile(profile: &toml::Value, missions: &[Mission], lang: &str) -> Vec<MatrixGroup> {
    let str_list = |v: Option<&toml::Value>| -> Vec<String> {
        match v {
            Some(toml::Value::Array(a)) => a
                .iter()
                .filter_map(|x| match x {
                    toml::Value::String(s) => Some(s.clone()),
                    toml::Value::Table(t) => t.get("title").and_then(|t| t.as_str()).map(String::from),
                    _ => None,
                })
                .collect(),
            Some(toml::Value::String(s)) => vec![s.clone()],
            _ => vec![],
        }
    };

    let mut tech = Vec::new();
    if let Some(skills) = profile.get("skills").and_then(|v| v.as_table()) {
        for v in skills.values() {
            for s in str_list(Some(v)) {
                push_unique(&mut tech, &s);
            }
        }
    }
    for s in str_list(profile.get("key_skills")) {
        push_unique(&mut tech, &s);
    }
    let mut domains = Vec::new();
    for s in str_list(profile.get("areas_of_expertise")) {
        // "Title: description" keeps only the title.
        push_unique(&mut domains, s.split(':').next().unwrap_or(&s));
    }
    let mut sectors = Vec::new();
    for s in str_list(profile.get("sectors")) {
        push_unique(&mut sectors, &s);
    }

    let now = now_month();
    let every: Vec<String> = tech.iter().chain(&domains).chain(&sectors).cloned().collect();
    let mut groups = Vec::new();
    for (kind, names, cap) in [("tech", tech, 20usize), ("domains", domains, 10), ("sectors", sectors, 8)] {
        if names.is_empty() {
            continue;
        }
        let mut items: Vec<(i32, MatrixItem)> = names
            .into_iter()
            .map(|name| {
                let months = months_for(&name, missions, &every, now);
                (months, MatrixItem { name, ..Default::default() })
            })
            .collect();
        // Most experience first; ties keep the profile's order.
        items.sort_by(|a, b| b.0.cmp(&a.0));
        items.truncate(cap);
        groups.push(MatrixGroup {
            category: category_label(kind, lang).to_string(),
            items: items.into_iter().map(|(_, it)| it).collect(),
        });
    }
    fill_defaults(&mut groups, missions);
    groups
}

// ── TOML ────────────────────────────────────────────────────────────────────

/// The matrix stored in a parsed cv_params.toml.
pub fn read(profile: &toml::Value) -> Vec<MatrixGroup> {
    let Some(arr) = profile.get("skills_matrix").and_then(|v| v.as_array()) else {
        return vec![];
    };
    arr.iter()
        .filter_map(|g| g.as_table())
        .map(|g| MatrixGroup {
            category: g.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            items: g
                .get("items")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|i| i.as_table())
                        .map(|i| MatrixItem {
                            name: i.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            years: match i.get("years") {
                                Some(toml::Value::Integer(n)) => n.to_string(),
                                Some(toml::Value::Float(f)) => f.round().to_string(),
                                Some(toml::Value::String(s)) => s.clone(),
                                _ => String::new(),
                            },
                            level: i.get("level").and_then(|v| v.as_integer()).unwrap_or(0).clamp(0, 4) as u8,
                            ..Default::default()
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

/// `groups` as a TOML array of tables, ready to set on a document.
pub fn to_toml_edit(groups: &[MatrixGroup]) -> toml_edit::ArrayOfTables {
    let mut out = toml_edit::ArrayOfTables::new();
    for g in groups {
        let mut gt = toml_edit::Table::new();
        gt["category"] = toml_edit::value(g.category.trim());
        let mut items = toml_edit::ArrayOfTables::new();
        for it in g.items.iter().filter(|i| !i.name.trim().is_empty()) {
            let mut t = toml_edit::Table::new();
            t["name"] = toml_edit::value(it.name.trim());
            let y = it.years.trim();
            if let Ok(n) = y.parse::<i64>() {
                t["years"] = toml_edit::value(n);
            } else if !y.is_empty() {
                t["years"] = toml_edit::value(y);
            }
            if it.level > 0 {
                t["level"] = toml_edit::value(it.level.min(4) as i64);
            }
            items.push(t);
        }
        gt.insert("items", toml_edit::Item::ArrayOfTables(items));
        out.push(gt);
    }
    out
}

/// Replace the matrix in a cv_params.toml text, leaving the rest as it was.
pub fn write_into(toml_text: &str, groups: &[MatrixGroup]) -> Result<String, String> {
    let mut doc: toml_edit::DocumentMut = toml_text.parse().map_err(|e| format!("Invalid cv_params.toml: {e}"))?;
    if groups.is_empty() {
        doc.remove("skills_matrix");
    } else {
        doc.insert("skills_matrix", toml_edit::Item::ArrayOfTables(to_toml_edit(groups)));
    }
    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(y: i32, mo: i32) -> Month {
        y * 12 + mo - 1
    }

    #[test]
    fn periods_in_the_usual_shapes() {
        let now = m(2026, 11);
        assert_eq!(period("Jan 2020 - Present", now), Some((m(2020, 1), now)));
        assert_eq!(period("2019 - 2021", now), Some((m(2019, 1), m(2021, 12))));
        assert_eq!(period("03/2018 – 12/2020", now), Some((m(2018, 3), m(2020, 12))));
        assert_eq!(period("mars 2018 - aujourd'hui", now), Some((m(2018, 3), now)));
        assert_eq!(period("Depuis 2021", now), Some((m(2021, 1), now)));
        assert_eq!(period("2018-03 - 2019-06", now), Some((m(2018, 3), m(2019, 6))));
        assert_eq!(period("2022", now), Some((m(2022, 1), m(2022, 12))));
        assert_eq!(period("", now), None);
    }

    #[test]
    fn overlapping_missions_count_once() {
        let missions = vec![
            Mission { date: "2015 - 2018".into(), text: "Java, Spring Boot".into() },
            Mission { date: "2017 - 2020".into(), text: "Environnement technique : Java, Kafka".into() },
            Mission { date: "2022 - 2023".into(), text: "Rust".into() },
        ];
        let now = m(2026, 11);
        assert_eq!(months_for("Java / Spring Boot", &missions, &[], now), 6 * 12);
        assert_eq!(months_for("Rust", &missions, &[], now), 24);
        assert_eq!(months_for("JavaScript", &missions, &[], now), 0);
    }

    #[test]
    fn a_longer_row_keeps_its_mentions() {
        let missions = vec![
            Mission { date: "2020 - 2021".into(), text: "Azure DevOps".into() },
            Mission { date: "2023 - 2023".into(), text: "Azure Functions, Azure DevOps".into() },
        ];
        let now = m(2026, 11);
        let rows = vec!["Azure (Functions, KeyVault)".to_string(), "Azure DevOps".to_string()];
        assert_eq!(months_for("Azure (Functions, KeyVault)", &missions, &rows, now), 12);
        assert_eq!(months_for("Azure DevOps", &missions, &rows, now), 36);
        // Without its own row, Azure DevOps is Azure experience.
        assert_eq!(months_for("Azure", &missions, &[], now), 36);
    }

    #[test]
    fn whole_words_only() {
        assert!(mentions("Built in C and Rust", "C"));
        assert!(!mentions("Built in C++", "C"));
        assert!(!mentions("C# services", "C"));
        assert!(mentions("Node.js, TypeScript", "Node.js"));
        assert!(!mentions("JavaScript", "Java"));
        assert!(mentions("used Java.", "Java"));
        assert!(mentions("CI/CD pipelines", "CI/CD"));
        assert!(mentions("Azure (CosmosDB)", "cosmosdb"));
    }

    #[test]
    fn aliases_split_lists_but_not_ci_cd() {
        assert_eq!(aliases("Java / Spring Boot"), vec!["Java / Spring Boot", "Java", "Spring Boot"]);
        assert_eq!(aliases("CI/CD"), vec!["CI/CD"]);
        assert_eq!(aliases("Pharma & Biotech"), vec!["Pharma & Biotech", "Pharma", "Biotech"]);
        assert_eq!(aliases("Azure (CosmosDB, KeyVault)"), vec!["Azure (CosmosDB, KeyVault)", "Azure", "CosmosDB", "KeyVault"]);
    }

    #[test]
    fn user_values_are_kept() {
        let missions = vec![Mission { date: "2010 - 2020".into(), text: "Rust".into() }];
        let mut groups = vec![MatrixGroup {
            category: "Tech".into(),
            items: vec![
                MatrixItem { name: "Rust".into(), years: "3".into(), ..Default::default() },
                MatrixItem { name: "Rust".into(), ..Default::default() },
            ],
        }];
        fill_defaults(&mut groups, &missions);
        assert_eq!(groups[0].items[0].years, "3");
        assert_eq!(groups[0].items[0].years_source.as_deref(), Some(SRC_USER));
        assert_eq!(groups[0].items[0].level, 3);
        assert_eq!(groups[0].items[1].years, "11");
        assert_eq!(groups[0].items[1].level, 4);
        assert_eq!(groups[0].items[1].level_source.as_deref(), Some(SRC_ESTIMATE));
    }

    #[test]
    fn toml_round_trip_keeps_other_keys() {
        let text = "name = \"A\"\nkey_figures = [{ value = \"15+\", label = \"ans\" }]\n";
        let groups = vec![MatrixGroup {
            category: "Tech".into(),
            items: vec![
                MatrixItem { name: "Rust".into(), years: "5".into(), level: 4, ..Default::default() },
                MatrixItem { name: "Java".into(), years: "15+".into(), level: 3, ..Default::default() },
            ],
        }];
        let out = write_into(text, &groups).unwrap();
        assert!(out.contains("key_figures"));
        let v: toml::Value = toml::from_str(&out).unwrap();
        let back = read(&v);
        assert_eq!(back[0].items[0].years, "5");
        assert_eq!(back[0].items[1].years, "15+");
        assert_eq!(back[0].items[1].level, 3);
        assert_eq!(read(&toml::from_str(&write_into(&out, &[]).unwrap()).unwrap()).len(), 0);
    }
}
