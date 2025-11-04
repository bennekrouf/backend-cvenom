#!/bin/bash
set -e

echo "Starting CVenom backend server..."
cd /opt/cvenom/backend

# Set production environment - single ENV variable
export ENVIRONMENT=production

# Create directories
echo "Ensuring directories exist..."
mkdir -p /var/cvenom/tenant-data /var/cvenom/output /opt/cvenom/templates

# Copy templates if needed
if [ ! "$(ls -A /opt/cvenom/templates 2>/dev/null)" ]; then
  echo "Copying templates..."
  cp -r templates/* /opt/cvenom/templates/
fi

# Initialize database if needed
if [ ! -f "/var/cvenom/tenants.db" ]; then
  echo "Initializing database..."
  cargo run --release -- tenant init

  if [ ! -z "$DEFAULT_DOMAIN" ] && [ ! -z "$DEFAULT_TENANT" ]; then
    echo "Adding domain tenant: $DEFAULT_DOMAIN -> $DEFAULT_TENANT"
    cargo run --release -- tenant add-domain "$DEFAULT_DOMAIN" "$DEFAULT_TENANT"
  fi
fi

echo "Starting server..."
exec cargo run --release -- "$@"
