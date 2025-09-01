#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text
#import "experiences.typ" : get_work_experience

#let details = toml("cv_params.toml")

// don't forget this
#show: doc => conf(details, doc)

#get_work_experience()

= #get_text("technical_skills")
#if "skills" in details {
  show_skills(details.skills)
} else {
  [No skills data found in configuration]
}

= #get_text("certifications_education")
#if "education" in details {
  for item in details.education {
    dated_experience(
      item.title,
      date: item.date
    )
  }
} else {
  [No education data found in configuration]
}

= #get_text("languages")
#if "languages" in details {
  let lang_items = ()
  if "native" in details.languages {
    lang_items = lang_items + details.languages.native
  }
  if "fluent" in details.languages {
    lang_items = lang_items + details.languages.fluent
  }
  if "intermediate" in details.languages {
    lang_items = lang_items + details.languages.intermediate
  }
  if "basic" in details.languages {
    lang_items = lang_items + details.languages.basic
  }
  
  if lang_items.len() > 0 {
    experience_details(..lang_items)
  }
} else {
  [No language data found in configuration]
}
