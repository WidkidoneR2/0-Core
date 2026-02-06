# ═══════════════════════════════════════════════════════════
# 🌲 FAELIGHT FOREST - ZSH SHELL CONFIGURATION
# Version 9.3.0 - FAST Edition ⚡
# ═══════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════
# 🛡️ PROTECTION & ERROR HANDLING
# ═══════════════════════════════════════════════════════════

setopt NO_BANG_HIST NO_HIST_EXPAND
setopt AUTO_PUSHD PUSHD_IGNORE_DUPS PUSHD_SILENT

# ═══════════════════════════════════════════════════════════
# 🎨 ENVIRONMENT & PATH
# ═══════════════════════════════════════════════════════════

export PATH="$HOME/.local/bin:$HOME/bin:$HOME/0-core/scripts:$HOME/.cargo/bin:$PATH"
export EDITOR=nvim
export VISUAL=nvim
export FM_EDITOR=nvim

# ═══════════════════════════════════════════════════════════
# ⚡ COMPLETIONS (OPTIMIZED)
# ═══════════════════════════════════════════════════════════

# Only rebuild cache once per day
autoload -Uz compinit
setopt EXTENDEDGLOB
for dump in ~/.zcompdump(N.mh+24); do
  compinit
done
unsetopt EXTENDEDGLOB
compinit -C  # Always use cached version

# ═══════════════════════════════════════════════════════════
# 🎨 PROMPT
# ═══════════════════════════════════════════════════════════

eval "$(starship init zsh)"

# ═══════════════════════════════════════════════════════════
# 📦 PLUGINS (Load synchronously but fast)
# ═══════════════════════════════════════════════════════════

source ~/.zsh/zsh-autosuggestions/zsh-autosuggestions.zsh
source ~/.zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# ═══════════════════════════════════════════════════════════
# 📦 CONFIGS
# ═══════════════════════════════════════════════════════════

source ~/.config/zsh/completions.zsh
source ~/.config/zsh/functions.zsh
source ~/.config/zsh/aliases.zsh
    source ~/.config/zsh/minimal-prompt.zsh  # Fallback prompt

# ═══════════════════════════════════════════════════════════
# 🎯 HISTORY
# ═══════════════════════════════════════════════════════════

HISTFILE=~/.zsh_history
HISTSIZE=50000
SAVEHIST=50000
setopt EXTENDED_HISTORY HIST_EXPIRE_DUPS_FIRST HIST_IGNORE_DUPS HIST_IGNORE_SPACE HIST_VERIFY SHARE_HISTORY

# ═══════════════════════════════════════════════════════════
# 🌲 WELCOME
# ═══════════════════════════════════════════════════════════

if [[ -o interactive ]]; then
    faelight-fetch
    echo -e "\033[1;32m🌲 Welcome to Faelight Forest v9.3.0 - Sway Edition!\033[0m"
    checkupdates 2>/dev/null &!
    echo "This is my Happy Place!!!"
    echo "💡 Quick: doctor | health | int list | keys"
fi

# ═══════════════════════════════════════════════════════════
# 🎯 KEY BINDINGS
# ═══════════════════════════════════════════════════════════

bindkey -e
bindkey '^R' history-incremental-search-backward
bindkey '^S' history-incremental-search-forward
