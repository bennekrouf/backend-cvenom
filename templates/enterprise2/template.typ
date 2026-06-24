#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label, nonempty

// ── Palette ───────────────────────────────────────────────────────────────────
// User-customizable via `sys.inputs` (see core/branding.rs). Every default
// here matches the literal value this template used historically — keys that
// the resolver doesn't forward fall through to these defaults, so legacy
// profiles render unchanged.
#let sidebar_bg  = rgb("#F3F0FF")   // light purple tint for sidebar (fixed)
#let light_rule  = rgb("#DDD8F5")   // subtle separator (fixed)
#let _u_primary  = sys.inputs.at("primary_color",   default: none)
#let _u_sec      = sys.inputs.at("secondary_color", default: none)
#let _u_accent   = sys.inputs.at("accent_color",    default: none)
#let primary     = if _u_primary != none { rgb(_u_primary) } else { rgb("#5236AB") }
#let accent      = if _u_accent  != none { rgb(_u_accent)  } else { rgb("#E31937") }
#let secondary   = if _u_sec     != none { rgb(_u_sec)     } else { rgb("#6B6B8A") }

#let sidebar_width = 6.5cm

#let default_font      = "Liberation Sans"
#let default_math_font = "Times"

// ── Language helpers ───────────────────────────────────────────────────────────
#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Technical Specializations",
      "certifications_education": "Education & Certifications",
      "languages":                "Languages",
      "work_experience":          "Professional Experience",
      "key_competencies":         "Qualifications",
      "sectors":                  "Industry Expertise",
      "summary":                  "Profile",
      "diplomas":                 "Diplomas",
      "certifications":           "Certifications",
      "areas_of_expertise":       "Areas of Expertise",
      "tools":                    "Tools & Software",
      "other_experience":         "Other Experience",
      "cgi_experience":           "CGI Experience",
    ),
    "fr": (
      "technical_skills":         "Spécialisations techniques",
      "certifications_education": "Formations & Certifications",
      "languages":                "Langues",
      "work_experience":          "Expérience professionnelle",
      "key_competencies":         "Qualifications",
      "sectors":                  "Expertise sectorielle",
      "summary":                  "Profil",
      "diplomas":                 "Diplômes",
      "certifications":           "Certifications",
      "areas_of_expertise":       "Domaines d'expertise",
      "tools":                    "Outils & Logiciels",
      "other_experience":         "Autres expériences",
      "cgi_experience":           "Expérience",
    ),
    "de": (
      "technical_skills":         "Technische Spezialisierungen",
      "certifications_education": "Bildung & Zertifizierungen",
      "languages":                "Sprachen",
      "work_experience":          "Berufserfahrung",
      "key_competencies":         "Qualifikationen",
      "sectors":                  "Branchenexpertise",
      "summary":                  "Profil",
      "diplomas":                 "Abschlüsse",
      "certifications":           "Zertifizierungen",
      "areas_of_expertise":       "Fachgebiete",
      "tools":                    "Werkzeuge & Software",
      "other_experience":         "Weitere Erfahrung",
      "cgi_experience":           "Erfahrung",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Sidebar section heading ────────────────────────────────────────────────────
#let sidebar_section(title) = {
  v(0.8em)
  block(width: 100%)[
    #text(size: 10pt, weight: "bold", fill: primary, upper(title))
    #v(0.1em)
    #line(length: 100%, stroke: 1pt + primary)
  ]
  v(0.25em)
}

// ── Main content section heading (purple band) ─────────────────────────────────
#let section(title) = {
  v(0.5em)
  block(
    width: 100%,
    fill: primary,
    inset: (x: 10pt, y: 5pt),
    radius: 2pt,
  )[
    #text(size: 10.5pt, weight: "bold", fill: white, upper(title))
  ]
  v(0.3em)
}

// ── Sub-section heading ────────────────────────────────────────────────────────
#let subsection(title, date_text: none) = {
  v(0.3em)
  grid(
    columns: (1fr, auto),
    align: (left + bottom, right + bottom),
    text(size: 10pt, weight: "bold", fill: primary, title),
    if date_text != none {
      text(size: 8.5pt, fill: secondary, style: "italic", date_text)
    },
  )
  line(length: 100%, stroke: 0.5pt + light_rule)
  v(0.2em)
}

// ── Competency tag ─────────────────────────────────────────────────────────────
#let competency_tag(label) = {
  box(
    inset: (x: 6pt, y: 3pt),
    radius: 2pt,
    fill: white,
    stroke: 0.5pt + primary,
    text(size: 8pt, fill: primary, label)
  )
  h(3pt)
  v(2pt)
}

#let show_competencies(items) = {
  items.map(i => competency_tag(i)).join()
}

// ── Sector badge ───────────────────────────────────────────────────────────────
#let sector_badge(label) = {
  box(
    inset: (x: 6pt, y: 3pt),
    radius: 2pt,
    fill: primary,
    text(size: 8pt, fill: white, weight: "bold", label)
  )
  h(3pt)
  v(3pt)
}

// ── Bullet list ────────────────────────────────────────────────────────────────
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none { color = accent }
  if symbol == none { symbol = sym.bullet }
  list(
    indent: 4pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 9pt, it)),
  )
}

#let date(color: none, content) = {
  if color == none { color = secondary }
  [#h(1fr) #text(weight: "regular", size: 8.5pt, fill: color, content)]
}

// ── Mission entry (main content area) ─────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  block(
    width: 100%,
    stroke: (left: 2.5pt + accent),
    inset: (left: 9pt, top: 4pt, bottom: 4pt, right: 4pt),
  )[
    #grid(
      columns: (1fr, auto),
      align: (left + top, right + top),
      [
        #if nonempty(title) [
          #text(size: 10.5pt, weight: "bold", fill: primary, title)
        ]
        #if company != none [
          #if nonempty(title) { linebreak() }
          #text(size: 9pt, fill: secondary, style: "italic", company)
        ]
      ],
      if date != none {
        box(
          inset: (x: 5pt, y: 2pt),
          radius: 2pt,
          fill: sidebar_bg,
          text(size: 8pt, fill: primary, date)
        )
      },
    )
    // Skip description when there's no role — without it, the description
    // block reads as a fake role beneath the company name.
    #if description != none and nonempty(title) {
      v(0.1em)
      text(size: 9pt, style: "italic", fill: secondary, description)
    }
    #if content != none {
      v(0.15em)
      content
    }
  ]
  v(0.4em)
}

// ── Skills table ───────────────────────────────────────────────────────────────
#let show_skills(separator: none, color: none, skills) = {
  if color == none { color = primary }
  if type(skills) == dictionary and skills.len() > 0 {
    let skills_array = ()
    for (key, value) in skills.pairs() {
      if key != "" and value != none {
        if type(value) == array {
          let filtered = value.filter(v => v != "" and v != none)
          if filtered.len() > 0 {
            skills_array.push(text(weight: "bold", size: 9pt, fill: primary, skill_label(key)))
            skills_array.push(filtered.map(box).join(text(fill: color, "  ·  ")))
          }
        } else if type(value) == str and value != "" {
          skills_array.push(text(weight: "bold", size: 9pt, fill: primary, skill_label(key)))
          skills_array.push(text(size: 9pt, value))
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
        fill: (_, row) => if calc.odd(row) { sidebar_bg } else { white },
        ..skills_array,
      )
    }
  }
}

// ── Header ─────────────────────────────────────────────────────────────────────
#let show_header(details) = {
  block(
    width: 100%,
    fill: primary,
    inset: (x: 1.2cm, y: 0.7cm),
    radius: (bottom-right: 6pt),
  )[
    #grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      column-gutter: 1em,
      [
        #text(size: 22pt, weight: "bold", fill: white,
          details.at("name", default: ""))
        #linebreak()
        #text(size: 11pt, fill: rgb("#D4CAFF"),
          details.at("job_title", default: details.at("title", default: "")))
        #v(0.4em)
        #set text(size: 8pt, fill: white.darken(15%))
        #let parts = ()
        #if details.at("email", default: "") != "" {
          parts.push(link("mailto:" + details.email,
            text(fill: rgb("#D4CAFF"), details.email)))
        }
        #if details.at("phonenumber", default: "") != "" {
          parts.push(details.phonenumber)
        }
        #if details.at("address", default: "") != "" {
          parts.push(details.address)
        }
        #parts.join("  ·  ")
        #if details.at("links", default: none) != none {
          let processed = process_links(details.links, color: rgb("#D4CAFF"))
          if processed.len() > 0 {
            linebreak()
            processed.join("  ·  ")
          }
        }
      ],
      [
        #let _pic  = sys.inputs.at("picture", default: none)
        #let _logo = sys.inputs.at("company_logo.png", default: none)
        #let show_photo = details.at("styling", default: (:)).at("show_photo", default: true)
        // Horizontal layout: logo to the left of the photo, both vertically
        // centered. A stacked (top/bottom) arrangement reads as two siblings
        // competing for the same slot; side-by-side keeps each at its own
        // visual role — corporate mark + person. Either may be absent; the
        // horizontal stack collapses around a missing element.
        #stack(
          dir: ltr,
          spacing: 14pt,
          if _logo != none {
            // No background — the logo sits directly on the primary header
            // band so a transparent-PNG mark blends with the rest of the
            // header. Sized smaller than the photo so the photo remains the
            // dominant element (people read faces faster than marks).
            box(
              height: 70pt,
              align(center + horizon,
                image(_logo, width: 70pt, height: 40pt, fit: "contain")),
            )
          },
          if _pic != none and show_photo {
            block(
              clip: true,
              radius: 50%,
              stroke: 2pt + white,
              image(_pic, width: 70pt, height: 70pt, fit: "cover"),
            )
          },
        )
      ]
    )
  ]
}

// ── Sidebar content builder ────────────────────────────────────────────────────
#let build_sidebar(details) = {
  pad(x: 14pt, y: 10pt)[
    // Years of experience callout
    #if details.at("summary", default: "") != "" {
      let sum = details.summary
      let yoe_match = sum.find(regex("\d+\s*years?"))
      if yoe_match != none {
        align(center)[
          #box(
            inset: (x: 10pt, y: 6pt),
            radius: 4pt,
            fill: primary,
          )[
            #text(size: 11pt, weight: "bold", fill: white, yoe_match)
            #linebreak()
            #text(size: 7.5pt, fill: rgb("#D4CAFF"), "of experience")
          ]
        ]
      }
    }

    // Industry / Sector Expertise
    #if details.at("sectors", default: none) != none {
      let sectors = details.sectors
      let domain_list = if type(sectors) == dictionary {
        sectors.at("domains", default: ())
      } else if type(sectors) == array {
        sectors
      } else { () }

      if domain_list.len() > 0 {
        sidebar_section(get_text("sectors"))
        domain_list.map(s => sector_badge(s)).join()
      }
    }

    // Key Competencies / Qualifications
    #if details.at("key_competencies", default: none) != none {
      sidebar_section(get_text("key_competencies"))
      for item in details.key_competencies {
        experience_details(color: primary, item)
      }
    }

    // Tools & Software
    #if details.at("tools", default: none) != none {
      sidebar_section(get_text("tools"))
      text(size: 8.5pt, fill: rgb("#333355"), details.tools)
    }

    // Languages
    #if "languages" in details {
      sidebar_section(get_text("languages"))
      let lvl = details.languages
      let lang_items = ()
      if "native"       in lvl { for l in lvl.native       { lang_items.push(l + " — Native") } }
      if "fluent"       in lvl { for l in lvl.fluent       { lang_items.push(l + " — Fluent") } }
      if "intermediate" in lvl { for l in lvl.intermediate { lang_items.push(l + " — Intermediate") } }
      if "basic"        in lvl { for l in lvl.basic        { lang_items.push(l + " — Basic") } }
      for l in lang_items {
        experience_details(color: primary, l)
      }
    }

    // Education & Certifications
    #if "education" in details {
      sidebar_section(get_text("certifications_education"))
      let diplomas = details.education.filter(item => item.at("type", default: "education") == "diploma")
      for item in diplomas {
        text(weight: "bold", size: 8.5pt, item.title)
        linebreak()
        text(size: 8pt, fill: secondary, item.date)
        v(0.2em)
      }
      let certs = details.education.filter(item => item.at("type", default: "education") != "diploma")
      if certs.len() > 0 {
        for item in certs {
          text(size: 8.5pt, item.title)
          linebreak()
          text(size: 8pt, fill: secondary, item.date)
          v(0.2em)
        }
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
  show link: set text(fill: primary)
  show list: set text(size: 9pt)
  show "C++": box
  show heading.where(level: 1): none

  set text(font: ("Arial", "Helvetica", "Liberation Sans", "DejaVu Sans"), ligatures: false, size: 9.5pt)
  set par(justify: true, leading: 0.65em)
  set page(
    paper: "a4",
    margin: (top: 0cm, left: 0cm, bottom: 1.0cm, right: 0cm),
    footer-descent: 0%,
    header-ascent: 0%,
    background: place(
      left + top,
      rect(width: sidebar_width, height: 100%, fill: sidebar_bg)
    ),
  )
  set list(indent: 4pt, marker: text(fill: accent, sym.bullet))

  show_header(details)

  // Two-column body: sidebar left, main content right
  grid(
    columns: (sidebar_width, 1fr),
    column-gutter: 0pt,
    build_sidebar(details),
    pad(x: 14pt, top: 8pt, bottom: 0pt)[
      #doc
    ]
  )
}
