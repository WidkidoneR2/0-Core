# ═══════════════════════════════════════════════════════════
# 🔧 TAB COMPLETIONS - Enhanced Edition
# Version 9.3.0 - Fast, Smart, Beautiful
# ═══════════════════════════════════════════════════════════

# Custom completions directory
fpath=(~/.config/zsh/completions $fpath)

# Already loaded in .zshrc with optimization
# (compinit -C for cached version)

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🎨 COMPLETION STYLING
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Enable menu selection with arrow keys
zstyle ':completion:*' menu select

# Case-insensitive matching
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|[._-]=* r:|=*' 'l:|=* r:|=*'

# Partial word completion
zstyle ':completion:*' completer _complete _match _approximate

# Approximate completion (typo tolerance)
zstyle ':completion:*:match:*' original only
zstyle ':completion:*:approximate:*' max-errors 1 numeric

# Group results by category
zstyle ':completion:*' group-name ''
zstyle ':completion:*:descriptions' format '%F{cyan}-- %d --%f'
zstyle ':completion:*:messages' format '%F{purple}-- %d --%f'
zstyle ':completion:*:warnings' format '%F{red}-- no matches --%f'

# Color completion listings (eza/ls colors)
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}

# Process completion
zstyle ':completion:*:processes' command 'ps -au$USER'
zstyle ':completion:*:*:kill:*:processes' list-colors '=(#b) #([0-9]#)*=0=01;31'

# Man page completion
zstyle ':completion:*:manuals' separate-sections true
zstyle ':completion:*:manuals.(^1*)' insert-sections true

# Cache completions (faster!)
zstyle ':completion:*' use-cache on
zstyle ':completion:*' cache-path ~/.cache/zsh/completion-cache

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 📁 DIRECTORY JUMPING
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Smart cd - don't show files, only directories
zstyle ':completion:*:cd:*' tag-order local-directories directory-stack path-directories
zstyle ':completion:*:cd:*' ignore-parents parent pwd

# Recent directories (from pushd stack)
zstyle ':completion:*:*:cd:*:directory-stack' menu yes select
zstyle ':completion:*:-tilde-:*' group-order 'named-directories' 'path-directories' 'users' 'expand'

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🌲 0-CORE TOOL COMPLETIONS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# ──────────────────────────────────────────────
# core-diff completions
# ──────────────────────────────────────────────
_core_diff() {
    local -a modes options packages
    
    modes=(
        'since:Compare to commit/tag'
        'working-tree:Explicit uncommitted changes'
        'summary:Stats only'
    )
    
    options=(
        '--open:Open diff tool (delta|meld)'
        '--verbose:Show individual files'
        '-v:Show individual files'
        '--quiet:Minimal output'
        '-q:Minimal output'
        '--high-risk:Show only critical/high'
        '--help:Show help'
        '-h:Show help'
        '--version:Show version'
    )
    
    # Get packages from 0-core directory (cached)
    packages=(${(f)"$(ls -d ~/0-core/*/ 2>/dev/null | xargs -n1 basename | grep -v -E '^(scripts|docs|automation|hooks|system|packages|installation|archive|INTENT)$')"})
    
    case "$words[2]" in
        --open)
            _values 'tool' delta meld
            ;;
        since)
            _values 'ref' $(git -C ~/0-core tag 2>/dev/null) HEAD~1 HEAD~2 HEAD~3
            ;;
        *)
            _describe -t modes 'mode' modes
            _describe -t options 'option' options
            _describe -t packages 'package' packages
            ;;
    esac
}
compdef _core_diff core-diff

# ──────────────────────────────────────────────
# dotctl completions
# ──────────────────────────────────────────────
_dotctl() {
    local -a commands packages
    
    commands=(
        'status:Show system and package versions'
        'bump:Bump package version'
        'history:Show package changelog'
        'health:Run system health check'
        'help:Show help'
    )
    
    packages=(${(f)"$(ls -d ~/0-core/*/ 2>/dev/null | xargs -n1 basename | grep -v -E '^(scripts|docs|automation|hooks|system|packages|installation|archive|INTENT)$')"})
    
    case "$words[2]" in
        bump|history)
            _describe -t packages 'package' packages
            ;;
        *)
            _describe -t commands 'command' commands
            ;;
    esac
}
compdef _dotctl dotctl

# ──────────────────────────────────────────────
# intent completions
# ──────────────────────────────────────────────
_intent() {
    local -a commands categories
    
    commands=(
        'add:Add new intent (interactive)'
        'list:List all intents'
        'show:Show specific intent'
        'search:Search intents by keyword/tag'
    )
    
    categories=(
        'decisions:Major architectural choices'
        'experiments:Things we tried'
        'philosophy:Core beliefs and principles'
        'future:Planned features and vision'
    )
    
    case "$words[2]" in
        list)
            _describe -t categories 'category' categories
            ;;
        show)
            _values 'id' $(find ~/0-core/intents -name "*.md" 2>/dev/null | xargs -n1 basename | sed 's/-.*//' | sort -u)
            ;;
        *)
            _describe -t commands 'command' commands
            ;;
    esac
}
compdef _intent intent

# ──────────────────────────────────────────────
# faelight-git completions
# ──────────────────────────────────────────────
_faelight_git() {
    local -a git_commands
    
    git_commands=(
        'add:Stage files'
        'commit:Commit changes'
        'push:Push to remote'
        'pull:Pull from remote'
        'status:Show status'
        'diff:Show differences'
        'log:Show commit log'
        'branch:Manage branches'
        'checkout:Switch branches'
        'clone:Clone repository'
    )
    
    _describe -t commands 'git command' git_commands
}
compdef _faelight_git faelight-git fg

# ──────────────────────────────────────────────
# faelight-fm completions
# ──────────────────────────────────────────────
_faelight_fm() {
    _files -/
}
compdef _faelight_fm faelight-fm fm

# ──────────────────────────────────────────────
# doctor/dot-doctor completions
# ──────────────────────────────────────────────
_dot_doctor() {
    _message 'run health check'
}
compdef _dot_doctor dot-doctor doctor d

# ──────────────────────────────────────────────
# bump-system-version completions
# ──────────────────────────────────────────────
_bump_system_version() {
    local -a flags
    
    flags=(
        '--dry-run:Preview without changes'
        '--help:Show help'
    )
    
    _describe -t flags 'option' flags
}
compdef _bump_system_version bump-system-version bump

# ──────────────────────────────────────────────
# profile completions
# ──────────────────────────────────────────────
_profile() {
    local -a commands profiles
    
    commands=(
        'list:List available profiles'
        'switch:Switch to profile'
        'current:Show current profile'
    )
    
    profiles=(
        'default:Default profile'
        'work:Work profile'
        'dev:Development profile'
        'gaming:Gaming profile'
        'low-power:Low power profile'
    )
    
    case "$words[2]" in
        switch)
            _describe -t profiles 'profile' profiles
            ;;
        *)
            _describe -t commands 'command' commands
            ;;
    esac
}
compdef _profile profile prof

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🚀 GIT COMPLETIONS (Enhanced)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Git checkout branch completion (cached)
_git_checkout_branch() {
    local -a branches
    branches=(${(f)"$(git branch 2>/dev/null | sed 's/^..//')"})
    _describe -t branches 'branch' branches
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 📦 PACKAGE MANAGER COMPLETIONS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# paru completions (via pacman)
compdef _pacman paru
compdef _pacman yay

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🎯 COMPLETION HELPERS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Accept completion and add space
bindkey '^I' complete-word          # Tab
bindkey '^[[Z' reverse-menu-complete # Shift+Tab

bindkey -M menuselect '^C' send-break

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🎯 MENU SELECTION KEYBINDINGS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Load menu selection module
zmodload zsh/complist

# Menu navigation (arrow keys work automatically)
# Use Ctrl+N/P for menu navigation
bindkey -M menuselect '^N' down-line-or-history
bindkey -M menuselect '^P' up-line-or-history

# Accept with Enter
bindkey -M menuselect '^M' .accept-line

# Cancel with Escape or Ctrl+C
bindkey -M menuselect '^[' send-break
bindkey -M menuselect '^C' send-break

# Use Ctrl+Space to accept and continue
bindkey -M menuselect '^ ' accept-and-hold

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 🎯 MENU SELECTION KEYBINDINGS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Load menu selection module FIRST
zmodload -i zsh/complist

# Then set up keybindings (arrow keys work by default!)
# These are optional - for Ctrl+N/P navigation
bindkey -M menuselect '^N' down-line-or-history   # Ctrl+N
bindkey -M menuselect '^P' up-line-or-history     # Ctrl+P
bindkey -M menuselect '^M' .accept-line           # Enter
bindkey -M menuselect '^[' send-break             # Escape
