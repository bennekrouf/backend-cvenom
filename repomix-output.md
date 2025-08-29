This file is a merged representation of a subset of the codebase, containing files not matching ignore patterns, combined into a single document by Repomix.

# File Summary

## Purpose
This file contains a packed representation of the entire repository's contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.

## File Format
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Repository files (if enabled)
5. Multiple file entries, each consisting of:
  a. A header with the file path (## File: path/to/file)
  b. The full contents of the file in a code block

## Usage Guidelines
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.

## Notes
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Files matching these patterns are excluded: samples, prompts
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded
- Files are sorted by Git change count (files with more changes are at the bottom)

# Directory Structure
```
data/
  john-doe/
    cv_params.toml
    experiences_en.typ
    experiences_fr.typ
    README.md
  mohamed-bennekrouf/
    cv_params.toml
    experiences_en.typ
    experiences_fr.typ
  mohamed2/
    cv_params.toml
    experiences_en.typ
    experiences_fr.typ
    README.md
src/
  lib.rs
  main.rs
  web.rs
templates/
  cv_keyteo.typ
  cv.typ
  experiences_template.typ
  person_template.toml
  template.typ
.gitignore
Cargo.toml
cv_keyteo.typ
cv_params.toml
experiences_en.typ
README.md
template.typ
```

# Files

## File: cv_keyteo.typ
````
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
````

## File: cv_params.toml
````toml
# Personal Information
name = "Mohamed Bennekrouf"
phonenumber = "+41 76 483 75 40"
email = "mb@mayorana.ch"
address = "1162 Saint-Prex - Switzerland"
picture = "profile.png"

[links]
github = "https://github.com/bennekrouf"
linkedin = "https://www.linkedin.com/in/bennekrouf"
personal = "https://mayorana.ch"

# Technical Skills
[skills]
Languages = ["Rust (3y)", "Typescript (5y)", "JavaScript (>10y)", "Java (>10y)"]
Frameworks = ["NodeJS", "React", "Angular"]
Others = ["gRPC", "Git", "Linux", "OpenShift", "MongoDB", "SQL"]

# Education & Certifications
[[education]]
title = "OpenShift - Administration I"
date = "(2024)"

[[education]]
title = "Computer Science University Degree (Programming) - Lyon Claude Bernard"
date = "(1998)"

[[education]]
title = "Science high-school diploma – Lycée La Pléiade"
date = "(1996)"

# Languages
[languages]
native = ["French (Mother tongue)"]
fluent = ["English (Fluent)"]
````

## File: experiences_en.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Mayorana - Rust / an AI startup
    #dated_experience(  
      "Technical Lead Developer",  
      date: "2022 - Present",  
      content: [  
        #experience_details(  
          "Developed Rust Substrate smart contracts with frontend for allfeat.com, a blockchain-based music service platform built on Substrate. Created a react-native mobile app for one customer to use these services."
        )
        #experience_details(  
          "Built an NLP/LLM based program that converts sentences to API calls. Based on Rust / Ollama / Deepseek R1 / Iggy messaging and Micro-services. See api0.ai."  
        )
        #experience_details(  
          "Developed a log library based on Rust/gRPC. See crate grpc_logger."  
        )  
      ]  
    )
    
    == Concreet & eJust - Startup CTO roles
    #dated_experience(  
      "CTO - Founding software engineering - Lead development teams",  
      date: "2016 - 2021",  
      description: "Led technical development for two startups: Concreet (real estate SAAS platform) and eJust (justice arbitration platform)",  
      content: [  
        #experience_details(  
          "Built Concreet's SAAS platform with Node.js for real estate project management, including mobile app for field collaboration tracking."  
        )  
        #experience_details(  
          "Migrated eJust's monolithic Java backend to a multi-tenant architecture for justice arbitration and mediation."  
        )
        #experience_details(  
          "Implemented core backend features, CI/CD pipelines, AWS infrastructure management, and automated deployments."  
        )  
      ]  
    )
    
    == Inpart.io
    #dated_experience(
      "Lead Frontend Developer",
      date: "2012 - 2016",
      description: "SASS platform for pharmaceutical professionals",
      content: [
        #experience_details(
          "Led frontend development team using Angular and React for an enterprise application serving 5000+ pharmaceutical professionals."
        )
        #experience_details(
          "Developed mobile applications using PhoneGap for cross-platform deployment and offline functionality."
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "Lead Developer - Architect",
      date: "2007 - 2015",
      content: [
        #experience_details(
          "Developed POCs using Beacon and GPS detection technologies for mobile applications."
        )
        #experience_details(
          "Data analysis and database architecture to build a new credit scoring system (Coface services)."
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 - 2006",
      content: [
        #experience_details(
          "Developed architectural components (proxy, cache, authentication) and UI components (Banques Populaires)"
        )
        #experience_details(
          "Developed a back-end data persistence framework using: MQ-Series / WebSphere / COBOL programs (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}
````

## File: data/john-doe/cv_params.toml
````toml
# Personal Information
name = "john-doe"
phonenumber = ""
email = ""
address = ""
picture = "profile.png"

[links]
github = ""
linkedin = ""
personal = ""

# Technical Skills
[skills]
Languages = [""]
Frameworks = [""]
Others = [""]

# Education & Certifications
[[education]]
title = ""
date = ""

# Languages
[languages]
native = [""]
fluent = [""]
intermediate = [""]
basic = [""]
````

## File: data/john-doe/experiences_en.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
````

## File: data/john-doe/experiences_fr.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
````

## File: data/john-doe/README.md
````markdown
# john-doe CV Data

Add your profile image as `profile.png` in this directory.

Edit the following files:
- `cv_params.toml` - Personal information and skills
- `experiences_*.typ` - Work experience for each language
````

## File: data/mohamed-bennekrouf/cv_params.toml
````toml
# Personal Information
name = "Mohamed Bennekrouf"
phonenumber = "+41 76 483 75 40"
email = "mb@mayorana.ch"
address = "1162 Saint-Prex - Switzerland"
picture = "profile.png"

[links]
github = "https://github.com/bennekrouf"
linkedin = "https://www.linkedin.com/in/bennekrouf"
personal = "https://mayorana.ch"

# Technical Skills
[skills]
Languages = ["Rust (3y)", "Typescript (5y)", "JavaScript (>10y)", "Java (>10y)"]
Frameworks = ["NodeJS", "React", "Angular"]
Others = ["gRPC", "Git", "Linux", "OpenShift", "MongoDB", "SQL"]

# Education & Certifications
[[education]]
title = "OpenShift - Administration I"
date = "(2024)"

[[education]]
title = "Computer Science University Degree (Programming) - Lyon Claude Bernard"
date = "(1998)"

[[education]]
title = "Science high-school diploma – Lycée La Pléiade"
date = "(1996)"

# Languages
[languages]
native = ["French (Mother tongue)"]
fluent = ["English (Fluent)"]
````

## File: data/mohamed-bennekrouf/experiences_en.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Mayorana - Rust / an AI startup
    #dated_experience(  
      "Technical Lead Developer",  
      date: "2022 - Present",  
      content: [  
        #experience_details(  
          "Developed Rust Substrate smart contracts with frontend for allfeat.com, a blockchain-based music service platform built on Substrate. Created a react-native mobile app for one customer to use these services."
        )
        #experience_details(  
          "Built an NLP/LLM based program that converts sentences to API calls. Based on Rust / Ollama / Deepseek R1 / Iggy messaging and Micro-services. See api0.ai."  
        )
        #experience_details(  
          "Developed a log library based on Rust/gRPC. See crate grpc_logger."  
        )  
      ]  
    )
    
    == Concreet & eJust - Startup CTO roles
    #dated_experience(  
      "CTO - Founding software engineering - Lead development teams",  
      date: "2016 - 2021",  
      description: "Led technical development for two startups: Concreet (real estate SAAS platform) and eJust (justice arbitration platform)",  
      content: [  
        #experience_details(  
          "Built Concreet's SAAS platform with Node.js for real estate project management, including mobile app for field collaboration tracking."  
        )  
        #experience_details(  
          "Migrated eJust's monolithic Java backend to a multi-tenant architecture for justice arbitration and mediation."  
        )
        #experience_details(  
          "Implemented core backend features, CI/CD pipelines, AWS infrastructure management, and automated deployments."  
        )  
      ]  
    )
    
    == Inpart.io
    #dated_experience(
      "Lead Frontend Developer",
      date: "2012 - 2016",
      description: "SASS platform for pharmaceutical professionals",
      content: [
        #experience_details(
          "Led frontend development team using Angular and React for an enterprise application serving 5000+ pharmaceutical professionals."
        )
        #experience_details(
          "Developed mobile applications using PhoneGap for cross-platform deployment and offline functionality."
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "Lead Developer - Architect",
      date: "2007 - 2015",
      content: [
        #experience_details(
          "Developed POCs using Beacon and GPS detection technologies for mobile applications."
        )
        #experience_details(
          "Data analysis and database architecture to build a new credit scoring system (Coface services)."
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 - 2006",
      content: [
        #experience_details(
          "Developed architectural components (proxy, cache, authentication) and UI components (Banques Populaires)"
        )
        #experience_details(
          "Developed a back-end data persistence framework using: MQ-Series / WebSphere / COBOL programs (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}
````

## File: data/mohamed-bennekrouf/experiences_fr.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Expérience Professionnelle
    == Mayorana
    #dated_experience(
      "Freelance - Développeur Principal",
      date: "2022 - Présent",
      content: [
        #experience_details(
          "Programme Rust pour une migration de données, développement d'un programme d'analyse de dette technique, développement d'un Agent IA (basé sur BERT) soutenu par des micro-services rust (API matcher, messagerie, client mail)"
        )
        #experience_details(
          "En tant que développeur blockchain expérimenté : intégration de portefeuilles dans une application, interaction avec des smart contracts BNB, développement d'une application de suivi en temps réel du token DAI, développement de pallet substrate et corrections de bugs substrate (blockchain musicale Allfeat), création d'un token SPL Solana (RIBH sur Raydium)"
        )
        #experience_details(
          "Développement d'applications mobiles avec contributions open-source (Voir les dépôts github.com/bennekrouf/mayo*). Développement d'une application d'apprentissage avec un rust / rocket / sled (Cf similar-sled / ribh.io)."
        )
      ]
    )
    
    == Concreet
    #dated_experience(
      "CTO - Responsable Ingénierie Logicielle",
      date: "2016 - 2021",
      description: "Plateforme SAAS utilisée pour gérer des projets immobiliers et suivre les activités des collaborateurs via une application mobile",
      content: [
        #experience_details(
          "Définition de la stack technique et initiation du développement des composants architecturaux de l'application : scaffolding, couche fondamentale backend (authentification, connecteurs de base de données)"
        )
        #experience_details(
          "Direction et mentorat de l'équipe de développement : réunion quotidienne, revue de code, discussions techniques, refactoring et optimisation du code. Configuration Devops : Pipelines pour CI/CD, scripting bash, intégration AWS S3, planificateur de processus PM2"
        )
      ]
    )
    
    == CGI
    #dated_experience(
      "Développeur Principal - Architecte",
      date: "2007 -- 2015",
      content: [
        #experience_details(
          "Direction d'une équipe frontend pour développer une application d'entreprise utilisée par plus de 5000 professionnels pharmaceutiques (inpart.io), développement de POCs utilisant Beacon, détection GPS pour application mobile"
        )
        #experience_details(
          "Analyse de données et architecture de base de données pour construire un nouveau système de scoring crédit (Services Coface)"
        )
      ]
    )
    
    == Accenture
    #dated_experience(
      "Consultant",
      date: "1998 -- 2006",
      content: [
        #experience_details(
          "Développement de composants architecturaux (proxy, cache, authentification) et de composants UI (Banques Populaires)"
        )
        #experience_details(
          "Développement d'un framework de persistance de données backend utilisant : MQ-Series / WebSphere / programmes COBOL (Banque Générale du Luxembourg)"
        )
      ]
    )
  ]
}
````

## File: data/mohamed2/cv_params.toml
````toml
# Personal Information
name = "mohamed2"
phonenumber = ""
email = ""
address = ""
picture = "profile.png"

[links]
github = ""
linkedin = ""
personal = ""

# Technical Skills
[skills]
Languages = [""]
Frameworks = [""]
Others = [""]

# Education & Certifications
[[education]]
title = ""
date = ""

# Languages
[languages]
native = [""]
fluent = [""]
intermediate = [""]
basic = [""]
````

## File: data/mohamed2/experiences_en.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
````

## File: data/mohamed2/experiences_fr.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
````

## File: data/mohamed2/README.md
````markdown
# mohamed2 CV Data

Add your profile image as `profile.png` in this directory.

Edit the following files:
- `cv_params.toml` - Personal information and skills
- `experiences_*.typ` - Work experience for each language
````

## File: templates/cv.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")

// don't forget this
#show: doc => conf(details, doc)

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
````

## File: templates/experiences_template.typ
````
#import "template.typ": conf, date, dated_experience, experience_details, section
#let get_work_experience() = {
  [
    = Work Experience
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
````

## File: templates/template.typ
````
// global variables
#let default_primary_color = rgb("#14A4E6")
#let default_secondary_color = rgb("#757575")
#let default_link_color = rgb("#14A4E6")
#let default_font = "Carlito"
#let default_math_font = "Times"
#let default_separator = text(
  // this is because in some fonts (notably computer modern), it shows the
  // vertical line as a horizontal one
  font: "Carlito",
  " \u{007c} ",
)

// dictionary of common icons and values
#let get_default_icons(color: none) = {
  if color == none {
    color = default_primary_color
  }
  (
    "github": ("displayname": "GitHub", "logo": text(font: "Font Awesome 6 Brands", "\u{f09b}")),
    "linkedin": (
      "displayname": "LinkedIn",
      "logo": text(font: "Font Awesome 6 Brands", "\u{f08c}"),
    ),
    "personal": (
      "displayname": "Personal",
      "logo": text(font: "Font Awesome 6 Free Solid", "\u{f268}"),
    ),
    // annoyingly, Debian does not ship a version of FontAwesome which supports
    // the ORCID logo, hence here I draw my own approximation of it using Typst
    //  primitives
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em, circle(
      radius: 0.5em,
      fill: color,
      inset: 0pt,
      align(center + horizon, text(size: 0.8em, fill: white, "iD")),
    ))),
  )
}

/* join dictionaries (kind of like Python's {**a, **b}) */
#let join_dicts(..args) = {
  let result = (:)
  for arg in args.pos() {
    for (key, value) in arg.pairs(){
      result.insert(key, value)
    }
  }
  result
}

/* function that applies a color to a link */
#let colorlink(color: none, url, body) = {
  if color == none {
    color = default_link_color
  }
  text(fill: color, link(url)[#body<colorlink>])
}

/* function that processes links */
#let process_links(color: none, icons: none, links) = {
  if icons == none {
    icons = default_icons
  } else {
    // if the user supplies a custom dictionary, update the default one
    icons = join_dicts(get_default_icons(color: color), icons)
  }
  links.pairs().map(
    it => text(
      fill: color,
      link(
        it.at(1),
        icons.at(it.at(0), default: (:)).at("logo", default: "") + " " + icons.at(it.at(0), default: (:)).at("displayname", default: ""),
      ),
    ),
  )
}

/* the section(s) that are colored and have a line */
#let section(primary_color: none, secondary_color: none, title) = {
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  heading(level: 1, grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  ))
}

/* custom bulleted list */
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none {
    color = default_primary_color
  }
  if symbol == none {
    symbol = sym.bullet
  }
  list(
    indent: 5pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 10pt, [#it<experience_details>])),
  )
}

#let date(color: none, content) = {
  if color == none {
    color = default_secondary_color
  }
  [#h(1fr) #text(weight: "regular", size: 10pt, fill: color, content)]
}

/* experience that has an optional date and an optional description */
#let dated_experience(title, date: none, description: none, content: none) = {
  [
    == #title #h(1fr) #text(weight: "regular", size: 10pt, date) <dated_experience_header>

    #text(weight: "regular", description)<dated_experience_description>

    #content
  ]
}

/* display skills (a dictionary) */
#let show_skills(separator: none, color: none, skills) = {
  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_primary_color
  }

  let skills_array = ()
  for (key, value) in skills.pairs() {
    skills_array.push([*#key*])
    skills_array.push(value.map(box).join(text(fill: color, separator)))
  }

  table(
    columns: 2,
    column-gutter: 2%,
    row-gutter: -0.2em,
    align: (right, left),
    stroke: none,
    ..skills_array,
  )
  v(-1em)
}

/* return text info about a person */
#let show_details_text(
  alignment: center + horizon,
  icons: none,
  separator: none,
  color: none,
  details,
) = {
  let show_line_from_dict(dict, key) = {
    if dict.at(key, default: none) != none [#dict.at(key) \ ]
  }

  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_link_color
  }

  if icons == none {
    icons = get_default_icons(color: color)
  } else {
    icons = join_dicts(get_default_icons(color: color), icons)
  }

  align(
    alignment,
    [
      #text(size: 14pt, details.at("name", default: none))\
      #show_line_from_dict(details, "address")
      #show_line_from_dict(details, "phonenumber")
      #text(
        size: 13pt,
        fill: color,
        (link("mailto:" + details.email)[#raw(details.email)]),
      ) \
      #if details.at("links", default: none) != none {
        process_links(details.links, color: color, icons: icons).join(text(fill: color, separator))
      }
    ],
  )
}

/* the main info about the person (including picture) */
#let show_details(icons: none, separator: none, color: none, details) = {
  if details.at("picture", default: "").len() > 0 {
    grid(
      columns: (0.5fr, 1fr, 2.5fr),
      align(right + horizon, image(details.picture, width: 90%)),
      h(1fr),
      show_details_text(icons: icons, separator: separator, color: color, details),
    )
  } else {
    show_details_text(
      // TODO figure out why the `center + horizon` alignment causes issues
      alignment: center + top,
      icons: icons,
      separator: separator,
      color: color,
      details,
    )
  }
  v(-1em)
}

/* the main configuration */
#let conf(
  primary_color: none,
  secondary_color: none,
  link_color: none,
  font: none,
  math_font: none,
  separator: none,
  list_point: none,
  details,
  doc,
) = {
  // TODO figure out if there's a simpler way to parse this
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  if link_color == none {
    link_color = default_link_color
  }

  if font == none {
    font = default_font
  }

  if math_font == none {
    math_font = default_math_font
  }

  if separator == none {
    separator = text(
      fill: primary_color,
      // this is because in some fonts (notably computer modern), it shows the
      // vertical line as a horizontal one
      text(font: "Carlito", " \u{007c} "),
    )
  }

  if list_point == none {
    list_point = sym.bullet
  }

  // custom show rules
  show math.equation: set text(font: math_font)
  show heading.where(level: 1): title => grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  )
  show heading.where(level: 2): set text(size: 11pt)
  show heading.where(level: 3): set text(weight: "regular")
  show heading.where(level: 2): set block(spacing: 0.7em)
  show heading.where(level: 3): set block(spacing: 0.7em)

  show link: set text(fill: primary_color)
  show list: set text(size: 10pt)
  // see https://github.com/typst/typst/issues/1941
  show "C++": box

  // custom set rules
  set text(font: font, ligatures: false)
  set par(justify: true)

  set page(
    margin: (top: 0.8cm, left: 1.5cm, bottom: 1.5cm, right: 1.5cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set page(footer: [
    #line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    )

    #eval(details.footer, mode: "markup")
  ]) if details.at("footer", default: "").len() > 0

  set list(indent: 5pt, marker: text(fill: primary_color, list_point))

  show_details(details, color: primary_color)

  // the actual content of the document
  doc
}
````

## File: .gitignore
````
/target

/tmp_workspace
````

## File: templates/cv_keyteo.typ
````
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
````

## File: Cargo.toml
````toml
[package]
name = "curi"
version = "0.1.0"
edition = "2021"

[lib]
name = "cv_generator"
path = "src/lib.rs"

[dependencies]
anyhow = "1.0.93"
clap = { version = "4.5.20", features = ["derive"] }
rocket = { version = "0.5.1", features = ["json"] }
serde = { version = "1.0.219", features = ["derive"] }
tokio = { version = "1.47.1", features = ["full"] }
````

## File: src/web.rs
````rust
use anyhow::Result;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, routes, State, fairing::{Fairing, Info, Kind}};
use rocket::http::{Status, Header, ContentType};
use rocket::{Request, Response};
use rocket::response::{self, Responder};
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use std::path::PathBuf;
use std::fs;
use crate::{CvConfig, CvGenerator, CvTemplate, list_templates};

pub struct PdfResponse(Vec<u8>);

impl<'r> Responder<'r, 'static> for PdfResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::PDF)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GenerateRequest {
    pub person: String,
    pub lang: Option<String>,
    pub template: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePersonRequest {
    pub person: String,
}

#[derive(FromForm)]
pub struct UploadForm<'f> {
    pub person: String,
    pub file: TempFile<'f>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePersonResponse {
    pub success: bool,
    pub message: String,
    pub person_dir: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub file_path: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GenerateResponse {
    pub success: bool,
    pub message: String,
    pub pdf_path: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplatesResponse {
    pub success: bool,
    pub templates: Vec<TemplateInfo>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub templates_dir: PathBuf,
}

#[post("/generate", data = "<request>")]
pub async fn generate_cv(
    request: Json<GenerateRequest>, 
    config: &State<ServerConfig>
) -> Result<PdfResponse, Status> {
    let lang = request.lang.as_deref().unwrap_or("en");
    let template_str = request.template.as_deref().unwrap_or("default");
    
    let template = match CvTemplate::from_str(template_str) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Invalid template: {}", template_str);
            return Err(Status::BadRequest);
        }
    };
    
    let cv_config = CvConfig::new(&request.person, lang)
        .with_template(template)
        .with_data_dir(config.data_dir.clone())
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());
    
    match CvGenerator::new(cv_config) {
        Ok(generator) => {
            match generator.generate() {
                Ok(pdf_path) => {
                    match fs::read(&pdf_path) {
                        Ok(pdf_data) => {
                            // Clean up the generated file after reading
                            let _ = fs::remove_file(&pdf_path);
                            Ok(PdfResponse(pdf_data))
                        },
                        Err(e) => {
                            eprintln!("Failed to read PDF file: {}", e);
                            Err(Status::InternalServerError)
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Generation error: {}", e);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(e) => {
            eprintln!("Config error: {}", e);
            Err(Status::BadRequest)
        }
    }
}

#[post("/create", data = "<request>")]
pub async fn create_person(
    request: Json<CreatePersonRequest>,
    config: &State<ServerConfig>
) -> Result<Json<CreatePersonResponse>, Status> {
    let cv_config = CvConfig::new(&request.person, "en")
        .with_data_dir(config.data_dir.clone())
        .with_output_dir(config.output_dir.clone())
        .with_templates_dir(config.templates_dir.clone());
    
    let generator = CvGenerator { config: cv_config };
    
    match generator.create_person() {
        Ok(_) => {
            let person_dir = generator.config.person_data_dir();
            Ok(Json(CreatePersonResponse {
                success: true,
                message: format!("Person directory created successfully for {}", request.person),
                person_dir: person_dir.to_string_lossy().to_string(),
            }))
        },
        Err(e) => {
            eprintln!("Person creation error: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/upload-picture", data = "<upload>")]
pub async fn upload_picture(
    mut upload: Form<UploadForm<'_>>,
    config: &State<ServerConfig>
) -> Result<Json<UploadResponse>, Status> {
    // Check if person directory exists
    let person_dir = config.data_dir.join(&upload.person);
    if !person_dir.exists() {
        return Ok(Json(UploadResponse {
            success: false,
            message: format!("Person directory not found: {}", upload.person),
            file_path: String::new(),
        }));
    }
    
    // Validate file type (basic check)
    let content_type = upload.file.content_type();
    let is_image = content_type.map_or(false, |ct| {
        ct.is_png() || ct.is_jpeg() || ct.top() == "image"
    });
    
    if !is_image {
        return Ok(Json(UploadResponse {
            success: false,
            message: "Invalid file type. Please upload an image file (PNG, JPG, etc.)".to_string(),
            file_path: String::new(),
        }));
    }
    
    // Save file as profile.png in person's directory
    let target_path = person_dir.join("profile.png");
    
    match upload.file.persist_to(&target_path).await {
        Ok(_) => {
            Ok(Json(UploadResponse {
                success: true,
                message: format!("Profile picture uploaded successfully for {}", upload.person),
                file_path: target_path.to_string_lossy().to_string(),
            }))
        },
        Err(e) => {
            eprintln!("File upload error: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/templates")]
pub async fn get_templates(config: &State<ServerConfig>) -> Json<TemplatesResponse> {
    match list_templates(&config.templates_dir) {
        Ok(templates) => {
            let template_infos = templates.into_iter().map(|name| {
                let description = match name.as_str() {
                    "default" => "Standard CV layout",
                    "keyteo" => "CV with Keyteo branding and logo at the top of every page",
                    _ => "Custom template",
                };
                TemplateInfo {
                    name,
                    description: description.to_string(),
                }
            }).collect();

            Json(TemplatesResponse {
                success: true,
                templates: template_infos,
            })
        },
        Err(e) => {
            eprintln!("Failed to list templates: {}", e);
            Json(TemplatesResponse {
                success: false,
                templates: vec![
                    TemplateInfo {
                        name: "default".to_string(),
                        description: "Standard CV layout".to_string(),
                    }
                ],
            })
        }
    }
}


#[get("/health")]
pub async fn health() -> Json<&'static str> {
    Json("OK")
}

pub async fn start_web_server(
    data_dir: PathBuf, 
    output_dir: PathBuf, 
    templates_dir: PathBuf
) -> Result<()> {
    let server_config = ServerConfig {
        data_dir,
        output_dir, 
        templates_dir,
    };

    let _rocket = rocket::build()
        .attach(Cors)
        .manage(server_config)
        .mount("/api", routes![generate_cv, create_person, upload_picture, get_templates, health])
        .launch()
        .await;

    Ok(())
}
````

## File: templates/person_template.toml
````toml
# Personal Information
name = "{{name}}"
phonenumber = ""
email = ""
address = ""
picture = "profile.png"

# Job title
job_title = "Technical Lead"

# Consultant information  
consultant_name = "{{name}}"

# Manager information
manager_info = "Anthony Levavasseur – alevavasseur@keyteo.ch – 078 230 36 38"

# Keyteo logo (separate from company logo if needed)
keyteo_logo = "logo.png"
company_logo = "logo.png"


# Company/Logo information (for logo template)
company_name = ""
company_logo = "logo.png"

[links]
github = ""
linkedin = ""
personal = ""

# Technical Skills
[skills]
Languages = [""]
Frameworks = [""]
Others = [""]

# Education & Certifications
[[education]]
title = ""
date = ""

# Languages
[languages]
native = [""]
fluent = [""]
intermediate = [""]
basic = [""]
````

## File: template.typ
````
// global variables
#let default_primary_color = rgb("#14A4E6")
#let default_secondary_color = rgb("#757575")
#let default_link_color = rgb("#14A4E6")
#let default_font = "Carlito"
#let default_math_font = "Times"
#let default_separator = text(
  // this is because in some fonts (notably computer modern), it shows the
  // vertical line as a horizontal one
  font: "Carlito",
  " \u{007c} ",
)

// dictionary of common icons and values
#let get_default_icons(color: none) = {
  if color == none {
    color = default_primary_color
  }
  (
    "github": ("displayname": "GitHub", "logo": text(font: "Font Awesome 6 Brands", "\u{f09b}")),
    "linkedin": (
      "displayname": "LinkedIn",
      "logo": text(font: "Font Awesome 6 Brands", "\u{f08c}"),
    ),
    "personal": (
      "displayname": "Personal",
      "logo": text(font: "Font Awesome 6 Free Solid", "\u{f268}"),
    ),
    // annoyingly, Debian does not ship a version of FontAwesome which supports
    // the ORCID logo, hence here I draw my own approximation of it using Typst
    //  primitives
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em, circle(
      radius: 0.5em,
      fill: color,
      inset: 0pt,
      align(center + horizon, text(size: 0.8em, fill: white, "iD")),
    ))),
  )
}

/* join dictionaries (kind of like Python's {**a, **b}) */
#let join_dicts(..args) = {
  let result = (:)
  for arg in args.pos() {
    for (key, value) in arg.pairs(){
      result.insert(key, value)
    }
  }
  result
}

/* function that applies a color to a link */
#let colorlink(color: none, url, body) = {
  if color == none {
    color = default_link_color
  }
  text(fill: color, link(url)[#body<colorlink>])
}

/* function that processes links */
#let process_links(color: none, icons: none, links) = {
  if icons == none {
    icons = default_icons
  } else {
    // if the user supplies a custom dictionary, update the default one
    icons = join_dicts(get_default_icons(color: color), icons)
  }
  links.pairs().map(
    it => text(
      fill: color,
      link(
        it.at(1),
        icons.at(it.at(0), default: (:)).at("logo", default: "") + " " + icons.at(it.at(0), default: (:)).at("displayname", default: ""),
      ),
    ),
  )
}

/* the section(s) that are colored and have a line */
#let section(primary_color: none, secondary_color: none, title) = {
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  heading(level: 1, grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  ))
}

/* custom bulleted list */
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none {
    color = default_primary_color
  }
  if symbol == none {
    symbol = sym.bullet
  }
  list(
    indent: 5pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 10pt, [#it<experience_details>])),
  )
}

#let date(color: none, content) = {
  if color == none {
    color = default_secondary_color
  }
  [#h(1fr) #text(weight: "regular", size: 10pt, fill: color, content)]
}

/* experience that has an optional date and an optional description */
#let dated_experience(title, date: none, description: none, content: none) = {
  [
    == #title #h(1fr) #text(weight: "regular", size: 10pt, date) <dated_experience_header>

    #text(weight: "regular", description)<dated_experience_description>

    #content
  ]
}

/* display skills (a dictionary) */
#let show_skills(separator: none, color: none, skills) = {
  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_primary_color
  }

  let skills_array = ()
  for (key, value) in skills.pairs() {
    skills_array.push([*#key*])
    skills_array.push(value.map(box).join(text(fill: color, separator)))
  }

  table(
    columns: 2,
    column-gutter: 2%,
    row-gutter: -0.2em,
    align: (right, left),
    stroke: none,
    ..skills_array,
  )
  v(-1em)
}

/* return text info about a person */
#let show_details_text(
  alignment: center + horizon,
  icons: none,
  separator: none,
  color: none,
  details,
) = {
  let show_line_from_dict(dict, key) = {
    if dict.at(key, default: none) != none [#dict.at(key) \ ]
  }

  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_link_color
  }

  if icons == none {
    icons = get_default_icons(color: color)
  } else {
    icons = join_dicts(get_default_icons(color: color), icons)
  }

  align(
    alignment,
    [
      #text(size: 14pt, details.at("name", default: none))\
      #show_line_from_dict(details, "address")
      #show_line_from_dict(details, "phonenumber")
      #text(
        size: 13pt,
        fill: color,
        (link("mailto:" + details.email)[#raw(details.email)]),
      ) \
      #if details.at("links", default: none) != none {
        process_links(details.links, color: color, icons: icons).join(text(fill: color, separator))
      }
    ],
  )
}

/* the main info about the person (including picture) */
#let show_details(icons: none, separator: none, color: none, details) = {
  if details.at("picture", default: "").len() > 0 {
    grid(
      columns: (0.5fr, 1fr, 2.5fr),
      align(right + horizon, image(details.picture, width: 90%)),
      h(1fr),
      show_details_text(icons: icons, separator: separator, color: color, details),
    )
  } else {
    show_details_text(
      // TODO figure out why the `center + horizon` alignment causes issues
      alignment: center + top,
      icons: icons,
      separator: separator,
      color: color,
      details,
    )
  }
  v(-1em)
}

/* the main configuration */
#let conf(
  primary_color: none,
  secondary_color: none,
  link_color: none,
  font: none,
  math_font: none,
  separator: none,
  list_point: none,
  details,
  doc,
) = {
  // TODO figure out if there's a simpler way to parse this
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  if link_color == none {
    link_color = default_link_color
  }

  if font == none {
    font = default_font
  }

  if math_font == none {
    math_font = default_math_font
  }

  if separator == none {
    separator = text(
      fill: primary_color,
      // this is because in some fonts (notably computer modern), it shows the
      // vertical line as a horizontal one
      text(font: "Carlito", " \u{007c} "),
    )
  }

  if list_point == none {
    list_point = sym.bullet
  }

  // custom show rules
  show math.equation: set text(font: math_font)
  show heading.where(level: 1): title => grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  )
  show heading.where(level: 2): set text(size: 11pt)
  show heading.where(level: 3): set text(weight: "regular")
  show heading.where(level: 2): set block(spacing: 0.7em)
  show heading.where(level: 3): set block(spacing: 0.7em)

  show link: set text(fill: primary_color)
  show list: set text(size: 10pt)
  // see https://github.com/typst/typst/issues/1941
  show "C++": box

  // custom set rules
  set text(font: font, ligatures: false)
  set par(justify: true)

  set page(
    margin: (top: 0.8cm, left: 1.5cm, bottom: 1.5cm, right: 1.5cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set page(footer: [
    #line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    )

    #eval(details.footer, mode: "markup")
  ]) if details.at("footer", default: "").len() > 0

  set list(indent: 5pt, marker: text(fill: primary_color, list_point))

  show_details(details, color: primary_color)

  // the actual content of the document
  doc
}
````

## File: src/main.rs
````rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use cv_generator::{CvConfig, CvGenerator, CvTemplate, list_persons, list_templates, web::start_web_server};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, default_value = "data")]
    data_dir: PathBuf,
    
    #[arg(short, long, default_value = "output")]
    output_dir: PathBuf,
    
    #[arg(short, long, default_value = "templates")]
    templates_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        person: String,
        #[arg(short, long, default_value = "en")]
        lang: String,
        #[arg(short, long, default_value = "default")]
        template: String,
        #[arg(short, long)]
        watch: bool,
    },
    Create {
        person: String,
    },
    List,
    ListTemplates,
    Server {
        #[arg(short, long, default_value = "8000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate { person, lang, template, watch } => {
            let cv_template = CvTemplate::from_str(&template)?;
            
            let config = CvConfig::new(&person, &lang)
                .with_template(cv_template)
                .with_data_dir(cli.data_dir)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);
            
            let generator = CvGenerator::new(config)?;
            
            if watch {
                generator.watch()
            } else {
                generator.generate().map(|_| ())
            }
        }
        
        Commands::Create { person } => {
            let config = CvConfig::new(&person, "en")
                .with_data_dir(cli.data_dir)
                .with_output_dir(cli.output_dir)
                .with_templates_dir(cli.templates_dir);
            
            let generator = CvGenerator { config };
            generator.create_person_unchecked()
        }
        
        Commands::List => {
            let persons = list_persons(&cli.data_dir)?;
            
            if persons.is_empty() {
                println!("No persons found in {}", cli.data_dir.display());
            } else {
                println!("Available persons:");
                for person in persons {
                    println!("  {}", person);
                }
            }
            
            Ok(())
        }
        
        Commands::ListTemplates => {
            let templates = list_templates(&cli.templates_dir)?;
            
            if templates.is_empty() {
                println!("No templates found in {}", cli.templates_dir.display());
            } else {
                println!("Available templates:");
                for template in templates {
                    println!("  {}", template);
                }
            }
            
            Ok(())
        }

        Commands::Server { port: _ } => {
            println!("Starting CV generator web server on http://0.0.0.0:8000");
            println!("Endpoints:");
            println!("  POST /api/generate      - Generate CV");
            println!("  POST /api/create        - Create person");
            println!("  POST /api/upload-picture - Upload profile picture");
            println!("  GET  /api/templates     - List available templates");
            println!("  GET  /api/health        - Health check");
            
            start_web_server(
                cli.data_dir,
                cli.output_dir,
                cli.templates_dir
            ).await
        }
    }
}
````

## File: README.md
````markdown
# CV Generator

A multi-tenant CV generator written in Rust using Typst for PDF compilation with support for multiple templates.

## Features

- **Multi-tenant**: Support for multiple persons with isolated data
- **Multi-language**: English and French support
- **Multi-template**: Choose from different CV layouts
- **Web API**: RESTful API with full CORS support
- **Watch mode**: Auto-recompile on file changes
- **Profile pictures**: Upload and manage profile images
- **Template system**: Extensible template architecture

## Available Templates

- **default**: Standard CV layout
- **keyteo**: CV with Keyteo branding and logo at the top of every page

## CLI Usage

### Generate a CV
```bash
# Generate with default template
cargo run -- generate mohamed-bennekrouf --lang en

# Generate with keyteo template
cargo run -- generate mohamed-bennekrouf --lang en --template keyteo

# Generate French version with keyteo template
cargo run -- generate mohamed-bennekrouf --lang fr --template keyteo
```

### Watch mode (auto-recompile on changes)
```bash
cargo run -- generate mohamed-bennekrouf --lang en --template keyteo --watch
```

### List available persons
```bash
cargo run -- list
```

### List available templates
```bash
cargo run -- list-templates
```

### Create a new person directory
```bash
cargo run -- create new-person-name
```

## Web Server

Start the web server:
```bash
cargo run -- server
```

The server runs on `http://localhost:8000` with full CORS support.

### API Endpoints

#### List Available Templates
```bash
curl http://localhost:8000/api/templates
```

Response:
```json
{
  "success": true,
  "templates": [
    {
      "name": "default",
      "description": "Standard CV layout"
    },
    {
      "name": "keyteo",
      "description": "CV with Keyteo branding and logo at the top of every page"
    }
  ]
}
```

#### Upload Profile Picture
```bash
curl -X POST http://localhost:8000/api/upload-picture \
  -F "person=john-doe" \
  -F "file=@/path/to/your/image.jpg"
```

Response:
```json
{
  "success": true,
  "message": "Profile picture uploaded successfully for john-doe",
  "file_path": "data/john-doe/profile.png"
}
```

Accepts any image format and automatically saves as `profile.png` in the person's directory.

#### Create New Person
```bash
curl -X POST http://localhost:8000/api/create \
  -H "Content-Type: application/json" \
  -d '{"person": "john-doe"}'
```

Response:
```json
{
  "success": true,
  "message": "Person directory created successfully for john-doe",
  "person_dir": "data/john-doe"
}
```

This creates:
- `data/john-doe/cv_params.toml` - Personal info template with logo support
- `data/john-doe/experiences_*.typ` - Experience templates for all languages
- `data/john-doe/README.md` - Instructions

#### Generate CV (Returns PDF file)
```bash
# Generate with default template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en"}' \
  --output cv.pdf

# Generate with keyteo template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en", "template": "keyteo"}' \
  --output cv_keyteo.pdf

# Generate French version with keyteo template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "fr", "template": "keyteo"}' \
  --output cv_keyteo_fr.pdf
```

The response is a PDF file with `Content-Type: application/pdf`.

#### Health Check
```bash
curl http://localhost:8000/api/health
```

### Request Parameters

- `person`: Required. Name of the person (must match directory name)
- `lang`: Optional. Language code (`en` or `fr`). Default: `en`
- `template`: Optional. Template name (`default` or `keyteo`). Default: `default`

## Directory Structure

```
data/
  person-name/
    cv_params.toml      # Personal info and configuration
    experiences_*.typ   # Experience files per language
    profile.png         # Profile image
    logo.png           # Company logo (for logo template)
templates/
  cv.typ             # Default CV template
  cv_keyteo.typ      # Keyteo CV template
  template.typ       # Base template functions
  person_template.toml # Template for new persons
output/              # Generated PDFs (format: person_template_lang.pdf)
```

## Configuration File Format

The `cv_params.toml` file supports the following structure:

```toml
# Personal Information
name = "Your Name"
phonenumber = "+1 234 567 8900"
email = "your.email@example.com"
address = "Your Address"
picture = "profile.png"

# Company/Logo information (for logo template)
company_name = "Your Company"
company_logo = "logo.png"

[links]
github = "https://github.com/username"
linkedin = "https://linkedin.com/in/username"
personal = "https://yourwebsite.com"

# Technical Skills
[skills]
Languages = ["Rust (3y)", "Python (5y)", "JavaScript (>10y)"]
Frameworks = ["React", "Angular", "Node.js"]
Others = ["Docker", "Git", "Linux"]

# Education & Certifications
[[education]]
title = "Computer Science Degree"
date = "(2020)"

[[education]]
title = "AWS Certification"
date = "(2023)"

# Languages
[languages]
native = ["English (Native)"]
fluent = ["French (Fluent)"]
intermediate = ["Spanish (Intermediate)"]
basic = ["German (Basic)"]
```

## Template Development

### Adding New Templates

1. Create a new `.typ` file in the `templates/` directory
2. Follow the naming convention: `cv_templatename.typ`
3. Update the `CvTemplate` enum in `src/lib.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub enum CvTemplate {
       Default,
       Logo,
       YourNewTemplate,  // Add here
   }
   ```
4. Add the template mapping in the `template_file()` method
5. Add the string conversion in `from_str()` and `all()` methods

### Template Structure

Templates should:
- Import from `template.typ` for base functionality
- Import the appropriate language experiences file
- Load configuration from `cv_params.toml`
- Follow the established styling patterns

Example template structure:
```typst
#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")

// Custom template configuration
#show: doc => conf(details, doc)

// Template-specific customizations here

#get_work_experience()
// ... rest of template
```

## Supported Languages
- `en` (English) - default
- `fr` (French)

## Generated File Naming

PDFs are generated with the format: `{person}_{template}_{lang}.pdf`

Examples:
- `john-doe_default_en.pdf`
- `john-doe_keyteo_fr.pdf`

## Requirements

- Rust (latest stable)
- Typst CLI tool
- Font Awesome fonts (for icons)

## Development Notes

- Uses generics over trait objects for better performance
- Implements clear error handling without unwrap()
- YAML configuration loading with tracing for logging
- Modular template system for easy extension
````

## File: src/lib.rs
````rust
// Updated CvConfig to support templates
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use std::process::Command;
use std::collections::HashMap;

pub mod web;

/// Available CV templates
#[derive(Debug, Clone)]
pub enum CvTemplate {
    Default,
    Keyteo,
}

impl CvTemplate {
    pub fn as_str(&self) -> &str {
        match self {
            CvTemplate::Default => "default",
            CvTemplate::Keyteo => "keyteo",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "default" => Ok(CvTemplate::Default),
            "keyteo" => Ok(CvTemplate::Keyteo),
            _ => anyhow::bail!("Unsupported template: {}. Use default, keyteo", s),
        }
    }
    
    pub fn template_file(&self) -> &str {
        match self {
            CvTemplate::Default => "cv.typ",
            CvTemplate::Keyteo => "cv_keyteo.typ",
        }
    }
    
    pub fn all() -> Vec<&'static str> {
        vec!["default", "keyteo"]
    }
}

/// Template processing for creating new persons
pub struct TemplateProcessor {
    templates_dir: PathBuf,
}

impl TemplateProcessor {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self { templates_dir }
    }
    
    /// Process a template file by replacing placeholders
    pub fn process_template(&self, template_content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template_content.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Create person directory with template-based files
    pub fn create_person_from_templates(&self, person_name: &str, data_dir: &PathBuf) -> Result<()> {
        let person_dir = data_dir.join(person_name);
        fs::create_dir_all(&person_dir)
            .context("Failed to create person directory")?;
 
        // Create variables for template processing
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), person_name.to_string());

        // Process and create cv_params.toml
        let toml_template_path = self.templates_dir.join("person_template.toml");
        if toml_template_path.exists() {
            let template_content = fs::read_to_string(&toml_template_path)
                .context("Failed to read person_template.toml")?;
            let processed_content = self.process_template(&template_content, &variables);

            let output_path = person_dir.join("cv_params.toml");
            fs::write(&output_path, processed_content)
                .context("Failed to write cv_params.toml")?;
        }
        
        // Create experience files for all supported languages
        let experience_template_path = self.templates_dir.join("experiences_template.typ");
        if experience_template_path.exists() {
            let template_content = fs::read_to_string(&experience_template_path)
                .context("Failed to read experiences_template.typ")?;
            
            let languages = ["en", "fr"];
            for lang in &languages {
                let output_path = person_dir.join(format!("experiences_{}.typ", lang));
                fs::write(&output_path, &template_content)
                    .with_context(|| format!("Failed to write experiences_{}.typ", lang))?;
            }
        }
        
        // Create placeholder profile image info
        let readme_path = person_dir.join("README.md");
        let readme_content = format!(
            "# {} CV Data\n\nAdd your profile image as `profile.png` in this directory.\n\nEdit the following files:\n- `cv_params.toml` - Personal information and skills\n- `experiences_*.typ` - Work experience for each language\n\n## Available Templates:\n- default: Standard CV layout\n- keyteo: CV with Keyteo logo at the top of every page\n",
            person_name
        );
        fs::write(&readme_path, readme_content)
            .context("Failed to write README.md")?;
        
        Ok(())
    }
}

/// Multi-tenant CV configuration
pub struct CvConfig {
    pub person_name: String,
    pub lang: String,
    pub template: CvTemplate,
    pub output_dir: PathBuf,
    pub data_dir: PathBuf,
    pub templates_dir: PathBuf,
}

impl CvConfig {
    pub fn new(person_name: &str, lang: &str) -> Self {
        Self {
            person_name: person_name.to_string(),
            lang: lang.to_string(),
            template: CvTemplate::Default,
            output_dir: PathBuf::from("output"),
            data_dir: PathBuf::from("data"),
            templates_dir: PathBuf::from("templates"),
        }
    }
    
    pub fn with_template(mut self, template: CvTemplate) -> Self {
        self.template = template;
        self
    }
    
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }
    
    pub fn with_data_dir(mut self, dir: PathBuf) -> Self {
        self.data_dir = dir;
        self
    }
    
    pub fn with_templates_dir(mut self, dir: PathBuf) -> Self {
        self.templates_dir = dir;
        self
    }
    
    /// Get person's data directory
    pub fn person_data_dir(&self) -> PathBuf {
        self.data_dir.join(&self.person_name)
    }
    
    /// Get person's config file path
    pub fn person_config_path(&self) -> PathBuf {
        self.person_data_dir().join("cv_params.toml")
    }
    
    /// Get person's experiences file path
    pub fn person_experiences_path(&self) -> PathBuf {
        self.person_data_dir().join(format!("experiences_{}.typ", self.lang))
    }

    /// Get person's profile image path
    pub fn person_image_path(&self) -> PathBuf {
        self.person_data_dir().join("profile.png")
    }
    
    /// Get the template file to use for compilation
    pub fn template_file_path(&self) -> PathBuf {
        self.templates_dir.join(self.template.template_file())
    }
}

/// Multi-tenant CV Generator
pub struct CvGenerator {
    pub config: CvConfig,
}

impl CvGenerator {
    pub fn new(config: CvConfig) -> Result<Self> {
        // Validate language
        if !["fr", "en"].contains(&config.lang.as_str()) {
            anyhow::bail!("Unsupported language: {}. Use fr, en", config.lang);
        }
        
        // Check if person's data directory exists
        let person_dir = config.person_data_dir();
        if !person_dir.exists() {
            anyhow::bail!("Person directory not found: {}. Create it with required files.", person_dir.display());
        }
        
        // Validate required files exist
        let config_path = config.person_config_path();
        let experiences_path = config.person_experiences_path();
        let template_path = config.template_file_path();
        
        if !config_path.exists() {
            anyhow::bail!("Config file not found: {}", config_path.display());
        }
        
        if !experiences_path.exists() {
            anyhow::bail!("Experiences file not found: {}", experiences_path.display());
        }
        
        if !template_path.exists() {
            anyhow::bail!("Template file not found: {}", template_path.display());
        }
        
        Ok(Self { config })
    }
    
    /// Generate the CV PDF
    pub fn generate(&self) -> Result<PathBuf> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;
        
        let output_path = self.compile_cv()?;
        
        self.cleanup_workspace()?;
        
        println!("✅ Successfully compiled CV for {} ({} template, {} lang) to {}", 
                self.config.person_name, 
                self.config.template.as_str(),
                self.config.lang, 
                output_path.display());
        
        Ok(output_path)
    }
    
    /// Watch for changes and regenerate
    pub fn watch(&self) -> Result<()> {
        self.setup_output_dir()?;
        self.prepare_workspace()?;
        
        let output_path = self.config.output_dir.join(format!("{}_{}_{}.pdf", 
            self.config.person_name, 
            self.config.template.as_str(),
            self.config.lang));
        
        println!("👀 Watching for changes for {} ({} template)...", 
                self.config.person_name, self.config.template.as_str());
        
        let status = Command::new("typst")
            .arg("watch")
            .arg(self.config.template.template_file())
            .arg(&output_path)
            .status()
            .context("Failed to execute typst watch command")?;

        if !status.success() {
            anyhow::bail!("Typst watch failed");
        }
        
        Ok(())
    }
    
    /// Create person's data directory structure using templates (bypassing validation)
    pub fn create_person_unchecked(&self) -> Result<()> {
        let template_processor = TemplateProcessor::new(self.config.templates_dir.clone());
        template_processor.create_person_from_templates(&self.config.person_name, &self.config.data_dir)?;
        
        let person_dir = self.config.person_data_dir();
        println!("Created person directory structure for: {}", self.config.person_name);
        println!("  Directory: {}", person_dir.display());
        println!("  Files created:");
        println!("    - cv_params.toml (edit your personal info)");
        println!("    - experiences_*.typ (for all languages)");
        println!("    - README.md (instructions)");
        println!("  Available templates: {}", CvTemplate::all().join(", "));
        println!("  Next steps:");
        println!("    1. Add your profile image as: profile.png");
        println!("    2. Edit cv_params.toml with your information");
        println!("    3. Update experiences_*.typ files with your work history");
        
        Ok(())
    }
    
    /// Create person's data directory structure using templates
    pub fn create_person(&self) -> Result<()> {
        self.create_person_unchecked()
    }
    
    fn setup_output_dir(&self) -> Result<()> {
        println!("Setting up directories...");
        fs::create_dir_all(&self.config.output_dir)
            .context("Failed to create output directory")?;
        
        // Create temporary workspace directory
        fs::create_dir_all("tmp_workspace")
            .context("Failed to create temporary workspace directory")?;
            
        Ok(())
    }
    
    fn prepare_workspace(&self) -> Result<()> {
        println!("Preparing workspace in tmp_workspace/...");
        
        // Change to temporary workspace directory
        std::env::set_current_dir("tmp_workspace")
            .context("Failed to change to temporary workspace")?;
        
        // Copy person's config to workspace
        let config_source = PathBuf::from("..").join(self.config.person_config_path());
        let config_dest = PathBuf::from("cv_params.toml");
        println!("Copying config from {} to {}", config_source.display(), config_dest.display());
        fs::copy(&config_source, &config_dest)
            .context("Failed to copy person config")?;
        
        // Copy person's experiences file  
        let exp_source = PathBuf::from("..").join(self.config.person_experiences_path());
        let exp_dest = PathBuf::from(format!("experiences_{}.typ", self.config.lang));
        println!("Copying experiences from {} to {}", exp_source.display(), exp_dest.display());
        fs::copy(&exp_source, &exp_dest)
            .context("Failed to copy person experiences")?;
        
        // Copy person's profile image
        let person_image_png = PathBuf::from("..").join(self.config.person_image_path());
        if person_image_png.exists() {
            let profile_dest = PathBuf::from("profile.png");
            println!("Copying profile image from {} to {}", person_image_png.display(), profile_dest.display());
            fs::copy(&person_image_png, &profile_dest)
                .context("Failed to copy person image")?;
        } else {
            println!("No profile image found at {}", person_image_png.display());
        }
        
        // Copy template-specific logo from templates directory
        let template_logo_name = match self.config.template {
            CvTemplate::Keyteo => "keyteo_logo.png",
            CvTemplate::Default => "default_logo.png",
        };
        let template_logo_source = PathBuf::from("..").join(&self.config.templates_dir).join(template_logo_name);
        if template_logo_source.exists() {
            let logo_dest = PathBuf::from(template_logo_name);
            println!("Copying template logo from {} to {}", template_logo_source.display(), logo_dest.display());
            fs::copy(&template_logo_source, &logo_dest)
                .context("Failed to copy template logo")?;
        } else {
            println!("No template-specific logo found at {}", template_logo_source.display());
        }
        
        // Copy the specific template file
        let template_file = PathBuf::from("..").join(&self.config.template_file_path());
        let template_dest = PathBuf::from(self.config.template.template_file());
        if template_file.exists() {
            println!("Copying template from {} to {}", template_file.display(), template_dest.display());
            fs::copy(&template_file, &template_dest)
                .context("Failed to copy template file")?;
        }

        // Copy base template.typ if it exists
        let base_template = PathBuf::from("..").join(&self.config.templates_dir).join("template.typ");
        let base_dest = PathBuf::from("template.typ");
        if base_template.exists() {
            println!("Copying base template from {} to {}", base_template.display(), base_dest.display());
            fs::copy(&base_template, &base_dest)
                .context("Failed to copy template.typ")?;
        }

        Ok(())
    }

    fn compile_cv(&self) -> Result<PathBuf> {
        let output_path = PathBuf::from("..").join(&self.config.output_dir).join(format!("{}_{}_{}.pdf", 
            self.config.person_name, 
            self.config.template.as_str(),
            self.config.lang));
        
        println!("Compiling with typst...");
        println!("Template file: {}", self.config.template.template_file());
        println!("Output path: {}", output_path.display());
        
        let mut cmd = Command::new("typst");
        cmd.arg("compile")
        .arg(self.config.template.template_file())
        .arg(&output_path);
        
        let output = cmd.output()
            .context("Failed to execute typst command")?;

        if !output.status.success() {
            println!("Typst compilation failed!");
            println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Typst compilation failed");
        }
        
        // Check if file actually exists
        if !output_path.exists() {
            anyhow::bail!("PDF was not created at expected path: {}", output_path.display());
        }
        
        println!("PDF created successfully at {}", output_path.display());
        
        Ok(output_path)
    }
    
    fn cleanup_workspace(&self) -> Result<()> {
        // Change back to root directory
        std::env::set_current_dir("..")
            .context("Failed to change back to root directory")?;
        
        println!("Cleaning up temporary workspace...");
        
        // Remove entire temporary workspace directory
        if PathBuf::from("tmp_workspace").exists() {
            fs::remove_dir_all("tmp_workspace")
                .context("Failed to remove temporary workspace directory")?;
            println!("Temporary workspace cleaned up");
        }
        
        Ok(())
    }
}

/// List all available persons
pub fn list_persons(data_dir: &PathBuf) -> Result<Vec<String>> {
    let mut persons = Vec::new();
    
    if !data_dir.exists() {
        return Ok(persons);
    }
    
    let entries = fs::read_dir(data_dir)
        .context("Failed to read data directory")?;
        
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if person has required files
                let config_path = path.join("cv_params.toml");
                if config_path.exists() {
                    persons.push(name.to_string());
                }
            }
        }
    }
    
    persons.sort();
    Ok(persons)
}

/// List all available templates
pub fn list_templates(templates_dir: &PathBuf) -> Result<Vec<String>> {
    let mut templates = Vec::new();
    
    for template in CvTemplate::all() {
        let template_path = templates_dir.join(match template {
            "default" => "cv.typ",
            "keyteo" => "cv_keyteo.typ",
            _ => continue,
        });
        
        if template_path.exists() {
            templates.push(template.to_string());
        }
    }
    
    if templates.is_empty() {
        templates.push("default".to_string()); // Always have default as fallback
    }
    
    Ok(templates)
}

/// Convenience function for quick CV generation
pub fn generate_cv(person_name: &str, lang: &str, template: Option<&str>, output_dir: Option<PathBuf>) -> Result<PathBuf> {
    let mut config = CvConfig::new(person_name, lang);
    
    if let Some(template_str) = template {
        let template = CvTemplate::from_str(template_str)?;
        config = config.with_template(template);
    }
    
    if let Some(dir) = output_dir {
        config = config.with_output_dir(dir);
    }
    
    let generator = CvGenerator::new(config)?;
    generator.generate()
}
````
