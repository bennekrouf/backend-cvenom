// enterprise3 — CGI corporate CV, 2026 format.
//
// Page 1: header (name, title, tagline, key figures, red corner mark), purple
// sidebar (in brief, languages, certifications, education) and a main column
// (why <first name>, expertise, key skills). Page 2: sectors and achievements
// when the profile has them. Then the missions and an optional skills matrix.
//
// Everything beyond the standard profile (name, title, summary, skills,
// languages, education, experiences) is optional in cv_params.toml:
//   tagline = "…"                       line under the title
//   availability, mobility = "…"        "In brief" block
//   [[key_figures]] value/label         the three figures under the header
//   key_competencies = ["…"]            "Why <first name>" bullets
//   [[areas_of_expertise]] title/description (or "Title: description")
//   key_skills = ["…"]                  the pills ("Key skills")
//   [[certifications]] title/date/issuer
//   sectors = ["…"], achievements = ["…"]
//   [[skills_matrix]] category + [[skills_matrix.items]] name/years/level (1-4)
//   company_name, company_tagline       footer ("CGI Suisse", "Insights you can act on")
// Without them the template falls back on the summary and the skills.
//
// The palette is CGI's and stays fixed: a profile's own colours would turn the
// brand template into something else.

#import "font_config.typ": font_config, get_icon
#import "common.typ": get_lang, skill_label, nonempty

// ── Palette (from the CGI Word template) ──────────────────────────────────────
#let navy      = rgb("#200A58")
#let red       = rgb("#E31937")
#let purple    = rgb("#5236AB")
#let gray      = rgb("#6B6880")
#let ink       = rgb("#1F1F1F")
#let lavender  = rgb("#E6E3F3")
#let lav_soft  = rgb("#F5F3FB")
#let lav_line  = rgb("#CBC3E6")
#let lav_text  = rgb("#D9D1F5")
#let rule_gray = rgb("#E4E2EC")

// Keep the historical names other templates export, for any shared caller.
#let primary   = purple
#let secondary = gray
#let accent    = red

// ── Geometry (A4) ─────────────────────────────────────────────────────────────
#let page_h    = 29.7cm
#let page_w    = 21cm
#let margin_x  = 1.8cm
#let margin_t  = 2.4cm
#let sidebar_w = 6.6cm      // purple band on page 1
#let header_h  = 7.2cm      // where the band starts
#let main_x    = 7.6cm      // left edge of page 1's main column

// ── Texts ─────────────────────────────────────────────────────────────────────
#let get_text(key) = {
  let texts = (
    "en": (
      "why": "Why", "expertise": "Expertise", "key_skills": "Key skills",
      "sectors": "Sectors", "achievements": "Achievements",
      "work_experience": "Professional experience", "skills_matrix": "Skills summary",
      "in_brief": "In brief", "availability": "Availability", "mobility": "Mobility",
      "languages": "Languages", "certifications": "Training and certifications",
      "education": "Education", "context": "Context", "tech_env": "Technical environment",
      "today": "Present", "years": "years", "year": "year",
      "years_exp": "of IT experience", "residence": "Place of residence",
      "cv": "Curriculum Vitae", "confidential": "Confidential", "page": "Page",
      "col_skill": "Skill", "col_years": "Years", "col_level": "Level",
      "legend": "Skill level:  1 = basic   ·   2 = intermediate   ·   3 = advanced   ·   4 = expert",
      "summary": "Profile",
    ),
    "fr": (
      "why": "Pourquoi", "expertise": "Expertise", "key_skills": "Compétences clés",
      "sectors": "Secteurs", "achievements": "Réalisations",
      "work_experience": "Expériences professionnelles", "skills_matrix": "Résumé des compétences",
      "in_brief": "En bref", "availability": "Disponibilité", "mobility": "Mobilité",
      "languages": "Langues", "certifications": "Perfectionnement et certifications",
      "education": "Formation", "context": "Contexte", "tech_env": "Environnement technique",
      "today": "Aujourd’hui", "years": "ans", "year": "an",
      "years_exp": "d'expérience en IT", "residence": "Lieu de résidence",
      "cv": "Curriculum Vitae", "confidential": "Confidentiel", "page": "Page",
      "col_skill": "Compétence", "col_years": "Années", "col_level": "Niveau",
      "legend": "Niveau de compétence :  1 = de base   ·   2 = intermédiaire   ·   3 = avancé   ·   4 = expert",
      "summary": "Profil",
    ),
    "de": (
      "why": "Warum", "expertise": "Expertise", "key_skills": "Kernkompetenzen",
      "sectors": "Branchen", "achievements": "Erfolge",
      "work_experience": "Berufserfahrung", "skills_matrix": "Kompetenzübersicht",
      "in_brief": "Auf einen Blick", "availability": "Verfügbarkeit", "mobility": "Mobilität",
      "languages": "Sprachen", "certifications": "Weiterbildung und Zertifizierungen",
      "education": "Ausbildung", "context": "Kontext", "tech_env": "Technische Umgebung",
      "today": "Heute", "years": "Jahre", "year": "Jahr",
      "years_exp": "IT-Erfahrung", "residence": "Wohnort",
      "cv": "Lebenslauf", "confidential": "Vertraulich", "page": "Seite",
      "col_skill": "Kompetenz", "col_years": "Jahre", "col_level": "Niveau",
      "legend": "Kompetenzniveau:  1 = Grundkenntnisse   ·   2 = fortgeschritten   ·   3 = sehr gut   ·   4 = Experte",
      "summary": "Profil",
    ),
  )
  texts.at(get_lang(), default: texts.en).at(key, default: key)
}

// ── Small helpers ─────────────────────────────────────────────────────────────
#let field(details, key) = {
  let v = details.at(key, default: none)
  if nonempty(v) { v } else { none }
}

#let as_list(v) = {
  if v == none { () }
  else if type(v) == array { v.filter(x => nonempty(x)) }
  else if type(v) == dictionary { v.at("domains", default: v.at("items", default: ())) }
  else if type(v) == str and v.trim() != "" { (v,) }
  else { () }
}

// Plain text of heading content ("== Concreet & eJust").
#let plain(c) = {
  if type(c) == str { c }
  else if c.has("text") { c.text }
  else if c.has("children") { c.children.map(plain).join() }
  else if c.has("body") { plain(c.body) }
  else if c == [ ] { " " }
  else if c.func() == smartquote { if c.double { "\"" } else { "'" } }
  else { "" }
}

// "Mohamed Bennekrouf" → "Mohamed BENNEKROUF", as CGI writes names.
#let display_name(name) = {
  let parts = name.trim().split(" ").filter(p => p != "")
  if parts.len() < 2 { name }
  else { parts.slice(0, -1).join(" ") + " " + upper(parts.last()) }
}

#let first_name(name) = name.trim().split(" ").first()

// "09/2025 - Present" → "09.2025 – Aujourd’hui".
#let fmt_date(d) = {
  if not nonempty(d) { return "" }
  let current = ("present", "présent", "heute", "aujourd'hui", "aujourd’hui", "current", "now", "")
  d.split(regex("\s+[-–—]\s+|\s*[-–—]\s*$")).map(p => {
    let p = p.trim()
    if lower(p) in current { get_text("today") } else { p.replace("/", ".") }
  }).filter(p => p != "").join(" – ")
}

// ── Decorations ───────────────────────────────────────────────────────────────
// The red corner mark at the top right of page 1.
#let corner_mark = polygon(
  fill: red,
  (0cm, 0cm), (3.6cm, 0cm), (3.6cm, 3.6cm), (2.1cm, 3.6cm), (2.1cm, 1.5cm), (0cm, 1.5cm),
)

// The purple sidebar of page 1: a vertical gradient with two diagonal bands.
#let sidebar_band = {
  let h = page_h - header_h
  block(
    width: sidebar_w, height: h, clip: true,
    radius: (top-right: 0.75cm),
    fill: gradient.linear(rgb("#5236AB"), rgb("#2A145F"), angle: 90deg),
  )[
    #place(top + left, polygon(
      fill: white.transparentize(93%),
      (0cm, h * 0.86), (sidebar_w, h * 0.46), (sidebar_w, h * 0.56), (0cm, h * 0.97),
    ))
    #place(top + left, polygon(
      fill: black.transparentize(88%),
      (0cm, h * 0.97), (sidebar_w, h * 0.56), (sidebar_w, h * 0.62), (0cm, h * 1.06),
    ))
  ]
}

// An outlined star, the icon in front of each area of expertise.
#let star_icon = {
  let pts = range(10).map(i => {
    let r = if calc.even(i) { 5.2pt } else { 2.3pt }
    let a = -90deg + i * 36deg
    (5.5pt + r * calc.cos(a), 5.5pt + r * calc.sin(a))
  })
  box(width: 11pt, height: 11pt, polygon(stroke: 0.8pt + purple, ..pts))
}

#let level_dots(level, of: 4, on: red, off: rgb("#CFCFD6"), size: 4.2pt) = {
  let n = calc.max(0, calc.min(of, int(level)))
  range(of).map(i => box(circle(radius: size / 2, fill: if i < n { on } else { off }))).join(h(1.6pt))
}

// ── Section headings ──────────────────────────────────────────────────────────
// Main section title: navy capitals over a thin rule.
#let section(title) = {
  v(0.9em, weak: true)
  block(sticky: true, width: 100%, below: 0.75em)[
    #text(size: 15pt, weight: "bold", fill: navy, upper(title))
    #v(-0.45em)
    #line(length: 100%, stroke: 0.6pt + rule_gray)
  ]
}

// Sidebar title: spaced white capitals over a light rule.
#let sidebar_section(title) = {
  v(2.6em, weak: true)
  block(width: 100%, below: 0.8em)[
    #text(size: 9pt, weight: "bold", fill: white, tracking: 1.6pt, upper(title))
    #v(-0.5em)
    #line(length: 100%, stroke: 0.5pt + lav_line)
  ]
}

// ── Experiences (called by experiences.typ) ───────────────────────────────────
// `== Company` comes before each dated_experience: keep it for the entry line.
#let _company = state("enterprise3-company", none)

#let _env_prefix = regex("^(?i)(environnement technique|environnement|technical environment|tech(nical)? stack|technologies|stack|technische umgebung)\s*:\s*")

#let tech_env_box(items) = {
  v(1em)
  block(width: 100%, fill: lav_soft, inset: (x: 0pt, y: 3pt), outset: (right: 0pt))[
    #text(size: 9.5pt, weight: "bold", fill: purple, get_text("tech_env"))#text(size: 9.5pt, fill: ink)[ :  #items]
  ]
}

// Bullets with CGI's red marker. A bullet starting with "Environnement
// technique :" (or "Technologies:", "Tech stack:"…) becomes the shaded
// technical-environment line instead.
#let experience_details(color: none, symbol: none, ..args) = {
  let marker_color = if color == none { red } else { color }
  let items = args.pos()
  let bullets = ()
  let env = none
  for it in items {
    if type(it) == str and it.trim().find(_env_prefix) != none {
      env = it.trim().replace(_env_prefix, "")
    } else {
      bullets.push(it)
    }
  }
  if bullets.len() > 0 {
    list(
      indent: 2pt, body-indent: 10pt, spacing: 0.75em,
      marker: text(fill: marker_color, size: 9pt, baseline: -0.5pt, sym.bullet),
      ..bullets.map(b => text(size: 10pt, fill: ink, b)),
    )
  }
  if env != none { tech_env_box(env) }
}

#let date(color: none, content) = {
  [#h(1fr) #text(size: 10.5pt, weight: "bold", fill: if color == none { purple } else { color }, content)]
}

#let dated_experience(title, date: none, description: none, content: none, company: none) = context {
  let comp = if company != none { company } else { _company.get() }
  let comp = if comp == none { "" } else { plain(comp) }
  // "CGI – Mission Oryx · Suisse": the client or mission part is set apart.
  let parts = comp.split(regex("\s+[–—-]\s+"))
  let org = parts.first()
  let rest = if parts.len() > 1 { parts.slice(1).join(" – ") } else { "" }

  v(2.2em, weak: true)
  block(width: 100%, breakable: true)[
    #block(sticky: true, below: 0.9em)[
      #grid(
        columns: (1fr, auto),
        align: (left + bottom, right + bottom),
        column-gutter: 1em,
        [
          #text(size: 11pt, weight: "bold", fill: navy, org)
          #if rest != "" [
            #text(size: 11pt, weight: "bold", fill: navy, [ –])
            #text(size: 10pt, style: "italic", fill: gray, rest)
          ]
        ],
        if nonempty(date) { text(size: 11pt, weight: "bold", fill: purple, fmt_date(date)) },
      )
    ]
    #if nonempty(title) {
      block(sticky: true, above: 0.9em, below: 0.9em,
        text(size: 11pt, weight: "bold", fill: navy, title))
    }
    #if nonempty(description) {
      block(above: 0.9em, below: 0.8em, par(justify: true,
        text(size: 10pt, fill: ink)[#get_text("context") – #description]))
    }
    #if content != none { content }
  ]
}

// ── Page 1: header ────────────────────────────────────────────────────────────
#let key_figures(details) = {
  let figures = details.at("key_figures", default: ())
  if type(figures) == array and figures.len() > 0 {
    return figures.filter(f => type(f) == dictionary and nonempty(f.at("value", default: none)))
      .map(f => (value: f.value, label: f.at("label", default: "")))
  }
  // Fallback: years of experience from the summary, and the country lived in.
  let out = ()
  let summary = details.at("summary", default: "")
  if nonempty(summary) {
    let m = summary.match(regex("(\d+\+?)\s*(years|year|ans|an|Jahre)"))
    if m != none {
      out.push((value: m.captures.first() + " " + get_text("years"), label: get_text("years_exp")))
    }
  }
  let address = details.at("address", default: "")
  if nonempty(address) {
    out.push((value: address.split(",").last().trim(), label: get_text("residence")))
  }
  out
}

#let show_header(details) = {
  let name = details.at("name", default: "")
  let title = details.at("title", default: details.at("job_title", default: ""))
  let tagline = field(details, "tagline")
  let tagline = if tagline != none { tagline } else {
    // Fallback: the profile's own skill headings, as CGI's subtitle lists them.
    let tech = details.at("skills", default: (:)).at("technical", default: ())
    if type(tech) == array and tech.len() > 0 { tech.slice(0, calc.min(4, tech.len())).join(" · ") } else { none }
  }
  let pic = sys.inputs.at("picture", default: none)
  let show_photo = details.at("styling", default: (:)).at("show_photo", default: true)

  block(width: 100%, height: header_h - margin_t, spacing: 0pt)[
    // Photo on the left, above the sidebar.
    #if pic != none and show_photo {
      place(top + left, dy: 0.7cm, block(
        clip: true, radius: 10pt, width: 3.4cm, height: 3.4cm,
        image(pic, width: 3.4cm, height: 3.4cm, fit: "cover"),
      ))
    }
    #place(top + left, dx: main_x - margin_x - 1.2cm, dy: 0.75cm, block(width: 12.6cm)[
      #set par(justify: false, leading: 0.4em)
      // Long names shrink rather than wrap under the red mark.
      #let n = display_name(name)
      #let size = if n.len() > 24 { 21pt } else if n.len() > 20 { 24pt } else { 26pt }
      #text(size: size, weight: "bold", fill: navy, n)
      #if nonempty(title) {
        v(0.75em, weak: true)
        text(size: 15pt, weight: "bold", fill: navy, title)
      }
      #if tagline != none {
        v(0.75em, weak: true)
        text(size: 11.5pt, fill: gray, tagline)
      }
      #let figures = key_figures(details)
      #if figures.len() > 0 {
        v(1.3em, weak: true)
        grid(
          columns: figures.slice(0, calc.min(3, figures.len())).map(_ => 4.15cm),
          row-gutter: 0.3em,
          ..figures.slice(0, calc.min(3, figures.len())).map(f => [
            #text(size: 16pt, weight: "bold", fill: purple, f.value) \
            #v(-0.35em) #text(size: 9.5pt, fill: gray, f.label)
          ]),
        )
      }
    ])
  ]
}

// ── Page 1: sidebar ───────────────────────────────────────────────────────────
#let language_rows(details) = {
  let lv = details.at("languages", default: (:))
  if type(lv) != dictionary { return () }
  let levels = (("native", 5, "C2"), ("fluent", 4, "C1"), ("intermediate", 3, "B1"), ("basic", 2, "A2"))
  let rows = ()
  for (key, dots, cefr) in levels {
    for l in as_list(lv.at(key, default: ())) { rows.push((l, dots, cefr)) }
  }
  rows
}

#let certifications(details) = {
  let certs = ()
  for c in as_list(details.at("certifications", default: ())) {
    if type(c) == str { certs.push((title: c, date: "", issuer: "")) }
    else {
      certs.push((
        title: c.at("title", default: c.at("name", default: "")),
        date: str(c.at("date", default: c.at("year", default: ""))),
        issuer: c.at("issuer", default: c.at("organization", default: "")),
      ))
    }
  }
  for e in as_list(details.at("education", default: ())) {
    if type(e) == dictionary and e.at("type", default: "") in ("certification", "cert", "training") {
      certs.push((title: e.at("title", default: ""), date: e.at("date", default: ""), issuer: e.at("location", default: "")))
    }
  }
  certs
}

#let build_sidebar(details) = {
  set text(fill: white, size: 9.5pt)
  set par(justify: false, leading: 0.5em)

  // In brief
  let availability = field(details, "availability")
  let mobility = field(details, "mobility")
  if availability != none or mobility != none {
    sidebar_section(get_text("in_brief"))
    for (label, value) in ((get_text("availability"), availability), (get_text("mobility"), mobility)) {
      if value != none {
        block(below: 0.7em)[
          #text(weight: "bold", size: 10pt, label) \
          #text(fill: lav_text, value)
        ]
      }
    }
  }

  // Languages
  let langs = language_rows(details)
  if langs.len() > 0 {
    sidebar_section(get_text("languages"))
    grid(
      columns: (1fr, auto, 1.2cm),
      row-gutter: 0.75em, column-gutter: 0.5em,
      align: (left + horizon, left + horizon, center + horizon),
      ..langs.map(((name, dots, cefr)) => (
        text(weight: "bold", size: 10pt, name),
        level_dots(dots, of: 5, on: white, off: white.transparentize(55%), size: 4.6pt),
        text(size: 9.5pt, cefr),
      )).flatten(),
    )
  }

  // Certifications
  let certs = certifications(details)
  if certs.len() > 0 {
    sidebar_section(get_text("certifications"))
    for c in certs.slice(0, calc.min(6, certs.len())) {
      block(below: 0.95em)[
        #text(weight: "bold", size: 10pt)[#if nonempty(c.date) [#c.date · ]#c.title]
        #if nonempty(c.issuer) [
          \ #h(2pt)#text(size: 8pt, sym.bullet)#h(8pt)#text(size: 10pt, c.issuer)
        ]
      ]
    }
  }

  // Education
  let diplomas = as_list(details.at("education", default: ())).filter(e =>
    type(e) != dictionary or not (e.at("type", default: "") in ("certification", "cert", "training")))
  if diplomas.len() > 0 {
    sidebar_section(get_text("education"))
    for e in diplomas.slice(0, calc.min(3, diplomas.len())) {
      let title = if type(e) == str { e } else { e.at("title", default: "") }
      let parts = title.split(regex("\s+[–—-]\s+"))
      let degree = parts.first()
      let school = parts.slice(1).join(" – ")
      let meta = if type(e) == dictionary {
        (e.at("location", default: ""), e.at("date", default: "")).filter(nonempty).join(" · ")
      } else { "" }
      block(below: 1em)[
        #text(weight: "bold", size: 10pt, upper(degree))
        #if school != "" [ \ #text(size: 9.5pt, fill: lav_text, upper(school))]
        #if nonempty(meta) [ \ #text(size: 9.5pt, fill: lav_text, meta)]
      ]
    }
  }
}

// ── Page 1: main column ───────────────────────────────────────────────────────
#let why_items(details) = {
  let items = as_list(details.at("key_competencies", default: ()))
  if items.len() > 0 { return items.slice(0, calc.min(6, items.len())) }
  // Fallback: the summary, a sentence per bullet.
  let summary = details.at("summary", default: "")
  if not nonempty(summary) { return () }
  let sentences = summary.split(". ").map(s => s.trim()).filter(s => s != "")
  let out = sentences.map(s => if s.ends-with(".") { s } else { s + "." })
  out.slice(0, calc.min(6, out.len()))
}

#let expertise_items(details) = {
  let areas = as_list(details.at("areas_of_expertise", default: ()))
  if areas.len() > 0 {
    return areas.slice(0, calc.min(4, areas.len())).map(a => {
      if type(a) == dictionary {
        (title: a.at("title", default: ""), description: a.at("description", default: ""))
      } else {
        let i = a.position(":")
        if i == none { (title: a, description: "") }
        else { (title: a.slice(0, i).trim(), description: a.slice(i + 1).trim()) }
      }
    })
  }
  // Fallback: the skill groups.
  let skills = details.at("skills", default: (:))
  if type(skills) != dictionary { return () }
  let out = ()
  for (key, values) in skills.pairs() {
    let vals = as_list(values)
    if key != "technical" and vals.len() > 0 {
      out.push((title: skill_label(key), description: vals.join(", ") + "."))
    }
  }
  out.slice(0, calc.min(4, out.len()))
}

#let key_skill_items(details) = {
  let items = as_list(details.at("key_skills", default: ()))
  if items.len() == 0 {
    let skills = details.at("skills", default: (:))
    if type(skills) == dictionary {
      for key in ("programming_languages", "frameworks", "tools") {
        items += as_list(skills.at(key, default: ()))
      }
      if items.len() == 0 { for (_, v) in skills.pairs() { items += as_list(v) } }
    }
  }
  items.slice(0, calc.min(9, items.len()))
}

#let pill(label, dark: false, outlined: false) = box(
  inset: (x: 17pt, y: 7.5pt),
  radius: 3pt,
  fill: if outlined { white } else if dark { navy } else { lavender },
  stroke: if outlined { 0.8pt + lav_line } else { none },
  text(size: 10pt, fill: if dark { white } else { ink }, label),
)

#let pills(items, outlined: false) = {
  set par(leading: 1.6em, justify: false)
  items.enumerate().map(((i, s)) =>
    pill(s, dark: not outlined and calc.rem(i, 3) == 0, outlined: outlined)
  ).join(h(8pt))
}

#let build_main(details) = {
  set par(justify: false)
  let why = why_items(details)
  if why.len() > 0 {
    section(get_text("why") + " " + first_name(details.at("name", default: "")))
    list(
      indent: 0pt, body-indent: 16pt, spacing: 0.85em,
      marker: text(fill: ink, size: 9pt, baseline: -0.5pt, sym.bullet),
      ..why.map(w => text(size: 10.5pt, fill: ink, w)),
    )
  }

  let areas = expertise_items(details)
  if areas.len() > 0 {
    v(1.2em)
    section(get_text("expertise"))
    for a in areas {
      grid(
        columns: (16pt, 1fr),
        column-gutter: 4pt,
        align: (left + top, left + top),
        pad(top: 1pt, star_icon),
        [
          #text(size: 11pt, weight: "bold", fill: navy, a.title)
          #if nonempty(a.description) [ \ #text(size: 10pt, fill: gray, a.description)]
        ],
      )
      v(0.45em)
    }
  }

  let ks = key_skill_items(details)
  if ks.len() > 0 {
    v(0.9em)
    section(get_text("key_skills"))
    pills(ks)
  }
}

// ── Page 2: sectors and achievements ──────────────────────────────────────────
#let _sectors(details) = as_list(details.at("sectors", default: ()))
#let _achievements(details) = as_list(details.at("achievements", default: details.at("key_achievements", default: ())))
#let has_page_two(details) = _sectors(details).len() > 0 or _achievements(details).len() > 0

#let build_page_two(details) = {
  let sectors = _sectors(details)
  let achievements = _achievements(details)
  let sectors_block = if sectors.len() > 0 [
    #section(get_text("sectors"))
    #pills(sectors, outlined: true)
  ]
  let achievements_block = if achievements.len() > 0 [
    #section(get_text("achievements"))
    #experience_details(..achievements)
  ]
  if sectors.len() > 0 and achievements.len() > 0 {
    grid(columns: (1fr, 1fr), column-gutter: 1.2cm, sectors_block, achievements_block)
  } else {
    if sectors.len() > 0 { sectors_block } else { achievements_block }
  }
}

// ── Skills matrix (last page) ─────────────────────────────────────────────────
#let show_skills_matrix(details) = {
  let groups = as_list(details.at("skills_matrix", default: ()))
  if groups.len() == 0 { return }
  pagebreak(weak: true)
  section(get_text("skills_matrix"))
  v(0.8em)
  let cells = ()
  for (gi, g) in groups.enumerate() {
    if gi > 0 { cells.push(table.cell(colspan: 3, stroke: none, inset: 0pt, v(0.9em))) }
    cells.push(table.cell(colspan: 3, fill: lavender, inset: (x: 8pt, y: 8pt),
      text(size: 10pt, weight: "bold", fill: purple, upper(g.at("category", default: "")))))
    for it in g.at("items", default: ()) {
      let years = it.at("years", default: "")
      let years = if type(years) == int {
        str(years) + " " + if years == 1 { get_text("year") } else { get_text("years") }
      } else { str(years) }
      cells.push(text(size: 9.5pt, fill: ink, it.at("name", default: "")))
      cells.push(text(size: 9.5pt, weight: "bold", fill: ink, years))
      cells.push(level_dots(it.at("level", default: 0)))
    }
  }
  table(
    columns: (1fr, 2.6cm, 3.2cm),
    align: (left + horizon, center + horizon, center + horizon),
    inset: (x: 8pt, y: 6.5pt),
    stroke: (x, y) => if y > 0 { (bottom: 0.5pt + rgb("#EEECF4")) },
    table.header(
      ..(get_text("col_skill"), get_text("col_years"), get_text("col_level")).enumerate().map(((i, h)) =>
        table.cell(fill: navy, inset: (x: 8pt, y: 8pt), align: if i == 0 { left + horizon } else { center + horizon },
          text(size: 10pt, weight: "bold", fill: white, upper(h)))),
    ),
    ..cells,
  )
  v(1.2em)
  text(size: 9pt, style: "italic", fill: gray, get_text("legend"))
}

// ── Running header and footer ─────────────────────────────────────────────────
#let logo_image(height) = {
  let logo = sys.inputs.at("company_logo.png", default: none)
  if logo != none { image(logo, height: height, fit: "contain") }
}

#let running_header(details) = {
  let name = details.at("name", default: "")
  let title = details.at("title", default: details.at("job_title", default: ""))
  set par(justify: false)
  block(width: 100%)[
    #grid(
      columns: (auto, 1fr),
      column-gutter: 10pt,
      align: (left + bottom, left + bottom),
      logo_image(0.75cm),
      text(size: 9pt, fill: gray, (display_name(name), title).filter(nonempty).join("  ·  ")),
    )
    #v(-0.3em)
    #line(length: 100%, stroke: 0.5pt + rule_gray)
  ]
}

#let running_footer(details) = context {
  let n = here().page()
  set text(size: 8.5pt, fill: gray)
  if n == 1 {
    let tagline = field(details, "company_tagline")
    align(right, box(
      grid(columns: 2, column-gutter: 8pt, align: horizon,
        logo_image(0.42cm),
        if tagline != none { text(size: 8pt, fill: rgb("#555555"), tagline) }),
    ))
  } else {
    let company = field(details, "company_name")
    grid(
      columns: (1fr, auto),
      (company, get_text("cv"), get_text("confidential")).filter(x => x != none).join("  ·  "),
      [#get_text("page") #n / #counter(page).final().first()],
    )
  }
}

// ── Document ──────────────────────────────────────────────────────────────────
#let conf(details, doc) = {
  set text(font: ("Arial", "Helvetica", "Liberation Sans", "DejaVu Sans"), lang: get_lang(), size: 10pt, fill: ink, ligatures: false)
  set par(justify: false, leading: 0.55em, spacing: 0.9em)
  show link: set text(fill: purple)
  show "C++": box
  show heading.where(level: 1): none
  show heading.where(level: 2): it => _company.update(it.body)
  show heading.where(level: 3): it => _company.update(it.body)

  set page(
    paper: "a4",
    margin: (top: margin_t, bottom: 2cm, x: margin_x),
    header: context { if here().page() > 1 { running_header(details) } },
    header-ascent: 35%,
    footer: running_footer(details),
    footer-descent: 40%,
    background: context {
      if here().page() == 1 {
        place(top + left, dx: page_w - 1.8cm - 3.6cm + 0.6cm, dy: 0.92cm, corner_mark)
        place(top + left, dy: header_h, sidebar_band)
      }
    },
  )

  // Page 1
  show_header(details)
  grid(
    columns: (sidebar_w - margin_x - 0.45cm, main_x - sidebar_w + 0.45cm, 1fr),
    pad(top: 1.4cm, build_sidebar(details)),
    [],
    pad(top: 0.9cm, build_main(details)),
  )

  // Page 2 (only when the profile has sectors or achievements)
  pagebreak()
  if has_page_two(details) {
    build_page_two(details)
    pagebreak()
  }

  doc
}
