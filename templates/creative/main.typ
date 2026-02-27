#import "template.typ": conf, date, dated_experience, experience_details, section, get_text
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

#show: doc => conf(details, doc)

// ── Work Experience ────────────────────────────────────────────────────────────
#section(get_text("work_experience"))
#get_work_experience()
