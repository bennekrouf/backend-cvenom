#import "font_config.typ": font_config, get_icon

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary   = rgb("#2D3748")   // slate dark
#let accent    = rgb("#4299E1")   // bright blue
#let secondary = rgb("#718096")   // muted gray
#let sidebar_bg = rgb("#F7FAFC")  // very light gray sidebar

#let default_font      = "Carlito"
#let default_math_font = "Times"
#let default_separator = text(font: "Carlito", fill: accent, " \u{007c} ")

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Technical Skills",
      "certifications_education": "Education & Certifications",
      "languages":                "Languages",
      "work_experience":          "Experience",
      "projects":                 "Projects",
      "summary":                  "About",
    ),
    "fr": (
      "technical_skills":         "Compétences techniques",
      "certifications_education": "Formations & Certifications",
      "languages":                "Langues",
      "work_experience":          "Expérience professionnelle",
      "projects":                 "Projets",
      "summary":                  "Profil",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Icon helpers (reused from default) ────────────────────────────────────────
#let get_default_icons(color: none) = {
  if color == none { color = accent }
  (
    "github":        ("displayname": "GitHub",   "logo": get_icon("github",   font_type: "brands")),
    "linkedin":      ("displayname": "LinkedIn", "logo": get_icon("linkedin", font_type: "brands")),
    "personal_info": ("displayname": "Web",      "logo": get_icon("personal_info", font_type: "solid")),
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em,
      circle(radius: 0.5em, fill: color, inset: 0pt,
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

#let colorlink(color: none, url, body) = {
  if color == none { color = accent }
  text(fill: color, link(url)[#body])
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

// ── Section heading — left accent bar style ────────────────────────────────────
#let section(title) = {
  v(0.4em)
  block(width: 100%, inset: 0pt)[
    #grid(
      columns: (4pt, 1fr),
      column-gutter: 6pt,
      rect(width: 4pt, height: 1.2em, fill: accent, radius: 2pt),
      align(left + horizon,
        text(size: 11pt, weight: "bold", fill: primary, upper(title))
      )
    )
  ]
  v(0.3em)
}

// ── Sidebar section heading ────────────────────────────────────────────────────
#let sidebar_section(title) = {
  v(0.5em)
  text(size: 9.5pt, weight: "bold", fill: accent, upper(title))
  v(0.1em)
  line(length: 100%, stroke: 0.5pt + accent)
  v(0.2em)
}

// ── Skill chips ────────────────────────────────────────────────────────────────
#let skill_chip(label) = {
  box(
    inset: (x: 5pt, y: 2pt),
    radius: 3pt,
    fill: accent.lighten(80%),
    text(size: 8.5pt, fill: primary, label)
  )
}

// ── Skills block (sidebar) ─────────────────────────────────────────────────────
#let show_skills_sidebar(skills) = {
  if type(skills) == dictionary and skills.len() > 0 {
    for (cat, items) in skills.pairs() {
      if cat != "" and items != none {
        text(size: 8.5pt, weight: "bold", fill: secondary, upper(cat))
        v(0.15em)
        if type(items) == array and items.len() > 0 {
          let filtered = items.filter(v => v != "" and v != none)
          filtered.map(i => { skill_chip(i); h(2pt) }).join()
        } else if type(items) == str and items != "" {
          skill_chip(items)
        }
        v(0.4em)
      }
    }
  }
}

// ── Language level indicator ───────────────────────────────────────────────────
#let lang_level_dot(filled) = {
  if filled {
    circle(radius: 3pt, fill: accent)
  } else {
    circle(radius: 3pt, stroke: 0.8pt + accent, fill: white)
  }
}

#let show_lang_level(name, level) = {
  // level: 1-4 (native=4, fluent=3, intermediate=2, basic=1)
  grid(
    columns: (1fr, auto),
    align: (left, right),
    text(size: 8.5pt, fill: primary, name),
    grid(
      columns: (auto, auto, auto, auto),
      column-gutter: 2pt,
      ..range(4).map(i => lang_level_dot(i < level))
    )
  )
  v(0.1em)
}

// ── Experience entry ───────────────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  grid(
    columns: (1fr, auto),
    align: (left, right),
    [
      #text(size: 11pt, weight: "bold", fill: primary, title)
      #if company != none [ #text(size: 9.5pt, fill: accent, " @ " + company) ]
    ],
    text(size: 8.5pt, fill: secondary, date)
  )
  if description != none {
    v(0.1em)
    text(size: 9.5pt, style: "italic", fill: secondary, description)
  }
  if content != none {
    v(0.15em)
    content
  }
  v(0.6em)
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

// ── Header (name + contact) ────────────────────────────────────────────────────
#let show_header(details) = {
  block(width: 100%, fill: primary, inset: (x: 16pt, y: 12pt), radius: (top: 4pt, bottom: 0pt))[
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      [
        #text(size: 22pt, weight: "bold", fill: white,
          details.at("name", default: ""))
        #linebreak()
        #text(size: 12pt, fill: accent,
          details.at("job_title", default:
            details.at("title", default: "")))
      ],
      [
        #set text(size: 8.5pt, fill: white)
        #if details.at("email", default: "") != "" {
          text(fill: accent.lighten(60%),
            link("mailto:" + details.email, details.email))
          linebreak()
        }
        #if details.at("phonenumber", default: "") != "" {
          details.phonenumber
          linebreak()
        }
        #if details.at("address", default: "") != "" {
          details.address
        }
      ]
    )
  ]
  // accent bar under header
  rect(width: 100%, height: 3pt, fill: accent, radius: (top: 0pt, bottom: 4pt))
}

// ── Sidebar ────────────────────────────────────────────────────────────────────
#let show_sidebar(details) = {
  block(width: 100%, fill: sidebar_bg, inset: 10pt, radius: 4pt)[

    // Links
    #if details.at("links", default: none) != none {
      let links = process_links(details.links, color: accent)
      if links.len() > 0 {
        for l in links { l; linebreak() }
        v(0.3em)
      }
    }

    // Picture
    #if details.at("picture", default: "").len() > 0 and details.at("styling", default: (:)).at("show_photo", default: false) {
      if sys.inputs.at("picture", default: none) != none {
        align(center,
          block(clip: true, radius: 50%,
            image(details.picture, width: 80pt, height: 80pt, fit: "cover")))
        v(0.5em)
      }
    }

    // Skills
    #if "skills" in details {
      sidebar_section(get_text("technical_skills"))
      show_skills_sidebar(details.skills)
    }

    // Languages
    #if "languages" in details {
      sidebar_section(get_text("languages"))
      let lvl = details.languages
      if "native"       in lvl { for l in lvl.native       { show_lang_level(l, 4) } }
      if "fluent"       in lvl { for l in lvl.fluent       { show_lang_level(l, 3) } }
      if "intermediate" in lvl { for l in lvl.intermediate { show_lang_level(l, 2) } }
      if "basic"        in lvl { for l in lvl.basic        { show_lang_level(l, 1) } }
    }

    // Education
    #if "education" in details {
      sidebar_section(get_text("certifications_education"))
      for item in details.education {
        text(size: 8.5pt, weight: "bold", fill: primary, item.title)
        linebreak()
        text(size: 8pt, fill: secondary, item.date)
        v(0.3em)
      }
    }
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
    margin: (top: 0.6cm, left: 0cm, bottom: 1cm, right: 0cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, sym.bullet))

  // Full-width header
  pad(x: 0pt)[#show_header(details)]

  // Two-column body
  pad(x: 0.8cm)[
    #grid(
      columns: (28%, 4%, 68%),
      align: (top, top, top),
      show_sidebar(details),
      [],  // gutter
      doc
    )
  ]
}
