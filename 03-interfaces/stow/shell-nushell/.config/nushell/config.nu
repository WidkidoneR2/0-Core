# 🌲 Faelight Forest - Nushell Configuration
# Version 8.3.0

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
alias 0c = cd ~/0-core
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

def welcome [] {
    # Get system info
   let health = try {
    let result = (^dot-doctor | complete)
    if ($result.stdout | str contains "100%") {
        "🟢 100%"
    } else if ($result.stdout | str contains "92%") {
        "🟢 92%"
    } else {
        "⚠️ Check"
    }
} catch {
    "🔴 Error"
} 
    
    let core_status = if ("~/.core-locked" | path expand | path exists) {
        "🔒 locked"
    } else {
        "🔓 unlocked"
    }
    
    let profile = try {
        open ~/.dotprofile | str trim
    } catch {
        "DEF"
    }
    
    let uptime = try {
        sys host | get uptime
    } catch {
        "N/A"
    }
    
    let kernel = try {
        ^uname -r | str trim
    } catch {
        "unknown"
    }
    
    # Print banner
    print $"(ansi green)🌲 Faelight Forest v8.3.0(ansi reset)"
    print "────────────────────────"
    print $"profile   ($profile)"
    print $"core      ($core_status)"
    print $"health    ($health)"
    print "wm        sway"
    print $"term      ($env.TERM)"
    print "shell     nushell"
    print $"kernel    ($kernel)"
    print $"uptime    ($uptime)"
    print "host      faelight"
    print ""
    print $"(ansi green)🌲 Welcome to Faelight Forest v8.3.0 - Nushell Edition!(ansi reset)"
    print $"(ansi cyan)This is my Happy Place!!!(ansi reset)"
    print $"(ansi yellow)💡 Quick: doctor | int list | keys(ansi reset)"
}

echo '
welcome
source ~/.cache/starship/init.nu' >> ~/.config/nushell/config.nu

welcome
source ~/.cache/starship/init.nu
source $"($nu.home-dir)/.cargo/env.nu"
