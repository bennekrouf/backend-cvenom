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
        fill: rgb("#049cb0"),
        width: 100%,
        inset: (x: 15pt, y: 8pt),
        text(size: 14pt, weight: "bold", fill: white, title)
      )
    )
  ]
  // v(1em)
}

// Function to handle logo display gracefully
#let show_logo() = {
  // Check if company_logo.png was provided as input
  if sys.inputs.at("company_logo.png", default: none) != none {
    // Logo exists - display it
    align(center + horizon)[
      #image("company_logo.png", width: 150pt, height: 60pt, fit: "contain")
    ]
  } else {
    // Fallback: Show professional text logo
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

// Don't use the default conf layout - we'll create custom layout
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
  header-ascent: -20pt,
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
  if "manager_name" in details and "manager_email" in details and "manager_phone" in details { 
    text(size: 11pt, details.manager_name + " – " + details.manager_email + " – " + details.manager_phone) 
  } else { 
    text(size: 11pt, "Anthony Levavasseur – alevavasseur@keyteo.ch – 078 230 36 38") 
  }
)

#v(0.5em)

#keyteo_section(if details.at("lang", default: "en") == "fr" { "Points clés" } else { "Key insights" })
#if "key_insights" in details {
  experience_details(..details.key_insights)
} else {
  experience_details(
    "Experienced technical lead with proven track record in startup environments",
    "Expert in modern development stacks with focus on Rust and microservices architecture", 
    "Strong background in AI/ML integration and blockchain development",
    "Demonstrated ability to scale teams and deliver complex technical solutions"
  )
}

#keyteo_section(if details.at("lang", default: "en") == "fr" { "Compétences" } else { "Technical Skills" })
#if "skills" in details {
  show_skills(details.skills)
} else {
  [No skills data found in configuration]
}

#keyteo_section(if details.at("lang", default: "en") == "fr" { "Formation" } else { "Certifications & Education" })
#if "education" in details {
  // Diplomas section
  let diplomas = details.education.filter(item => item.at("type", default: "education") == "diploma")
  if diplomas.len() > 0 {
    text(weight: "bold", if details.at("lang", default: "en") == "fr" { "Diplômes" } else { "Diplomas" })
    for item in diplomas {
      experience_details(item.title + " " + item.date)
    }
  }
  
  // Certifications section  
  let certifications = details.education.filter(item => item.at("type", default: "education") != "diploma")
  if certifications.len() > 0 {
    text(weight: "bold", if details.at("lang", default: "en") == "fr" { "Certifications" } else { "Certifications" })
    for item in certifications {
      experience_details(item.title + " " + item.date)
    }
  }
} else {
  [No education data found in configuration]
}

#keyteo_section(if details.at("lang", default: "en") == "fr" { "Langues" } else { "Languages" })
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

#block[
  #keyteo_section(if details.at("lang", default: "en") == "fr" { "Expérience Professionnelle" } else { "Work Experience" })
  #get_work_experience()
]
