# CV Generator

A multi-tenant CV generator written in Rust using Typst for PDF compilation with Firebase authentication and SQLite-based tenant management.

## Features

- **Multi-tenant**: Complete data isolation between tenants using SQLite
- **Firebase Authentication**: Google sign-in with JWT token verification
- **Multi-language**: English and French support
- **Multi-template**: Choose from different CV layouts
- **Web API**: RESTful API with full CORS support
- **Watch mode**: Auto-recompile on file changes
- **Profile pictures**: Upload and manage profile images
- **Template system**: Extensible template architecture

## Quick Start

### 1. Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Typst
cargo install typst-cli

# Install dependencies
cargo add anyhow clap --features derive rocket --features json,secrets serde --features derive tokio --features full jsonwebtoken reqwest --features json tracing tracing-subscriber --features env-filter sqlx --features runtime-tokio-rustls,sqlite,chrono,uuid chrono --features serde uuid --features v4 csv
```

### 2. Initialize Database and Tenants
```bash
# Create directories
mkdir -p data output templates

# Initialize the tenant database
cargo run -- tenant init

# Add tenants manually
cargo run -- tenant add mohamed.bennekrouf@gmail.com keyteo

# Check it is added
cargo run -- tenant check mohamed.bennekrouf@gmail.com


# OR import from CSV
echo "email,tenant_name
user1@company.com,company-a
user2@startup.io,startup-b" > tenants.csv

cargo run -- tenant import tenants.csv

# Verify tenants
cargo run -- tenant list
```

### 3. Start the Server
```bash
cargo run -- server
```

The server runs on `http://localhost:8000` with multi-tenant authentication.

## Authentication & Authorization

The system uses Firebase Authentication + SQLite tenant management:

1. **Firebase Auth**: Users sign in with Google
2. **Tenant Check**: Email must exist in SQLite `tenants` table
3. **Data Isolation**: Each tenant gets separate data directory

### User Flow
- User signs in with Google → Gets Firebase ID token
- Frontend sends token with API requests
- Backend verifies token + checks tenant authorization
- Access granted only if both succeed

## CLI Usage

### Tenant Management
```bash
# Initialize database
cargo run -- tenant init

# Add single tenant
cargo run -- tenant add user@example.com company-name

# Import from CSV file
cargo run -- tenant import tenants.csv

# List all tenants
cargo run -- tenant list

# Check if email is authorized
cargo run -- tenant check user@example.com

# Deactivate tenant
cargo run -- tenant remove user@example.com
```

### CV Generation
```bash
# Generate with default template
cargo run -- generate person-name --lang en

# Generate with specific template
cargo run -- generate person-name --lang en --template keyteo

# Watch mode (auto-recompile on changes)
cargo run -- generate person-name --lang en --watch


# Add domain tenant for keyteo.ch
cargo run -- tenant add-domain keyteo.ch keyteo

# Test it works
cargo run -- tenant check mohamed.bennekrouf@keyteo.ch
cargo run -- tenant check alevavasseur@keyteo.ch  
cargo run -- tenant check anyone@keyteo.ch

# List all tenants
cargo run -- tenant list
```

### File Management
```bash
# Create new person directory
cargo run -- create new-person-name

# List available persons
cargo run -- list

# List available templates
cargo run -- list-templates
```

## API Endpoints

### Public Endpoints
```bash
GET  /api/health        # Health check
GET  /api/templates     # List available templates
```

### Protected Endpoints (require Firebase auth + tenant registration)
```bash
POST /api/generate      # Generate CV PDF
POST /api/create        # Create person directory
POST /api/upload-picture # Upload profile picture
GET  /api/me            # Get current user + tenant info
```

### Example API Usage
```bash
# Get templates (public)
curl http://localhost:8000/api/templates

# Generate CV (requires auth header)
curl -X POST http://localhost:8000/api/generate \
  -H "Authorization: Bearer <firebase-id-token>" \
  -H "Content-Type: application/json" \
  -d '{"person": "john-doe", "lang": "en", "template": "default"}' \
  --output cv.pdf
```

## Directory Structure

```
data/
  tenants.db            # SQLite tenant database
  tenants/              # Tenant-isolated data
    company-a/          # Tenant-specific directory
      john-doe/         # Person directory
        cv_params.toml  # Personal info
        experiences_*.typ # Experience files
        profile.png     # Profile image
templates/
  cv.typ               # Default template
  cv_keyteo.typ        # Keyteo template
  template.typ         # Base template functions
output/                # Generated PDFs
```

## Configuration

### Firebase Setup
Update your Firebase project ID in `src/web.rs`:
```rust
let mut auth_config = AuthConfig::new("your-project-id".to_string());
```

### Database Location
Default: `data/tenants.db`
Change with: `--data-dir /path/to/data`

## Available Templates

- **default**: Standard CV layout
- **keyteo**: CV with Keyteo branding and logo

## Troubleshooting

### Database Issues
```bash
# Check if database exists
ls -la data/tenants.db

# Re-initialize database
rm data/tenants.db
cargo run -- tenant init
```

### Authentication Issues
- Verify Firebase project ID matches your setup
- Check that user email exists in tenant table
- Ensure Firebase ID token is valid

### Permission Issues
```bash
# Fix directory permissions
chmod 755 data
chmod 644 data/tenants.db
```

## Development

### Adding New Tenants
```bash
# Via CLI
cargo run -- tenant add new-user@company.com new-company

# Via CSV
echo "new-user@company.com,new-company" >> tenants.csv
cargo run -- tenant import tenants.csv
```

### Multi-tenant Testing
1. Add test users to tenant database
2. Sign in via Firebase in frontend
3. Verify data isolation between tenants
4. Check tenant-specific directories are created

## Security Features

- Firebase JWT token verification
- Tenant-based authorization
- Data isolation per tenant
- Request logging with tenant context
- No shared data between tenants

Built with Rust for performance and type safety, following clear error handling patterns without unwrap calls.
