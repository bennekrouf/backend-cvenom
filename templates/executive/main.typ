#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text, achievement_box
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Executive Summary ──────────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 10pt, details.summary)
}

// ── Key Achievements ───────────────────────────────────────────────────────────
#if details.at("key_achievements", default: none) != none {
  section(get_text("key_achievements"))
  achievement_box(details.key_achievements)
}

// ── Professional Experience ────────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()

// ── Core Competencies ──────────────────────────────────────────────────────────
#if "skills" in details {
  section(get_text("technical_skills"))
  show_skills(details.skills)
}

// ── Education & Certifications ─────────────────────────────────────────────────
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
