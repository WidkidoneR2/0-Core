# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - FISH SHELL CONFIGURATION
# Version 2.7 - Enhanced & Protected Edition
# Clean, organized, and beautiful
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# 🛡️ PROTECTION & ERROR HANDLING (NEW!)
# ═══════════════════════════════════════════════════════════

# Disable greeting (we have custom one below)
set -g fish_greeting

# Enable better glob handling (Fish 3.0+)
set fish_features qmark-noglob 3.0

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

# Add local bin to PATH
fish_add_path ~/.local/bin
fish_add_path ~/bin
set -gx PATH ~/.local/bin $PATH

# ═══════════════════════════════════════════════════════════
# 📂 FILE NAVIGATION (Modern ls with exa/eza)
# ═══════════════════════════════════════════════════════════

# Basic listings
alias ls='eza --icons --group-directories-first'
alias ll='eza -lah --icons --group-directories-first --git'
alias la='eza -a --icons --group-directories-first'
alias l='eza -lh --icons --group-directories-first'
alias lt='eza -lah --icons --sort=modified --reverse'
alias lsize='eza -lah --icons --sort=size --reverse'
alias tree='eza --tree --icons --group-directories-first'

# Directory navigation
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias cdp='cd -'

# Quick directory jumps
alias desk='cd ~/Desktop'
alias docs='cd ~/Documents'
alias down='cd ~/Downloads'
alias pics='cd ~/Pictures'
alias vids='cd ~/Videos'
alias conf='cd ~/.config'
alias hyprconf='cd ~/.config/hypr'
alias nvimconf='cd ~/.config/nvim'
alias fishconf='cd ~/.config/fish'

# ═══════════════════════════════════════════════════════════
# 📦 PACKAGE MANAGEMENT (Arch/Pacman/Yay)
# ═══════════════════════════════════════════════════════════

# Pacman shortcuts
alias pacu='sudo pacman -Syu'
alias paci='sudo pacman -S'
alias pacs='pacman -Ss'
alias pacr='sudo pacman -R'
alias pacrem='sudo pacman -Rns'
alias pacinfo='pacman -Qi'
alias paclist='pacman -Qqe'

# Yay shortcuts
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
alias update-all='yay -Syu $argv'
alias clean-all='yay -Sc && yay -Yc && sudo pacman -Rns (pacman -Qtdq) 2>/dev/null || true'
alias fix-keys='sudo pacman-key --init && sudo pacman-key --populate && sudo pacman-key --refresh-keys'

# ═══════════════════════════════════════════════════════════
# 🔧 GIT SHORTCUTS
# ═══════════════════════════════════════════════════════════

# LazyGit (best way!)
alias lg='lazygit'

# Basic git
alias g='git'
alias gs='git status'
alias gss='git status -s'

# Logs
alias gl='git log --oneline --graph -10'
alias gla='git log --oneline --graph --all'
alias glog='git log --oneline --graph --all --decorate'

# Add and commit
alias ga='git add'
alias gaa='git add --all'
alias gc='git commit -m'
alias gca='git commit --amend'
alias gcam='git commit -am'

# Push and pull
alias gp='git push'
alias gpl='git pull'
alias gf='git fetch'

# Branches
alias gb='git branch'
alias gba='git branch -a'
alias gbd='git branch -d'
alias gbD='git branch -D'
alias gco='git checkout'
alias gcb='git checkout -b'
alias gcm='git checkout main'

# Diff and show
alias gd='git diff'
alias gds='git diff --staged'
alias gsh='git show'
alias gd='git diff --color=always'
alias gdp='git diff --color=always | less -R'

# Stash
alias gst='git stash'
alias gstp='git stash pop'
alias gstl='git stash list'
alias gstd='git stash drop'

# Undo/Reset
alias gundo='git reset HEAD~1'
alias gunstage='git reset HEAD'
alias greset='git reset --hard'
alias gclean='git clean -fd'

# Clone
alias gcl='git clone'

# Dotfiles management
alias dotfiles='cd ~/dotfiles'
alias dotsave='cd ~/dotfiles && git add -A && git commit -m "Update dotfiles" && git push'
alias dotpull='cd ~/dotfiles && git pull'
alias dotlog='cd ~/dotfiles && git log --oneline -10'

# ═══════════════════════════════════════════════════════════
# 💻 SYSTEM MONITORING & INFO
# ═══════════════════════════════════════════════════════════

# System info
alias ff='fastfetch'
alias neofetch='fastfetch'
alias df='df -h'
alias du='du -h'
alias duh='du -sh * | sort -hr'
alias free='free -h'
alias psa='ps auxf'
alias weather='curl wttr.in'

# Process management
alias psg='ps aux | grep -v grep | grep -i -e VSZ -e'
alias mem='ps auxf | sort -nr -k 4 | head -10'
alias cpu='ps auxf | sort -nr -k 3 | head -10'

# Network
alias myip='curl -s ifconfig.me'
alias localip='ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v 127.0.0.1'
alias pingg='ping -c 5 google.com'
alias ports='sudo ss -tulanp'
alias listening='sudo lsof -i -P -n | grep LISTEN'

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

# LazyVim management
alias lazyvim-update='nvim --headless "+Lazy! sync" +qa'
alias lazyvim-clean='nvim --headless "+Lazy! clean" +qa'

# Yazi file manager
alias y='yazi'

# ═══════════════════════════════════════════════════════════
# 🔍 MODERN CLI TOOLS
# ═══════════════════════════════════════════════════════════

# bat (better cat)
alias cat='bat --paging=never'
alias catp='bat --paging=always'
alias catt='bat --style=plain'

# fd (better find)
alias search='fd'
alias findf='fd --type f'
alias findd='fd --type d'

# fzf (fuzzy finder)
alias fcd='cd (fd --type d | fzf)'
alias vf='nvim (fd --type f | fzf)'
alias preview='fzf --preview "bat --color=always {}"'

# ═══════════════════════════════════════════════════════════
# 📁 FILE MANAGERS
# ═══════════════════════════════════════════════════════════

# Thunar (File Manager) - Run in the background
function thunar
    command thunar $argv &>/dev/null &
end

# Meld - Visual diff/merge tool (only if installed)
if type -q meld
    alias compare='meld'
    alias diff-dirs='meld' # Compare two directories
    alias merge='meld' # 3-way merge

    # 🔍 Dotfiles Verification with Meld
    alias verify-hypr='meld ~/.config/hypr ~/dotfiles/hypr/.config/hypr'
    alias verify-waybar='meld ~/.config/waybar ~/dotfiles/waybar/.config/waybar'
    alias verify-kitty='meld ~/.config/kitty ~/dotfiles/kitty/.config/kitty'
    alias verify-fish='meld ~/.config/fish ~/dotfiles/fish/.config/fish'
    alias verify-nvim='meld ~/.config/nvim ~/dotfiles/nvim/.config/nvim'
    alias verify-all='meld ~/.config ~/dotfiles'
end

# Yazi (terminal file manager)
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
# 🖥️  HYPRLAND & WAYBAR
# ═══════════════════════════════════════════════════════════

# Hyprland controls
alias hypr-reload='hyprctl reload'
alias hypr-info='hyprctl clients'
alias hypr-windows='hyprctl clients | grep class'

# Waybar
alias waybar-restart='killall waybar && uwsm-app -- waybar > /dev/null 2>&1 & disown'
alias waybar-reload='killall waybar && uwsm-app -- waybar > /dev/null 2>&1 & disown'

# ═══════════════════════════════════════════════════════════
# ⚡ POWER MANAGEMENT
# ═══════════════════════════════════════════════════════════

alias ssn='shutdown now'
alias sr='reboot'
alias logout='hyprctl dispatch exit'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'

# ═══════════════════════════════════════════════════════════
# 🛠️ QUICK UTILITIES
# ═══════════════════════════════════════════════════════════

# Common actions
alias c='clear'
alias h='history'
alias reload='source ~/.config/fish/config.fish'
alias path='echo $PATH | tr " " "\n"'
alias now='date +"%T"'
alias nowdate='date +"%Y-%m-%d"'
alias timestamp='date +"%Y%m%d_%H%M%S"'
alias please='sudo (history | head -1)'
alias fucking='sudo (history | head -1)'

# File operations
alias chx='chmod +x'
alias extract='tar -xzvf'
alias targz='tar -czf'
alias untar='tar -xvf'

# Copy/paste helpers
alias yp='pwd | wl-copy'
alias yf='basename $PWD | wl-copy'

# ═══════════════════════════════════════════════════════════
# 🌐 WEB & COMMUNICATION
# ═══════════════════════════════════════════════════════════

# AI Assistants
alias chatgpt='xdg-open "https://chat.openai.com"'
alias claude='xdg-open "https://claude.ai"'

# Common sites
alias youtube='xdg-open "https://youtube.com"'
alias gmail='xdg-open "https://gmail.com"'

# Communication
alias signal='signal-desktop'

# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST QUICK ACCESS
# ═══════════════════════════════════════════════════════════

# Documentation shortcuts
alias guide='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md'
alias faelight='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md'
alias vguide='nvim ~/faelight-forest-docs/COMPLETE_GUIDE.md'

# 📋 Planning & Ideas (Local Only)
alias roadmap='nvim ~/dotfiles/docs/planning/ROADMAP.md'
alias ideas='nvim ~/dotfiles/docs/planning/ROADMAP.md'
alias planning='cd ~/dotfiles/docs/planning && ls'

# 🎨 Theme Management
alias theme-dark='~/dotfiles/scripts/theme-switch.sh dark'
alias theme-light='~/dotfiles/scripts/theme-switch.sh light'
alias theme-toggle='~/dotfiles/scripts/theme-switch.sh toggle'
alias theme='~/dotfiles/scripts/theme-switch.sh status'

# Dotfiles management
alias dotfiles='cd ~/dotfiles'
alias dotbackup='cd ~/dotfiles && ./backup.sh'
alias dotpush='cd ~/dotfiles && git add -A && git commit -m "Update dotfiles $(date +%Y-%m-%d)" && git push'
alias dotpull='cd ~/dotfiles && git pull && ./install.sh'
alias dotstatus='cd ~/dotfiles && git status'
alias sync-dotfiles='cd ~/dotfiles && git add -A && git commit -m "🌲 Auto-sync dotfiles $(date +%Y-%m-%d)" && git push origin main'

# Faelight Forest Blueprint
alias safe-update='~/.local/bin/safe-update'
alias save-packages='~/.local/bin/save-packages'
alias auto-sync='~/.local/bin/auto-sync'
alias snapshots='sudo snapper -c root list'
alias snapshot='sudo snapper -c root create --description'

# Quick reference sections (FIXED for v2.5+)
alias keys='bat ~/dotfiles/docs/KEYBINDINGS.md'
alias fishhelp='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md | grep -A 200 "Fish Shell Reference"'
alias vimhelp='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md | grep -A 200 "LazyVim Reference"'
alias workspaces='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md | grep -A 100 "Workspaces"'
alias colors='bat ~/faelight-forest-docs/COMPLETE_GUIDE.md | grep -A 10 "Faelight Forest Colors"'

# System health check
alias health='echo "=== SYSTEM HEALTH ===" && btop --quit-after-cycles 1 2>/dev/null || true && echo && df -h / && echo && free -h && echo && uptime'
alias sysinfo='fastfetch'

# Faelight docs directory
alias faedocs='cd ~/faelight-forest-docs && ll'

# Date management
alias update-dates='~/.local/bin/update-dates'

# 🛡️ Gitleaks - Secret Scanning
alias scan-secrets='gitleaks detect --no-git -v'
alias scan-staged='gitleaks protect --staged -v'

# Security audit
alias audit-secrets='cd ~/dotfiles && echo "🔍 Scanning for secrets..." && grep -r "password\|api_key\|secret.*=\|token.*=" . --exclude-dir=.git --exclude=.gitignore -i | grep -v "Binary" || echo "✅ No secrets found!"'

# ═══════════════════════════════════════════════════════════
# 🔒 Security Audit Aliases (v2.7.2)
# ═══════════════════════════════════════════════════════════

# Full audit (saves score to file)
alias audit-full='sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'

# Quick audit (saves score to file)  
alias audit-quick='sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'

# Show saved score (instant!)
alias security-score='test -f ~/.lynis-score && echo "🛡️  Hardening Index: "(cat ~/.lynis-score)"/100" || echo "Run audit-full or audit-quick first"'

# Weekly security routine
alias security-check='sudo pacman -Syu && echo "---" && arch-audit && echo "---" && audit-quick'

# Individual checks
alias vuln-check='arch-audit | grep -i "High risk"'

# Fail2ban monitoring
alias jail-status='sudo fail2ban-client status'
alias ban-list='sudo fail2ban-client status sshd'

# ═══════════════════════════════════════════════════════════
# 🔍 Dotfile Intelligence Tools (v2.8.0)
# ═══════════════════════════════════════════════════════════

alias doctor='dot-doctor'
alias health-check='dot-doctor'
alias keys-check='keyscan'

# ═══════════════════════════════════════════════════════════
# 📝 Productivity Apps (v2.7.2)
# ═══════════════════════════════════════════════════════════

# Note-taking
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
# 🎨 FISH SHELL COLORS (Faelight Forest Theme)
# ═══════════════════════════════════════════════════════════

# Syntax highlighting colors
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

# Pager colors
set -g fish_pager_color_progress 5bb7a5
set -g fish_pager_color_prefix 5bb7a5 --bold
set -g fish_pager_color_completion e8f5d5
set -g fish_pager_color_description 557d68

# ═══════════════════════════════════════════════════════════
# 🚀 WELCOME MESSAGE
# ═══════════════════════════════════════════════════════════

if status is-interactive
    # Show fastfetch on new shell
    fastfetch
    # Custom greeting
    echo ""
    set_color -o 00ff00
    echo "🌲 Welcome to Faelight Forest v2.8.0!"
    set_color normal
    echo "This is my Happy Place!!!"
    echo ""
    echo "💡 Quick commands: guide | doctor | alias | health | roadmap"
end

set -g fish_greeting ""

# Apply theme colors on shell start
set_fish_colors

# ═══════════════════════════════════════════════════════════
# 🌲 END OF FAELIGHT FOREST CONFIGURATION
# Version 2.8.0 - Enhanced & Protected Edition
# ═══════════════════════════════════════════════════════════
