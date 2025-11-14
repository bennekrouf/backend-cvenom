// templates/font_config.typ

// Font configuration - update these to match installed fonts
#let font_config = (
  // Use Font Awesome 7 since that's what you have installed
  brands_font: "Font Awesome 7 Brands",
  solid_font: "Font Awesome 7 Free",
  regular_font: "Font Awesome 7 Free",
  
  // Fallback fonts if Font Awesome is not available
  fallback_font: "Arial",
  
  // Icon unicode values (these stay the same across versions)
  icons: (
    github: "\u{f09b}",
    linkedin: "\u{f08c}", 
    personal_info: "\u{f268}",
    website: "\u{f0ac}",
  )
)

// Helper function to get icon with font
#let get_icon(icon_name, font_type: "solid") = {
  let font_family = if font_type == "brands" {
    font_config.brands_font
  } else if font_type == "solid" {
    font_config.solid_font  
  } else {
    font_config.regular_font
  }
  
  let icon_code = font_config.icons.at(icon_name, default: "")
  if icon_code != "" {
    text(font: font_family, icon_code)
  } else {
    text("") // Empty if icon not found
  }
}
