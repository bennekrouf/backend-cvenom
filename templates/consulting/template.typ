#import "font_config.typ": font_config, get_icon

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary   = rgb("#023E8A")   // deep corporate blue
#let accent    = rgb("#0096C7")   // ocean teal
#let secondary = rgb("#6B7280")   // neutral gray
#let light_bg  = rgb("#EFF6FF")   // very light blue tint

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Technical Skills",
      "certifications_education": "Education & Certifications",
      "languages":                "Languages",
      "work_experience":          "Mission Experience",
      "key_competencies":         "Key Competencies",
      "sectors":                  "Sector Expertise",
      "availability":             "Availability",
      "summary":                  "Profile",
      "diplomas":                 "Diplomas",
      "certifications":           "Certifications",
    ),
    "fr": (
      "technical_skills":         "Compétences techniques",
      "certifications_education": "Formations & Certifications",
      "languages":                "Langues",
      "work_experience":          "Expériences missions",
      "key_competencies":         "Compétences clés",
      "sectors":                  "Secteurs d'expertise",
      "availability":             "Disponibilité",
      "summary":                  "Profil",
      "diplomas":                 "Diplômes",
      "certifications":           "Certifications",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Icon helpers ───────────────────────────────────────────────────────────────
#let get_default_icons(color: none) = {
  if color == none { color = accent }
  (
    "github":        ("displayname": "GitHub",   "logo": get_icon("github",        font_type: "brands")),
    "linkedin":      ("displayname": "LinkedIn", "logo": get_icon("linkedin",      font_type: "brands")),
    "personal_info": ("displayname": "Web",      "logo": get_icon("personal_info", font_type: "solid")),
  )
}

#let join_dicts(..args) = {
  let result = (:)
  for arg in args.pos() {
    for (key, value) in arg.pairs() { result.insert(key, value) }
  }
  result
}

#let process_links(color: none, icons: none, links) = {
  if icons == none { icons = get_default_icons(color: color) }
  else { icons = join_dicts(get_default_icons(color: color), icons) }
  let link_pairs = ()
  if type(links) == array {
    for l in links { if l != "" and l != none { link_pairs.push(("personal_info", l)) } }
  } else if type(links) == dictionary {
    for (key, value) in links.pairs() {
      if value != "" and value != none and type(value) == str { link_pairs.push((key, value)) }
    }
  }
  if link_pairs.len() > 0 {
    link_pairs.map(it => {
      let key = it.at(0); let url = it.at(1)
      text(fill: color, link(url,
        icons.at(key, default: (:)).at("logo", default: "") + " " +
        icons.at(key, default: (:)).at("displayname", default: key)))
    })
  } else { () }
}

// ── Section heading — solid blue band ─────────────────────────────────────────
#let section(title) = {
  v(0.5em)
  block(
    width: 100%,
    fill: primary,
    inset: (x: 10pt, y: 6pt),
    radius: 3pt,
  )[
    #text(size: 11pt, weight: "bold", fill: white, upper(title))
  ]
  v(0.3em)
}

// ── Competency tag ─────────────────────────────────────────────────────────────
#let competency_tag(label) = {
  box(
    inset: (x: 7pt, y: 3pt),
    radius: 2pt,
    fill: light_bg,
    stroke: 0.5pt + accent,
    text(size: 8.5pt, fill: primary, label)
  )
  h(3pt)
}

// ── Key competencies grid ──────────────────────────────────────────────────────
#let show_competencies(items) = {
  items.map(i => competency_tag(i)).join()
}

// ── Sector badge ───────────────────────────────────────────────────────────────
#let sector_badge(label) = {
  box(
    inset: (x: 7pt, y: 3pt),
    radius: 2pt,
    fill: accent,
    text(size: 8.5pt, fill: white, weight: "bold", label)
  )
  h(3pt)
}

// ── Availability badge ─────────────────────────────────────────────────────────
#let availability_box(text_content) = {
  box(
    inset: (x: 10pt, y: 5pt),
    radius: 3pt,
    fill: rgb("#DCFCE7"),
    stroke: 0.7pt + rgb("#16A34A"),
  )[
    #text(size: 9.5pt, weight: "bold", fill: rgb("#15803D"),
      "✓  " + text_content)
  ]
}

// ── Skills table (2-col) ───────────────────────────────────────────────────────
#let show_skills(separator: none, color: none, skills) = {
  if color == none { color = accent }
  if type(skills) == dictionary and skills.len() > 0 {
    let skills_array = ()
    for (key, value) in skills.pairs() {
      if key != "" and value != none {
        skills_array.push(text(weight: "bold", size: 9.5pt, fill: primary, key))
        if type(value) == array and value.len() > 0 {
          let filtered = value.filter(v => v != "" and v != none)
          skills_array.push(filtered.map(box).join(text(fill: color, "  ·  ")))
        } else if type(value) == str and value != "" {
          skills_array.push(text(size: 9.5pt, value))
        } else {
          skills_array.push([—])
        }
      }
    }
    if skills_array.len() > 0 {
      table(
        columns: 2,
        column-gutter: 2%,
        row-gutter: 0em,
        align: (right, left),
        stroke: none,
        fill: (_, row) => if calc.odd(row) { light_bg } else { white },
        ..skills_array,
      )
    }
  }
}

// ── Bullet list ────────────────────────────────────────────────────────────────
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none { color = accent }
  if symbol == none { symbol = sym.bullet }
  list(
    indent: 5pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 9.5pt, it)),
  )
}

#let date(color: none, content) = {
  if color == none { color = secondary }
  [#h(1fr) #text(weight: "regular", size: 9pt, fill: color, content)]
}

// ── Mission experience entry ───────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  block(
    width: 100%,
    stroke: (left: 3pt + accent),
    inset: (left: 10pt, top: 4pt, bottom: 4pt, right: 4pt),
  )[
    #grid(
      columns: (1fr, auto),
      align: (left + top, right + top),
      [
        #text(size: 11pt, weight: "bold", fill: primary, title)
        #if company != none [
          #linebreak()
          #text(size: 9.5pt, fill: accent, company)
        ]
      ],
      box(
        inset: (x: 5pt, y: 2pt),
        radius: 2pt,
        fill: light_bg,
        text(size: 8.5pt, fill: primary, date)
      )
    )
    #if description != none {
      v(0.15em)
      text(size: 9.5pt, style: "italic", fill: secondary, description)
    }
    #if content != none {
      v(0.15em)
      content
    }
  ]
  v(0.5em)
}

// ── Header ─────────────────────────────────────────────────────────────────────
#let show_header(details) = {
  block(
    width: 100%,
    fill: primary,
    inset: (x: 1.5cm, y: 0.8cm),
    radius: (bottom: 6pt),
  )[
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      [
        #text(size: 22pt, weight: "bold", fill: white,
          details.at("name", default: ""))
        #linebreak()
        #text(size: 11pt, fill: accent,
          details.at("job_title", default:
            details.at("title", default: "")))
        #linebreak()
        #v(0.3em)
        // Contact
        #set text(size: 8.5pt, fill: white.darken(10%))
        #let parts = ()
        #if details.at("email", default: "") != "" {
          parts.push(link("mailto:" + details.email, text(fill: accent.lighten(40%), details.email)))
        }
        #if details.at("phonenumber", default: "") != "" {
          parts.push(details.phonenumber)
        }
        #if details.at("address", default: "") != "" {
          parts.push(details.address)
        }
        #parts.join("  ·  ")
        // Links
        #if details.at("links", default: none) != none {
          let processed = process_links(details.links, color: accent.lighten(40%))
          if processed.len() > 0 {
            linebreak()
            processed.join("  ·  ")
          }
        }
      ],
      // Availability badge on right of header
      if details.at("availability", default: "") != "" {
        block(inset: (x: 10pt, y: 6pt), radius: 4pt, fill: rgb("#16A34A"))[
          #text(size: 9pt, weight: "bold", fill: white,
            "✓  " + details.availability)
        ]
      }
    )
  ]
}

// ── Main conf ──────────────────────────────────────────────────────────────────
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
  show math.equation: set text(font: default_math_font)
  show link: set text(fill: accent)
  show list: set text(size: 9.5pt)
  show "C++": box

  set text(font: ("Arial", "Helvetica"), ligatures: false)
  set par(justify: true)
  set page(
    margin: (top: 0cm, left: 0cm, bottom: 1.2cm, right: 0cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, sym.bullet))

  show_header(details)

  pad(x: 1.5cm)[
    #doc
  ]
}
