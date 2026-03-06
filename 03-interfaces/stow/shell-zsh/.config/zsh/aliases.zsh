# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH ALIASES
# Version 9.3.0 - Enhanced Organization
# Total: 300+ aliases organized by purpose
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# ⚡ ULTRA-FAST SHORTCUTS (1-letter aliases)
# Quick access to most-used commands for maximum speed
# ═══════════════════════════════════════════════════════════

alias c='clear'                           # Clean terminal
alias d='doctor'                          # System health check
alias f='faelight'                        # Main CLI
alias g='git'                             # Git shortcut
alias h='history'                         # Command history
alias l='eza -lh --icons --group-directories-first'  # List files
alias t='teach'                           # Teaching tool
alias v='nvim'                            # Neovim
alias y='yazi'                            # File manager
alias b='bat --paging=never'              # Bat viewer

# ═══════════════════════════════════════════════════════════
# 🌲 0-CORE TOOLS (Faelight Ecosystem)
# Custom Rust tools for intentional system management
# ═══════════════════════════════════════════════════════════

# ─── Core Management ───
alias bar='faelight-bar'
alias bootstrap='faelight-bootstrap'
alias bump='bump-system-version'
alias dash='faelight-dashboard'
alias dashboard='faelight-dashboard'
alias dmenu='faelight-palette'
alias dot='dotctl'
alias fm='faelight-fm'
alias launcher='faelight-palette'
alias link='faelight-link'
alias lock='faelight-lock'
alias menu='faelight-menu'
alias notify='faelight-notify'
alias term='faelight-term'
alias zone='faelight-zone'

# ─── Health & Monitoring ───
alias doctor='dot-doctor'
alias check-health='dot-doctor'
alias health='dot-doctor'
alias drift='entropy-check'
alias entropy='entropy-check'
alias audit='echo "🏥 Running full audit..." && dot-doctor && entropy-check && security-score'

# ─── Updates & Maintenance ───
alias fu='faelight-update'
alias sec='security-audit'
alias sb='faelight-sandbox'
alias sb-diff='faelight-sandbox diff'
alias sb-status='faelight-sandbox status'
alias sb-clear='faelight-sandbox clear'
alias sb-snap='faelight-sandbox snapshot'
alias sb-snaps='faelight-sandbox snapshots'
alias sb-restore='faelight-sandbox restore'
alias sec-scan='security-audit scan'
alias sec-report='security-audit report'
alias sec-history='security-audit history'
alias sec='security-audit'
alias sb='faelight-sandbox'
alias sb-diff='faelight-sandbox diff'
alias sb-status='faelight-sandbox status'
alias sb-clear='faelight-sandbox clear'
alias sb-snap='faelight-sandbox snapshot'
alias sb-snaps='faelight-sandbox snapshots'
alias sb-restore='faelight-sandbox restore'
alias sec-scan='security-audit scan'
alias sec-report='security-audit report'
alias sec-history='security-audit history'                # System update
alias topgrade='faelight-update'           # Topgrade replaced by faelight-update
alias fudr='faelight-update --dry-run'     # Check updates without applying
alias fui='faelight-update --interactive' # Interactive update
alias fuup='faelight-update'              # Quick update
alias update='safe-update'
alias safe-update='~/0-core/scripts/safe-update'
alias safe-up='snap-now && safe-update && dot-doctor'

# ─── Git & Version Control ───
alias fg='faelight-git'
alias fga='faelight-git add'
alias fgc='faelight-git commit'
alias fgp='faelight-git push'
alias fgs='faelight-git status'
alias hooks='faelight-hooks'

# ─── Protection & Security ───
alias lock-core='~/0-core/scripts/core-protect lock'
alias unlock-core='~/0-core/scripts/core-protect unlock'
alias edit-core='~/0-core/scripts/core-protect edit'
alias core-status='~/0-core/scripts/core-protect status'

# ─── Intent System ───
alias int='intent'
alias inta='intent add'
alias intc='intent complete'
alias intl='intent list'
alias ints='intent show'
alias guard='intent-guard'

# ─── File & Link Management ───
alias fl='faelight-link'
alias stow-check='faelight-link status'
alias stow-fix='faelight-link clean'

# ─── Snapshots & Backups ───
alias snap='faelight-snapshot'
alias snapshot='faelight-snapshot'
alias snapcreate='faelight-snapshot create'
alias snaplist='faelight-snapshot list'
alias snap-now='faelight-snapshot create "Manual snapshot at $(date +%Y%m%d_%H%M%S)"'
alias snap-before='echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created!"'

# ─── Utilities ───
alias ff='faelight-fetch'
alias getver='get-version'
alias ver='get-version'
alias recent='recent-files'

# ─── Shortened Tool Names ───
alias f-bar='faelight-bar'
alias f-daemon='faelight-daemon'
alias daemon-status='systemctl --user status faelight-daemon'
alias daemon-log='journalctl --user -u faelight-daemon -n 20 --no-pager'
alias f-bootstrap='faelight-bootstrap'
alias f-dmenu='faelight-palette'
alias f-fm='faelight-fm'
alias f-guard='intent-guard'
alias f-hooks='faelight-hooks'
alias f-launch='faelight-palette'
alias f-link='faelight-link'
alias f-lock='faelight-lock'
alias f-menu='faelight-menu'
alias f-notify='faelight-notify'
alias f-recent='recent-files'
alias f-term='faelight-term'
alias f-ver='get-version'
alias f-zone='faelight-zone'

# ═══════════════════════════════════════════════════════════
# 📁 NAVIGATION (Quick Directory Jumps)
# Fast access to common directories in the numbered system
# ═══════════════════════════════════════════════════════════

alias core='~/0-core/scripts/core'  # v2 orchestrator binary

# ── Core v3 — Event Ledger (Phase 1) ─────────────────────
alias cls='core link sync'           # sync all dotfiles — one command
alias clp='core link plan'           # preview before syncing
alias ce='core events list'
alias cew='core events watch'          # live event stream
alias cpl='core plugin list'            # plugin registry
alias cpa='core plugin add'             # register plugin
alias cps='core plugin status'          # plugin status         # live event stream          # today's events
alias ces='core events since'        # ces 1h / ces 30m / ces 2d
alias cef='core events filter'       # cef git / cef doctor

# ── Core v4 — Checkpoint System (Phase 1) ────────────────
alias cif="core intent focus"           # focus an intent
alias ciu="core intent unfocus"         # clear focus
alias cis="core intent status"          # show focused intent
alias cid="core intent drift"           # detect drift
alias cistart="core intent start"       # start intent (planned→in-progress)
alias cicomplete="core intent complete" # complete intent
alias cin="core intent new"             # new intent from template
alias cibd="core intent burndown"       # completion burndown chart
alias civ="core intent velocity"        # velocity metrics
alias cibr="core intent branch"         # git branch name for intent
alias cideps="core intent deps"         # dependency graph

alias cpc="core checkpoint create"       # create checkpoint
alias cpl="core checkpoint list"         # list checkpoints
alias cpr="core checkpoint restore"      # recovery report for checkpoint
alias cplg="core checkpoint last-good"   # find last 95%+ checkpoint

alias ssd="core security debt"          # security debt report
alias sst="core security trend"         # security trend over time
alias ssh2="core security history"      # scan history
alias cpss="core checkpoint snapshot"    # btrfs snapshot of @home
alias cpsl="core checkpoint snapshots"   # list btrfs snapshots
alias cpd="core checkpoint diff"         # diff since checkpoint

# ── Core v3 — Causality Engine (Phase 2) ─────────────────
alias csd='core simulate doctor'
alias cdt='core doctor trend'            # health trend analysis
alias cdf='core doctor forecast'         # health forecast      # predict health — no writes
alias csu='core simulate update'      # preview updates — no writes
alias cw='core why summary'          # today's activity summary
alias cwh='core why health'          # health trajectory
alias cwd='core why domain'          # cwd git / cwd doctor / cwd security
alias ctr='core trace last'          # last 10 events with detail
alias ctrd='core trace domain'       # ctrd git / ctrd doctor
alias 0core='cd ~/0-core'            # navigate to 0-core root

# ── New Tools (installed 2026-02-26) ─────────────────────
alias top='btm'                      # bottom — better htop
alias repo='onefetch'                # git repo summary
alias bench='hyperfine'              # benchmarking
alias extract='ouch decompress'      # smart archive extraction
alias compress='ouch compress'       # smart archive creation
alias diff='difft'                   # difftastic — semantic diff
alias loc='tokei'                    # lines of code stats
alias loch='tokei ~/0-core/rust-tools --sort lines'  # 0-core LOC
alias cdcore='cd ~/0-core'
alias src='cd ~/1-src'
alias work='cd ~/2-work'
alias ws='workspace-view'
alias keep='cd ~/3-keep'
alias conf='cd ~/.config'
alias docs='cd ~/Documents'
alias down='cd ~/Downloads'
alias pics='cd ~/Pictures'
alias vids='cd ~/Videos'
alias desk='cd ~/Desktop'
alias tmp='cd ~/9-temp'
alias secrets='cd ~/secrets'

# ─── Navigation Shortcuts ───
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'
alias cdp='cd -'

# ─── Config Directories ───
alias nvimconf='cd ~/.config/nvim'
alias swayconf='cd ~/.config/sway'
alias zshconf='cd ~/.config/zsh'

# ═══════════════════════════════════════════════════════════
# 🚀 GIT & DEVELOPMENT
# Version control and development workflows
# ═══════════════════════════════════════════════════════════

# ─── Git Shortcuts ───
alias g='git'
alias ga='git add'
alias gaa='git add -A'
alias gc='git commit -m'
alias gca='git commit --amend'
alias gcam='git commit -am'
alias gp='git push'
alias gl='git pull'
alias gst='git status'
alias gss='git status -s'
alias gd='git diff'
alias gds='git diff --staged'
alias gdp='git diff --color=always | less -R'
alias glog='git log --oneline -10'
alias gla='git log --oneline --graph --all'
alias gb='git branch'
alias gba='git branch -a'
alias gbd='git branch -d'
alias gbD='git branch -D'
alias gco='git checkout'
alias gcb='git checkout -b'
alias gf='git fetch'
alias gsh='git show'
alias gstash='git stash'
alias gstl='git stash list'
alias gstp='git stash pop'
alias gclean='git clean -fd'
alias greset='git reset --hard'
alias gundo='git reset HEAD~1'
alias gunstage='git reset HEAD'
alias gcl='git clone'

# ─── Lazygit ───
alias lg='lazygit'

# ─── Quick Commits ───
alias qc='git commit -m "Quick update: $(date +%Y-%m-%d)"'
alias qcp='git commit -m "Quick update: $(date +%Y-%m-%d)" && git push'

# ─── Dotfile Management ───
alias dotgit='cd ~/0-core && git'
alias dotsave='cd ~/0-core && git add -A && git commit -m "Update configs" && git push'
alias dotpush='cd ~/0-core && git add -A && git commit -m "Update configs $(date +%Y-%m-%d)" && git push'
alias dotstatus='cd ~/0-core && git status'
alias dotadd='dotctl add'
alias dotlist='dotctl list'
alias dotrem='dotctl remove'

# ─── Core Diff Tools ───
alias cdiff='core-diff'
alias cds='core-diff summary'
alias cdv='core-diff --verbose'
alias cdd='core-diff --open delta'
alias cdm='core-diff --open meld'
alias cdh='core-diff --high-risk'
alias cdlast='core-diff since HEAD~1'
alias cdrel='core-diff since $(git describe --tags --abbrev=0 2>/dev/null || echo HEAD)'
alias cdcheck='cdiff && dot-doctor'
alias cdreview='cdv && cdh'
alias cdbar='core-diff faelight-bar'
alias cdsway='core-diff wm-sway'
alias cdzsh='core-diff shell-zsh'
alias cdnvim='core-diff editor-nvim'

# ─── Security & Secrets ───
alias scan-secrets='gitleaks detect --no-git -v'
alias scan-staged='gitleaks protect --staged -v'
alias pre-commit='echo "🔍 Pre-commit checks..." && gitleaks protect --staged -v && dot-doctor && echo "✅ Safe to commit!"'
alias secrets-mount='gocryptfs ~/secrets.encrypted ~/secrets && echo "🔓 Secrets mounted"'
alias secrets-unmount='fusermount -u ~/secrets && echo "🔒 Secrets locked"'

# ─── Archaeology (Git History) ───
alias arch='archaeology-0-core'
alias arch0='archaeology-0-core'
alias archint='archaeology-0-core --by-intent'
alias archsince='archaeology-0-core --since'
alias archtime='archaeology-0-core --timeline'
alias archwk='archaeology-0-core --this-week'

# ═══════════════════════════════════════════════════════════
# 📦 PACKAGE MANAGEMENT (Paru/Pacman)
# System package installation and maintenance
# ═══════════════════════════════════════════════════════════

# ─── Paru (AUR Helper) ───
alias yay='paru --color=auto'             # Compatibility alias
alias yayi='paru -S'
alias yayr='paru -R'
alias yays='paru -Ss'
alias yayu='paru -Syu'
alias yup='paru -Syu'

# ─── Pacman Operations ───
alias paci='paru -S'                      # Install package
alias pacr='paru -R'                      # Remove package
alias pacu='paru -Syu'                    # Update system
alias pacs='pacman -Ss'                   # Search packages
alias pacinfo='pacman -Qi'                # Package info
alias paclist='pacman -Qqe'               # List installed

# ─── Maintenance ───
alias ins='paru -S'                       # Install package
alias uns='paru -Rns'                     # Uninstall + remove deps
alias orphan-clean='paru -Rns $(paru -Qtdq) 2>/dev/null || true'
alias cleanup='faelight-cleanup'
alias f-cleanup='faelight-cleanup'
alias clean-all='paru -Sc && paru -Yc'
alias orphans='pacman -Qtdq'
alias unlock='sudo rm /var/lib/pacman/db.lck'

# ─── Mirrors & Updates ───
alias mirror='sudo reflector --verbose --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'
alias fix-keys='sudo pacman-key --init && sudo pacman-key --populate && sudo pacman-key --refresh-keys'

# ═══════════════════════════════════════════════════════════
# ⚙️  SYSTEM OPERATIONS
# System management, monitoring, and control
# ═══════════════════════════════════════════════════════════

# ─── System Info ───
alias sysinfo='fastfetch'
alias neofetch='fastfetch'
alias sysver='uname -r'
alias card='echo "╔════════════════════════════════════════╗" && echo "║  🌲 FAELIGHT FOREST v9.3.0            ║" && echo "║  🏥 Health: $(dot-doctor | grep "Health:" | awk "{print \$2}")                        ║" && echo "║  📦 Tools: 40 Production Ready         ║" && echo "║  🔒 Security: Hardened                 ║" && echo "╚════════════════════════════════════════╝"'

# ─── Power Management ───
alias sr='reboot'
alias ssn='shutdown now'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'
alias logout='swaymsg exit'

# ─── Process Management ───
alias psa='ps auxf'
alias psg='ps aux | grep -v grep | grep -i -e VSZ -e'
alias cpu='ps auxf | sort -nr -k 3 | head -10'
alias mem='ps auxf | sort -nr -k 4 | head -10'

# ─── Network ───
alias ports='sudo ss -tulanp'
alias listening='sudo lsof -i -P -n | grep LISTEN'
alias myip='curl -s ifconfig.me'
alias localip='ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v 127.0.0.1'
alias pingg='ping -c 5 google.com'

# ─── Security & Monitoring ───
alias security-check='sudo pacman -Syu && echo "---" && arch-audit && echo "---" && audit-quick'
alias security-score='test -f ~/.lynis-score && echo "🛡️  Hardening Index: $(cat ~/.lynis-score)/100" || echo "Run audit-full or audit-quick first"'
alias audit-full='sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias audit-quick='sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score'
alias full-audit='dot-doctor && entropy-check && security-check'
alias system-health='dot-doctor && lynis audit system --quick'

# ─── Fail2ban ───
alias jail-status='sudo fail2ban-client status'
alias ban-list='sudo fail2ban-client status sshd'

# ─── Sway WM ───
alias sway-reload='swaymsg reload'
alias sway-info='swaymsg -t get_tree'
alias bar-restart='~/0-core/scripts/launch-bar'

# ─── Disk & Storage ───
alias df='df -h'
alias du='du -h'
alias duh='du -sh * | sort -hr'
alias free='free -h'

# ─── Snapshots ───
alias snapshots='sudo snapper -c root list'
alias snapper-create='sudo snapper -c root create --description'

# ═══════════════════════════════════════════════════════════
# 🎨 UI & DISPLAY
# Listing, viewing, and display utilities
# ═══════════════════════════════════════════════════════════

# ─── Listing (eza) ───
alias ls='eza --icons --group-directories-first'
alias la='eza -a --icons --group-directories-first'
alias ll='eza -lah --icons --group-directories-first --git'
alias lt='eza -lah --icons --sort=modified --reverse'
alias lsize='eza -lah --icons --sort=size --reverse'
alias tree='eza --tree --icons --group-directories-first'

# ─── File Viewing (bat) ───
alias ccat='/usr/bin/cat'                 # Original cat
alias cat='bat --paging=never'            # Replaced with bat
alias catp='bat --paging=always'          # Paged bat
alias catt='bat --style=plain'            # Plain bat

# ─── Search & Find ───
alias search='fd'
alias findf='fd --type f'
alias findd='fd --type d'
alias fcd='cd $(fd --type d | fzf)'
alias vf='nvim $(fd --type f | fzf)'
alias preview='fzf --preview "bat --color=always {}"'

# ─── Key Bindings ───
alias keys='bat ~/0-core/docs/KEYBINDINGS.md'
alias keybinds='keyscan'
alias conflicts='keyscan'

# ═══════════════════════════════════════════════════════════
# ✏️  EDITORS & NEOVIM
# Editor shortcuts and configurations
# ═══════════════════════════════════════════════════════════

alias nv='nvim'
alias vi='nvim'
alias vim='nvim'
alias svi='sudo nvim'
alias lazy='nvim'

# ─── Neovim Distributions ───
alias astro='NVIM_APPNAME=astronvim nvim'
alias chad='NVIM_APPNAME=nvchad nvim'

# ─── Neovim Management ───
alias lazyvim-update='nvim --headless "+Lazy! sync" +qa'
alias lazyvim-clean='nvim --headless "+Lazy! clean" +qa'

# ─── Config Editing ───
alias nzsh='nvim ~/.config/zsh/.zshrc'
alias nsway='nvim ~/.config/sway/config'
alias nbar='nvim ~/0-core/rust-tools/faelight-bar/src/main.rs'

# ═══════════════════════════════════════════════════════════
# 🛠️  UTILITIES & HELPERS
# Miscellaneous useful commands
# ═══════════════════════════════════════════════════════════

# ─── Time & Date ───
alias now='date +"%T"'
alias nowdate='date +"%Y-%m-%d"'
alias timestamp='date +"%Y%m%d_%H%M%S"'

# ─── Archive Operations ───
alias extract='tar -xzvf'
alias targz='tar -czf'
alias untar='tar -xvf'

# ─── File Operations ───
alias chx='chmod +x'

# ─── Clipboard ───
alias yp='pwd | wl-copy'                  # Yank path
alias yf='basename $PWD | wl-copy'        # Yank filename

# ─── Web Shortcuts ───
alias gmail='xdg-open "https://gmail.com"'
alias youtube='xdg-open "https://youtube.com"'
alias chatgpt='xdg-open "https://chat.openai.com"'
alias claude='xdg-open "https://claude.ai"'
alias weather='curl wttr.in'

# ─── Workspace & Profiles ───
alias prof='profile'
alias prof-list='profile list'
alias prof-switch='profile switch'
alias wsa='workspace-view --active'
alias wss='workspace-view --summary'

# ─── Documentation ───
alias guide='bat ~/0-core/COMPLETE_GUIDE.md'
alias changelog='bat ~/0-core/CHANGELOG.md'
alias roadmap='nvim ~/0-core/docs/planning/ROADMAP.md'
alias planning='cd ~/0-core/docs/planning && ls'

# ─── Fun Shortcuts ───
alias please='sudo !!'
alias fucking='sudo !!'

# ─── Shell Management ───
alias reload='source ~/.config/zsh/.zshrc'
alias s='source ~/.zshrc'
alias path='echo $PATH | tr ":" "\n"'

# ─── Status & Overview ───
alias status='dot-doctor && echo "" && git status'
alias overview='fastfetch && echo "" && dot-doctor && echo "" && git -C ~/0-core status -s'
alias check-updates='update-check'
alias weekly='weekly-check'
alias lastup='latest-update'
alias latest='latest-update'
alias forest-ver='echo "🌲 Faelight Forest v9.3.0"'

# ─── Release Management ───
alias release-prep='echo "📦 Preparing release..." && bump-system-version && compile-changelog.sh && git status'
alias compile-log='~/0-core/scripts/compile-changelog.sh'
alias mklog='~/0-core/scripts/compile-changelog.sh'

# ─── Direnv ───
alias envrc-allow='direnv allow'
alias envrc-deny='direnv deny'
alias envrc-status='direnv status'
alias envrc-check='bat .envrc'
alias envrc-inspect='bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow"'

# ═══════════════════════════════════════════════════════════
# 🎯 END OF ALIASES
# Total: 300+ aliases for maximum productivity!
# ═══════════════════════════════════════════════════════════

# Auth health monitoring (added 2026-02-11)
alias auth-health='~/0-core/scripts/check-auth-health'
alias reset-auth='~/0-core/scripts/reset-auth'

# ─── faelight-clipboard ───
alias clip='faelight-clipboard'
alias cb='faelight-clipboard'
alias cbh='faelight-clipboard history'
alias cbp='faelight-clipboard pick'

# ─── faelight-pulse ───
alias pulse='faelight-pulse'
alias fp='faelight-pulse'
alias pulse-doc='faelight-pulse --domain doctor'
alias pulse-git='faelight-pulse --domain git'
alias pulse-json='faelight-pulse --json'

# ─── faelight-niri-bridge ───
alias niri-bridge='faelight-niri-bridge'
alias nb='faelight-niri-bridge'

# ─── faelight-forecast ───
alias forecast='faelight-forecast'
alias ff='faelight-forecast'
alias ffp='faelight-forecast --plain'

# ─── faelight-clipboard ───
alias clip='faelight-clipboard'
alias cb='faelight-clipboard'
alias cbh='faelight-clipboard history'
alias cbp='faelight-clipboard pick'

# ─── faelight-pulse ───
alias pulse='faelight-pulse'
alias fp='faelight-pulse'
alias pulse-doc='faelight-pulse --domain doctor'
alias pulse-git='faelight-pulse --domain git'
alias pulse-json='faelight-pulse --json'

# ─── faelight-niri-bridge ───
alias niri-bridge='faelight-niri-bridge'
alias nb='faelight-niri-bridge'

# ─── faelight-forecast ───
alias forecast='faelight-forecast'
alias ff='faelight-forecast'
alias ffp='faelight-forecast --plain'
