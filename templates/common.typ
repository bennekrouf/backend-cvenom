// templates/common.typ — shared utilities for all CV templates
// Import this alongside font_config.typ to avoid duplicating helpers.

#import "font_config.typ": font_config, get_icon

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

// ── Value helper ───────────────────────────────────────────────────────────────
// True when the value is something we should render: not `none`, not an empty
// string, not whitespace-only. Trimming matters because LLM imports sometimes
// emit a single space as a placeholder, which an `== ""` check would miss.
// Used by templates' `dated_experience` to skip empty roles / descriptions
// instead of leaving a blank line (or letting the description impersonate
// the missing role).
#let nonempty(v) = {
  v != none and not (type(v) == str and v.trim() == "")
}

// ── Skill subsection label translation ─────────────────────────────────────────
// Translates skill keys (technical, programming_languages, …) used as labels
// inside the Skills section. Falls back to a humanized version of the key if
// unknown (replaces underscores with spaces and capitalizes).
#let skill_label(key) = {
  let lang = get_lang()
  let labels = (
    "en": (
      "technical": "Technical",
      "programming_languages": "Programming Languages",
      "frameworks": "Frameworks",
      "tools": "Tools",
      "soft_skills": "Soft Skills",
      "certifications": "Certifications",
    ),
    "fr": (
      "technical": "Compétences techniques",
      "programming_languages": "Langages de programmation",
      "frameworks": "Frameworks",
      "tools": "Outils",
      "soft_skills": "Savoir-être",
      "certifications": "Certifications",
    ),
    "de": (
      "technical": "Technisch",
      "programming_languages": "Programmiersprachen",
      "frameworks": "Frameworks",
      "tools": "Werkzeuge",
      "soft_skills": "Soft Skills",
      "certifications": "Zertifizierungen",
    ),
  )
  let dict = labels.at(lang, default: labels.en)
  dict.at(key, default: {
    // Humanize unknown keys: snake_case → "Snake Case"
    let parts = key.split("_")
    parts.map(p => if p.len() > 0 { upper(p.slice(0, 1)) + p.slice(1) } else { p }).join(" ")
  })
}

// ── Dictionary merge ──────────────────────────────────────────────────────────
#let join_dicts(..args) = {
  let result = (:)
  for arg in args.pos() {
    for (key, value) in arg.pairs() { result.insert(key, value) }
  }
  result
}

// ── Default social-link icons (superset — includes ORCID) ─────────────────────
#let get_default_icons(color: none) = {
  (
    "github":        ("displayname": "GitHub",   "logo": get_icon("github",        font_type: "brands")),
    "linkedin":      ("displayname": "LinkedIn", "logo": get_icon("linkedin",      font_type: "brands")),
    "personal_info": ("displayname": "Web",      "logo": get_icon("personal_info", font_type: "solid")),
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em,
      circle(radius: 0.5em, fill: color, inset: 0pt,
        align(center + horizon, text(size: 0.8em, fill: white, "iD"))))),
  )
}

// ── Link processing (handles both array and dictionary formats) ───────────────
#let process_links(color: none, icons: none, links) = {
  let resolved_icons = if icons == none {
    get_default_icons(color: color)
  } else {
    join_dicts(get_default_icons(color: color), icons)
  }
  let link_pairs = ()
  if type(links) == array {
    for l in links {
      if l != "" and l != none { link_pairs.push(("personal_info", l)) }
    }
  } else if type(links) == dictionary {
    for (key, value) in links.pairs() {
      if value != "" and value != none and type(value) == str {
        link_pairs.push((key, value))
      }
    }
  }
  if link_pairs.len() > 0 {
    link_pairs.map(it => {
      let key = it.at(0); let url = it.at(1)
      text(fill: color, link(url,
        resolved_icons.at(key, default: (:)).at("logo", default: "") + " " +
        resolved_icons.at(key, default: (:)).at("displayname", default: key)))
    })
  } else { () }
}
