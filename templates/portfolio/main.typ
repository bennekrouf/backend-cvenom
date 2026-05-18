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

