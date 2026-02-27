#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills_sidebar, get_text
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

// don't forget this
#show: doc => conf(details, doc)

// ── Summary / About ────────────────────────────────────────────────────────────
#if details.at("summary", default: "") != "" {
  section(get_text("summary"))
  text(size: 9.5pt, fill: rgb("#2D3748"), details.summary)
  v(0.3em)
}

// ── Work Experience ────────────────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()
