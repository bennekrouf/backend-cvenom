#import "font_config.typ": font_config, get_icon

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary    = rgb("#1C1C1E")   // near-black
#let accent     = rgb("#E85D75")   // vibrant coral/rose
#let accent2    = rgb("#F4A261")   // warm amber (secondary accent)
#let secondary  = rgb("#6B7280")   // muted gray
#let sidebar_bg = rgb("#1C1C1E")   // dark sidebar
#let sidebar_fg = rgb("#F9FAFB")   // light text on sidebar

#let default_font      = "Carlito"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_lang() = { sys.inputs.at("lang", default: "en") }

#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Tools & Skills",
      "certifications_education": "Education",
      "languages":                "Languages",
      "work_experience":          "Experience",
      "projects":                 "Projects & Portfolio",
      "summary":                  "About Me",
      "software":                 "Software",
      "disciplines":              "Disciplines",
    ),
    "fr": (
      "technical_skills":         "Outils & Compétences",
      "certifications_education": "Formation",
      "languages":                "Langues",
      "work_experience":          "Expérience",
      "projects":                 "Projets & Portfolio",
      "summary":                  "À propos",
      "software":                 "Logiciels",
      "disciplines":              "Disciplines",
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
    "personal_info": ("displayname": "Portfolio", "logo": get_icon("personal_info", font_type: "solid")),
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

// ── Sidebar section heading ────────────────────────────────────────────────────
#let sidebar_section(title) = {
  v(0.6em)
  text(size: 9.5pt, weight: "bold", fill: accent, upper(title))
  v(0.15em)
  line(length: 100%, stroke: 0.5pt + accent.lighten(50%))
  v(0.25em)
}

// ── Main area section heading ──────────────────────────────────────────────────
#let section(title) = {
  v(0.5em)
  grid(
    columns: (auto, 1fr),
    column-gutter: 8pt,
    align: horizon,
    rect(width: 18pt, height: 3pt, fill: accent, radius: 2pt),
    text(size: 11pt, weight: "bold", fill: primary, upper(title))
  )
  v(0.3em)
}

// ── Skill pill (sidebar) ───────────────────────────────────────────────────────
#let skill_pill(label) = {
  box(
    inset: (x: 6pt, y: 3pt),
    radius: 20pt,
    stroke: 0.7pt + accent.lighten(60%),
    text(size: 8pt, fill: sidebar_fg, label)
  )
  h(3pt)
}

// ── Show skills in sidebar (pill style) ────────────────────────────────────────
#let show_skills_sidebar(skills) = {
  if type(skills) == dictionary and skills.len() > 0 {
    for (cat, items) in skills.pairs() {
      if cat != "" and items != none {
        text(size: 8pt, fill: accent2, weight: "bold", upper(cat))
        v(0.2em)
        if type(items) == array and items.len() > 0 {
          let filtered = items.filter(v => v != "" and v != none)
          filtered.map(i => skill_pill(i)).join()
        } else if type(items) == str and items != "" {
          skill_pill(items)
        }
        v(0.4em)
      }
    }
  }
}

// ── Proficiency dots (sidebar languages) ──────────────────────────────────────
#let prof_dots(level, max: 4) = {
  grid(
    columns: range(max).map(_ => auto),
    column-gutter: 3pt,
    ..range(max).map(i =>
      circle(radius: 3.5pt,
        fill: if i < level { accent } else { white.darken(15%) }))
  )
}

#let show_lang_sidebar(name, level) = {
  grid(
    columns: (1fr, auto),
    align: (left + horizon, right + horizon),
    text(size: 8.5pt, fill: sidebar_fg, name),
    prof_dots(level)
  )
  v(0.2em)
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

// ── Experience entry ───────────────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #text(size: 11pt, weight: "bold", fill: primary, title)
      #if company != none [
        #text(size: 9pt, fill: accent, " · " + company)
      ]
    ],
    box(
      inset: (x: 5pt, y: 2pt),
      radius: 10pt,
      fill: accent.lighten(88%),
      text(size: 8pt, fill: accent, date)
    )
  )
  if description != none {
    v(0.1em)
    text(size: 9.5pt, style: "italic", fill: secondary, description)
  }
  if content != none {
    v(0.15em)
    content
  }
  v(0.7em)
}

// ── Portfolio link button ──────────────────────────────────────────────────────
#let portfolio_button(url) = {
  box(
    inset: (x: 12pt, y: 6pt),
    radius: 4pt,
    fill: accent,
  )[
    #text(size: 9.5pt, weight: "bold", fill: white,
      link(url, "🔗 " + get_text("projects")))
  ]
}

// ── Sidebar ────────────────────────────────────────────────────────────────────
#let show_sidebar(details) = {
  block(
    width: 100%,
    height: 100%,
    fill: sidebar_bg,
    inset: 12pt,
  )[
    // Photo
    #if details.at("picture", default: "").len() > 0 and details.at("styling", default: (:)).at("show_photo", default: false) {
      if sys.inputs.at("picture", default: none) != none {
        align(center,
          block(clip: true, radius: 50%,
            image(details.picture, width: 75pt, height: 75pt, fit: "cover")))
        v(0.6em)
      }
    }

    // Name + title on sidebar (mobile-card feel)
    #align(center)[
      #text(size: 13pt, weight: "bold", fill: sidebar_fg,
        details.at("name", default: ""))
      #linebreak()
      #text(size: 9pt, fill: accent,
        details.at("job_title", default:
          details.at("title", default: "")))
    ]
    #v(0.3em)
    #line(length: 100%, stroke: 0.5pt + accent.lighten(50%))

    // Contact info
    #v(0.3em)
    #if details.at("email", default: "") != "" {
      text(size: 8pt, fill: sidebar_fg,
        link("mailto:" + details.email, text(fill: accent, details.email)))
      linebreak()
    }
    #if details.at("phonenumber", default: "") != "" {
      text(size: 8pt, fill: sidebar_fg, details.phonenumber)
      linebreak()
    }
    #if details.at("address", default: "") != "" {
      text(size: 8pt, fill: sidebar_fg, details.address)
    }

    // Links
    #if details.at("links", default: none) != none {
      let links = process_links(details.links, color: accent)
      if links.len() > 0 {
        v(0.3em)
        for l in links { l; linebreak() }
      }
    }

    // Portfolio CTA
    #if details.at("portfolio", default: "") != "" {
      v(0.5em)
      align(center, portfolio_button(details.portfolio))
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
      if "native"       in lvl { for l in lvl.native       { show_lang_sidebar(l, 4) } }
      if "fluent"       in lvl { for l in lvl.fluent       { show_lang_sidebar(l, 3) } }
      if "intermediate" in lvl { for l in lvl.intermediate { show_lang_sidebar(l, 2) } }
      if "basic"        in lvl { for l in lvl.basic        { show_lang_sidebar(l, 1) } }
    }

    // Education
    #if "education" in details {
      sidebar_section(get_text("certifications_education"))
      for item in details.education {
        text(size: 8.5pt, weight: "bold", fill: sidebar_fg, item.title)
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
    margin: (top: 0cm, left: 0cm, bottom: 0cm, right: 0cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, sym.bullet))

  // Full-bleed two-column layout
  grid(
    columns: (30%, 70%),
    rows: (auto,),
    show_sidebar(details),
    // Right content column
    pad(x: 1.2cm, y: 0.8cm)[
      // Big name header on main area
      #v(0.5em)
      #block(
        stroke: (left: 5pt + accent),
        inset: (left: 12pt, top: 4pt, bottom: 4pt),
      )[
        #text(size: 24pt, weight: "bold", fill: primary,
          details.at("name", default: ""))
        #linebreak()
        #text(size: 12pt, fill: accent,
          details.at("job_title", default:
            details.at("title", default: "")))
      ]
      #v(0.5em)

      // Summary
      #if details.at("summary", default: "") != "" {
        section(get_text("summary"))
        text(size: 9.5pt, fill: primary, details.summary)
      }

      #doc
    ]
  )
}
