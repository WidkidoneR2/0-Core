function fish_prompt
    # Get current theme
    set theme (cat ~/dotfiles/themes/current.txt 2>/dev/null || echo "dark")

    # Set colors based on theme
    if test "$theme" = light
        # Light theme - vivid colors
        set timestamp_color "#8b5cf6" # Purple - different from blue!
        set folder_color "#1a6b54" # Dark teal
        set success_color "#00ff00" # NEON GREEN ✔
        set error_color "#ff0000" # NEON RED ✗
        set prompt_color "#c7256e" # Magenta
    else
        # Dark theme - bright variety
        set timestamp_color "#7400e7" # Purple!
        set folder_color "#d4ff00" # Brighter lime yellow
        set success_color "#00ff00" # NEON GREEN ✔
        set error_color "#ff0000" # NEON Red ✗
        set prompt_color "#56d4dd" # Cyan
    end

    # Render prompt
    set_color -o $timestamp_color
    echo -n "["(date "+%H:%M:%S")"] "
    set_color -o $folder_color
    echo -n "📁 "(basename (pwd))" "
    if test $status -eq 0
        set_color -o $success_color
        echo -n "✔ "
    else
        set_color -o $error_color
        echo -n "✗ "
    end
    set_color -o $prompt_color
    echo -n "❯ "
    set_color normal
end
