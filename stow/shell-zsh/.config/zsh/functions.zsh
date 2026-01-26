# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH FUNCTIONS
# Version 8.0.0 - Organized Functions
# ═══════════════════════════════════════════════════════════

# Sudo wrapper (fixes @ in password issues)
sudo() {
    command sudo "$@"
}

# Command not found handler
command_not_found_handler() {
    echo "🐚 Command not found: $1"
    echo "💡 Check your spelling or install it with: paci $1"
    return 127
}

# Yazi with cd-on-quit
ya() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")"
    yazi "$@" --cwd-file="$tmp"
    if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
        cd -- "$cwd"
    fi
    rm -f -- "$tmp"
}

# Alias help function
alias-help() {
  echo "📋 Alias Categories (260+ total):"
  echo ""
  echo "🔒 Core Protection: lock-core, unlock-core, edit-core"
  echo "📂 Navigation: core, src, work, .., cd ~1"
  echo "📁 File Mgmt: ls, ll, tree, b (bat), search (fd)"
  echo "📦 Packages: pacu, paci, ins, yup, cleanup"
  echo "🔧 Git: lg, gst, gaa, gcm, gp, gl"
  echo "🔍 Core-Diff: cdiff, cds, cdh, cdm, cdsway"
  echo "💻 System: doctor, ff, df, mem, cpu"
  echo "📝 Editor: v, nzsh, nsway, nbar"
  echo "🖥️  Desktop: sway-reload, bar-restart"
  echo "🔐 Security: audit-quick, scan-secrets"
  echo "📚 Docs: keys, guide, roadmap"
  echo ""
  echo "📖 Full reference: bat ~/0-core/docs/ALIASES.md"
  echo "🔍 Search: alias | grep <keyword>"
}

# Productivity Apps
notes() {
    notesnook >/dev/null 2>&1 &
    disown
}

kp() {
    keepassxc >/dev/null 2>&1 &
    disown
}

# Weekly maintenance check
weekly-check() {
    echo ""
    echo "🗓️  Weekly Maintenance Check"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "This will:"
    echo "  1. Create pre/post snapshots"
    echo "  2. Run system updates (with auto-recovery)"
    echo "  3. Check for .pacnew files"
    echo "  4. Run health check"
    echo ""
    echo "⚠️  This requires user interaction"
    echo "⚠️  You control when this runs (no automation)"
    echo ""
    
    read "response?Continue? (y/N): "
    echo ""
    
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo "🚀 Starting maintenance..."
        echo ""
        ~/0-core/scripts/safe-update
    else
        echo "❌ Cancelled - no changes made"
    fi
}

# Check for updates
update-check() {
    echo ""
    echo "🔍 Checking for available updates..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Check official repos
    echo "📦 Official repositories:"
    local updates=$(checkupdates 2>/dev/null | wc -l)
    
    if [ $updates -gt 0 ]; then
        echo "   ⚠️  $updates updates available"
    else
        echo "   ✅ System is up to date"
    fi
    
    echo ""
    
    # Check AUR
    echo "📦 AUR packages:"
    local aur_updates=$(yay -Qua 2>/dev/null | wc -l)
    
    if [ $aur_updates -gt 0 ]; then
        echo "   ⚠️  $aur_updates AUR updates available"
    else
        echo "   ✅ AUR packages up to date"
    fi
    
    echo ""
    
    if [ $updates -gt 0 ] || [ $aur_updates -gt 0 ]; then
        echo "💡 Run 'safe-update' or 'weekly-check' to update"
    else
        echo "🎉 Everything is up to date!"
    fi
    
    echo ""
}

# dotctl wrapper
dotctl() {
    ~/0-core/scripts/dotctl "$@"
}

# sync-0-core function
sync-0-core() {
    local was_locked=false
    
    echo "🔄 Syncing 0-core..."
    echo ""
    
    # Check if locked
    if lsattr ~/0-core 2>/dev/null | head -1 | grep -q -- '----i'; then
        was_locked=true
        echo "🔓 Core is locked, unlocking temporarily..."
        unlock-core
        echo ""
    fi
    
    # Navigate and sync
    cd ~/0-core
    
    # Pull changes
    echo "⬇️  Pulling latest changes..."
    if git pull; then
        echo ""
        echo "⬆️  Pushing local changes..."
        git push
    else
        echo ""
        echo "❌ Pull failed - resolve conflicts manually"
        if [ "$was_locked" = true ]; then
            echo "⚠️  Core left unlocked for conflict resolution"
        fi
        return 1
    fi
    
    echo ""
    
    # Re-lock if it was locked
    if [ "$was_locked" = true ]; then
        echo "🔒 Re-locking core..."
        lock-core
    fi
    
    echo ""
    echo "✅ Sync complete!"
}

# dot-doctor wrapper
dot-doctor() {
    ~/0-core/scripts/dot-doctor "$@"
}

# Git guardrails - Prevent dangerous git operations
git() {
  # Only apply guardrails in 0-core
  if [[ $PWD != $HOME/0-core* ]]; then
    command git "$@"
    return $?
  fi
  
  local cmd="$1"
  
  case "$cmd" in
    commit)
      # Block commits if core is locked
      if lsattr -d ~/0-core 2>/dev/null | grep -q -- '----i'; then
        echo "🔒 0-core is LOCKED"
        echo "❌ Commit blocked to protect immutable core"
        echo "💡 Run: unlock-core"
return 1
      fi
      ;;
  esac
  
  # Execute the actual git command
  command git "$@"
}

# intent-guard preexec hook
preexec() {
    # Run before every command
    intent-guard check-command "$1" 2>&1
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        # Command was rejected - cancel execution
        # Send SIGINT to current shell to abort
        kill -INT $$
    fi
}
