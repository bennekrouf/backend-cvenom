#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills, get_text, structured_experience_full
#import "experiences.typ": get_work_experience, get_key_insights

#let details = toml("cv_params.toml")

// Override section function for this template's visual style
#let section(title) = {
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
        fill: rgb("#049cb0"),
        width: 100%,
        inset: (x: 15pt, y: 8pt),
        text(size: 14pt, weight: "bold", fill: white, title)
      )
    )
  ]
}

// Logo display function
#let show_logo() = {
  if sys.inputs.at("company_logo.png", default: none) != none {
    align(center + horizon)[
      #image("company_logo.png", width: 150pt, height: 60pt, fit: "contain")
    ]
  } else {
    rect(
      width: 160pt,
      height: 70pt,
      fill: rgb("#049cb0"),
      radius: 4pt,
      align(center + horizon)[
        #text(size: 18pt, weight: "bold", fill: white, "KEYTEO")
      ]
    )
  }
}

#set page(
 header: [
  #v(20pt)
  #align(center)[
      #show_logo()
  ]
  #v(20pt)
],
  footer: [
    #grid(
      columns: (1fr, 2fr, 1fr),
      align: (left, center, right),
      text(size: 9pt, get_text("skills_file")),
      text(size: 9pt, get_text("confidential_document")),
      context text(size: 9pt, [#counter(page).display()/#counter(page).final().first()])
    )
    #v(-0.2em)
    #line(length: 100%, stroke: 0.5pt + rgb("#14A4E6"))
    #v(-0.2em)
    #align(center)[
      #text(size: 7pt, get_text("website"))
    ]
  ],
  header-ascent: -20pt,
  footer-descent: 25pt,
  margin: (top: 2.8cm, left: 1.5cm, bottom: 2.8cm, right: 1.5cm),
)

#v(1em)

#align(center)[
  #text(size: 18pt, weight: "bold", 
    if "job_title" in details { details.job_title } else { "Technical Lead" }
  )
]

#v(1.5em)

#grid(
  columns: 2,
  column-gutter: 4em,
  row-gutter: 1em,
  text(size: 12pt, weight: "bold", fill: rgb("#14A4E6"), "Consultant"),
  if "consultant_name" in details { 
    text(size: 11pt, details.consultant_name) 
  } else { 
    text(size: 11pt, details.at("name", default: "")) 
  },
  text(size: 12pt, weight: "bold", fill: rgb("#14A4E6"), "Keyteo Business Manager"),
  if "manager_name" in details and "manager_email" in details and "manager_phone" in details { 
    text(size: 11pt, details.manager_name + " – " + details.manager_email + " – " + details.manager_phone) 
  } else { 
    text(size: 11pt, "Anthony Levavasseur – alevavasseur@mycompany.ch – 078 230 36 38") 
  }
)

#v(0.5em)

#section(get_text("key_insights"))
#experience_details(..get_key_insights())

#section(get_text("technical_skills"))
#if "skills" in details {
  show_skills(details.skills)
} else {
  [No skills data found in configuration]
}

#section(get_text("certifications_education"))
#if "education" in details {
  let diplomas = details.education.filter(item => item.at("type", default: "education") == "diploma")
  if diplomas.len() > 0 {
    text(weight: "bold", get_text("diplomas"))
    for item in diplomas {
      experience_details(item.title + " " + item.date)
    }
  }
  
  let certifications = details.education.filter(item => item.at("type", default: "education") != "diploma")
  if certifications.len() > 0 {
    text(weight: "bold", get_text("certifications"))
    for item in certifications {
      experience_details(item.title + " " + item.date)
    }
  }
} else {
  [No education data found in configuration]
}

#section(get_text("languages"))
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
    grid(
      columns: 2,
      column-gutter: 2em,
      ..lang_items.map(lang => 
        list(
          indent: 5pt,
          marker: text(fill: rgb("#14A4E6"), sym.bullet),
          [#lang]
        )
      )
    )
  }
} else {
  [No language data found in configuration]
}

#block[
  #section(get_text("work_experience"))
  #get_work_experience()
]
