# CVenom Multi-tenant CV Generator

A Rust-based multi-tenant CV generation system with Firebase authentication and tenant isolation.

## Breaking Change: Configuration

**🚨 IMPORTANT: Config.yaml has been removed. All configuration is now via mandatory environment variables.**

## Required Environment Variables

The application will **crash on startup** if any of these variables are missing:

```bash
# Logging
LOG_PATH_CVENOM="/var/log/cvenom.log"

# Server
ROCKET_PORT="4002"
CV_SERVICE_URL="http://localhost:8080"

# Storage paths
CVENOM_TENANT_DATA_PATH="/var/cvenom/tenant-data"
CVENOM_OUTPUT_PATH="/var/cvenom/output" 
CVENOM_TEMPLATES_PATH="/opt/cvenom/templates"
CVENOM_DATABASE_PATH="/var/cvenom/tenants.db"

# Services
JOB_MATCHING_API_URL="http://127.0.0.1:5555"
SERVICE_TIMEOUT="60"
```

## Quick Start

1. **Set environment variables:**
```bash
cp set_env_example.sh set_env.sh
# Edit set_env.sh with your values
source set_env.sh
```

2. **Initialize database:**
```bash
cargo run -- tenant init
```

3. **Add tenants:**
```bash
# Add domain tenant (all @mycompany.ch emails)
cargo run -- tenant add-domain mycompany.ch mycompany

# Add specific email
cargo run -- tenant add user@company.com company-name
```

4. **Start server:**
```bash
cargo run
```

## Docker Deployment

1. **Set environment variables:**
```bash
cp docker.env .env
# Edit .env with your values
```

2. **Build and run:**
```bash
docker build -t cvenom .
docker run --env-file .env -p 4002:4002 cvenom
```

## Migration from config.yaml

If you previously used config.yaml, convert your settings:

**Old config.yaml:**
```yaml
production:
  tenant_data_path: "/var/cvenom/tenant-data"
  output_path: "/var/cvenom/output"
  templates_path: "/opt/cvenom/templates"
  database_path: "/var/cvenom/tenants.db"

job_matching:
  api_url: "http://127.0.0.1:5555"
  timeout_seconds: 120
```

**New environment variables:**
```bash
export CVENOM_TENANT_DATA_PATH="/var/cvenom/tenant-data"
export CVENOM_OUTPUT_PATH="/var/cvenom/output"
export CVENOM_TEMPLATES_PATH="/opt/cvenom/templates"
export CVENOM_DATABASE_PATH="/var/cvenom/tenants.db"
export JOB_MATCHING_API_URL="http://127.0.0.1:5555"
export SERVICE_TIMEOUT="60"
```

## Cargo Dependencies

Add the following dependencies without versions:

```bash
cargo add anyhow
cargo add async-recursion
cargo add chrono --features serde
cargo add csv
cargo add graflog
cargo add jsonwebtoken
cargo add reqwest --features json,multipart
cargo add rocket --features json,secrets
cargo add serde --features derive
cargo add serde_json
cargo add sqlx --features runtime-tokio-rustls,sqlite,chrono,uuid
cargo add tokio --features full
cargo add toml
cargo add uuid --features v4
```

## CLI Commands

```bash
# Tenant management
cargo run -- tenant init
cargo run -- tenant add <email> <tenant-name>
cargo run -- tenant add-domain <domain> <tenant-name>
cargo run -- tenant list
cargo run -- tenant check <email>

# CV generation
cargo run -- generate <profile> --lang <en|fr> --template <template>
cargo run -- create <profile-name>
cargo run -- list
cargo run -- list-templates
```

## API Endpoints

### Public
- `GET /health` - Health check
- `GET /templates` - List templates

### Protected (Firebase auth + tenant)
- `POST /generate` - Generate CV PDF
- `POST /create` - Create profile
- `POST /upload-picture` - Upload profile picture
- `POST /analyze-job-fit` - LinkedIn job analysis
- `GET /me` - Current user info

### Through api0 (MCP gateway)

cvenom is an api0 tenant (dashboard: app.api0.ai, workspace "cvenom"). api0 calls
this backend with a Google identity token plus `X-User-Email`; `src/auth.rs`
checks both. Values (set in `ecosystem.config.js`):

| | |
|---|---|
| `CVENOM_OIDC_AUDIENCE` (expected `aud`) | `https://api.cvenom.com/api0/tenant/c752f139-9fad-47c9-b10b-9983dcce8333` |
| `CVENOM_OIDC_SERVICE_ACCOUNT` (pinned signer) | `firebase-adminsdk-fbsvc@cvenom-de582.iam.gserviceaccount.com` |
| Issuer | `accounts.google.com` |

Unset audience disables this path; an audience with no signer refuses every
api0 call. The api0 workspace must match: Access tab, auth mode `google_sa`,
same service account and audience, connection "Google identity token".
Who can connect: the workspace's Google sign-in (OAuth client in Firebase
project `cvenom-de582`; full client id to check in the Google Cloud console).

Connecting from claude.ai: server URL `https://gateway.api0.ai/mcp/cvenom-mcp`,
"Use your own OAuth client", Client ID `cvenom-mcp`, no secret. Do not pick
automatic registration: it gives api0's generic client, not the cvenom workspace.

## Directory Structure

```
data/
  tenants.db              # SQLite tenant database
  tenants/                # Tenant-isolated data
    mycompany/               # Tenant directory
      user@domain.com/    # User directory
        cv_params.toml    # Personal info
        experiences_*.typ # Experience files
        profile.png       # Profile image
templates/                # CV templates
output/                   # Generated PDFs
```

## Environment Examples

### Development
```bash
export LOG_PATH_CVENOM="./logs/cvenom.log"
export ROCKET_PORT="4002"
export CV_SERVICE_URL="http://localhost:8080"
export CVENOM_TENANT_DATA_PATH="./data/tenants"
export CVENOM_OUTPUT_PATH="./output"
export CVENOM_TEMPLATES_PATH="./templates"
export CVENOM_DATABASE_PATH="./data/tenants.db"
export JOB_MATCHING_API_URL="http://127.0.0.1:5555"
export SERVICE_TIMEOUT="30"
```

### Production
```bash
export LOG_PATH_CVENOM="/var/log/cvenom.log"
export ROCKET_PORT="4002"
export CV_SERVICE_URL="https://cv-service.example.com"
export CVENOM_TENANT_DATA_PATH="/var/cvenom/tenant-data"
export CVENOM_OUTPUT_PATH="/var/cvenom/output"
export CVENOM_TEMPLATES_PATH="/opt/cvenom/templates"
export CVENOM_DATABASE_PATH="/var/cvenom/tenants.db"
export JOB_MATCHING_API_URL="https://job-matching.example.com"
export SERVICE_TIMEOUT="60"
```

## Security Features

- Firebase JWT token verification
- Tenant-based authorization
- Data isolation per tenant
- No shared data between tenants
- Mandatory environment variable validation

## Error Handling

The application uses static error handling with `Result<T, E>` types and `trace` logging throughout. No `unwrap()` calls are used for robust error handling.

Built with Rust for performance and type safety.
