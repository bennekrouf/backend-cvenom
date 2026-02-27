#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text, publication_entry, grant_entry, interest_tag
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Research Profile ───────────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 10pt, details.summary)
}

// ── Research Interests ─────────────────────────────────────────────────────────
#if details.at("research_interests", default: none) != none {
  section(get_text("research_interests"))
  wrap(details.research_interests.map(i => interest_tag(i)).join())
  v(0.3em)
}

// ── Research & Academic Experience ────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()

// ── Publications ───────────────────────────────────────────────────────────────
#if details.at("publications", default: none) != none {
  section(get_text("publications"))
  for pub in details.publications {
    publication_entry(pub)
  }
}

// ── Grants & Funding ──────────────────────────────────────────────────────────
#if details.at("grants", default: none) != none {
  section(get_text("grants"))
  for g in details.grants {
    grant_entry(g)
  }
}

// ── Methods & Tools ────────────────────────────────────────────────────────────
#if "skills" in details {
  section(get_text("technical_skills"))
  show_skills(details.skills)
}

// ── Education ─────────────────────────────────────────────────────────────────
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

// ── Languages ─────────────────────────────────────────────────────────────────
#if "languages" in details {
  section(get_text("languages"))
  let lang_items = ()
  let lvl = details.languages
  if "native"       in lvl { for l in lvl.native       { lang_items.push(l + " (Native)") } }
  if "fluent"       in lvl { for l in lvl.fluent       { lang_items.push(l + " (Fluent)") } }
  if "intermediate" in lvl { for l in lvl.intermediate { lang_items.push(l + " (Intermediate)") } }
  if "basic"        in lvl { for l in lvl.basic        { lang_items.push(l + " (Basic)") } }
  if lang_items.len() > 0 {
    grid(
      columns: (1fr, 1fr, 1fr),
      column-gutter: 1em,
      ..lang_items.map(l => text(size: 10pt, l))
    )
  }
}
