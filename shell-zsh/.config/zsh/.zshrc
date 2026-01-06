# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH SHELL CONFIGURATION
# Version v5.1.0 - Zsh Migration Edition
# Clean, organized, and intentional
# Migrated from Fish for better bash compatibility
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# 🛡️ PROTECTION & ERROR HANDLING
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

# Disable history expansion (fixes git commit message issues)
setopt NO_BANG_HIST          # Don't treat ! specially
setopt NO_HIST_EXPAND        # Don't expand history references

# Directory history (Fish-like cd navigation)
setopt AUTO_PUSHD            # Auto push old dir to stack
setopt PUSHD_IGNORE_DUPS     # No duplicates in stack
setopt PUSHD_SILENT          # Don't print stack on pushd/popd

# ═══════════════════════════════════════════════════════════
# 🎨 ENVIRONMENT & PATH
# ═══════════════════════════════════════════════════════════

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/0-core/scripts:$PATH"

# Editor
export EDITOR=nvim
export VISUAL=nvim


# ═══════════════════════════════════════════════════════════
# 🔒 CORE PROTECTION (0-core Immutability)
# ═══════════════════════════════════════════════════════════

alias lock-core='~/0-core/scripts/core-protect lock'
alias unlock-core='~/0-core/scripts/core-protect unlock'
alias edit-core='~/0-core/scripts/core-protect edit'
alias core-status='~/0-core/scripts/core-protect status'

# ═══════════════════════════════════════════════════════════
# 🔄 SMART UPDATE SYSTEM (Manual Control)
# ═══════════════════════════════════════════════════════════

alias safe-update='~/0-core/scripts/safe-update'
alias weekly='weekly-check'
alias check-updates='update-check'

# ═══════════════════════════════════════════════════════════
# 📂 NAVIGATION & DIRECTORY MANAGEMENT
# ═══════════════════════════════════════════════════════════

# Numbered structure (0-core philosophy)
alias core='cd ~/0-core'
alias src='cd ~/1-src'
alias work='cd ~/2-work'
alias keep='cd ~/3-keep'
alias tmp='cd ~/9-temp'

# Quick navigation
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias cdp='cd -'

# Common directories
alias desk='cd ~/Desktop'
alias docs='cd ~/Documents'
alias down='cd ~/Downloads'
alias pics='cd ~/Pictures'
alias vids='cd ~/Videos'

# Config directories
alias conf='cd ~/.config'
alias hyprconf='cd ~/.config/hypr'
alias nvimconf='cd ~/.config/nvim'
alias zshconf='cd ~/.config/zsh'

# ═══════════════════════════════════════════════════════════
# 📁 FILE MANAGEMENT (Modern Tools)
# ═══════════════════════════════════════════════════════════

# Eza (modern ls)
alias ls='eza --icons --group-directories-first'
alias ll='eza -lah --icons --group-directories-first --git'
alias la='eza -a --icons --group-directories-first'
alias l='eza -lh --icons --group-directories-first'
alias lt='eza -lah --icons --sort=modified --reverse'
alias lsize='eza -lah --icons --sort=size --reverse'
alias tree='eza --tree --icons --group-directories-first'

# Bat (better cat) - DO NOT alias cat directly (breaks pipes/scripts)
alias b='bat --paging=never'        # Quick colorized view
alias catp='bat --paging=always'    # Paged view
alias catt='bat --style=plain'      # Plain style
alias ccat='/usr/bin/cat'           # Explicit plain cat if needed

# Fd (better find)
alias search='fd'
alias findf='fd --type f'
alias findd='fd --type d'

# Fzf (fuzzy finder)
alias fcd='cd $(fd --type d | fzf)'
alias vf='nvim $(fd --type f | fzf)'
alias preview='fzf --preview "bat --color=always {}"'

# Yazi (file manager)
alias y='yazi'
alias yy='yazi'
alias fm='yazi'

# Yazi with cd-on-quit
ya() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")"
    yazi "$@" --cwd-file="$tmp"
    if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
        cd -- "$cwd"
    fi
    rm -f -- "$tmp"
}

# ═══════════════════════════════════════════════════════════
# 📦 PACKAGE MANAGEMENT (Arch/Pacman/Yay)
# ═══════════════════════════════════════════════════════════

# Pacman
alias pacu='sudo pacman -Syu'
alias paci='sudo pacman -S'
alias pacs='pacman -Ss'
alias pacr='sudo pacman -R'
alias pacrem='sudo pacman -Rns'
alias pacinfo='pacman -Qi'
alias paclist='pacman -Qqe'

# Yay
alias yay='yay --color=auto'
alias yayu='yay -Syu'
alias yays='yay -Ss'
alias yayi='yay -S'
alias yayr='yay -R'
alias ins='yay -S'
alias uns='yay -Rns'
alias yup='yay -Syu'

# Maintenance
alias cleanup='sudo pacman -Rns $(pacman -Qtdq) 2>/dev/null || true'
alias unlock='sudo rm /var/lib/pacman/db.lck'
alias orphans='pacman -Qtdq'
alias mirror='sudo reflector --verbose --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'
alias clean-all='yay -Sc && yay -Yc && sudo pacman -Rns $(pacman -Qtdq) 2>/dev/null || true'
alias fix-keys='sudo pacman-key --init && sudo pacman-key --populate && sudo pacman-key --refresh-keys'

# ═══════════════════════════════════════════════════════════
# 🔧 GIT & VERSION CONTROL
# ═══════════════════════════════════════════════════════════

# LazyGit (best!)
alias lg='lazygit'

# Basic
alias g='git'
alias gst='git status'
alias gss='git status -s'

# Add & Commit
alias ga='git add'
alias gaa='git add -A'
alias gcm='git commit -m'
alias gca='git commit --amend'
alias gcam='git commit -am'

# Push & Pull
alias gp='git push'
alias gl='git pull'
alias gf='git fetch'

# Logs
alias glog='git log --oneline -10'
alias gla='git log --oneline --graph --all'

# Branches
alias gb='git branch'
alias gba='git branch -a'
alias gbd='git branch -d'
alias gbD='git branch -D'
alias gco='git checkout'
alias gcb='git checkout -b'

# Diff
alias gd='git diff'
alias gds='git diff --staged'
alias gdp='git diff --color=always | less -R'
alias gsh='git show'

# Stash
alias gstash='git stash'
alias gstp='git stash pop'
alias gstl='git stash list'

# Undo/Reset
alias gundo='git reset HEAD~1'
alias gunstage='git reset HEAD'
alias greset='git reset --hard'
alias gclean='git clean -fd'

# Clone
alias gcl='git clone'

# 0-core Management
alias dotsave='cd ~/0-core && git add -A && git commit -m "Update configs" && git push'
alias dotpush='cd ~/0-core && git add -A && git commit -m "Update configs $(date +%Y-%m-%d)" && git push'
alias dotstatus='cd ~/0-core && git status'

# ═══════════════════════════════════════════════════════════
# 🔍 CORE-DIFF ALIASES (Quick Access)
# ═══════════════════════════════════════════════════════════

# Quick checks
alias cdiff='core-diff'                          # Short form
alias cds='core-diff summary'                    # Quick stats
alias cdh='core-diff --high-risk'                # High-risk only
alias cdv='core-diff --verbose'                  # Show files

# Visual inspection
alias cdm='core-diff --open meld'                # Open Meld
alias cdd='core-diff --open delta'               # Delta terminal diff

# Historical comparisons
alias cdlast='core-diff since HEAD~1'            # Since last commit
alias cdrel='core-diff since $(git describe --tags --abbrev=0 2>/dev/null || echo HEAD)'  # Since last release

# Package-specific shortcuts (customize as needed)
alias cdhypr='core-diff wm-hypr'
alias cdway='core-diff bar-waybar'
alias cdzsh='core-diff shell-zsh'
alias cdnvim='core-diff editor-nvim'

# Combined workflows
alias cdcheck='cdiff && dot-doctor'              # Morning check
alias cdreview='cdv && cdh'                      # Pre-commit review

# ═══════════════════════════════════════════════════════════
# 💻 SYSTEM MONITORING & HEALTH
# ═══════════════════════════════════════════════════════════

# System info
alias ff='fastfetch'
alias neofetch='fastfetch'
alias sysinfo='fastfetch'

# Health checks
alias doctor='dot-doctor'
alias health='dot-doctor'
alias check-health='dot-doctor'
alias system-health='dot-doctor && lynis audit system --quick'

# Disk & Memory
alias df='df -h'
alias du='du -h'
alias duh='du -sh * | sort -hr'
alias free='free -h'

# Processes
alias psa='ps auxf'
alias psg='ps aux | grep -v grep | grep -i -e VSZ -e'
alias mem='ps auxf | sort -nr -k 4 | head -10'
alias cpu='ps auxf | sort -nr -k 3 | head -10'

# Network
alias myip='curl -s ifconfig.me'
alias localip='ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v v5.1.0.1'
alias pingg='ping -c 5 google.com'
alias ports='sudo ss -tulanp'
alias listening='sudo lsof -i -P -n | grep LISTEN'
alias weather='curl wttr.in'

# Snapshots
alias snapshots='sudo snapper -c root list'
alias snapshot='sudo snapper -c root create --description'

# ═══════════════════════════════════════════════════════════
# 📝 EDITOR SHORTCUTS
# ═══════════════════════════════════════════════════════════

# Neovim
alias v='nvim'
alias vi='nvim'
alias vim='nvim'
alias nv='nvim'
alias svi='sudo nvim'

# Quick config editing
alias nzsh='nvim ~/.config/zsh/.zshrc'
alias nhypr='nvim ~/.config/hypr/hyprland.conf'
alias nwaybar='nvim ~/.config/waybar/config.jsonc'
alias nwaybar-style='nvim ~/.config/waybar/style.css'
alias nkitty='nvim ~/.config/kitty/kitty.conf'

# LazyVim
alias lazyvim-update='nvim --headless "+Lazy! sync" +qa'
alias lazyvim-clean='nvim --headless "+Lazy! clean" +qa'

# ═══════════════════════════════════════════════════════════
# 🖥️  HYPRLAND & DESKTOP ENVIRONMENT
# ═══════════════════════════════════════════════════════════

# Hyprland
alias hypr-reload='hyprctl reload'
alias hypr-info='hyprctl clients'
alias hypr-windows='hyprctl clients | grep class'

# Waybar
alias waybar-restart='killall waybar && uwsm-app -- waybar > /dev/null 2>&1 & disown'
alias waybar-reload='killall waybar && uwsm-app -- waybar > /dev/null 2>&1 & disown'

# Power management
alias ssn='shutdown now'
alias sr='reboot'
alias logout='hyprctl dispatch exit'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'

# ═══════════════════════════════════════════════════════════
# 🛠️ UTILITIES & QUICK ACTIONS
# ═══════════════════════════════════════════════════════════

# Shell
alias c='clear'
alias h='history'
alias reload='source ~/.config/zsh/.zshrc'
alias path='echo $PATH | tr ":" "\n"'

# Date & Time
alias now='date +"%T"'
alias nowdate='date +"%Y-%m-%d"'
alias timestamp='date +"%Y%m%d_%H%M%S"'

# Sudo shortcuts
alias please='sudo !!'
alias fucking='sudo !!'

# File operations
alias chx='chmod +x'
alias extract='tar -xzvf'
alias targz='tar -czf'
alias untar='tar -xvf'

# Clipboard
alias yp='pwd | wl-copy'
alias yf='basename $PWD | wl-copy'

# Alias help function
alias-help() {
  echo "📋 Alias Categories (188+ total):"
  echo ""
  echo "🔒 Core Protection: lock-core, unlock-core, edit-core"
  echo "📂 Navigation: core, src, work, .., cd ~1"
  echo "📁 File Mgmt: ls, ll, tree, b (bat), search (fd)"
  echo "📦 Packages: pacu, paci, ins, yup, cleanup"
  echo "🔧 Git: lg, gst, gaa, gcm, gp, gl"
  echo "🔍 Core-Diff: cdiff, cds, cdh, cdm, cdhypr"
  echo "💻 System: doctor, ff, df, mem, cpu"
  echo "📝 Editor: v, nzsh, nhypr, nwaybar"
  echo "🖥️  Desktop: hypr-reload, waybar-restart"
  echo "🔐 Security: audit-quick, scan-secrets"
  echo "📚 Docs: keys, guide, roadmap"
  echo ""
  echo "📖 Full reference: bat ~/0-core/docs/ALIASES.md"
  echo "🔍 Search: alias | grep <keyword>"
}

# ═══════════════════════════════════════════════════════════
# 🔐 SECURITY & AUDITING
# ═══════════════════════════════════════════════════════════

# Lynis audits
alias audit-full='sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias audit-quick='sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias security-score='test -f ~/.lynis-score && echo "🛡️  Hardening Index: $(cat ~/.lynis-score)/100" || echo "Run audit-full or audit-quick first"'
alias security-check='sudo pacman -Syu && echo "---" && arch-audit && echo "---" && audit-quick'

# Secret scanning
alias scan-secrets='gitleaks detect --no-git -v'
alias scan-staged='gitleaks protect --staged -v'

# Fail2ban
alias jail-status='sudo fail2ban-client status'
alias ban-list='sudo fail2ban-client status sshd'

# ═══════════════════════════════════════════════════════════
# 📚 FAELIGHT FOREST DOCUMENTATION
# ═══════════════════════════════════════════════════════════

# Quick reference
alias keys='bat ~/0-core/docs/KEYBINDINGS.md'
alias guide='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md'
alias faelight='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md'

# Planning
alias roadmap='nvim ~/0-core/docs/planning/ROADMAP.md'
alias ideas='nvim ~/0-core/docs/planning/ROADMAP.md'
alias planning='cd ~/0-core/docs/planning && ls'

# ═══════════════════════════════════════════════════════════
# 💼 PRODUCTIVITY APPS
# ═══════════════════════════════════════════════════════════

# Notes
notes() {
    uwsm-app -- notesnook >/dev/null 2>&1 &
    disown
}

notesnook() {
    uwsm-app -- notesnook >/dev/null 2>&1 &
    disown
}

# Password manager
kp() {
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
}

keepass() {
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
}

pass() {
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
}

# ═══════════════════════════════════════════════════════════
# 🌐 WEB & BROWSERS
# ═══════════════════════════════════════════════════════════

# AI Assistants
alias chatgpt='xdg-open "https://chat.openai.com"'
alias claude='xdg-open "https://claude.ai"'

# Common sites
alias youtube='xdg-open "https://youtube.com"'
alias gmail='xdg-open "https://gmail.com"'

# Qutebrowser
alias qb='qutebrowser'
alias qute='qutebrowser'

# ═══════════════════════════════════════════════════════════
# 🔄 UPDATE FUNCTIONS
# ═══════════════════════════════════════════════════════════

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

# ═══════════════════════════════════════════════════════════
# 🛠️ 0-CORE UTILITY FUNCTIONS
# ═══════════════════════════════════════════════════════════

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

# dot-doctor wrapper (calls the script in scripts/)
dot-doctor() {
    ~/0-core/scripts/dot-doctor "$@"
}

# ═══════════════════════════════════════════════════════════
# 🔒 Git Guardrails - Prevent dangerous git operations
# ═══════════════════════════════════════════════════════════
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
      
    push)
      # Warn on push to main
      local branch=$(command git symbolic-ref --short HEAD 2>/dev/null)
      if [[ "$branch" == "main" ]]; then
        echo "⚠️  Pushing directly to MAIN in 0-core"
        echo ""
        read "ans?Proceed? (type 'push-main'): "
        if [[ "$ans" != "push-main" ]]; then
          echo "❌ Push cancelled"
          return 1
        fi
      fi
      ;;
  esac
  
  # Execute the actual git command
  command git "$@"
}

# Escape hatch - bypass guardrails
alias git!='/usr/bin/git'

# ═══════════════════════════════════════════════════════════
# 🔐 DIRENV (Per-Directory Environments)
# ═══════════════════════════════════════════════════════════

# Initialize direnv
eval "$(direnv hook zsh)"

# Security aliases
alias envrc-check='bat .envrc'
alias envrc-inspect='bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow"'
alias envrc-allow='direnv allow'
alias envrc-deny='direnv deny'
alias envrc-status='direnv status'

# ═══════════════════════════════════════════════════════════
# 🎨 ZSH PLUGINS & ENHANCEMENTS
# ═══════════════════════════════════════════════════════════

# Autosuggestions (Fish-like) - BRIGHTER COLOR
source ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh
ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'  # Brighter gray (was too dim)

# Syntax highlighting (Fish-like) - Faelight Forest colors
source ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
source ~/.config/zsh/completions.zsh


# ═══════════════════════════════════════════════════════════
# ⚠️  DANGEROUS COMMAND HIGHLIGHTING (v5.1.0)
# Visual warnings for destructive commands
# ═══════════════════════════════════════════════════════════

# Enable pattern highlighting
ZSH_HIGHLIGHT_HIGHLIGHTERS=(main brackets pattern)

# Dangerous patterns - RED background
typeset -A ZSH_HIGHLIGHT_PATTERNS

# rm variants (with and without sudo/command prefix)
ZSH_HIGHLIGHT_PATTERNS+=('rm -rf *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('rm -fr *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('rm -rf /*' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo rm -rf *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo rm -fr *' 'fg=white,bold,bg=red')

# chmod 777 (world-writable)
ZSH_HIGHLIGHT_PATTERNS+=('chmod 777 *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('chmod -R 777 *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo chmod 777 *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo chmod -R 777 *' 'fg=white,bold,bg=red')

# dd (disk destroyer)
ZSH_HIGHLIGHT_PATTERNS+=('dd if=*' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo dd if=*' 'fg=white,bold,bg=red')

# Filesystem formatters
ZSH_HIGHLIGHT_PATTERNS+=('mkfs*' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo mkfs*' 'fg=white,bold,bg=red')

# Additional dangerous commands
ZSH_HIGHLIGHT_PATTERNS+=(':(){ :|:& };:' 'fg=white,bold,bg=red')  # Fork bomb
ZSH_HIGHLIGHT_PATTERNS+=('> /dev/sda' 'fg=white,bold,bg=red')     # Disk overwrite
ZSH_HIGHLIGHT_PATTERNS+=('chmod -R 000 *' 'fg=white,bold,bg=red') # Permission nuke

# Faelight Forest syntax colors
ZSH_HIGHLIGHT_STYLES[command]='fg=cyan,bold'           # Commands in cyan
ZSH_HIGHLIGHT_STYLES[alias]='fg=cyan,bold'             # Aliases in cyan
ZSH_HIGHLIGHT_STYLES[builtin]='fg=cyan,bold'           # Builtins in cyan
ZSH_HIGHLIGHT_STYLES[function]='fg=cyan,bold'          # Functions in cyan
ZSH_HIGHLIGHT_STYLES[arg0]='fg=cyan,bold'              # First command in cyan
ZSH_HIGHLIGHT_STYLES[default]='fg=green'               # Default text (like "status")
ZSH_HIGHLIGHT_STYLES[unknown-token]='fg=red,bold'      # Errors in red
ZSH_HIGHLIGHT_STYLES[reserved-word]='fg=magenta'       # Keywords
ZSH_HIGHLIGHT_STYLES[suffix-alias]='fg=cyan,bold'      # Suffix aliases
ZSH_HIGHLIGHT_STYLES[precommand]='fg=cyan,underline'   # Precommands
ZSH_HIGHLIGHT_STYLES[path]='fg=green'                  # Paths in green
ZSH_HIGHLIGHT_STYLES[path_pathseparator]='fg=green'    # Path separators
ZSH_HIGHLIGHT_STYLES[path_prefix]='fg=green'           # Path prefixes
ZSH_HIGHLIGHT_STYLES[globbing]='fg=yellow'             # Globs in yellow
ZSH_HIGHLIGHT_STYLES[single-hyphen-option]='fg=green'  # Options like -h
ZSH_HIGHLIGHT_STYLES[double-hyphen-option]='fg=green'  # Options like --help
ZSH_HIGHLIGHT_STYLES[back-quoted-argument]='fg=yellow' # Backticks
ZSH_HIGHLIGHT_STYLES[single-quoted-argument]='fg=yellow'   # Single quotes
ZSH_HIGHLIGHT_STYLES[double-quoted-argument]='fg=yellow'   # Double quotes
ZSH_HIGHLIGHT_STYLES[dollar-quoted-argument]='fg=yellow'   # Dollar quotes
ZSH_HIGHLIGHT_STYLES[dollar-double-quoted-argument]='fg=yellow' # Variables in quotes
ZSH_HIGHLIGHT_STYLES[assign]='fg=magenta'              # Variable assignments
ZSH_HIGHLIGHT_STYLES[redirection]='fg=magenta'         # Redirections
ZSH_HIGHLIGHT_STYLES[comment]='fg=240'                 # Comments (dim)

# Starship prompt
eval "$(starship init zsh)"

# ═══════════════════════════════════════════════════════════
# 🚀 WELCOME MESSAGE
# ═══════════════════════════════════════════════════════════

if [[ -o interactive ]]; then
    # Show fastfetch
    fastfetch
    
    # Custom greeting with dynamic latest update
    echo ""
    echo -e "\033[1;32m🌲 Welcome to Faelight Forest v5.1.0!\033[0m"
    
    # Show latest package update (if script exists)
    if [[ -x ~/0-core/scripts/latest-update ]]; then
        local latest=$(~/0-core/scripts/latest-update)
        if [[ -n "$latest" ]]; then
            echo -e "\033[0;36m   Latest: $latest\033[0m"
        fi
    fi
    
    echo ""
    echo "This is my Happy Place!!!"
    echo ""
    echo "💡 Quick: doctor | health | intent list | keys"
    echo ""
fi

# ═══════════════════════════════════════════════════════════
# ⚠️  DANGEROUS COMMAND WARNING (v5.1.0)
# Shows warning BEFORE execution - doesn't block, just warns
# ═══════════════════════════════════════════════════════════

dangerous_command_warning() {
    local cmd="$1"
    
    # Patterns to warn about (regex)
    local -a dangerous_patterns=(
        'rm[[:space:]]+-[^[:space:]]*[rf][^[:space:]]*[rf]'  # rm -rf, rm -fr, rm -rf -v, etc.
        'rm[[:space:]].*[[:space:]]/($|[[:space:]])'         # rm anything ending with /
        'chmod[[:space:]].*777'                               # chmod 777
        'chmod[[:space:]]+-R[[:space:]]+000'                  # chmod -R 000
        'dd[[:space:]]+if='                                   # dd if=
        'mkfs\.'                                              # mkfs.*
        '>[[:space:]]*/dev/[sh]d[a-z]'                        # > /dev/sda
        'mv[[:space:]]+.+[[:space:]]+/dev/null'               # mv to /dev/null
        ':\(\)\{[[:space:]]*:\|:&[[:space:]]*\};:'            # Fork bomb
    )
    
    for pattern in "${dangerous_patterns[@]}"; do
        if [[ "$cmd" =~ $pattern ]]; then
            echo ""
            echo -e "\033[1;37;41m ⚠️  DANGEROUS COMMAND DETECTED \033[0m"
            echo -e "\033[1;31m    $cmd\033[0m"
            echo -e "\033[0;33m    Press Enter to execute, Ctrl+C to abort\033[0m"
            echo ""
            return 0
        fi
    done
}

# Hook into preexec
preexec_functions+=(dangerous_command_warning)

# ═══════════════════════════════════════════════════════════
# 🌲 END OF FAELIGHT FOREST CONFIGURATION
# Version v5.1.0 - Zsh Migration Edition
# ═══════════════════════════════════════════════════════════
export PATH="$HOME/.cargo/bin:$PATH"
