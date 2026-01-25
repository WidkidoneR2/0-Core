# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH SHELL CONFIGURATION
# Version 8.0.0 - Faelight Forest
# Clean, modular, and intentional
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# 🛡️ PROTECTION & ERROR HANDLING
# ═══════════════════════════════════════════════════════════
# Disable history expansion (fixes git commit message issues)
setopt NO_BANG_HIST          # Don't treat ! specially
setopt NO_HIST_EXPAND        # Don't expand history references

# Directory history (Fish-like cd navigation)
setopt AUTO_PUSHD            # Auto push old dir to stack
setopt PUSHD_IGNORE_DUPS     # No duplicates in stack
setopt PUSHD_SILENT          # Don't print stack on pushd/popd

# ═══════════════════════════════════════════════════════════
# 🎨 ENVIRONMENT & PATH
# ═══════════════════════════════════════════════════════════
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/0-core/scripts:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"

# Editor
export EDITOR=nvim
export VISUAL=nvim

# ═══════════════════════════════════════════════════════════
# 🔐 DIRENV (Per-Directory Environments)
# ═══════════════════════════════════════════════════════════
eval "$(direnv hook zsh)"

# ═══════════════════════════════════════════════════════════
# 🎨 ZSH PLUGINS & ENHANCEMENTS
# ═══════════════════════════════════════════════════════════
# Autosuggestions (Fish-like)
if [[ -f ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh ]]; then
    source ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh
    ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=244'
fi

# Syntax highlighting (Fish-like)
if [[ -f ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]]; then
    source ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
fi

# ═══════════════════════════════════════════════════════════
# ⚠️  DANGEROUS COMMAND HIGHLIGHTING
# ═══════════════════════════════════════════════════════════
ZSH_HIGHLIGHT_HIGHLIGHTERS=(main brackets pattern)
typeset -A ZSH_HIGHLIGHT_PATTERNS

ZSH_HIGHLIGHT_PATTERNS+=('rm -rf *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('rm -fr *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('sudo rm -rf *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('chmod 777 *' 'fg=white,bold,bg=red')
ZSH_HIGHLIGHT_PATTERNS+=('dd if=*' 'fg=white,bold,bg=red')

# Faelight Forest syntax colors
ZSH_HIGHLIGHT_STYLES[command]='fg=cyan,bold'
ZSH_HIGHLIGHT_STYLES[alias]='fg=cyan,bold'
ZSH_HIGHLIGHT_STYLES[builtin]='fg=cyan,bold'
ZSH_HIGHLIGHT_STYLES[unknown-token]='fg=red,bold'
ZSH_HIGHLIGHT_STYLES[path]='fg=green'
ZSH_HIGHLIGHT_STYLES[globbing]='fg=yellow'
ZSH_HIGHLIGHT_STYLES[single-quoted-argument]='fg=yellow'
ZSH_HIGHLIGHT_STYLES[double-quoted-argument]='fg=yellow'

# ═══════════════════════════════════════════════════════════
# 📦 LOAD MODULAR COMPONENTS
# ═══════════════════════════════════════════════════════════
# Load completions
if [[ -f ~/.config/zsh/completions.zsh ]]; then
    source ~/.config/zsh/completions.zsh
fi

# Load functions (utilities, git guardrails, etc.)
if [[ -f ~/.config/zsh/functions.zsh ]]; then
    source ~/.config/zsh/functions.zsh
fi

# Load aliases (263 aliases organized by category)
if [[ -f ~/.config/zsh/aliases.zsh ]]; then
    source ~/.config/zsh/aliases.zsh
fi

# ═══════════════════════════════════════════════════════════
# 🌟 STARSHIP PROMPT
# ═══════════════════════════════════════════════════════════
eval "$(starship init zsh)"

# ═══════════════════════════════════════════════════════════
# 🚀 WELCOME MESSAGE
# ═══════════════════════════════════════════════════════════
if [[ -o interactive ]]; then
    faelight-fetch
    echo ""
    echo -e "\033[1;32m🌲 Welcome to Faelight Forest v8.2.0 - Sway Edition!\033[0m"
    if [[ -x ~/0-core/scripts/latest-update ]]; then
        local latest=$(~/0-core/scripts/latest-update)
        if [[ -n "$latest" ]]; then
            echo -e "\033[0;36m   Latest: $latest\033[0m"
        fi
    fi
    echo ""
    echo "This is my Happy Place!!!"
    echo ""
    echo "💡 Quick: doctor | health | int list | keys"
    echo ""
fi

# ═══════════════════════════════════════════════════════════
# 🌲 END OF FAELIGHT FOREST CONFIGURATION
# ═══════════════════════════════════════════════════════════
export TERM=foot
export COLORTERM=truecolor
alias ff='faelight-fetch'
fpath=(~/.zsh/completions $fpath)
