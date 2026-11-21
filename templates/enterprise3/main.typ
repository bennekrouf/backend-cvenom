#import "template.typ": *
#import "experiences.typ": get_work_experience

#let details = toml("cv_params.toml")

// Page 1 (cover with sidebar) and page 2 (sectors, achievements) are built by
// conf from cv_params.toml; the body below is the experience pages.
#show: doc => conf(details, doc)

#section(get_text("work_experience"))
#get_work_experience()

#show_skills_matrix(details)
