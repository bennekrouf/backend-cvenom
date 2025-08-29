#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")

// Custom section function with layered rectangle effect
#let keyteo_section(title) = {
  block(
    fill: gray,
    width: 100%,
    inset: 0pt,
    outset: 0pt,
  )[
    #move(
      dx: -4pt,
      dy: -4pt,
      block(
        fill: rgb("#14A4E6"),
        width: 100%,
        inset: (x: 15pt, y: 8pt),
        text(size: 14pt, weight: "bold", fill: white, title)
      )
    )
  ]
  v(1em)
}

// Don't use the default conf layout - we'll create custom layout
#set page(
 header: [
 #align(center)[
    #image("keyteo_logo.png", width: 180pt, height: 80pt, fit: "contain")
 ]
  #line(length: 100%, stroke: 0.5pt + rgb("#14A4E6"))
],
  footer: [
    #grid(
      columns: (1fr, 2fr, 1fr),
      align: (left, center, right),
      text(size: 9pt, "Skills file"),
      text(size: 9pt, "Confidential document, reproduction prohibited"),
      context text(size: 9pt, [#counter(page).display()/#counter(page).final().first()])
    )
    #v(-0.2em)
    #line(length: 100%, stroke: 0.5pt + rgb("#14A4E6"))
    #v(-0.2em)
    #align(center)[
      #text(size: 7pt, "www.keyteo.ch")
    ]
  ],
  header-ascent: 25pt,
  footer-descent: 25pt,
  margin: (top: 2.8cm, left: 1.5cm, bottom: 2.8cm, right: 1.5cm),
)

// Custom first page layout for Keyteo template
#v(1em)

// Job title centered
#align(center)[
  #text(size: 18pt, weight: "bold", 
    if "job_title" in details { details.job_title } else { "Technical Lead" }
  )
]

#v(1.5em)

// Two rows layout for consultant info
#grid(
  columns: 2,
  column-gutter: 4em,
  row-gutter: 1em,
  // First row
  text(size: 12pt, weight: "bold", fill: rgb("#14A4E6"), "Consultant"),
  if "consultant_name" in details { 
    text(size: 11pt, details.consultant_name) 
  } else { 
    text(size: 11pt, details.at("name", default: "")) 
  },
  // Second row  
  text(size: 12pt, weight: "bold", fill: rgb("#14A4E6"), "Keyteo Business Manager"),
  if "manager_info" in details { 
    text(size: 11pt, details.manager_info) 
  } else { 
    text(size: 11pt, "Anthony Levavasseur – alevavasseur@keyteo.ch – 078 230 36 38") 
  }
)

#v(0.5em)

#keyteo_section("Key insights")



#keyteo_section("Technical Skills")
#if "skills" in details {
  show_skills(details.skills)
} else {
  [No skills data found in configuration]
}

#keyteo_section("Certifications & Education")
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

#keyteo_section("Languages")
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

#get_work_experience()
