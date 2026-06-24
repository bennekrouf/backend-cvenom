#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, join_dicts, get_default_icons, process_links, skill_label, nonempty

// ── Palette ───────────────────────────────────────────────────────────────────
// Conservative: dark navy + burgundy accents — conveys authority and tradition.
// User-customizable: primary_color → accent, secondary_color → secondary
#let _u_accent  = sys.inputs.at("primary_color",   default: none)
#let _u_sec     = sys.inputs.at("secondary_color",  default: none)
#let primary    = rgb("#1B2A4A")   // dark navy (fixed)
#let accent     = if _u_accent != none { rgb(_u_accent) } else { rgb("#7A2532") }
#let secondary  = if _u_sec    != none { rgb(_u_sec)    } else { rgb("#4A5568") }
#let rule_color = primary  // navy rules (follows primary)

#let default_font      = "Liberation Serif"
#let default_math_font = "Times"

// ── Language helpers ──────────────────────────────────────────────────────────
#let get_text(key) = {
  let lang = get_lang()
  let texts = (
    "en": (
      "technical_skills":         "Areas of Expertise",
      "certifications_education": "Education & Bar Admissions",
      "languages":                "Languages",
      "work_experience":          "Professional Experience",
      "practice_areas":           "Practice Areas",
      "publications":             "Publications & Presentations",
      "bar_admissions":           "Bar Admissions & Memberships",
      "summary":                  "Professional Profile",
    ),
    "fr": (
      "technical_skills":         "Domaines d'expertise",
      "certifications_education": "Formation & Inscriptions au Barreau",
      "languages":                "Langues",
      "work_experience":          "Parcours professionnel",
      "practice_areas":           "Domaines de pratique",
      "publications":             "Publications & Interventions",
      "bar_admissions":           "Inscriptions au Barreau & Affiliations",
      "summary":                  "Profil professionnel",
    ),
    "de": (
      "technical_skills":         "Fachgebiete",
      "certifications_education": "Ausbildung & Zulassungen",
      "languages":                "Sprachen",
      "work_experience":          "Berufserfahrung",
      "practice_areas":           "Rechtsgebiete",
      "publications":             "Publikationen & Vortraege",
      "bar_admissions":           "Zulassungen & Mitgliedschaften",
      "summary":                  "Berufsprofil",
    )
  )
  texts.at(lang, default: texts.en).at(key, default: key)
}

// ── Dividers ──────────────────────────────────────────────────────────────────
#let navy_rule() = {
  line(length: 100%, stroke: 0.8pt + primary)
}

#let thin_rule() = {
  line(length: 100%, stroke: 0.4pt + secondary.lighten(60%))
}

// ── Section heading — classic serif, navy with thin underline ─────────────────
#let section(title) = {
  v(0.7em)
  text(size: 11pt, weight: "bold", fill: primary, tracking: 0.5pt, upper(title))
  v(0.15em)
  navy_rule()
  v(0.35em)
}

// ── Bullet list — conservative square bullets ─────────────────────────────────
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none { color = accent }
  if symbol == none { symbol = sym.square.filled }
  list(
    indent: 5pt,
    marker: text(fill: color, size: 6pt, baseline: 1.5pt, symbol),
    ..args.pos().map(it => text(size: 9.5pt, it)),
  )
}

#let date(color: none, content) = {
  if color == none { color = secondary }
  [#h(1fr) #text(weight: "regular", size: 9pt, fill: color, content)]
}

// ── Experience entry ──────────────────────────────────────────────────────────
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  grid(
    columns: (1fr, auto),
    align: (left + top, right + top),
    [
      #if nonempty(title) [
        #text(size: 10.5pt, weight: "bold", fill: primary, title)
      ]
      #if company != none [
        #if nonempty(title) { linebreak() }
        #text(size: 9.5pt, fill: accent, style: "italic", company)
      ]
    ],
    text(size: 9pt, fill: secondary, date)
  )
  // Skip description when there's no role — without it, the description block
  // reads as a fake role beneath the company name.
  if description != none and nonempty(title) {
    v(0.2em)
    text(size: 9.5pt, style: "italic", fill: secondary, description)
  }
  if content != none {
    v(0.2em)
    content
  }
  v(0.6em)
}

// ── Practice area pills ───────────────────────────────────────────────────────
#let show_practice_areas(items) = {
  if type(items) == array and items.len() > 0 {
    let cells = items.map(item =>
      box(
        inset: (x: 8pt, y: 4pt),
        radius: 2pt,
        fill: primary.lighten(92%),
        stroke: 0.5pt + primary.lighten(70%),
        text(size: 9pt, fill: primary, item)
      )
    )
    // Wrap as inline flow
    for cell in cells {
      cell
      h(4pt)
    }
  }
}

// ── Skills grid ───────────────────────────────────────────────────────────────
#let show_skills(separator: none, color: none, skills) = {
  if color == none { color = accent }
  if type(skills) == dictionary and skills.len() > 0 {
    let items = ()
    for (cat, values) in skills.pairs() {
      if cat != "" and values != none {
        let val_str = if type(values) == array {
          values.filter(v => v != "").join(" · ")
        } else { str(values) }
        if val_str != "" {
          items.push(
            block(inset: (x: 4pt, y: 3pt))[
              #text(size: 9pt, weight: "bold", fill: primary, skill_label(cat) + ": ")
              #text(size: 9pt, fill: secondary, val_str)
            ]
          )
        }
      }
    }
    grid(
      columns: (1fr, 1fr),
      column-gutter: 1em,
      row-gutter: 0.05em,
      ..items
    )
  }
}

// ── Header — formal centered layout with thin lines ──────────────────────────
#let show_header(details) = {
  v(0.4em)

  // Optional photo
  let _pic = sys.inputs.at("picture", default: none)
  let show_photo = details.at("styling", default: (:)).at("show_photo", default: true)
  if _pic != none and show_photo {
    align(center,
      block(clip: true, radius: 3pt,
        image(_pic, width: 75pt, height: 90pt, fit: "cover")))
    v(0.3em)
  }

  align(center)[
    // Name — large serif
    #text(size: 24pt, weight: "bold", fill: primary, tracking: 1pt,
      upper(details.at("name", default: "")))
    #linebreak()
    #v(-0.2em)

    // Title — burgundy
    #text(size: 12pt, fill: accent, weight: "semibold",
      details.at("job_title", default:
        details.at("title", default: "")))
    #linebreak()
    #v(0.2em)

    // Double rule
    #navy_rule()
    #v(0.15em)

    // Contact line
    #set text(size: 9pt, fill: secondary)
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
    #parts.join(text(fill: secondary, "  |  "))

    // Links
    #if details.at("links", default: none) != none {
      let processed = process_links(details.links, color: accent)
      if processed.len() > 0 {
        linebreak()
        processed.join(text(fill: secondary, "  |  "))
      }
    }
  ]
  v(0.4em)
  navy_rule()
  v(0.3em)
}

// ── Main conf ─────────────────────────────────────────────────────────────────
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

  // Suppress the raw "= Work Experience" heading from experiences.typ —
  // main.typ provides its own translated section heading via get_text().
  show heading.where(level: 1): none

  // Serif-forward: Georgia / Palatino / fallback to Liberation Serif
  set text(font: ("Georgia", "Palatino Linotype", "Liberation Serif"), ligatures: false)
  set par(justify: true)
  set page(
    margin: (top: 1.5cm, left: 2.2cm, bottom: 1.5cm, right: 2.2cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set list(indent: 5pt, marker: text(fill: accent, size: 6pt, baseline: 1.5pt, sym.square.filled))

  show_header(details)
  doc
}
