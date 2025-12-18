# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - FISH SHELL CONFIGURATION
# Version 3.2.0 - Smart Update Edition
# Clean, organized, and intentional
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# 🛡️ PROTECTION & ERROR HANDLING
# ═══════════════════════════════════════════════════════════

# Disable default greeting (custom one below)
set -g fish_greeting

# Enable better glob handling (Fish 3.0+)
set fish_features qmark-noglob 3.0

# Sudo wrapper (fixes @ in password issues)
function sudo
    command sudo $argv
end

# Prevent accidental shell switches
alias bash='echo "⚠️  You are in Fish! Use: exec fish to reload. Really want bash? Use: command bash"'
alias sh='echo "⚠️  You are in Fish! Really want sh? Use: command sh"'

# Better command not found
function fish_command_not_found
    echo "🐠 Command not found: $argv[1]"
    echo "💡 Check your spelling or install it with: paci $argv[1]"
end

# ═══════════════════════════════════════════════════════════
# 🎨 ENVIRONMENT & PATH
# ═══════════════════════════════════════════════════════════

fish_add_path ~/.local/bin
fish_add_path ~/bin

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
alias fishconf='cd ~/.config/fish'

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

# Bat (better cat)
alias cat='bat --paging=never'
alias catp='bat --paging=always'
alias catt='bat --style=plain'

# Fd (better find)
alias search='fd'
alias findf='fd --type f'
alias findd='fd --type d'

# Fzf (fuzzy finder)
alias fcd='cd (fd --type d | fzf)'
alias vf='nvim (fd --type f | fzf)'
alias preview='fzf --preview "bat --color=always {}"'

# Yazi (file manager)
alias y='yazi'
alias yy='yazi'
alias fm='yazi'

# Yazi with cd-on-quit
function ya
    set tmp (mktemp -t "yazi-cwd.XXXXXX")
    yazi $argv --cwd-file="$tmp"
    if test -f "$tmp"
        if set cwd (cat -- "$tmp"); and [ -n "$cwd" ]; and [ "$cwd" != "$PWD" ]
            cd -- "$cwd"
        end
    end
    rm -f -- "$tmp"
end

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
alias cleanup='sudo pacman -Rns (pacman -Qtdq)'
alias unlock='sudo rm /var/lib/pacman/db.lck'
alias orphans='pacman -Qtdq'
alias mirror='sudo reflector --verbose --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'
alias clean-all='yay -Sc && yay -Yc && sudo pacman -Rns (pacman -Qtdq) 2>/dev/null || true'
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
alias localip='ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v 127.0.0.1'
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
alias nfish='nvim ~/.config/fish/config.fish'
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
alias reload='source ~/.config/fish/config.fish'
alias path='echo $PATH | tr " " "\n"'

# Date & Time
alias now='date +"%T"'
alias nowdate='date +"%Y-%m-%d"'
alias timestamp='date +"%Y%m%d_%H%M%S"'

# Sudo shortcuts
alias please='sudo (history | head -1)'
alias fucking='sudo (history | head -1)'

# File operations
alias chx='chmod +x'
alias extract='tar -xzvf'
alias targz='tar -czf'
alias untar='tar -xvf'

# Clipboard
alias yp='pwd | wl-copy'
alias yf='basename $PWD | wl-copy'

# ═══════════════════════════════════════════════════════════
# 🔐 SECURITY & AUDITING
# ═══════════════════════════════════════════════════════════

# Lynis audits
alias audit-full='sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias audit-quick='sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias security-score='test -f ~/.lynis-score && echo "🛡️  Hardening Index: "(cat ~/.lynis-score)"/100" || echo "Run audit-full or audit-quick first"'
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
function notes
    uwsm-app -- notesnook >/dev/null 2>&1 &
    disown
end

function notesnook
    uwsm-app -- notesnook >/dev/null 2>&1 &
    disown
end

# Password manager
function kp
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
end

function keepass
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
end

function pass
    uwsm-app -- keepassxc >/dev/null 2>&1 &
    disown
end

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
# 🎨 FISH SHELL COLORS (Faelight Forest Theme)
# ═══════════════════════════════════════════════════════════

# Syntax highlighting
set -g fish_color_command 5bb7a5 --bold
set -g fish_color_param e8f5d5
set -g fish_color_error c94c4c
set -g fish_color_normal e8f5d5
set -g fish_color_comment 557d68
set -g fish_color_keyword 8ed1a3
set -g fish_color_quote c7df63
set -g fish_color_redirection 8ed1a3
set -g fish_color_end 5bb7a5
set -g fish_color_operator 8ed1a3
set -g fish_color_escape c7df63
set -g fish_color_autosuggestion 557d68
set -g fish_color_cancel c94c4c
set -g fish_color_search_match --background=2e6146
set -g fish_color_selection --background=2e6146

# Pager
set -g fish_pager_color_progress 5bb7a5
set -g fish_pager_color_prefix 5bb7a5 --bold
set -g fish_pager_color_completion e8f5d5
set -g fish_pager_color_description 557d68

# Apply theme colors
set_fish_colors

# ═══════════════════════════════════════════════════════════
# 🔐 DIRENV (Per-Directory Environments)
# ═══════════════════════════════════════════════════════════

# Initialize direnv
direnv hook fish | source

# Security aliases
alias envrc-check='bat .envrc'
alias envrc-inspect='bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow"'
alias envrc-allow='direnv allow'
alias envrc-deny='direnv deny'
alias envrc-status='direnv status'

# ═══════════════════════════════════════════════════════════
# ⭐ STARSHIP PROMPT
# ═══════════════════════════════════════════════════════════

starship init fish | source

# ═══════════════════════════════════════════════════════════
# 🚀 WELCOME MESSAGE
# ═══════════════════════════════════════════════════════════

if status is-interactive
    # Show fastfetch
    fastfetch

    # Custom greeting with dynamic latest update
    echo ""
    set_color -o 00ff00
    echo "🌲 Welcome to Faelight Forest v3.3.1!"
    set_color normal

    # Show latest package update (if script exists)
    if test -x ~/0-core/scripts/latest-update
        set -l latest (~/0-core/scripts/latest-update)
        if test -n "$latest"
            set_color cyan
            echo "   Latest: $latest"
            set_color normal
        end
    end

    echo ""
    echo "This is my Happy Place!!!"
    echo ""
    echo "💡 Quick: guide | doctor | health | roadmap | keys"
    echo ""
end

# ═══════════════════════════════════════════════════════════
# 🌲 END OF FAELIGHT FOREST CONFIGURATION
# Version 3.2.0 - Smart Update Edition
# ═══════════════════════════════════════════════════════════
