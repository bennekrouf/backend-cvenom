#import "font_config.typ": font_config, get_icon

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary   = rgb("#1A1A2E")   // deep navy
#let accent    = rgb("#C9A84C")   // gold
#let secondary = rgb("#4A4A6A")   // muted slate
#let rule_color = rgb("#C9A84C")  // gold rules

#let default_font      = "Carlito"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Core Competencies",
      "certifications_education": "Education & Certifications",
      "languages":                "Languages",
      "work_experience":          "Professional Experience",
      "key_achievements":         "Key Achievements",
      "board_memberships":        "Board Memberships",
      "summary":                  "Executive Summary",
    ),
    "fr": (
      "technical_skills":         "Compétences clés",
      "certifications_education": "Formations & Certifications",
      "languages":                "Langues",
      "work_experience":          "Expérience professionnelle",
      "key_achievements":         "Réalisations clés",
      "board_memberships":        "Mandats & Conseils",
      "summary":                  "Résumé exécutif",
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

// ── Gold divider ───────────────────────────────────────────────────────────────
#let gold_rule() = {
  line(length: 100%, stroke: 0.7pt + accent)
}

// ── Section heading — classic with gold underline ──────────────────────────────
#let section(title) = {
  v(0.6em)
  text(size: 11.5pt, weight: "bold", fill: primary, upper(title))
  v(0.1em)
  gold_rule()
  v(0.3em)
}

// ── Bullet list ────────────────────────────────────────────────────────────────
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none { color = accent }
  if symbol == none { symbol = "›" }
  list(
    indent: 5pt,
    marker: text(fill: color, size: 13pt, symbol),
    ..args.pos().map(it => text(size: 10pt, it)),
  )
}

#let date(color: none, content) = {
  if color == none { color = secondary }
  [#h(1fr) #text(weight: "regular", size: 9.5pt, fill: color, content)]
}

// ── Experience entry ───────────────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #text(size: 11.5pt, weight: "bold", fill: primary, title)
      #if company != none [
        #linebreak()
        #text(size: 10pt, fill: accent, company)
      ]
    ],
    text(size: 9pt, fill: secondary, date)
  )
  if description != none {
    v(0.2em)
    text(size: 9.5pt, style: "italic", fill: secondary, description)
  }
  if content != none {
    v(0.2em)
    content
  }
  v(0.7em)
}

// ── Key achievement box ────────────────────────────────────────────────────────
#let achievement_box(items) = {
  block(
    width: 100%,
    stroke: (left: 3pt + accent),
    inset: (left: 10pt, top: 6pt, bottom: 6pt, right: 6pt),
    fill: accent.lighten(92%),
    radius: (right: 4pt),
  )[
    #for item in items {
      grid(
        columns: (auto, 1fr),
        column-gutter: 6pt,
        text(fill: accent, size: 12pt, "◆"),
        text(size: 10pt, fill: primary, item)
      )
      v(0.2em)
    }
  ]
}

// ── Skills grid (3-col) ────────────────────────────────────────────────────────
#let show_skills(separator: none, color: none, skills) = {
  if color == none { color = accent }
  if type(skills) == dictionary and skills.len() > 0 {
    let items = ()
    for (cat, values) in skills.pairs() {
      if cat != "" and values != none {
        let val_str = if type(values) == array {
          values.filter(v => v != "").join(" · ")
        } else { str(values) }
        items.push(
          block(inset: (x: 4pt, y: 4pt))[
            #text(size: 9pt, weight: "bold", fill: primary, cat + ": ")
            #text(size: 9pt, fill: secondary, val_str)
          ]
        )
      }
    }
    grid(
      columns: (1fr, 1fr),
      column-gutter: 1em,
      row-gutter: 0.1em,
      ..items
    )
  }
}

// ── Header ─────────────────────────────────────────────────────────────────────
#let show_header(details) = {
  v(0.5em)
  align(center)[
    #text(size: 26pt, weight: "bold", fill: primary,
      details.at("name", default: ""))
    #linebreak()
    #v(-0.3em)
    #gold_rule()
    #v(-0.1em)
    #text(size: 13pt, fill: accent, weight: "bold",
      details.at("job_title", default:
        details.at("title", default: "")))
    #linebreak()
    #v(0.2em)
    // Contact line
    #set text(size: 9.5pt, fill: secondary)
    #let parts = ()
    #if details.at("email", default: "") != "" {
      parts.push(link("mailto:" + details.email, text(fill: accent, details.email)))
    }
    #if details.at("phonenumber", default: "") != "" {
      parts.push(details.phonenumber)
    }
    #if details.at("address", default: "") != "" {
      parts.push(details.address)
    }
    #parts.join(text(fill: accent, "  ·  "))
    // Links
    #if details.at("links", default: none) != none {
      let processed = process_links(details.links, color: accent)
      if processed.len() > 0 {
        linebreak()
        processed.join(text(fill: accent, "  ·  "))
      }
    }
  ]
  v(0.5em)
  gold_rule()
  v(0.3em)
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
  show list: set text(size: 10pt)
  show "C++": box

  set text(font: ("Arial", "Helvetica"), ligatures: false)
  set par(justify: true)
  set page(
    margin: (top: 1.2cm, left: 2cm, bottom: 1.5cm, right: 2cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, "›"))

  show_header(details)
  doc
}
