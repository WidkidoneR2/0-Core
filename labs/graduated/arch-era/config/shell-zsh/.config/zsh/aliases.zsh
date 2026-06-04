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
alias d='core doctor run'                 # System health check
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
# removed stale (bump-system-version retired): alias bump='faelight-release publish'  # faelight-release replaces bump-system-version
# removed synonym: alias dash='faelight-dashboard'
alias dashboard='faelight-dashboard'
# removed synonym: alias dmenu='faelight-palette'
alias dot='dotctl'
alias fm='faelight-fm'
alias launcher='faelight-palette'
alias link='faelight-link'
alias lock='faelight-lock'
alias menu='faelight-menu'
alias notify='faelight-notify'
alias term='faelight-term'
# removed synonym: alias ft='faelight-term'
alias zone='faelight-zone'

# ─── Health & Monitoring ───
# removed stale (dot-doctor retired): alias doctor='dot-doctor'
# removed stale (dot-doctor retired): alias check-health='dot-doctor'
# removed stale (dot-doctor retired): alias health='dot-doctor'

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
# (duplicates removed 2026-03-29 — INT-163)
alias topgrade='faelight-update'           # Topgrade replaced by faelight-update
alias fudr='faelight-update --dry-run'     # Check updates without applying
alias fui='faelight-update --interactive' # Interactive update
alias fuup='faelight-update'              # Quick update
alias update='safe-update'
alias safe-update='~/0-core/scripts/safe-update'
# removed stale (dot-doctor retired): alias safe-up='snap-now && safe-update && dot-doctor'

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
# removed synonym: alias fl='faelight-link'
alias stow-check='faelight-link status'
alias stow-fix='faelight-link clean'

# ─── Snapshots & Backups ───
# removed synonym: alias snap='faelight-snapshot'
alias snapshot='faelight-snapshot'
alias snapcreate='faelight-snapshot create'
alias snaplist='faelight-snapshot list'
alias snap-now='faelight-snapshot create "Manual snapshot at $(date +%Y%m%d_%H%M%S)"'
alias snap-before='echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created!"'

# ─── Utilities ───
alias fae='faelight-digest'
# removed synonym: alias faelight='faelight-digest'
# removed synonym: alias getver='get-version'
alias ver='get-version'
alias recent='recent-files'

# ─── Shortened Tool Names ───
# removed synonym: alias f-bar='faelight-bar'
alias f-daemon='faelight-daemon'
alias daemon-status='systemctl --user status faelight-daemon'
alias daemon-log='journalctl --user -u faelight-daemon -n 20 --no-pager'
# removed synonym: alias f-bootstrap='faelight-bootstrap'
# removed synonym: alias f-dmenu='faelight-palette'
# removed synonym: alias f-fm='faelight-fm'
# removed synonym: alias f-guard='intent-guard'
# removed synonym: alias f-hooks='faelight-hooks'
# removed synonym: alias f-launch='faelight-palette'
# removed synonym: alias f-link='faelight-link'
# removed synonym: alias f-lock='faelight-lock'
# removed synonym: alias f-menu='faelight-menu'
# removed synonym: alias f-notify='faelight-notify'
# removed synonym: alias f-recent='recent-files'
# removed synonym: alias f-term='faelight-term'
# removed synonym: alias f-ver='get-version'
# removed synonym: alias f-zone='faelight-zone'

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
alias cplugs='core plugin list'   # was cpl — renamed INT-163
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
# ─── Core v6 — The Judgment Layer ───
alias decide='core decide'                        # record a decision with risk assessment
alias outcome='core decision outcome'             # record outcome of a decision
alias decisions='core decision list'              # list all decisions
alias dec='core decision list'                    # shorthand
alias deco='core decision list --open'            # pending decisions only
alias decshow='core decision show'                # show decision detail
alias decstats='core decision stats'              # correlation stats
alias hindsight='core hindsight'                  # decision success summary
alias advise='core advise'                        # judgment advisory
alias heuristics='core heuristics'               # auto-derived heuristics
alias lessons='core lessons'                      # human-readable wisdom
alias story='core story'                          # 30-day forest narrative
alias css='core simulate scenario'               # simulate a planned scenario
alias secadvise='core security advise'            # security judgment advisory

# ─── faelight-shell ───
alias fs='faelight-shell'                          # forest-native shell
alias deploy='~/0-core/scripts/deploy'            # deploy core or faelight-shell
alias rollback='~/0-core/scripts/rollback'        # rollback to previous version
alias forest-status='~/0-core/scripts/forest-status' # show active versions + chain

# ─── Core Audit — Tool Intelligence Layer ───
alias audit='core audit scan'                     # score all tools
alias auditshow='core audit show'                 # deep audit of a tool
alias auditstale='core audit stale'               # tools needing attention
alias auditcov='core audit coverage'              # documentation coverage

alias cw='core why summary'          # today's activity summary
alias cwh='core why health'          # health trajectory
alias cwv='core why visual'         # visual topology today
alias cwa='core why attention'       # attention fragmentation
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
alias keep='cd ~/3-keep'
alias conf='cd ~/.config'
alias cdocs='cd ~/Documents'  # was docs — renamed INT-163
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
# removed stale (sway retired): alias swayconf='cd ~/.config/sway'
alias zshconf='cd ~/.config/zsh'

# ═══════════════════════════════════════════════════════════
# 🚀 GIT & DEVELOPMENT
# Version control and development workflows
# ═══════════════════════════════════════════════════════════

# ─── Git Shortcuts ───
# removed duplicate: alias g='git'
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
# removed stale (dot-doctor retired): alias cdcheck='cdiff && dot-doctor'
alias cdreview='cdv && cdh'
alias cdbar='core-diff faelight-bar'
alias cdzsh='core-diff shell-zsh'
alias cdnvim='core-diff editor-nvim'

# ─── Security & Secrets ───
alias scan-secrets='gitleaks detect --no-git -v'
alias scan-staged='gitleaks protect --staged -v'
# removed stale (dot-doctor retired): alias pre-commit='echo "🔍 Pre-commit checks..." && gitleaks protect --staged -v && dot-doctor && echo "✅ Safe to commit!"'
alias secrets-mount='gocryptfs ~/secrets.encrypted ~/secrets && echo "🔓 Secrets mounted"'
alias secrets-unmount='fusermount -u ~/secrets && echo "🔒 Secrets locked"'

# ─── Archaeology (Git History) ───

# ═══════════════════════════════════════════════════════════
# 📦 PACKAGE MANAGEMENT (Paru/Pacman)
# System package installation and maintenance
# ═══════════════════════════════════════════════════════════

# ─── Paru (AUR Helper) ───
alias yay='paru --color=auto'             # Compatibility alias
alias yayi='paru -S'
alias yayr='paru -R'
alias yays='paru -Ss'
# removed synonym: alias yayu='paru -Syu'
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
# removed synonym: alias f-cleanup='faelight-cleanup'
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
# removed synonym: alias neofetch='fastfetch'
alias sysver='uname -r'
# removed stale (dot-doctor retired): alias card='echo "╔════════════════════════════════════════╗" && echo "║  🌲 FAELIGHT FOREST v9.3.0            ║" && echo "║  🏥 Health: $(dot-doctor | grep "Health:" | awk "{print \$2}")                        ║" && echo "║  📦 Tools: 40 Production Ready         ║" && echo "║  🔒 Security: Hardened                 ║" && echo "╚════════════════════════════════════════╝"'

# ─── Power Management ───
alias sr='reboot'
alias ssn='shutdown now'
alias suspend='systemctl suspend'
alias hibernate='systemctl hibernate'
# removed stale (sway retired): alias logout='swaymsg exit'

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
# removed stale (dot-doctor retired): alias system-health='dot-doctor && lynis audit system --quick'

# ─── Fail2ban ───
alias jail-status='sudo fail2ban-client status'
alias ban-list='sudo fail2ban-client status sshd'

# ─── Sway WM ───
# removed stale (sway retired): alias sway-reload='swaymsg reload'
# removed stale (sway retired): alias sway-info='swaymsg -t get_tree'
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
# removed synonym: alias conflicts='keyscan'

# ═══════════════════════════════════════════════════════════
# ✏️  EDITORS & NEOVIM
# Editor shortcuts and configurations
# ═══════════════════════════════════════════════════════════

# removed synonym: alias nv='nvim'
# removed synonym: alias vi='nvim'
# removed synonym: alias vim='nvim'
alias svi='sudo nvim'
# removed synonym: alias lazy='nvim'

# ─── Neovim Distributions ───
alias astro='NVIM_APPNAME=astronvim nvim'
alias chad='NVIM_APPNAME=nvchad nvim'

# ─── Neovim Management ───
alias lazyvim-update='nvim --headless "+Lazy! sync" +qa'
alias lazyvim-clean='nvim --headless "+Lazy! clean" +qa'

# ─── Config Editing ───
alias nzsh='nvim ~/.config/zsh/.zshrc'
# removed stale (sway retired): alias nsway='nvim ~/.config/sway/config'
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
# removed: tar extract alias — ouch decompress is canonical (INT-163)
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

# ─── Documentation ───
alias guide='bat ~/0-core/COMPLETE_GUIDE.md'
alias changelog='bat ~/0-core/CHANGELOG.md'
alias roadmap='nvim ~/0-core/docs/planning/ROADMAP.md'
alias planning='cd ~/0-core/docs/planning && ls'

# ─── Fun Shortcuts ───
alias please='sudo !!'
# removed synonym: alias fucking='sudo !!'

# ─── Shell Management ───
alias reload='source ~/.config/zsh/.zshrc'
alias s='source ~/.zshrc'
alias path='echo $PATH | tr ":" "\n"'

# ─── Status & Overview ───
# removed stale (dot-doctor retired): alias status='dot-doctor && echo "" && git status'
# removed stale (dot-doctor retired): alias overview='fastfetch && echo "" && dot-doctor && echo "" && git -C ~/0-core status -s'
alias check-updates='update-check'
alias weekly='weekly-check'
# removed synonym: alias lastup='latest-update'
alias latest='latest-update'
# removed stale (version string): alias forest-ver='echo "🌲 Faelight Forest v9.3.0"'

# ─── Release Management ───
# removed stale (bump-system-version retired): alias release-prep='echo "📦 Preparing release..." && bump-system-version && compile-changelog.sh && git status'
# removed stale (compile-changelog retired): alias compile-log='~/0-core/scripts/compile-changelog.sh'
# removed stale (compile-changelog retired): alias mklog='~/0-core/scripts/compile-changelog.sh'

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
# removed synonym: alias cb='faelight-clipboard'
alias cbh='faelight-clipboard history'
alias cbp='faelight-clipboard pick'

# ─── faelight-pulse ───
alias pulse='faelight-pulse'
# removed synonym: alias fp='faelight-pulse'
alias pulse-doc='faelight-pulse --domain doctor'
alias pulse-git='faelight-pulse --domain git'
alias pulse-json='faelight-pulse --json'

# ─── faelight-niri-bridge ───
# removed synonym: alias niri-bridge='faelight-niri-bridge'
alias nb='faelight-niri-bridge'

# ─── faelight-compositor ───
# removed synonym: alias fcomp='faelight-compositor'
alias fc='faelight-compositor'

# ─── faelight-forecast ───
alias forecast='faelight-forecast'
# removed synonym: alias ff='faelight-forecast'
alias ffp='faelight-forecast --plain'


# ─── faelight-release ───
alias release='faelight-release'
# removed synonym: alias fr='faelight-release'
alias fr-status='faelight-release status'
alias fr-history='faelight-release history'
alias fr-preview='faelight-release preview'

# ─── faelight-wallpaper ───
alias wallpaper='faelight-wallpaper'
# removed synonym: alias wp='faelight-wallpaper'

# ─── faelight-search ───
# alias fs=faelight-search  # removed — faelight-shell takes priority
# alias search='faelight-search'  # RETIRED 2026-03-26 — use ? in fsh or fd


# ─── core ledger (Core v5 Phase 1) ───
alias cledger='core ledger stats'  # was cls — renamed INT-163
alias clq='core ledger query'
alias cle='core ledger export'
alias cli='core ledger indexes'

# ─── core why deep (Core v5 Phase 3) ───
alias cwhs='core why health-since'
alias cwc='core why causal'
alias cwch='core why chain'

# ─── core why patterns (Core v5 Phase 4) ───
alias suggest='core why suggest'
alias correlate='core why correlate'

# ─── compositor intelligence (Core v5 Phase 5) ───
alias cww='core why workspace'
alias cwf='core why focus'

# faelight-vault
alias vault="faelight-vault"
# removed synonym: alias fv="faelight-vault"
alias fva="faelight-vault audit"
alias fvl="faelight-vault list"
alias fvg="faelight-vault generate"

# faelight-docs
# removed synonym: alias docs="faelight-docs"
alias fdocs="faelight-docs"
alias docs-sync="faelight-docs sync"
alias docs-check="faelight-docs check"
alias docs-status="faelight-docs status"