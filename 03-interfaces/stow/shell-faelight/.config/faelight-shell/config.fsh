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
alias d = "core doctor run"
alias v = "nvim"
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
alias fm = "faelight-fm"
alias menu = "faelight-menu"
alias bar = "faelight-bar"
alias bump = "faelight-release publish"
alias fu = "faelight-update"
alias sec = "security-audit"
alias lock = "faelight-lock"
alias notify = "faelight-notify"
alias term = "faelight-term"
alias palette = "faelight-palette"
alias vault = "faelight-vault"
alias forecast = "faelight-forecast"
alias pulse = "faelight-pulse"
alias clip = "faelight-clipboard"
alias ya = "yazi"

alias ls = "eza --icons"
alias ll = "eza -la --icons"
alias la = "eza -la --icons"

# Deploy pipeline (INT-164)
alias deploy = "~/0-core/scripts/deploy"
alias rollback = "~/0-core/scripts/rollback"
alias forest-status = "~/0-core/scripts/forest-status"

# Pre-command decision rules — INT-171
before_run {
    if command contains "paru -Syu" { warn "System update — run during maintenance window?" }
    if command contains "sudo rm" { block "Use rm directly — sudo rm is too dangerous" }
}
