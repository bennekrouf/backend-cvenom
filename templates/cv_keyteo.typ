#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")

// Custom configuration for Keyteo template with header on every page
#show: doc => conf(details, doc)

// Add Keyteo logo/header to every page - ensure it appears on first page
#set page(
  header: [
    #align(center)[
      #if "company_logo" in details and details.company_logo != "" {
        image(details.company_logo, width: 80pt)
      } else {
        text(size: 16pt, weight: "bold", fill: rgb("#14A4E6"), "KEYTEO")
      }
    ]
    #line(length: 100%, stroke: 0.5pt + rgb("#14A4E6"))
  ],
  footer: [
    #grid(
      columns: (1fr, 2fr, 1fr),
      align: (left, center, right),
      text(size: 7pt, "Skills file"),
      text(size: 7pt, "Confidential document, reproduction prohibited"),
      context text(size: 7pt, [#counter(page).display()/#counter(page).final().first()])
    )
    #v(-0.10em)
    #line(length: 100%, stroke: 0.5pt + rgb("#14A4E6"))
    #v(-0.10em)
    #align(center)[
      #text(size: 5pt, "www.keyteo.ch")
    ]
  ],
  header-ascent: 15pt,
  footer-descent: 25pt,
  margin: (top: 2.8cm, left: 1.5cm, bottom: 2.8cm, right: 1.5cm),
)

// Ensure content starts properly
#v(0.5em)

#get_work_experience()

= Technical Skills
#if "skills" in details {
  show_skills(details.skills)
} else {
  [No skills data found in configuration]
}

= Certifications & Education
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

= Languages
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
