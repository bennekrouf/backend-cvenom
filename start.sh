#!/bin/bash
set -e # Exit on any error

echo "Starting CVenom backend server..."

# Change to backend directory
cd /opt/cvenom/backend

# Create required directories with proper permissions
echo "Creating directories..."
sudo mkdir -p /var/cvenom/tenant-data /var/cvenom/output /opt/cvenom/templates
sudo chown -R $(whoami):$(whoami) /var/cvenom /opt/cvenom/templates

# Copy templates if they don't exist in production location
if [ ! "$(ls -A /opt/cvenom/templates)" ]; then
  echo "Copying templates to production location..."
  cp -r templates/* /opt/cvenom/templates/
fi

# Initialize database if it doesn't exist
if [ ! -f "/var/cvenom/tenants.db" ]; then
  echo "Initializing tenant database..."
  cargo run --release -- tenant init

  # Add default domain tenant if specified
  if [ ! -z "$DEFAULT_DOMAIN" ] && [ ! -z "$DEFAULT_TENANT" ]; then
    echo "Adding default domain tenant: $DEFAULT_DOMAIN -> $DEFAULT_TENANT"
    cargo run --release -- tenant add-domain "$DEFAULT_DOMAIN" "$DEFAULT_TENANT"
  fi
fi

# Start the server
echo "Starting server..."
exec cargo run --release -- "$@"
