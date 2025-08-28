# CV Generator

A multi-tenant CV generator written in Rust using Typst for PDF compilation with support for multiple templates.

## Features

- **Multi-tenant**: Support for multiple persons with isolated data
- **Multi-language**: English and French support
- **Multi-template**: Choose from different CV layouts
- **Web API**: RESTful API with full CORS support
- **Watch mode**: Auto-recompile on file changes
- **Profile pictures**: Upload and manage profile images
- **Template system**: Extensible template architecture

## Available Templates

- **default**: Standard CV layout
- **keyteo**: CV with Keyteo branding and logo at the top of every page

## CLI Usage

### Generate a CV
```bash
# Generate with default template
cargo run -- generate mohamed-bennekrouf --lang en

# Generate with keyteo template
cargo run -- generate mohamed-bennekrouf --lang en --template keyteo

# Generate French version with keyteo template
cargo run -- generate mohamed-bennekrouf --lang fr --template keyteo
```

### Watch mode (auto-recompile on changes)
```bash
cargo run -- generate mohamed-bennekrouf --lang en --template keyteo --watch
```

### List available persons
```bash
cargo run -- list
```

### List available templates
```bash
cargo run -- list-templates
```

### Create a new person directory
```bash
cargo run -- create new-person-name
```

## Web Server

Start the web server:
```bash
cargo run -- server
```

The server runs on `http://localhost:8000` with full CORS support.

### API Endpoints

#### List Available Templates
```bash
curl http://localhost:8000/api/templates
```

Response:
```json
{
  "success": true,
  "templates": [
    {
      "name": "default",
      "description": "Standard CV layout"
    },
    {
      "name": "keyteo",
      "description": "CV with Keyteo branding and logo at the top of every page"
    }
  ]
}
```

#### Upload Profile Picture
```bash
curl -X POST http://localhost:8000/api/upload-picture \
  -F "person=john-doe" \
  -F "file=@/path/to/your/image.jpg"
```

Response:
```json
{
  "success": true,
  "message": "Profile picture uploaded successfully for john-doe",
  "file_path": "data/john-doe/profile.png"
}
```

Accepts any image format and automatically saves as `profile.png` in the person's directory.

#### Create New Person
```bash
curl -X POST http://localhost:8000/api/create \
  -H "Content-Type: application/json" \
  -d '{"person": "john-doe"}'
```

Response:
```json
{
  "success": true,
  "message": "Person directory created successfully for john-doe",
  "person_dir": "data/john-doe"
}
```

This creates:
- `data/john-doe/cv_params.toml` - Personal info template with logo support
- `data/john-doe/experiences_*.typ` - Experience templates for all languages
- `data/john-doe/README.md` - Instructions

#### Generate CV (Returns PDF file)
```bash
# Generate with default template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en"}' \
  --output cv.pdf

# Generate with keyteo template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en", "template": "keyteo"}' \
  --output cv_keyteo.pdf

# Generate French version with keyteo template
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "fr", "template": "keyteo"}' \
  --output cv_keyteo_fr.pdf
```

The response is a PDF file with `Content-Type: application/pdf`.

#### Health Check
```bash
curl http://localhost:8000/api/health
```

### Request Parameters

- `person`: Required. Name of the person (must match directory name)
- `lang`: Optional. Language code (`en` or `fr`). Default: `en`
- `template`: Optional. Template name (`default` or `keyteo`). Default: `default`

## Directory Structure

```
data/
  person-name/
    cv_params.toml      # Personal info and configuration
    experiences_*.typ   # Experience files per language
    profile.png         # Profile image
    logo.png           # Company logo (for logo template)
templates/
  cv.typ             # Default CV template
  cv_keyteo.typ      # Keyteo CV template
  template.typ       # Base template functions
  person_template.toml # Template for new persons
output/              # Generated PDFs (format: person_template_lang.pdf)
```

## Configuration File Format

The `cv_params.toml` file supports the following structure:

```toml
# Personal Information
name = "Your Name"
phonenumber = "+1 234 567 8900"
email = "your.email@example.com"
address = "Your Address"
picture = "profile.png"

# Company/Logo information (for logo template)
company_name = "Your Company"
company_logo = "logo.png"

[links]
github = "https://github.com/username"
linkedin = "https://linkedin.com/in/username"
personal = "https://yourwebsite.com"

# Technical Skills
[skills]
Languages = ["Rust (3y)", "Python (5y)", "JavaScript (>10y)"]
Frameworks = ["React", "Angular", "Node.js"]
Others = ["Docker", "Git", "Linux"]

# Education & Certifications
[[education]]
title = "Computer Science Degree"
date = "(2020)"

[[education]]
title = "AWS Certification"
date = "(2023)"

# Languages
[languages]
native = ["English (Native)"]
fluent = ["French (Fluent)"]
intermediate = ["Spanish (Intermediate)"]
basic = ["German (Basic)"]
```

## Template Development

### Adding New Templates

1. Create a new `.typ` file in the `templates/` directory
2. Follow the naming convention: `cv_templatename.typ`
3. Update the `CvTemplate` enum in `src/lib.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub enum CvTemplate {
       Default,
       Logo,
       YourNewTemplate,  // Add here
   }
   ```
4. Add the template mapping in the `template_file()` method
5. Add the string conversion in `from_str()` and `all()` methods

### Template Structure

Templates should:
- Import from `template.typ` for base functionality
- Import the appropriate language experiences file
- Load configuration from `cv_params.toml`
- Follow the established styling patterns

Example template structure:
```typst
#import "template.typ": conf, date, dated_experience, experience_details, section, show_skills
#import "experiences_en.typ" : get_work_experience

#let details = toml("cv_params.toml")

// Custom template configuration
#show: doc => conf(details, doc)

// Template-specific customizations here

#get_work_experience()
// ... rest of template
```

## Supported Languages
- `en` (English) - default
- `fr` (French)

## Generated File Naming

PDFs are generated with the format: `{person}_{template}_{lang}.pdf`

Examples:
- `john-doe_default_en.pdf`
- `john-doe_keyteo_fr.pdf`

## Requirements

- Rust (latest stable)
- Typst CLI tool
- Font Awesome fonts (for icons)

## Development Notes

- Uses generics over trait objects for better performance
- Implements clear error handling without unwrap()
- YAML configuration loading with tracing for logging
- Modular template system for easy extension
