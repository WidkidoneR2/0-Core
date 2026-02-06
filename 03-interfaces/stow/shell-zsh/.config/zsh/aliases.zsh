# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH ALIASES
# Version 8.6.0 - Complete Alias Collection
# Total: ~270 aliases organized by category
# ═══════════════════════════════════════════════════════════

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
alias update='safe-update'

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
alias swayconf='cd ~/.config/sway'
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

# Bat (better cat)
alias b='bat --paging=never'
alias catp='bat --paging=always'
alias catt='bat --style=plain'
alias ccat='/usr/bin/cat'

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

# ═══════════════════════════════════════════════════════════
# 📦 PACKAGE MANAGEMENT (Arch/Pacman/Yay)
# ═══════════════════════════════════════════════════════════
# Pacman
alias pacu='paru -Syu'             # Update system
alias paci='paru -S'               # Install package
alias pacs='pacman -Ss'
alias pacr='paru -R'               # Remove package
alias pacrem='sudo pacman -Rns'
alias pacinfo='pacman -Qi'
alias paclist='pacman -Qqe'

# Yay
alias yay='paru --color=auto'
alias yayu='paru -Syu'
alias yays='paru -Ss'
alias yayi='paru -S'
alias yayr='paru -R'
alias ins='paru -S'                # Install package
alias uns='paru -Rns'              # Uninstall + remove deps
alias yup='paru -Syu'

# Maintenance
alias cleanup='paru -Rns $(paru -Qtdq) 2>/dev/null || true'
alias unlock='sudo rm /var/lib/pacman/db.lck'
alias orphans='pacman -Qtdq'
alias mirror='sudo reflector --verbose --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'
alias clean-all='paru -Sc && paru -Yc'
alias fix-keys='sudo pacman-key --init && sudo pacman-key --populate && sudo pacman-key --refresh-keys'

# ═══════════════════════════════════════════════════════════
# 🔧 GIT & VERSION CONTROL
# ═══════════════════════════════════════════════════════════
# LazyGit
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
alias cdcore='cd ~/0-core'
alias dotgit='cd ~/0-core && git'

# Escape hatch
alias git!='/usr/bin/git'

# ═══════════════════════════════════════════════════════════
# 🔍 CORE-DIFF ALIASES (Quick Access)
# ═══════════════════════════════════════════════════════════
alias cdiff='core-diff'
alias cds='core-diff summary'
alias cdh='core-diff --high-risk'
alias cdv='core-diff --verbose'
alias cdm='core-diff --open meld'
alias cdd='core-diff --open delta'
alias cdlast='core-diff since HEAD~1'
alias cdrel='core-diff since $(git describe --tags --abbrev=0 2>/dev/null || echo HEAD)'
alias cdsway='core-diff wm-sway'
alias cdbar='core-diff faelight-bar'
alias cdzsh='core-diff shell-zsh'
alias cdnvim='core-diff editor-nvim'
alias cdcheck='cdiff && dot-doctor'
alias cdreview='cdv && cdh'

# ═══════════════════════════════════════════════════════════
# 💻 SYSTEM MONITORING & HEALTH
# ═══════════════════════════════════════════════════════════
# System info
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
alias snapper-create='sudo snapper -c root create --description'

# ═══════════════════════════════════════════════════════════
# 📝 EDITOR SHORTCUTS
# ═══════════════════════════════════════════════════════════
alias v='nvim'
alias vi='nvim'
alias vim='nvim'
alias nv='nvim'
alias svi='sudo nvim'

# Quick config editing
alias nzsh='nvim ~/.config/zsh/.zshrc'
alias nsway='nvim ~/.config/sway/config'
alias nbar='nvim ~/0-core/rust-tools/faelight-bar/src/main.rs'

# LazyVim
alias lazyvim-update='nvim --headless "+Lazy! sync" +qa'
alias lazyvim-clean='nvim --headless "+Lazy! clean" +qa'

# Vim Distros (Universal File Management)
alias lazy='nvim'                              # LazyVim (default)
alias astro='NVIM_APPNAME=astronvim nvim'     # AstroVim
alias chad='NVIM_APPNAME=nvchad nvim'         # NvChad

# ═══════════════════════════════════════════════════════════
# 🖥️  SWAY & DESKTOP ENVIRONMENT
# ═══════════════════════════════════════════════════════════
alias sway-reload='swaymsg reload'
alias sway-info='swaymsg -t get_tree'
alias bar-restart='pkill faelight-bar; ~/0-core/scripts/faelight-bar & disown'

# Power management
alias ssn='shutdown now'
alias sr='reboot'
alias logout='swaymsg exit'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'

# ═══════════════════════════════════════════════════════════
# 🛠️ UTILITIES & QUICK ACTIONS
# ═══════════════════════════════════════════════════════════
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

# ═══════════════════════════════════════════════════════════
# 🔐 SECURITY & AUDITING
# ═══════════════════════════════════════════════════════════
alias audit-full='sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias audit-quick='sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias security-score='test -f ~/.lynis-score && echo "🛡️  Hardening Index: $(cat ~/.lynis-score)/100" || echo "Run audit-full or audit-quick first"'
alias security-check='sudo pacman -Syu && echo "---" && arch-audit && echo "---" && audit-quick'
alias scan-secrets='gitleaks detect --no-git -v'
alias scan-staged='gitleaks protect --staged -v'
alias jail-status='sudo fail2ban-client status'
alias ban-list='sudo fail2ban-client status sshd'

# ═══════════════════════════════════════════════════════════
# 📚 FAELIGHT FOREST DOCUMENTATION
# ═══════════════════════════════════════════════════════════
alias keys='bat ~/0-core/docs/KEYBINDINGS.md'
alias guide='bat ~/0-core/COMPLETE_GUIDE.md'
alias roadmap='nvim ~/0-core/docs/planning/ROADMAP.md'
alias planning='cd ~/0-core/docs/planning && ls'

# ═══════════════════════════════════════════════════════════
# 🌐 WEB & BROWSERS
# ═══════════════════════════════════════════════════════════
alias chatgpt='xdg-open "https://chat.openai.com"'
alias claude='xdg-open "https://claude.ai"'
alias youtube='xdg-open "https://youtube.com"'
alias gmail='xdg-open "https://gmail.com"'

# ═══════════════════════════════════════════════════════════
# 🔐 DIRENV (Per-Directory Environments)
# ═══════════════════════════════════════════════════════════
alias envrc-check='bat .envrc'
alias envrc-inspect='bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow"'
alias envrc-allow='direnv allow'
alias envrc-deny='direnv deny'
alias envrc-status='direnv status'

# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT TOOLS
# ═══════════════════════════════════════════════════════════
alias dashboard='faelight-dashboard'
alias dash='faelight-dashboard'
alias d='doctor'                    # System health check
alias snap='faelight-snapshot'
alias snapshot='faelight-snapshot'
alias snaplist='faelight-snapshot list'
alias snapcreate='faelight-snapshot create'
alias stow-check='faelight-stow'
alias stow-fix='faelight-stow --fix'
alias launcher='faelight-launcher'
alias powermenu='faelight-menu'
alias secrets-mount='gocryptfs ~/secrets.encrypted ~/secrets && echo "🔓 Secrets mounted"'
alias secrets-unmount='fusermount -u ~/secrets && echo "🔒 Secrets locked"'
alias secrets='cd ~/secrets'
alias entropy='entropy-check'
alias drift='entropy-check'
alias f='faelight'
alias lock='faelight-lock'
alias bump='bump-system-version'
alias prof='profile'
alias prof-list='profile list'
alias prof-switch='profile switch'
alias ff='faelight-fetch'
alias bar='faelight-bar'
alias f-bar='faelight-bar'
alias f-launch='faelight-launcher'
alias dmenu='faelight-dmenu'
alias f-dmenu='faelight-dmenu'
alias notify='faelight-notify'
alias f-notify='faelight-notify'
alias f-lock='faelight-lock'
alias menu='faelight-menu'
alias f-menu='faelight-menu'
alias term='faelight-term'
alias f-term='faelight-term'
alias guard='intent-guard'
alias f-guard='intent-guard'
alias bootstrap='faelight-bootstrap'
alias f-bootstrap='faelight-bootstrap'

# ═══════════════════════════════════════════════════════════
# 🔧 ARCHAEOLOGY & WORKSPACE TOOLS
# ═══════════════════════════════════════════════════════════
alias arch='archaeology-0-core'
alias arch0='archaeology-0-core'
alias archtime='archaeology-0-core --timeline'
alias archwk='archaeology-0-core --this-week'
alias archint='archaeology-0-core --by-intent'
alias archsince='archaeology-0-core --since'
alias ws='workspace-view'
alias wsa='workspace-view --active'
alias wss='workspace-view --summary'

# ═══════════════════════════════════════════════════════════
# 🆕 NEW TOOL ALIASES (v8.6.0)
# ═══════════════════════════════════════════════════════════
# Faelight Git Workflow
alias fg='faelight-git'
alias fgc='faelight-git commit'
alias fgs='faelight-git status'
alias fga='faelight-git add'
alias fgp='faelight-git push'

# Version & Update Info
alias latest='latest-update'
alias lastup='latest-update'
alias forest-ver='echo "🌲 Faelight Forest v8.0.0"'
alias sysver='uname -r'

# Keybinding & Config Analysis
alias keybinds='keyscan'
alias conflicts='keyscan'

# Stow Management (dotctl)
alias dot='dotctl'
alias dotadd='dotctl add'
alias dotrem='dotctl remove'
alias dotlist='dotctl list'

# Changelog & Documentation
alias changelog='bat ~/0-core/CHANGELOG.md'
alias changelog-draft='bat ~/0-core/CHANGELOG-v8.0.0-DRAFT.md'
alias compile-log='~/0-core/scripts/compile-changelog.sh'
alias mklog='~/0-core/scripts/compile-changelog.sh'

# Intent Ledger
alias int='intent'
alias intl='intent list'
alias ints='intent show'
alias inta='intent add'
alias intc='intent complete'

# ⭐ Production v1.0.0 Tools (NEW!)
alias fm='faelight-fm'          # File manager v1.0.0
alias f-fm='faelight-fm'
alias fl='faelight-link'        # Symlink manager v1.0.0
alias f-link='faelight-link'
alias link='faelight-link'
alias zone='faelight-zone'
alias f-zone='faelight-zone'
alias hooks='faelight-hooks'
alias f-hooks='faelight-hooks'
alias recent='recent-files'
alias f-recent='recent-files'
alias ver='get-version'
alias getver='get-version'
alias f-ver='get-version'
alias t='teach'

# ═══════════════════════════════════════════════════════════
# 🚀 COMPOSITE WORKFLOW ALIASES (Multi-tool Chains)
# ═══════════════════════════════════════════════════════════
alias pre-commit='echo "🔍 Pre-commit checks..." && gitleaks protect --staged -v && dot-doctor && echo "✅ Safe to commit!"'
alias audit='echo "🏥 Running full audit..." && dot-doctor && entropy-check && security-score'
alias full-audit='dot-doctor && entropy-check && security-check'
alias release-prep='echo "📦 Preparing release..." && bump-system-version && compile-changelog.sh && git status'
alias status='dot-doctor && echo "" && git status'
alias overview='fastfetch && echo "" && dot-doctor && echo "" && git -C ~/0-core status -s'
alias snap-now='faelight-snapshot create "Manual snapshot at $(date +%Y%m%d_%H%M%S)"'
alias snap-before='echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created! Proceed with operation."'
alias safe-up='snap-now && safe-update && dot-doctor'
alias qc='git commit -m "Quick update: $(date +%Y-%m-%d)"'
alias qcp='git commit -m "Quick update: $(date +%Y-%m-%d)" && git push'
alias card='echo "╔════════════════════════════════════════╗" && echo "║  🌲 FAELIGHT FOREST v8.0.0            ║" && echo "║  🏥 Health: $(dot-doctor | grep "Health:" | awk "{print \$2}")                        ║" && echo "║  📦 Tools: 30 Production Ready         ║" && echo "║  🔒 Security: Hardened                 ║" && echo "╚════════════════════════════════════════╝"'

# ═══════════════════════════════════════════════════════════
# 🌲 END OF ALIASES
# ═══════════════════════════════════════════════════════════

# faelight-update
alias fu='faelight-update'           # System update
alias fui='faelight-update --interactive'  # Interactive update
alias fuup='faelight-update'        # Quick update
