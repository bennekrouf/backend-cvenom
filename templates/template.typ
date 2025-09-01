// global variables
#let default_primary_color = rgb("#14A4E6")
#let default_secondary_color = rgb("#757575")
#let default_link_color = rgb("#14A4E6")
#let default_font = "Carlito"
#let default_math_font = "Times"
#let default_separator = text(
  // this is because in some fonts (notably computer modern), it shows the
  // vertical line as a horizontal one
  font: "Carlito",
  " \u{007c} ",
)

// dictionary of common icons and values
#let get_default_icons(color: none) = {
  if color == none {
    color = default_primary_color
  }
  (
    "github": ("displayname": "GitHub", "logo": text(font: "Font Awesome 6 Brands", "\u{f09b}")),
    "linkedin": (
      "displayname": "LinkedIn",
      "logo": text(font: "Font Awesome 6 Brands", "\u{f08c}"),
    ),
    "personal": (
      "displayname": "Personal",
      "logo": text(font: "Font Awesome 6 Free Solid", "\u{f268}"),
    ),
    // annoyingly, Debian does not ship a version of FontAwesome which supports
    // the ORCID logo, hence here I draw my own approximation of it using Typst
    //  primitives
    "orcid": ("displayname": "ORCID", "logo": box(baseline: 0.2em, circle(
      radius: 0.5em,
      fill: color,
      inset: 0pt,
      align(center + horizon, text(size: 0.8em, fill: white, "iD")),
    ))),
  )
}

/* join dictionaries (kind of like Python's {**a, **b}) */
#let join_dicts(..args) = {
  let result = (:)
  for arg in args.pos() {
    for (key, value) in arg.pairs(){
      result.insert(key, value)
    }
  }
  result
}

/* function that applies a color to a link */
#let colorlink(color: none, url, body) = {
  if color == none {
    color = default_link_color
  }
  text(fill: color, link(url)[#body<colorlink>])
}

/* function that processes links */
#let process_links(color: none, icons: none, links) = {
  if icons == none {
    icons = get_default_icons(color: color)
  } else {
    // if the user supplies a custom dictionary, update the default one
    icons = join_dicts(get_default_icons(color: color), icons)
  }
  
  // Handle both array and dictionary formats for links
  let link_pairs = ()
  
  if type(links) == "array" {
    // If links is an array, treat each item as a URL with default icon
    for link in links {
      if link != "" and link != none {
        link_pairs.push(("personal", link))
      }
    }
  } else if type(links) == "dictionary" {
    // If links is a dictionary, filter out empty/none values
    for (key, value) in links.pairs() {
      if value != "" and value != none and type(value) == "string" {
        link_pairs.push((key, value))
      }
    }
  }
  
  // Only process non-empty links
  if link_pairs.len() > 0 {
    link_pairs.map(
      it => {
        let key = it.at(0)
        let url = it.at(1)
        text(
          fill: color,
          link(
            url,
            icons.at(key, default: (:)).at("logo", default: "") + " " + icons.at(key, default: (:)).at("displayname", default: key),
          ),
        )
      }
    )
  } else {
    // Return empty array if no valid links
    ()
  }
}

/* the section(s) that are colored and have a line */
#let section(primary_color: none, secondary_color: none, title) = {
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  heading(level: 1, grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  ))
}

/* custom bulleted list */
#let experience_details(color: none, symbol: none, ..args) = {
  if color == none {
    color = default_primary_color
  }
  if symbol == none {
    symbol = sym.bullet
  }
  list(
    indent: 5pt,
    marker: text(fill: color, symbol),
    ..args.pos().map(it => text(size: 10pt, [#it<experience_details>])),
  )
}

#let date(color: none, content) = {
  if color == none {
    color = default_secondary_color
  }
  [#h(1fr) #text(weight: "regular", size: 10pt, fill: color, content)]
}

/* experience that has an optional date and an optional description */
#let dated_experience(title, date: none, description: none, content: none, company: none) = {
  [
    #block(
      stroke: (bottom: 0.5pt + rgb("#14A4E6")),
      inset: (bottom: 5pt),
      width: 100%,
      grid(
        columns: (1fr, auto),
        align: (left, right),
        [
          #text(size: 9pt, fill: rgb("#757575"), date) \
          #text(size: 11pt, weight: "bold", title)
        ],
        [
          #text(size: 11pt, weight: "bold", company)
        ]
      )
    )

    #if description != none [
      #text(weight: "regular", size: 10pt, description)
    ]

    #content
    #v(1em)
  ]
}

/* display skills (a dictionary) */
#let show_skills(separator: none, color: none, skills) = {
  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_primary_color
  }

  let skills_array = ()
  
  // Handle case where skills might be empty or malformed
  if type(skills) == "dictionary" and skills.len() > 0 {
    for (key, value) in skills.pairs() {
      if key != "" and value != none {
        skills_array.push([*#key*])
        
        // Handle both string arrays and single strings
        if type(value) == "array" and value.len() > 0 {
          let filtered_values = value.filter(v => v != "" and v != none)
          if filtered_values.len() > 0 {
            skills_array.push(filtered_values.map(box).join(text(fill: color, separator)))
          } else {
            skills_array.push([Not specified])
          }
        } else if type(value) == "string" and value != "" {
          skills_array.push([#value])
        } else {
          skills_array.push([Not specified])
        }
      }
    }
  }

  if skills_array.len() > 0 {
    table(
      columns: 2,
      column-gutter: 2%,
      row-gutter: -0.2em,
      align: (right, left),
      stroke: none,
      ..skills_array,
    )
  } else {
    [No skills information provided]
  }
}

/* return text info about a person */
#let show_details_text(
  alignment: center + horizon,
  icons: none,
  separator: none,
  color: none,
  details,
) = {
  let show_line_from_dict(dict, key) = {
    let value = dict.at(key, default: none)
    if value != none and value != "" [#value \ ]
  }

  if separator == none {
    separator = default_separator
  }

  if color == none {
    color = default_link_color
  }

  if icons == none {
    icons = get_default_icons(color: color)
  } else {
    icons = join_dicts(get_default_icons(color: color), icons)
  }

  align(
    alignment,
    [
      #text(size: 14pt, details.at("name", default: ""))\
      #show_line_from_dict(details, "address")
      #show_line_from_dict(details, "phonenumber")
      #if details.at("email", default: "") != "" {
        text(
          size: 13pt,
          fill: color,
          (link("mailto:" + details.email)[#raw(details.email)]),
        )
        linebreak()
      }
      #if details.at("links", default: none) != none {
        let processed_links = process_links(details.links, color: color, icons: icons)
        if processed_links.len() > 0 {
          processed_links.join(text(fill: color, separator))
        }
      }
    ],
  )
}

/* the main info about the person (including picture) */
#let show_details(icons: none, separator: none, color: none, details) = {
  if details.at("picture", default: "").len() > 0 {
    grid(
      columns: (0.5fr, 1fr, 2.5fr),
      {
        if sys.inputs.at("picture", default: none) != none {
          align(right + horizon, image(details.picture, width: 90%))
        } else {
          align(right + horizon, rect(width: 90%, height: 100pt, fill: gray.lighten(80%), 
            align(center + horizon, text(size: 10pt, fill: gray, "No Image"))))
        }
      },
      h(1fr),
      show_details_text(icons: icons, separator: separator, color: color, details),
    )
  } else {
    show_details_text(
      // TODO figure out why the `center + horizon` alignment causes issues
      alignment: center + top,
      icons: icons,
      separator: separator,
      color: color,
      details,
    )
  }
  v(-1em)
}

/* the main configuration */
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
  // TODO figure out if there's a simpler way to parse this
  if primary_color == none {
    primary_color = default_primary_color
  }

  if secondary_color == none {
    secondary_color = default_secondary_color
  }

  if link_color == none {
    link_color = default_link_color
  }

  if font == none {
    font = default_font
  }

  if math_font == none {
    math_font = default_math_font
  }

  if separator == none {
    separator = text(
      fill: primary_color,
      // this is because in some fonts (notably computer modern), it shows the
      // vertical line as a horizontal one
      text(font: "Carlito", " \u{007c} "),
    )
  }

  if list_point == none {
    list_point = sym.bullet
  }

  // custom show rules
  show math.equation: set text(font: math_font)
  show heading.where(level: 1): title => grid(
    columns: 2,
    gutter: 1%,
    text(fill: primary_color, [#title <section>]),
    line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    ),
  )
  show heading.where(level: 2): set text(size: 11pt)
  show heading.where(level: 3): set text(weight: "regular")
  show heading.where(level: 2): set block(spacing: 0.7em)
  show heading.where(level: 3): set block(spacing: 0.7em)

  show link: set text(fill: primary_color)
  show list: set text(size: 10pt)
  // see https://github.com/typst/typst/issues/1941
  show "C++": box

  // custom set rules
set text(font: ("Arial", "Helvetica"), ligatures: false)
  set par(justify: true)

  set page(
    margin: (top: 0.8cm, left: 1.5cm, bottom: 1.5cm, right: 1.5cm),
    footer-descent: 0%,
    header-ascent: 0%,
  )
  set page(footer: [
    #line(
      start: (0pt, 0.45em),
      length: 100%,
      stroke: (paint: secondary_color, thickness: 0.05em),
    )

    #eval(details.footer, mode: "markup")
  ]) if details.at("footer", default: "").len() > 0

  set list(indent: 5pt, marker: text(fill: primary_color, list_point))

  show_details(details, color: primary_color)

  // the actual content of the document
  doc
}
