#!/bin/bash

# Multi-tenant CV Generator Setup Script

echo "🏗️  Setting up Multi-tenant CV Generator..."

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p data/tenants
mkdir -p output
mkdir -p templates

# Initialize the database
echo "🗄️  Initializing tenant database..."
cargo run -- tenant init

# Import example tenants if CSV exists
if [ -f "tenants.csv" ]; then
  echo "📊 Importing tenants from tenants.csv..."
  cargo run -- tenant import tenants.csv
else
  echo "⚠️  No tenants.csv found. Creating example file..."
  cat >tenants.csv <<'EOF'
email,tenant_name
your.email@example.com,your-company
EOF
  echo "📝 Edit tenants.csv and run: cargo run -- tenant import tenants.csv"
fi

# Show current tenants
echo ""
echo "👥 Current tenants:"
cargo run -- tenant list

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit tenants.csv with your actual emails and tenant names"
echo "2. Run: cargo run -- tenant import tenants.csv"
echo "3. Start the server: cargo run -- server"
echo ""
echo "Tenant management commands:"
echo "  cargo run -- tenant add <email> <tenant-name>     # Add single tenant"
echo "  cargo run -- tenant list                          # List all tenants"
echo "  cargo run -- tenant check <email>                 # Check if email is authorized"
echo "  cargo run -- tenant remove <email>                # Deactivate tenant"
echo ""
