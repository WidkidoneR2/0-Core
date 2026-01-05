# ═══════════════════════════════════════════════════════════
# 🔧 TAB COMPLETIONS (v3.5.2)
# Discoverability for 0-Core tools
# ═══════════════════════════════════════════════════════════

# Initialize completion system
autoload -Uz compinit
compinit -d ~/.cache/zsh/zcompdump-$ZSH_VERSION

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
    
    # Get packages from 0-core directory
    packages=(${(f)"$(ls -d ~/0-core/*/ 2>/dev/null | xargs -n1 basename | grep -v -E '^(scripts|docs|automation|hooks|system|packages|installation|archive|INTENT)$')"})
    
    case "$words[2]" in
        --open)
            _values 'tool' delta meld
            ;;
        since)
            # Suggest git refs
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
            # Suggest intent IDs
            _values 'id' $(find ~/0-core/INTENT -name "*.md" 2>/dev/null | xargs -n1 basename | sed 's/-.*//' | sort -u)
            ;;
        *)
            _describe -t commands 'command' commands
            ;;
    esac
}
compdef _intent intent

# ──────────────────────────────────────────────
# dot-doctor completions (minimal - no flags)
# ──────────────────────────────────────────────
_dot_doctor() {
    # dot-doctor has no flags currently
    _message 'no options'
}
compdef _dot_doctor dot-doctor

# profile completions
_profile() {
    compadd list status history edit help default gaming work low-power
}
compdef _profile profile
