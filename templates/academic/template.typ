#import "font_config.typ": font_config, get_icon

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary   = rgb("#1E3A5F")   // dark academic blue
#let accent    = rgb("#2E7D32")   // university green
#let secondary = rgb("#546E7A")   // blue-gray
#let rule_clr  = rgb("#90A4AE")   // soft separator

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Methods & Tools",
      "certifications_education": "Education",
      "languages":                "Languages",
      "work_experience":          "Research & Academic Experience",
      "publications":             "Publications",
      "grants":                   "Grants & Funding",
      "research_interests":       "Research Interests",
      "teaching":                 "Teaching Experience",
      "awards":                   "Awards & Honours",
      "summary":                  "Research Profile",
    ),
    "fr": (
      "technical_skills":         "Méthodes & Outils",
      "certifications_education": "Formation",
      "languages":                "Langues",
      "work_experience":          "Expérience académique & recherche",
      "publications":             "Publications",
      "grants":                   "Financements & Bourses",
      "research_interests":       "Thèmes de recherche",
      "teaching":                 "Enseignement",
      "awards":                   "Prix & Distinctions",
      "summary":                  "Profil de recherche",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Icon helpers ───────────────────────────────────────────────────────────────
#let get_default_icons(color: none) = {
  if color == none { color = accent }
  (
    "github":        ("displayname": "GitHub",    "logo": get_icon("github",        font_type: "brands")),
    "linkedin":      ("displayname": "LinkedIn",  "logo": get_icon("linkedin",      font_type: "brands")),
    "personal_info": ("displayname": "Web",       "logo": get_icon("personal_info", font_type: "solid")),
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em,
      circle(radius: 0.5em, fill: accent, inset: 0pt,
        align(center + horizon, text(size: 0.8em, fill: white, "iD"))))),
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

// ── Section heading — classic academic style ───────────────────────────────────
#let section(title) = {
  v(0.6em)
  grid(
    columns: (auto, 1fr),
    column-gutter: 8pt,
    align: horizon,
    text(size: 11.5pt, weight: "bold", fill: primary, title),
    line(length: 100%, stroke: 0.8pt + rule_clr),
  )
  v(0.3em)
}

// ── Bullet list ────────────────────────────────────────────────────────────────
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none { color = accent }
  if symbol == none { symbol = sym.bullet }
  list(
    indent: 5pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 10pt, it)),
  )
}

#let date(color: none, content) = {
  if color == none { color = secondary }
  [#h(1fr) #text(weight: "regular", size: 9pt, fill: color, content)]
}

// ── Academic experience entry ──────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #text(size: 11pt, weight: "bold", fill: primary, title)
      #if company != none [
        #linebreak()
        #text(size: 9.5pt, fill: secondary, style: "italic", company)
      ]
    ],
    text(size: 9pt, fill: secondary, date)
  )
  if description != none {
    v(0.1em)
    text(size: 9.5pt, fill: secondary, description)
  }
  if content != none {
    v(0.15em)
    content
  }
  v(0.5em)
}

// ── Publication entry ──────────────────────────────────────────────────────────
#let publication_entry(pub) = {
  block(
    width: 100%,
    inset: (left: 12pt, top: 3pt, bottom: 3pt),
    stroke: (left: 2pt + accent),
  )[
    // Title in bold
    #if pub.at("url", default: "") != "" {
      text(size: 10pt, weight: "bold", fill: primary,
        link(pub.url, pub.at("title", default: "Untitled")))
    } else {
      text(size: 10pt, weight: "bold", fill: primary,
        pub.at("title", default: "Untitled"))
    }
    #linebreak()
    // Authors + venue + year
    #text(size: 9pt, fill: secondary,
      pub.at("authors", default: "") + "  ·  " +
      text(style: "italic", pub.at("venue", default: "")) +
      if pub.at("year", default: "") != "" { "  (" + pub.at("year", default: "") + ")" } else { "" }
    )
  ]
  v(0.2em)
}

// ── Grant entry ────────────────────────────────────────────────────────────────
#let grant_entry(g) = {
  grid(
    columns: (1fr, auto),
    align: (left, right),
    [
      #text(size: 10pt, weight: "bold", fill: primary, g.at("title", default: ""))
      #if g.at("funder", default: "") != "" [
        #linebreak()
        #text(size: 9pt, fill: secondary, g.at("funder", default: ""))
      ]
    ],
    [
      #if g.at("amount", default: "") != "" {
        box(inset: (x: 5pt, y: 2pt), radius: 2pt, fill: accent.lighten(88%),
          text(size: 9pt, weight: "bold", fill: accent, g.amount))
      }
      #if g.at("year", default: "") != "" {
        linebreak()
        text(size: 9pt, fill: secondary, g.year)
      }
    ]
  )
  v(0.3em)
}

// ── Research interest tag ──────────────────────────────────────────────────────
#let interest_tag(label) = {
  box(
    inset: (x: 7pt, y: 3pt),
    radius: 3pt,
    fill: accent.lighten(88%),
    stroke: 0.5pt + accent,
    text(size: 9pt, fill: primary, label)
  )
  h(4pt)
}

// ── Skills table ───────────────────────────────────────────────────────────────
#let show_skills(separator: none, color: none, skills) = {
  if color == none { color = accent }
  if type(skills) == dictionary and skills.len() > 0 {
    let skills_array = ()
    for (key, value) in skills.pairs() {
      if key != "" and value != none {
        skills_array.push(text(weight: "bold", size: 9.5pt, fill: primary, key))
        if type(value) == array and value.len() > 0 {
          let filtered = value.filter(v => v != "" and v != none)
          skills_array.push(filtered.join(", "))
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
        ..skills_array,
      )
    }
  }
}

// ── Header ─────────────────────────────────────────────────────────────────────
#let show_header(details) = {
  // Top ruled line in accent
  rect(width: 100%, height: 4pt, fill: primary)
  v(0.5em)
  grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #text(size: 22pt, weight: "bold", fill: primary,
        details.at("name", default: ""))
      #linebreak()
      #text(size: 11pt, fill: accent, weight: "bold",
        details.at("job_title", default:
          details.at("title", default: "")))
      #linebreak()
      #if details.at("address", default: "") != "" {
        text(size: 9.5pt, fill: secondary, details.address)
        linebreak()
      }
    ],
    [
      #set align(right)
      #let _pic = sys.inputs.at("picture", default: none)
      #let show_photo = details.at("styling", default: (:)).at("show_photo", default: false)
      #if _pic != none and show_photo {
        block(clip: true, radius: 50%,
          image(_pic, width: 70pt, height: 70pt, fit: "cover"))
        v(0.3em)
      }
      #if details.at("email", default: "") != "" {
        text(size: 9.5pt, fill: accent,
          link("mailto:" + details.email, details.email))
        linebreak()
      }
      #if details.at("phonenumber", default: "") != "" {
        text(size: 9.5pt, fill: secondary, details.phonenumber)
        linebreak()
      }
      #if details.at("links", default: none) != none {
        let processed = process_links(details.links, color: accent)
        if processed.len() > 0 {
          processed.join(text(fill: secondary, "  ·  "))
        }
      }
    ]
  )
  v(0.2em)
  line(length: 100%, stroke: 1.5pt + primary)
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
    margin: (top: 1cm, left: 1.8cm, bottom: 1.5cm, right: 1.8cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, sym.bullet))

  show_header(details)
  doc
}
