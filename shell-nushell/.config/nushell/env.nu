# 🌲 Faelight Forest - Nushell Environment

# Path
$env.PATH = ($env.PATH | split row (char esep) | prepend [
    $"($env.HOME)/0-core/scripts"
    $"($env.HOME)/.cargo/bin"
    $"($env.HOME)/.local/bin"
])

# Starship prompt
mkdir ~/.cache/starship
starship init nu | save -f ~/.cache/starship/init.nu

# Editor
$env.EDITOR = "nvim"
$env.VISUAL = "nvim"

# XDG
$env.XDG_CONFIG_HOME = $"($env.HOME)/.config"
$env.XDG_DATA_HOME = $"($env.HOME)/.local/share"
$env.XDG_STATE_HOME = $"($env.HOME)/.local/state"
$env.XDG_CACHE_HOME = $"($env.HOME)/.cache"
