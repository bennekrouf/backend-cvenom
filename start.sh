#!/bin/bash
set -e

echo "Starting CVenom backend server..."
cd /opt/cvenom/backend

# All environment variables are now mandatory - no defaults
# Export all required environment variables
export LOG_PATH_CVENOM="${LOG_PATH_CVENOM:?LOG_PATH_CVENOM environment variable required}"
export ROCKET_PORT="${ROCKET_PORT:?ROCKET_PORT environment variable required}"
export CV_SERVICE_URL="${CV_SERVICE_URL:?CV_SERVICE_URL environment variable required}"
export CVENOM_TENANT_DATA_PATH="${CVENOM_TENANT_DATA_PATH:?CVENOM_TENANT_DATA_PATH environment variable required}"
export CVENOM_OUTPUT_PATH="${CVENOM_OUTPUT_PATH:?CVENOM_OUTPUT_PATH environment variable required}"
export CVENOM_TEMPLATES_PATH="${CVENOM_TEMPLATES_PATH:?CVENOM_TEMPLATES_PATH environment variable required}"
export CVENOM_DATABASE_PATH="${CVENOM_DATABASE_PATH:?CVENOM_DATABASE_PATH environment variable required}"
export JOB_MATCHING_API_URL="${JOB_MATCHING_API_URL:?JOB_MATCHING_API_URL environment variable required}"
export SERVICE_TIMEOUT="${SERVICE_TIMEOUT:?SERVICE_TIMEOUT environment variable required}"

echo "Environment variables validated successfully"
echo "Tenant data path: $CVENOM_TENANT_DATA_PATH"
echo "Output path: $CVENOM_OUTPUT_PATH"
echo "Templates path: $CVENOM_TEMPLATES_PATH"
echo "Database path: $CVENOM_DATABASE_PATH"

# Create directories
echo "Ensuring directories exist..."
mkdir -p "$CVENOM_TENANT_DATA_PATH" "$CVENOM_OUTPUT_PATH" "$CVENOM_TEMPLATES_PATH"

# Create database directory if needed
mkdir -p "$(dirname "$CVENOM_DATABASE_PATH")"

# Copy templates if needed
if [ ! "$(ls -A "$CVENOM_TEMPLATES_PATH" 2>/dev/null)" ]; then
  echo "Copying templates..."
  cp -r templates/* "$CVENOM_TEMPLATES_PATH/"
fi

# Initialize database if needed
if [ ! -f "$CVENOM_DATABASE_PATH" ]; then
  echo "Initializing database..."
  cargo run --release -- tenant init

  if [ ! -z "$DEFAULT_DOMAIN" ] && [ ! -z "$DEFAULT_TENANT" ]; then
    echo "Adding domain tenant: $DEFAULT_DOMAIN -> $DEFAULT_TENANT"
    cargo run --release -- tenant add-domain "$DEFAULT_DOMAIN" "$DEFAULT_TENANT"
  fi
fi

echo "Starting server..."
exec cargo run --release -- "$@"
