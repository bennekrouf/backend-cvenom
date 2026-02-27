#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text, show_competencies, sector_badge
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Profile Summary ────────────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 10pt, details.summary)
}

// ── Key Competencies ───────────────────────────────────────────────────────────
#if details.at("key_competencies", default: none) != none {
  section(get_text("key_competencies"))
  show_competencies(details.key_competencies)
  v(0.3em)
}

// ── Sector Expertise ───────────────────────────────────────────────────────────
#if details.at("sectors", default: none) != none {
  let sectors = details.sectors
  let domain_list = if type(sectors) == dictionary {
    sectors.at("domains", default: ())
  } else if type(sectors) == array {
    sectors
  } else { () }

  if domain_list.len() > 0 {
    section(get_text("sectors"))
    wrap(domain_list.map(s => sector_badge(s)).join())
    v(0.3em)
  }
}

// ── Mission Experience ─────────────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()

// ── Technical Skills ───────────────────────────────────────────────────────────
#if "skills" in details {
  section(get_text("technical_skills"))
  show_skills(details.skills)
}

// ── Education & Certifications ─────────────────────────────────────────────────
#if "education" in details {
  section(get_text("certifications_education"))
  let diplomas = details.education.filter(item => item.at("type", default: "education") == "diploma")
  if diplomas.len() > 0 {
    text(weight: "bold", size: 10pt, get_text("diplomas"))
    for item in diplomas {
      experience_details(item.title + " — " + item.date)
    }
  }
  let certs = details.education.filter(item => item.at("type", default: "education") != "diploma")
  if certs.len() > 0 {
    text(weight: "bold", size: 10pt, get_text("certifications"))
    for item in certs {
      experience_details(item.title + " — " + item.date)
    }
  }
}

// ── Languages ─────────────────────────────────────────────────────────────────
#if "languages" in details {
  section(get_text("languages"))
  let lang_items = ()
  let lvl = details.languages
  if "native"       in lvl { for l in lvl.native       { lang_items.push(l + " — Native") } }
  if "fluent"       in lvl { for l in lvl.fluent       { lang_items.push(l + " — Fluent") } }
  if "intermediate" in lvl { for l in lvl.intermediate { lang_items.push(l + " — Intermediate") } }
  if "basic"        in lvl { for l in lvl.basic        { lang_items.push(l + " — Basic") } }
  if lang_items.len() > 0 {
    grid(
      columns: (1fr, 1fr, 1fr),
      column-gutter: 1em,
      row-gutter: 0.3em,
      ..lang_items.map(l => experience_details(l))
    )
  }
}
