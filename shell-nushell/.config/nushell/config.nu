# 🌲 Faelight Forest - Nushell Configuration
# Version 6.5.0

# ═══════════════════════════════════════════════════════════
# 🎨 APPEARANCE
# ═══════════════════════════════════════════════════════════

$env.config = {
    show_banner: false
    
    ls: {
        use_ls_colors: true
        clickable_links: true
    }
    
    table: {
        mode: rounded
        index_mode: always
        trim: {
            methodology: wrapping
            wrapping_try_keep_words: true
        }
    }
    
    completions: {
        case_sensitive: false
        quick: true
        partial: true
        algorithm: "fuzzy"
    }
    
    history: {
        max_size: 10000
        sync_on_enter: true
        file_format: "sqlite"
    }
    
    keybindings: [
        {
            name: clear_screen
            modifier: control
            keycode: char_l
            mode: [emacs, vi_normal, vi_insert]
            event: { send: clearscreen }
        }
    ]
}

# ═══════════════════════════════════════════════════════════
# 🔧 ALIASES - Faelight Forest Core
# ═══════════════════════════════════════════════════════════

# Navigation
alias core = cd ~/0-core
alias src = cd ~/1-src
alias projects = cd ~/2-projects
alias archive = cd ~/3-archive
alias media = cd ~/4-media

# Modern CLI
alias ls = eza --icons
alias ll = eza -la --icons --git
alias la = eza -a --icons
alias lt = eza --tree --icons --level=2
alias cat = bat

# Git
alias gs = git status
alias ga = git add
alias gc = git commit
alias gp = git push
alias gl = git log --oneline -10
alias gd = git diff

# Faelight Tools
alias fl = faelight
alias fls = faelight status
alias flh = faelight health

# System
alias update = sudo pacman -Syu
alias paci = sudo pacman -S
alias pacr = sudo pacman -Rns
alias pacs = pacman -Ss

# Quick edit
alias zshrc = nvim ~/.config/zsh/.zshrc
alias nuconfig = nvim ~/.config/nushell/config.nu

# ═══════════════════════════════════════════════════════════
# 🚀 STARTUP
# ═══════════════════════════════════════════════════════════

print $"(ansi green)🌲 Welcome to Faelight Forest v6.5.0 - Nushell Edition!(ansi reset)"
source ~/.cache/starship/init.nu
