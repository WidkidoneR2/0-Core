# ═══════════════════════════════════════════════════════════
# 🌲 MINIMAL PROMPT - Pure ZSH Fallback
# Instant (<1ms) - No external dependencies
# ═══════════════════════════════════════════════════════════

# Enable minimal prompt with: minimal_prompt
# Return to Starship with: starship_prompt

minimal_prompt() {
    # Disable Starship
    STARSHIP_DISABLED=1
    
    # Set up minimal prompt
    setopt PROMPT_SUBST
    
    PROMPT='%F{green}┌─%f %F{cyan}${${PWD/#$HOME/~}:t}%f $(minimal_git_info)$(minimal_exit_code)
%F{green}└─%f%F{green}❯%f '
    
    echo "⚡ Minimal prompt activated (instant!)"
    echo "Return to Starship with: starship_prompt"
}

starship_prompt() {
    unset STARSHIP_DISABLED
    eval "$(starship init zsh)"
    echo "🌲 Starship prompt restored!"
}

# Pure ZSH git info (fast!)
minimal_git_info() {
    # Check if in git repo
    git rev-parse --git-dir >/dev/null 2>&1 || return
    
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null)
    
    # Count changes (fast git plumbing)
    local modified=$(git diff --name-only 2>/dev/null | wc -l)
    local staged=$(git diff --staged --name-only 2>/dev/null | wc -l)
    
    # Build status string
    local status=""
    [[ $modified -gt 0 ]] && status="${status}!${modified}"
    [[ $staged -gt 0 ]] && status="${status}+${staged}"
    
    # Color based on changes
    if [[ -n $status ]]; then
        echo "%F{purple} ${branch}%f %F{yellow}[${status}]%f "
    else
        echo "%F{purple} ${branch}%f "
    fi
}

# Show exit code if non-zero
minimal_exit_code() {
    [[ $? -ne 0 ]] && echo "%F{red}✘%f "
}

# Optional: Zone indicator (if faelight-zone is fast)
minimal_zone() {
    if command -v faelight-zone >/dev/null 2>&1; then
        local zone=$(faelight-zone 2>/dev/null)
        [[ -n $zone ]] && echo "%F{green}${zone}%f "
    fi
}

# ═══════════════════════════════════════════════════════════
# USAGE:
#   minimal_prompt    - Switch to minimal
#   starship_prompt   - Switch back to Starship
# ═══════════════════════════════════════════════════════════
