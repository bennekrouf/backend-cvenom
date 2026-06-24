#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label, nonempty

// ── Palette ───────────────────────────────────────────────────────────────────
#let primary   = rgb("#0F172A")   // deep slate (fixed)
#let light_bg  = rgb("#F8FAFC")   // near-white page bg (fixed)
#let card_bg   = rgb("#FFFFFF")
#let rule_clr  = rgb("#E2E8F0")
// User-customizable: primary_color → accent, secondary_color → secondary
#let _u_accent  = sys.inputs.at("primary_color",   default: none)
#let _u_sec     = sys.inputs.at("secondary_color",  default: none)
#let accent     = if _u_accent != none { rgb(_u_accent) } else { rgb("#6366F1") }
#let accent2    = rgb("#EC4899")   // pink — second accent for tags (fixed)
#let secondary  = if _u_sec    != none { rgb(_u_sec)    } else { rgb("#64748B") }

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "projects":       "Projects",
      "skills":         "Technical Skills",
      "languages":      "Languages",
      "summary":        "About",
      "role":           "Role",
      "technologies":   "Tech",
      "highlights":     "Highlights",
      "view_project":   "View project",
      "portfolio_of":   "Portfolio of",
      "contact":        "Contact",
    ),
    "fr": (
      "projects":       "Projets",
      "skills":         "Compétences techniques",
      "languages":      "Langues",
      "summary":        "À propos",
      "role":           "Rôle",
      "technologies":   "Technologies",
      "highlights":     "Points clés",
      "view_project":   "Voir le projet",
      "portfolio_of":   "Portfolio de",
      "contact":        "Contact",
    ),
    "de": (
      "projects":       "Projekte",
      "skills":         "Technische Fähigkeiten",
      "languages":      "Sprachen",
      "summary":        "Über mich",
      "role":           "Rolle",
      "technologies":   "Technologien",
      "highlights":     "Highlights",
      "view_project":   "Projekt ansehen",
      "portfolio_of":   "Portfolio von",
      "contact":        "Kontakt",
    ),
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Tech badge ────────────────────────────────────────────────────────────────
#let tech_badge(label, color: none) = {
  if color == none { color = accent }
  box(
    inset: (x: 6pt, y: 3pt),
    radius: 10pt,
    fill: color.lighten(88%),
    stroke: 0.5pt + color.lighten(40%),
    text(size: 7.5pt, fill: color, weight: "medium", label)
  )
  h(3pt)
}

// ── Section heading ───────────────────────────────────────────────────────────
#let section(title) = {
  v(1.2em)
  grid(
    columns: (auto, 1fr),
    column-gutter: 10pt,
    align: horizon,
    text(size: 13pt, weight: "bold", fill: primary, upper(title)),
    line(stroke: 0.7pt + rule_clr),
  )
  v(0.6em)
}

// ── Project card ──────────────────────────────────────────────────────────────
#let project_card(
  title: "",
  role: none,
  date: none,
  description: none,
  technologies: (),
  highlights: (),
  url: none,
) = {
  block(
    width: 100%,
    radius: 8pt,
    stroke: 0.7pt + rule_clr,
    fill: card_bg,
    inset: (x: 16pt, top: 14pt, bottom: 14pt),
  )[
    // Title row
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      [
        #text(size: 12pt, weight: "bold", fill: primary, title)
        #if role != none [
          #h(8pt)
          #text(size: 9pt, fill: accent, style: "italic", role)
        ]
      ],
      if date != none {
        box(
          inset: (x: 8pt, y: 3pt),
          radius: 4pt,
          fill: light_bg,
          text(size: 8pt, fill: secondary, date)
        )
      },
    )

    // Description
    #if description != none {
      v(0.4em)
      text(size: 9.5pt, fill: secondary, description)
    }

    // Tech badges
    #if technologies.len() > 0 {
      v(0.5em)
      text(size: 7.5pt, fill: secondary, weight: "medium", upper(get_text("technologies")) + "  ")
      technologies.map(t => tech_badge(t)).join()
    }

    // Highlights
    #if highlights.len() > 0 {
      v(0.5em)
      for h in highlights {
        grid(
          columns: (8pt, 1fr),
          column-gutter: 4pt,
          align: (center + top, left),
          text(fill: accent, size: 9pt, "›"),
          text(size: 9pt, fill: primary, h),
        )
        v(0.15em)
      }
    }

    // URL
    #if url != none and url != "" {
      v(0.4em)
      text(size: 8pt, fill: accent, link(url, "↗ " + get_text("view_project") + ": " + url))
    }
  ]
  v(0.7em)
}

// ── Skills table ──────────────────────────────────────────────────────────────
#let show_skills(skills) = {
  if type(skills) == dictionary and skills.len() > 0 {
    let items = ()
    for (key, value) in skills.pairs() {
      if key != "" and value != none {
        if type(value) == array {
          let filtered = value.filter(v => v != "" and v != none)
          if filtered.len() > 0 {
            items.push(text(weight: "bold", size: 9pt, fill: primary, skill_label(key)))
            items.push(filtered.map(v => tech_badge(v, color: secondary)).join())
          }
        } else if type(value) == str and value != "" {
          items.push(text(weight: "bold", size: 9pt, fill: primary, skill_label(key)))
          items.push(text(size: 9pt, value))
        }
      }
    }
    if items.len() > 0 {
      table(
        columns: 2,
        column-gutter: 2%,
        row-gutter: 0.4em,
        align: (right, left),
        stroke: none,
        fill: (_, row) => if calc.odd(row) { light_bg } else { white },
        ..items,
      )
    }
  }
}

// ── Cover page ────────────────────────────────────────────────────────────────
#let show_cover(details) = {
  // Full-width indigo header band
  block(
    width: 100%,
    fill: primary,
    inset: (x: 1.5cm, top: 1.2cm, bottom: 1.0cm),
  )[
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      column-gutter: 1.5cm,
      [
        // Name
        #text(size: 28pt, weight: "bold", fill: white,
          details.at("name", default: ""))
        #v(0.3em)
        // Title
        #text(size: 13pt, fill: accent.lighten(50%),
          details.at("job_title", default: details.at("title", default: "")))
        #v(0.6em)
        // Contact line
        #set text(size: 8.5pt, fill: white.darken(20%))
        #let parts = ()
        #if details.at("email", default: "") != "" {
          parts.push(link("mailto:" + details.email,
            text(fill: accent.lighten(50%), details.email)))
        }
        #if details.at("phonenumber", default: "") != "" { parts.push(details.phonenumber) }
        #if details.at("address", default: "") != "" { parts.push(details.address) }
        #parts.join("  ·  ")
        // Links
        #if details.at("links", default: none) != none {
          let processed = process_links(details.links, color: accent.lighten(50%))
          if processed.len() > 0 { linebreak(); processed.join("  ·  ") }
        }
      ],
      // Brand logo + photo — same horizontal arrangement as enterprise2 so a
      // user generating a CV + portfolio pair sees consistent header framing.
      [
        #let _pic  = sys.inputs.at("picture", default: none)
        #let _logo = sys.inputs.at("company_logo.png", default: none)
        #let show_photo = details.at("styling", default: (:)).at("show_photo", default: true)
        #stack(
          dir: ltr,
          spacing: 14pt,
          if _logo != none {
            // No background — relies on a transparent-PNG logo to blend with
            // the dark slate header. If the logo's color matches the header,
            // upload a contrasting / white variant.
            box(
              height: 80pt,
              align(center + horizon,
                image(_logo, width: 80pt, height: 45pt, fit: "contain")),
            )
          },
          if _pic != none and show_photo {
            block(
              clip: true, radius: 50%, stroke: 2pt + accent.lighten(40%),
              image(_pic, width: 80pt, height: 80pt, fit: "cover"),
            )
          },
        )
      ]
    )
  ]

  // Accent stripe under header
  block(width: 100%, height: 4pt, fill: accent)

  pad(x: 1.5cm, top: 0.8cm, bottom: 0cm)[
    // Portfolio label
    #text(size: 9pt, fill: secondary, upper(get_text("portfolio_of") + " " + details.at("name", default: "")))
    #v(0.3em)

    // Summary / about
    #if details.at("summary", default: "") != "" {
      section(get_text("summary"))
      text(size: 10pt, fill: secondary, details.summary)
    }
  ]
}

// ── Main conf ─────────────────────────────────────────────────────────────────
#let conf(details, doc) = {
  show math.equation: set text(font: "Times")
  show link: set text(fill: accent)
  show "C++": box

  set text(font: ("Arial", "Helvetica", "Liberation Sans", "DejaVu Sans"), size: 10pt, ligatures: false)
  set par(justify: true, leading: 0.65em)
  set page(
    paper: "a4",
    fill: light_bg,
    margin: (top: 0cm, left: 0cm, right: 0cm, bottom: 1.2cm),
    footer: context [
      #pad(x: 1.5cm)[
        #line(length: 100%, stroke: 0.5pt + rule_clr)
        #v(0.2em)
        #grid(
          columns: (1fr, auto),
          text(size: 7.5pt, fill: secondary, details.at("name", default: "")),
          text(size: 7.5pt, fill: secondary, counter(page).display()),
        )
      ]
    ],
    footer-descent: 20pt,
  )

  show_cover(details)

  pad(x: 1.5cm, top: 0cm, bottom: 0cm)[
    #doc
  ]
}
