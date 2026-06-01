#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary    = rgb("#2D3748")   // slate dark (fixed)
#let sidebar_bg = rgb("#F7FAFC")   // very light gray sidebar (fixed)
// User-customizable: primary_color → accent, secondary_color → secondary
#let _u_accent  = sys.inputs.at("primary_color",   default: none)
#let _u_sec     = sys.inputs.at("secondary_color",  default: none)
#let accent     = if _u_accent != none { rgb(_u_accent) } else { rgb("#4299E1") }
#let secondary  = if _u_sec    != none { rgb(_u_sec)    } else { rgb("#718096") }

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"
#let default_separator = text(font: "Liberation Sans", fill: accent, " \u{007c} ")

// ── Language helpers ───────────────────────────────────────────────────────────
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
    ),
    "de": (
      "technical_skills":         "Technische Fähigkeiten",
      "certifications_education": "Bildung & Zertifizierungen",
      "languages":                "Sprachen",
      "work_experience":          "Erfahrung",
      "projects":                 "Projekte",
      "summary":                  "Über mich",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Color link helper ─────────────────────────────────────────────────────────
#let colorlink(color: none, url, body) = {
  if color == none { color = accent }
  text(fill: color, link(url)[#body])
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
        text(size: 8.5pt, weight: "bold", fill: secondary, upper(skill_label(cat)))
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

    // Picture — use sys.inputs directly; no need for a `picture` key in the TOML
    #let _pic = sys.inputs.at("picture", default: none)
    #if _pic != none and details.at("styling", default: (:)).at("show_photo", default: true) {
      align(center,
        block(clip: true, radius: 50%,
          image(_pic, width: 80pt, height: 80pt, fit: "cover")))
      v(0.5em)
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
  show heading.where(level: 1): none

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
