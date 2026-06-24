#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label, nonempty

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary    = rgb("#1C1C1E")   // near-black (fixed)
#let sidebar_bg = rgb("#1C1C1E")   // dark sidebar (fixed)
#let sidebar_fg = rgb("#F9FAFB")   // light text on sidebar (fixed)
// User-customizable: primary_color → accent, secondary_color → secondary
#let _u_accent  = sys.inputs.at("primary_color",   default: none)
#let _u_sec     = sys.inputs.at("secondary_color",  default: none)
#let accent     = if _u_accent != none { rgb(_u_accent) } else { rgb("#E85D75") }
#let accent2    = rgb("#F4A261")   // warm amber (secondary accent, fixed)
#let secondary  = if _u_sec    != none { rgb(_u_sec)    } else { rgb("#6B7280") }

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
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
    ),
    "de": (
      "technical_skills":         "Werkzeuge & Fähigkeiten",
      "certifications_education": "Bildung",
      "languages":                "Sprachen",
      "work_experience":          "Erfahrung",
      "projects":                 "Projekte & Portfolio",
      "summary":                  "Über mich",
      "software":                 "Software",
      "disciplines":              "Disziplinen",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
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
        // Compute non-empty items before deciding whether to render the heading
        let has_content = if type(items) == array {
          items.filter(v => v != "" and v != none).len() > 0
        } else if type(items) == str {
          items != ""
        } else {
          false
        }
        if has_content {
          text(size: 8pt, fill: accent2, weight: "bold", upper(skill_label(cat)))
          v(0.2em)
          if type(items) == array {
            let filtered = items.filter(v => v != "" and v != none)
            filtered.map(i => skill_pill(i)).join()
          } else {
            skill_pill(items)
          }
          v(0.4em)
        }
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
      #if nonempty(title) [
        #text(size: 11pt, weight: "bold", fill: primary, title)
      ]
      #if company != none [
        // " · " separator only when there's a title to separate from.
        #text(size: 9pt, fill: accent, if nonempty(title) { " · " } else { "" } + company)
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
    // Photo — use sys.inputs directly; no need for a `picture` key in the TOML
    #let _pic = sys.inputs.at("picture", default: none)
    #let _show_photo = details.at("styling", default: (:)).at("show_photo", default: true)
    #if _show_photo {
      align(center,
        if _pic != none {
          block(clip: true, radius: 50%,
            image(_pic, width: 75pt, height: 75pt, fit: "cover"))
        } else {
          block(
            width: 75pt, height: 75pt,
            radius: 50%,
            fill: sidebar_bg.lighten(10%),
            stroke: 0.7pt + accent.lighten(40%)
          )
        }
      )
      v(0.6em)
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
  show heading.where(level: 1): none

  set text(font: ("Arial", "Helvetica", "DejaVu Sans"), ligatures: false)
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
