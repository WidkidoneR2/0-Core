# faelight-shell configuration
# ~/.config/faelight-shell/config.fsh
#
# Syntax:
#   alias <name> = "<command>"
#   set <key> = <value>

# Common aliases
alias gs = "git status"
alias gc5 = "gc | first 5"

# Settings
set history_limit = 10000
set prompt_style = forest

# Tier 1 — Daily use
alias d = "/run/current-system/sw/bin/core doctor run"
alias v = "hx"
alias l = "eza -lh --icons --group-directories-first"
alias b = "bat --paging=never"
alias y = "yazi"
alias g = "git"
alias c = "clear"
alias gst = "git status"
alias gp = "git push"
alias gaa = "git add -A"
alias gcm = "git commit -m"

# Tier 2 — Forest tools
# Core binary -- explicit NixOS path
alias core = "/run/current-system/sw/bin/core"

alias fm = "faelight-fm"
alias menu = "faelight-logout"
alias bar = "systemctl --user status faelight-bar"
alias bump = "faelight-release"
alias fu = "faelight-update"
alias lock = "faelight-lock"
alias notify = "faelight-notify"
alias term = "faelight-term"
alias vault = "faelight-vault"
alias forecast = "core friday health-forecast"
alias inspect = "core nix inspect"
# awesome-nix tools (installed, now wired in -- see decision pattern INT-097)
alias ndiff = "nvd diff"
alias deptree = nix-tree
alias nhclean = "nh clean all --keep-since 7d --ask"
alias nhclean-dry = "nh clean all --keep-since 7d --dry"
alias nhclean-all = "nh clean all --ask"
alias clip = "faelight-clipboard"
alias ya = "yazi"

alias ls = "eza --icons"
alias ll = "eza -la --icons"
alias la = "eza -la --icons"

# Deploy pipeline (INT-164)
alias deploy = "~/0-core/faelight/packages/faelight/scripts/deploy"
alias rebuild = "/run/current-system/sw/bin/bash ~/0-core/faelight/packages/faelight/scripts/rebuild-record"
    alias rebuild-safe = "bash ~/0-core/faelight/packages/faelight/scripts/rebuild-safe"
    alias fm = "faelight-fm"
    alias fmd = "faelight-fm --dual"
alias rebuild-home = "sudo nixos-rebuild switch --flake ~/0-core#framework16"
alias rebuild-dry = "sudo nixos-rebuild dry-run --flake ~/0-core#framework16"
alias rebuild-check = "sudo nixos-rebuild dry-run --flake ~/0-core#framework16 && core doctor quick"
alias update-flake = "cd ~/0-core && nix flake update && bash ~/0-core/faelight/packages/faelight/scripts/rebuild-safe"
alias rollback = "~/0-core/faelight/packages/faelight/scripts/rollback"
alias forest-status = "~/0-core/faelight/packages/faelight/scripts/forest-status"

# Forest workflow aliases — INT-171
alias cistart = "core intent start"
alias cicomplete = "core intent complete"
alias intent = "/run/current-system/sw/bin/intent"
alias fg = "faelight-git"

# Pre-command decision rules — INT-171
before_run {
    if command contains "paru -Syu" { warn "System update — run during maintenance window?" }
    if command contains "sudo rm" { block "Use rm directly — sudo rm is too dangerous" }
}

alias gd = "GIT_EXTERNAL_DIFF=difft git diff"

set LIBVIRT_DEFAULT_URI = "qemu:///system"

# Friday learning mode -- off = silent (learns but does not suggest)
set friday_hints = off

# --- Migrated from runtime table (INT-060) ---
alias .. = cd ..
alias ... = cd ../..
alias .... = cd ../../..
alias ..... = cd ../../../..
alias 0core = cd ~/0-core
alias advise = core advise
alias audit = core audit scan
alias auditcov = core audit coverage
alias auditshow = core audit show
alias auditstale = core audit stale
alias ban-list = sudo fail2ban-client status sshd
alias bar-restart = "systemctl --user restart faelight-bar"
alias bench = hyperfine
alias cat = bat --paging=never --color=always
alias catp = bat --paging=always
alias catt = bat --style=plain
alias cbh = faelight-clipboard history
alias cbp = faelight-clipboard pick
alias cdcore = cd ~/0-core
alias cdf = core doctor forecast
alias cdocs = cd ~/Documents
alias cdp = cd -
alias cdt = core doctor trend
alias ce = core events list
alias cef = core events filter
alias ces = core events since
alias cew = core events watch
alias changelog = bat ~/0-core/CHANGELOG.md
alias chx = chmod +x
alias cibd = core intent burndown
alias cibr = core intent branch
alias cid = core intent drift
alias cideps = core intent deps
alias cif = core intent focus
alias cin = core intent new
alias cis = core intent status
alias ciu = core intent unfocus
alias civ = core intent velocity
alias cle = core ledger export
alias cledger = core ledger stats
alias cli = core ledger indexes
alias clq = core ledger query
alias compress = ouch compress
alias conf = cd ~/.config
alias correlate = core why correlate
alias cpa = core plugin add
alias cpc = core checkpoint create
alias cpd = core checkpoint diff
alias cpl = core checkpoint list
alias cplg = core checkpoint last-good
alias cplugs = core plugin list
alias cpr = core checkpoint restore
alias cps = core plugin status
alias cpsl = core checkpoint snapshots
alias cpss = core checkpoint snapshot
alias cpu = ps auxf | sort -nr -k 3 | head -10
alias csd = core simulate doctor
alias css = core simulate scenario
alias csu = core simulate update
alias ctr = core trace last
alias ctrd = core trace domain
alias cw = core why summary
alias cwa = core why attention
alias cwc = core why causal
alias cwch = core why chain
alias cwd = core why domain
alias cwf = core why focus
alias cwh = core why health
alias cwhs = core why health-since
alias cwv = core why visual
alias cww = core why workspace
alias daemon-log = journalctl --user -u faelight-daemon -n 20 --no-pager
alias daemon-status = systemctl --user status faelight-daemon
alias db = db-browse
alias dec = core decision list
alias decide = core decide
alias decisions = core decision list
alias deco = core decision list --open
alias decshow = core decision show
alias decstats = core decision stats
alias desk = cd ~/Desktop
alias df = df -h
alias diff = difft
alias docs-check = faelight-docs check
alias docs-status = faelight-docs status
alias docs-sync = faelight-docs sync
alias dotgit = cd ~/0-core && git
alias dotpush = cd ~/0-core && git add -A && git commit -m "Update configs $(date +%Y-%m-%d)" && git push
alias dotsave = cd ~/0-core && git add -A && git commit -m "Update configs" && git push
alias dotstatus = cd ~/0-core && git status
alias down = cd ~/Downloads
alias du = du -h
alias duh = du -sh * | sort -hr
alias envrc-allow = direnv allow
alias envrc-check = bat .envrc
alias envrc-deny = direnv deny
alias envrc-inspect = bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow
alias envrc-status = direnv status
alias extract = ouch decompress
alias f = faelight
alias f-daemon = faelight-daemon
alias fc = faelight-compositor
alias fcd = cd $(fd --type d | fzf)
alias fdocs = faelight-docs
alias fga = faelight-git add
alias fgc = faelight-git commit
alias fgp = faelight-git push
alias fgs = faelight-git status
alias findd = fd --type d
alias findf = fd --type f
alias fr-history = faelight-release history
alias fr-preview = faelight-release preview
alias fr-status = faelight-release status
alias free = free -h
alias fs = faelight-shell
alias fudr = faelight-update --dry-run
alias fui = faelight-update --interactive
alias fuup = faelight-update
alias fva = faelight-vault audit
alias fvg = faelight-vault generate
alias fvl = faelight-vault list
alias ga = git add
alias gb = git branch
alias gbD = git branch -D
alias gba = git branch -a
alias gbd = git branch -d
alias gc = git commit -m
alias gca = git commit --amend
alias gcam = git commit -am
alias gcb = git checkout -b
alias gcl = git clone
alias gclean = git clean -fd
alias gco = git checkout
alias gdp = git diff --color=always | less -R
alias gds = git diff --staged
alias gf = git fetch
alias gl = git pull
alias gla = git log --oneline --graph --all
alias glog = git log --oneline -10
alias greset = git reset --hard
alias gsh = git show
alias gss = git status -s
alias gstash = git stash
alias gstl = git stash list
alias gstp = git stash pop
alias guard = intent-guard
alias guide = bat ~/0-core/COMPLETE_GUIDE.md
alias gundo = git reset HEAD~1
alias gunstage = git reset HEAD
alias heuristics = core heuristics
alias hibernate = systemctl hibernate
alias hindsight = core hindsight
alias hooks = faelight-hooks
alias int = intent
alias int-active = intent list --active
alias inta = intent add
alias intc = intent complete
alias intl = intent list
alias ints = intent show
alias jail-status = sudo fail2ban-client status
alias jb = journalctl -b | tspin
alias jf = journalctl -f | tspin
alias journal = journalctl --no-pager | tspin
alias keep = cd ~/3-keep
alias keys = bat ~/0-core/docs/KEYBINDINGS.md
alias launcher = faelight-launcher
alias launch = faelight-launcher  # INT-084
alias lessons = core lessons
alias lg = lazygit
alias loc = tokei
alias localip = ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v 127.0.0.1
alias loch = tokei ~/0-core/rust-tools --sort lines
alias lsize = eza -lah --icons --sort=size --reverse
alias lt = eza -lah --icons --sort=modified --reverse
alias mem = ps auxf | sort -nr -k 4 | head -10
alias mycommits = gc
alias myip = curl -s ifconfig.me
alias now = date +"%T
alias nowdate = date +"%Y-%m-%d
alias outcome = core decision outcome
alias path = echo $PATH | tr ":" "\n
alias pics = cd ~/Pictures
alias pingg = ping -c 5 google.com
alias planning = cd ~/0-core/docs/planning && ls
alias ports = sudo ss -tulanp
alias preview = fzf --preview "bat --color=always {}
alias psa = ps auxf
alias psg = ps aux | grep -v grep | grep -i -e VSZ -e
alias qc = git commit -m "Quick update: $(date +%Y-%m-%d)
alias qcp = git commit -m "Quick update: $(date +%Y-%m-%d)" && git push
alias release = faelight-release
alias repo = onefetch
alias safe-update = /run/current-system/sw/bin/safe-update
alias sb = faelight-sandbox
alias sb-clear = faelight-sandbox clear
alias sb-diff = faelight-sandbox diff
alias sb-restore = faelight-sandbox restore
alias sb-snap = faelight-sandbox snapshot
alias sb-snaps = faelight-sandbox snapshots
alias sb-status = faelight-sandbox status
alias scan-secrets = gitleaks detect --no-git -v
alias scan-staged = gitleaks protect --staged -v
alias secadvise = core security advise
alias secrets = cd ~/secrets
alias secrets-mount = gocryptfs ~/secrets.encrypted ~/secrets && echo "🔓 Secrets mounted
alias secrets-unmount = fusermount -u ~/secrets && echo "🔒 Secrets locked
alias security-score = test -f ~/.lynis-score && echo "🛡️  Hardening Index: $(cat ~/.lynis-score)/100" || echo "Run audit-full or audit-quick first
alias snap-before = echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created!
alias sr = reboot
alias src = cd ~/1-src
alias ssd = core security debt
alias ssh2 = core security history
alias ssn = shutdown now
alias sst = core security trend
alias story = core story
alias suggest = core why suggest
alias suspend = systemctl suspend
alias svi = sudo hx
alias sysver = uname -r
alias t = teach
alias targz = tar -czf
alias timestamp = date +"%Y%m%d_%H%M%S
alias tmp = cd ~/9-temp
alias top = btm
alias topgrade = faelight-update
alias tree = eza --tree --icons --group-directories-first
alias untar = tar -xvf
alias update = safe-update
alias vf = nvim $(fd --type f | fzf)
alias vids = cd ~/Videos
alias wallpaper = faelight-wallpaper
alias weather = curl wttr.in
alias work = cd ~/2-work
alias yf = basename $PWD | wl-copy
alias yp = pwd | wl-copy
alias zone = faelight-zone
