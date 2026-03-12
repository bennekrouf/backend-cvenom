#!/bin/bash
# =============================================================================
# cvenom — backend-only deploy
# Triggered by GitHub Actions on push to main.
# Run as root: sudo bash /opt/cvenom/src/backend-cvenom/deploy.sh
# =============================================================================
set -euo pipefail

APP_DIR="/opt/cvenom"
SRC_DIR="$APP_DIR/src"
BIN_DIR="$APP_DIR/bin"
DATA_DIR="$APP_DIR/data"
DEPLOY_USER="ubuntu"
DEPLOY_KEY="/var/www/.ssh/id_ed25519"
BACKEND_SRC="$SRC_DIR/backend-cvenom"

RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
log()  { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err()  { echo -e "${RED}[X]${NC} $1"; exit 1; }
step() { echo -e "\n${CYAN}=== $1 ===${NC}\n"; }

[ "$EUID" -ne 0 ] && err "Run as root: sudo bash deploy.sh"

# =============================================================================
step "1/3 — Pull latest backend from git"
# =============================================================================

[ -d "$BACKEND_SRC/.git" ] || err "$BACKEND_SRC is not a git repo"

GIT_SSH_COMMAND="ssh -i $DEPLOY_KEY -o StrictHostKeyChecking=no" \
  git -c safe.directory='*' -C "$BACKEND_SRC" fetch origin

BRANCH=$(git -C "$BACKEND_SRC" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
git -c safe.directory='*' -C "$BACKEND_SRC" reset --hard "origin/$BRANCH"
log "backend-cvenom pulled (branch: $BRANCH)"

# Restore ownership so ubuntu can build
chown -R "$DEPLOY_USER:$DEPLOY_USER" "$BACKEND_SRC"
log "Ownership restored to $DEPLOY_USER"

# =============================================================================
step "2/3 — Rebuild backend-cvenom"
# =============================================================================

CARGO_HOME="/home/$DEPLOY_USER/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

echo "  → Running cargo build --release ..."
sudo -u "$DEPLOY_USER" HOME="/home/$DEPLOY_USER" \
  bash -c "source ~/.cargo/env && cd '$BACKEND_SRC' && cargo build --release 2>&1"

# Atomic binary replace — never overwrites a running binary in-place
cp "$BACKEND_SRC/target/release/cvenom" "$BIN_DIR/cvenom.new"
chmod 755 "$BIN_DIR/cvenom.new"
mv "$BIN_DIR/cvenom.new" "$BIN_DIR/cvenom"
chown "$DEPLOY_USER:$DEPLOY_USER" "$BIN_DIR/cvenom"
log "Binary replaced: $BIN_DIR/cvenom"

# Sync templates (Typst files, font configs, etc.)
TEMPLATES_SRC="$BACKEND_SRC/templates"
if [ -d "$TEMPLATES_SRC" ]; then
  rsync -a --delete "$TEMPLATES_SRC/" "$DATA_DIR/templates/"
  chown -R "$DEPLOY_USER:$DEPLOY_USER" "$DATA_DIR/templates"
  log "Templates synced"
else
  warn "No templates directory found at $TEMPLATES_SRC"
fi

# =============================================================================
step "3/3 — Restart cvenom-backend"
# =============================================================================

PM2=$(which pm2)
sudo -u "$DEPLOY_USER" HOME="/home/$DEPLOY_USER" $PM2 restart cvenom-backend
sudo -u "$DEPLOY_USER" HOME="/home/$DEPLOY_USER" $PM2 save

log "cvenom-backend restarted"
echo ""
echo -e "${GREEN}Backend deploy complete.${NC}"
sudo -u "$DEPLOY_USER" HOME="/home/$DEPLOY_USER" $PM2 list
