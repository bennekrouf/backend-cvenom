# CV Generator

A multi-tenant CV generator written in Rust using Typst for PDF compilation.

## CLI Usage

### Generate a CV
```bash
cargo run -- generate mohamed-bennekrouf --lang en
```

### Watch mode (auto-recompile on changes)
```bash
cargo run -- generate mohamed-bennekrouf --lang en --watch
```

### List available persons
```bash
cargo run -- list
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
- `data/john-doe/cv_params.toml` - Personal info template
- `data/john-doe/experiences_*.typ` - Experience templates for all languages
- `data/john-doe/README.md` - Instructions

#### Generate CV (Returns PDF file)
```bash
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en"}' \
  --output cv.pdf
```

Or save with a specific filename:
```bash
curl -X POST http://localhost:8000/api/generate \
  -H "Content-Type: application/json" \
  -d '{"person": "mohamed-bennekrouf", "lang": "en"}' \
  --output mohamed-bennekrouf_en.pdf
```

The response is a PDF file with `Content-Type: application/pdf`.

#### Health Check
```bash
curl http://localhost:8000/api/health
```

### Supported Languages
- `en` (English) - default
- `fr` (French)
- `ch` (Chinese)  
- `ar` (Arabic)

## Directory Structure

```
data/
  person-name/
    cv_params.toml      # Personal info and configuration
    experiences_*.typ   # Experience files per language
    profile.png         # Profile image
templates/
  cv.typ             # Main CV template
  template.typ       # Base template functions
output/              # Generated PDFs
```

## Requirements

- Rust (latest stable)
- Typst CLI tool
- Font Awesome fonts (for icons)
