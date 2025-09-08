pub fn normalize_person_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '_' | '.' | '-' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn normalize_language(lang: Option<&str>) -> String {
    let lang_lower = lang.unwrap_or("en").to_lowercase();
    match lang_lower.as_str() {
        "fr" | "french" | "français" => "fr".to_string(),
        "en" | "english" | "anglais" => "en".to_string(),
        _ => "en".to_string(),
    }
}
