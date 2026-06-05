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
# Core binary -- explicit NixOS path
alias core = "/run/current-system/sw/bin/core"

alias fm = "faelight-fm"
alias menu = "faelight-menu"
alias bar = "faelight-bar"
alias bump = "echo \"faelight-release disabled -- needs NixOS rebuild (INT-031)\""
alias fu = "faelight-update"
alias sec = "security-audit"
alias lock = "faelight-lock"
alias notify = "faelight-notify"
alias term = "faelight-term"
alias palette = "faelight-palette"
alias vault = "faelight-vault"
alias forecast = "core friday health-forecast"
alias pulse = "faelight-pulse"
alias clip = "faelight-clipboard"
alias ya = "yazi"

alias ls = "eza --icons"
alias ll = "eza -la --icons"
alias la = "eza -la --icons"

# Deploy pipeline (INT-164)
alias deploy = "~/0-core/pkgs/faelight/scripts/deploy"
alias rebuild = "sudo nixos-rebuild switch --flake ~/0-core#framework16"
    alias rebuild-safe = "bash ~/0-core/pkgs/faelight/scripts/rebuild-safe"
    alias fm = "faelight-fm"
    alias fmd = "faelight-fm --dual"
alias rebuild-home = "sudo nixos-rebuild switch --flake ~/0-core#framework16"
alias rebuild-dry = "sudo nixos-rebuild dry-run --flake ~/0-core#framework16"
alias rebuild-check = "sudo nixos-rebuild dry-run --flake ~/0-core#framework16 && core doctor quick"
alias update-flake = "cd ~/0-core && nix flake update && sudo nixos-rebuild switch --flake .#framework16"
alias rollback = "~/0-core/pkgs/faelight/scripts/rollback"
alias forest-status = "~/0-core/pkgs/faelight/scripts/forest-status"

# Forest workflow aliases — INT-171
alias cistart = "core intent start"
alias cicomplete = "core intent complete"
alias intent = "/run/current-system/sw/bin/intent"
alias lock-core = "core protect lock"
alias unlock-core = "core protect unlock"
alias fg = "faelight-git"

# Pre-command decision rules — INT-171
before_run {
    if command contains "paru -Syu" { warn "System update — run during maintenance window?" }
    if command contains "sudo rm" { block "Use rm directly — sudo rm is too dangerous" }
}

alias gd = "GIT_EXTERNAL_DIFF=difft git diff"

set LIBVIRT_DEFAULT_URI = "qemu:///system"
