#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text, show_practice_areas

#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Professional Profile ─────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 9.5pt, details.summary)
}

// ── Practice Areas (pills) ───────────────────────────────────────────────────
#if details.at("practice_areas", default: none) != none {
  section(get_text("practice_areas"))
  show_practice_areas(details.practice_areas)
}

// ── Professional Experience ──────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()

// ── Areas of Expertise / Skills ──────────────────────────────────────────────
#if "skills" in details {
  section(get_text("technical_skills"))
  show_skills(details.skills)
}

// ── Education & Bar Admissions ───────────────────────────────────────────────
#if "education" in details {
  section(get_text("certifications_education"))
  for item in details.education {
    dated_experience(
      item.title,
      date: item.date,
      company: item.at("location", default: none)
    )
  }
}

// ── Bar Admissions & Memberships ─────────────────────────────────────────────
#if details.at("bar_admissions", default: none) != none {
  section(get_text("bar_admissions"))
  for item in details.bar_admissions {
    grid(
      columns: (1fr, auto),
      align: (left, right),
      text(size: 9.5pt, weight: "bold", fill: rgb("#1B2A4A"), item.at("title", default: "")),
      text(size: 9pt, fill: rgb("#4A5568"), item.at("date", default: ""))
    )
    if item.at("description", default: "") != "" {
      text(size: 9pt, fill: rgb("#4A5568"), style: "italic", item.at("description", default: ""))
    }
    v(0.3em)
  }
}

// ── Publications ─────────────────────────────────────────────────────────────
#if details.at("publications", default: none) != none {
  section(get_text("publications"))
  for pub in details.publications {
    text(size: 9.5pt)[
      #text(weight: "bold", fill: rgb("#1B2A4A"), pub.at("title", default: ""))
      #if pub.at("journal", default: "") != "" [
        — #text(style: "italic", pub.at("journal", default: ""))
      ]
      #if pub.at("date", default: "") != "" [
        , #text(fill: rgb("#4A5568"), pub.at("date", default: ""))
      ]
    ]
    v(0.25em)
  }
}

// ── Languages ────────────────────────────────────────────────────────────────
#if "languages" in details {
  section(get_text("languages"))
  let lang_items = ()
  let lvl = details.languages
  let lang = sys.inputs.at("lang", default: "en")
  let labels = if lang == "fr" {
    ("native": "Langue maternelle", "fluent": "Courant", "intermediate": "Intermédiaire", "basic": "Notions")
  } else if lang == "de" {
    ("native": "Muttersprache", "fluent": "Fliessend", "intermediate": "Mittelstufe", "basic": "Grundkenntnisse")
  } else {
    ("native": "Native", "fluent": "Fluent", "intermediate": "Intermediate", "basic": "Basic")
  }
  if "native"       in lvl { for l in lvl.native       { lang_items.push(l + " (" + labels.native + ")") } }
  if "fluent"       in lvl { for l in lvl.fluent       { lang_items.push(l + " (" + labels.fluent + ")") } }
  if "intermediate" in lvl { for l in lvl.intermediate { lang_items.push(l + " (" + labels.intermediate + ")") } }
  if "basic"        in lvl { for l in lvl.basic        { lang_items.push(l + " (" + labels.basic + ")") } }
  if lang_items.len() > 0 {
    grid(
      columns: (1fr, 1fr, 1fr),
      column-gutter: 1em,
      ..lang_items.map(l => text(size: 9.5pt, l))
    )
  }
}
