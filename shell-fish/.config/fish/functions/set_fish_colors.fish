function set_fish_colors
    # Check current theme
    set theme (cat ~/0-core/themes/current.txt 2>/dev/null || echo "dark")

    if test "$theme" = light
        # Light theme - VERY DARK colors for readability
        set -g fish_color_command 1a6b54 --bold # Dark teal commands
        set -g fish_color_param 2a1f05 # Almost black params - NO YELLOW
        set -g fish_color_quote 2d8872 # Dark teal strings
        set -g fish_color_escape 1a6b54 # Dark teal ~
        set -g fish_color_valid_path 1a6b54 --underline # Dark teal paths - NO LIGHT GREEN
        set -g fish_color_redirection 1a6b54
        set -g fish_color_end 2d8872
        set -g fish_color_error c94c4c
        set -g fish_color_comment 3d6652
        set -g fish_color_autosuggestion 7ecfbd # Bright teal - readable!
        set -g fish_color_operator 1a6b54
        set -g fish_color_keyword 2d8872
    else
        # Dark theme - bright colors
        set -g fish_color_command f59e0b --bold # ORANGE commands (not blue!)
        set -g fish_color_param fbbf24 # Yellow params
        set -g fish_color_quote c7df63 # Lime strings
        set -g fish_color_escape c7df63 # Lime special chars
        set -g fish_color_valid_path c7df63 --underline # Lime paths
        set -g fish_color_redirection f59e0b # Orange
        set -g fish_color_end f59e0b # Orange
        set -g fish_color_error ff0066 # Neon pink
        set -g fish_color_comment 98e09a # Green
        set -g fish_color_autosuggestion 8ed1a3 # Soft green - matches your secondary color
        set -g fish_color_operator c7df63
        set -g fish_color_keyword c7df63
    end
end
