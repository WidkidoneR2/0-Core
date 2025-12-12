#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🧹 Weekly System Maintenance
# Omarchy v2.8.8
# Clean caches, update package lists, system health check
# ═══════════════════════════════════════════════════════════

set -euo pipefail

LOG_FILE="$HOME/.local/state/omarchy/maintenance.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

# Function to log messages
log() {
  echo "[$DATE] $1" | tee -a "$LOG_FILE"
}

log "🧹 Starting weekly maintenance..."

# 1. Clean package cache (keep last 3 versions)
log "Cleaning package cache..."
if paccache -rk3 &>/dev/null; then
  log "✅ Package cache cleaned"
else
  log "⚠️  paccache not found, skipping"
fi

# 2. Clean AUR cache
log "Cleaning AUR cache..."
if command -v yay &>/dev/null; then
  yay -Sc --noconfirm &>/dev/null || true
  log "✅ AUR cache cleaned"
fi

# 3. Clean cargo cache (if cargo-cache installed)
log "Cleaning cargo cache..."
if command -v cargo-cache &>/dev/null; then
  cargo-cache -a &>/dev/null || true
  log "✅ Cargo cache cleaned"
fi

# 4. Clean old logs
log "Cleaning old journal logs (keep 2 weeks)..."
sudo journalctl --vacuum-time=2weeks &>/dev/null || true
log "✅ Journal logs cleaned"

# 5. Update package database
log "Updating package database..."
if sudo pacman -Sy &>/dev/null; then
  log "✅ Package database updated"
fi

# 6. Check for orphaned packages
log "Checking for orphaned packages..."
ORPHANS=$(pacman -Qdtq 2>/dev/null || true)
if [[ -n "$ORPHANS" ]]; then
  log "⚠️  Found orphaned packages: $ORPHANS"
  log "   Run: sudo pacman -Rns \$(pacman -Qdtq)"
else
  log "✅ No orphaned packages"
fi

# 7. Run dot-doctor health check
log "Running system health check..."
HEALTH_OUTPUT=$(dot-doctor 2>&1 | tail -1)
if echo "$HEALTH_OUTPUT" | grep -q "100%"; then
  log "✅ System health: 100%"
else
  log "⚠️  System health check: $HEALTH_OUTPUT"
fi

log "🎉 Weekly maintenance complete!"
