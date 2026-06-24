#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label, nonempty

// ── Palette ───────────────────────────────────────────────────────────────────
#let light_bg   = rgb("#EFF6FF")   // very light blue tint (fixed)
// User-customizable: primary_color → primary+accent, secondary_color → secondary
#let _u_primary = sys.inputs.at("primary_color",   default: none)
#let _u_sec     = sys.inputs.at("secondary_color",  default: none)
#let primary    = if _u_primary != none { rgb(_u_primary) } else { rgb("#023E8A") }
#let accent     = if _u_primary != none { rgb(_u_primary).lighten(20%) } else { rgb("#0096C7") }
#let secondary  = if _u_sec     != none { rgb(_u_sec)     } else { rgb("#6B7280") }

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
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
    ),
    "de": (
      "technical_skills":         "Technische Kompetenzen",
      "certifications_education": "Bildung & Zertifizierungen",
      "languages":                "Sprachen",
      "work_experience":          "Projekterfahrung",
      "key_competencies":         "Kernkompetenzen",
      "sectors":                  "Branchenexpertise",
      "availability":             "Verfügbarkeit",
      "summary":                  "Profil",
      "diplomas":                 "Abschlüsse",
      "certifications":           "Zertifizierungen",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
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
        if type(value) == array {
          let filtered = value.filter(v => v != "" and v != none)
          if filtered.len() > 0 {
            skills_array.push(text(weight: "bold", size: 9.5pt, fill: primary, skill_label(key)))
            skills_array.push(filtered.map(box).join(text(fill: color, "  ·  ")))
          }
        } else if type(value) == str and value != "" {
          skills_array.push(text(weight: "bold", size: 9.5pt, fill: primary, skill_label(key)))
          skills_array.push(text(size: 9.5pt, value))
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
        #if nonempty(title) [
          #text(size: 11pt, weight: "bold", fill: primary, title)
        ]
        #if company != none [
          #if nonempty(title) { linebreak() }
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
    // Skip description when there's no role — without it, the description
    // block reads as a fake role beneath the company name.
    #if description != none and nonempty(title) {
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
      // Photo and/or availability badge on right of header
      [
        #let _pic = sys.inputs.at("picture", default: none)
        #let show_photo = details.at("styling", default: (:)).at("show_photo", default: true)
        #if _pic != none and show_photo {
          align(center,
            block(clip: true, radius: 50%,
              image(_pic, width: 70pt, height: 70pt, fit: "cover")))
          v(0.4em)
        }
        #if details.at("availability", default: "") != "" {
          block(inset: (x: 10pt, y: 6pt), radius: 4pt, fill: rgb("#16A34A"))[
            #text(size: 9pt, weight: "bold", fill: white,
              "✓  " + details.availability)
          ]
        }
      ]
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
  show heading.where(level: 1): none

  set text(font: ("Arial", "Helvetica", "DejaVu Sans"), ligatures: false)
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
