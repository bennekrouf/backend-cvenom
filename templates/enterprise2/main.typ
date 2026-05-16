#import "template.typ": conf, date, dated_experience, experience_details, section, subsection, show_skills, get_text, show_competencies, sector_badge
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Profile Summary ────────────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 9.5pt, details.summary)
  v(0.2em)
}

// ── Work Experience ────────────────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()

// ── Technical Specializations ──────────────────────────────────────────────────
#if "skills" in details {
  section(get_text("technical_skills"))
  show_skills(details.skills)
}

// ── Areas of Expertise ─────────────────────────────────────────────────────────
#if details.at("areas_of_expertise", default: none) != none {
  section(get_text("areas_of_expertise"))
  for item in details.areas_of_expertise {
    experience_details(item)
  }
}
