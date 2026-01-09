#!/bin/bash
set -e
BACKUP_DIR="${1:-/tmp/faelight-backup}"
echo "🌲 Faelight Forest Backup"
mkdir -p "$BACKUP_DIR"
cd ~/0-core
[[ -n $(git status --porcelain) ]] && git add -A && git commit -m "pre-reinstall" && git push
echo "✅ 0-core synced"
[[ -d ~/.ssh ]] && cp -r ~/.ssh "$BACKUP_DIR/ssh"
[[ -d ~/.gnupg ]] && cp -r ~/.gnupg "$BACKUP_DIR/gnupg"
[[ -d ~/secrets ]] && cp -r ~/secrets "$BACKUP_DIR/secrets"
[[ -d ~/vault ]] && cp -r ~/vault "$BACKUP_DIR/vault"
pacman -Qe >"$BACKUP_DIR/packages.txt"
echo "✅ Backup: $BACKUP_DIR"
echo "📋 Copy to USB!"
