#import "template.typ": conf, section, project_card, show_skills, tech_badge, get_text

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Projects ──────────────────────────────────────────────────────────────────
#let projects = details.at("projects", default: ())

#if projects.len() > 0 {
  section(get_text("projects"))

  for project in projects {
    project_card(
      title:        project.at("title",        default: "Untitled"),
      role:         project.at("role",         default: none),
      date:         project.at("date",         default: none),
      description:  project.at("description",  default: none),
      technologies: project.at("technologies", default: ()),
      highlights:   project.at("highlights",   default: ()),
      url:          project.at("url",          default: none),
    )
  }
}

// ── Technical Skills ──────────────────────────────────────────────────────────
#if "skills" in details {
  section(get_text("skills"))
  show_skills(details.skills)
  v(0.5em)
}

// ── Languages ─────────────────────────────────────────────────────────────────
#if "languages" in details {
  section(get_text("languages"))
  let lvl = details.languages
  let items = ()
  if "native"       in lvl { for l in lvl.native       { items.push(l + " — Native") } }
  if "fluent"       in lvl { for l in lvl.fluent       { items.push(l + " — Fluent") } }
  if "intermediate" in lvl { for l in lvl.intermediate { items.push(l + " — Intermediate") } }
  if "basic"        in lvl { for l in lvl.basic        { items.push(l + " — Basic") } }

  grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 1em,
    row-gutter: 0.4em,
    ..items.map(l => text(size: 9.5pt, l)),
  )
}
