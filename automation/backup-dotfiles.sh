#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🔄 Dotfiles Backup Automation
# Omarchy v2.8.8
# Auto-commit and push 0-core changes daily
# ═══════════════════════════════════════════════════════════

set -euo pipefail

DOTFILES_DIR="$HOME/0-core"
LOG_FILE="$HOME/.local/state/omarchy/backup.log"
DATE=$(date '+%Y-%m-%d %H:%M:%S')

# Ensure log directory exists
mkdir -p "$(dirname "$LOG_FILE")"

# Function to log messages
log() {
  echo "[$DATE] $1" | tee -a "$LOG_FILE"
}

# Change to 0-core directory
cd "$DOTFILES_DIR" || {
  log "ERROR: Cannot access 0-core directory"
  exit 1
}

# Check if there are changes
if [[ -z $(git status --porcelain) ]]; then
  log "No changes to backup"
  exit 0
fi

log "Changes detected, backing up..."

# Add all changes
git add -A

# Commit with automated message
git commit -m "auto: Daily backup $(date '+%Y-%m-%d')

Automated backup from systemd timer"

# Push to GitHub
if git push; then
  log "✅ Backup successful"
else
  log "❌ Backup failed - push error"
  exit 1
fi
