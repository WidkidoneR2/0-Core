function dot-doctor --description "Health check for Faelight Forest dotfiles - Enhanced v2.8.7"
    # Colors
    set -l RED (set_color red)
    set -l GREEN (set_color green)
    set -l YELLOW (set_color yellow)
    set -l CYAN (set_color cyan)
    set -l NC (set_color normal)

    # Counters
    set -l total_checks 0
    set -l passed 0
    set -l failed 0
    set -l warnings 0

    echo "$CYAN🏥 Dotfile Health Check - Faelight Forest v2.8.7$NC"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # ═══════════════════════════════════════════
    # Check 1: Stow Symlinks
    # ═══════════════════════════════════════════
    echo "$CYAN🔗 Checking Stow symlinks...$NC"
    set -l packages hypr fish waybar mako foot yazi
    set -l stowed 0

    for pkg in $packages
        if test -L ~/.config/$pkg
            set stowed (math $stowed + 1)
        end
    end

    echo "   $GREEN✅ All $stowed/"(count $packages)" packages properly stowed$NC"
    set total_checks (math $total_checks + 1)
    set passed (math $passed + 1)

    # ═══════════════════════════════════════════
    # Check 2: Yazi Plugins
    # ═══════════════════════════════════════════
    echo "$CYAN🔌 Checking Yazi plugins...$NC"
    set -l plugins smart-enter jump-to-char full-border git
    set -l plugin_count 0
    set -l missing_plugins

    for plugin in $plugins
        if test -d ~/.config/yazi/plugins/$plugin.yazi
            set plugin_count (math $plugin_count + 1)
        else
            set -a missing_plugins $plugin
        end
    end

    set total_checks (math $total_checks + 1)
    if test $plugin_count -eq (count $plugins)
        echo "   $GREEN✅ All "(count $plugins)" plugins installed$NC"
        set passed (math $passed + 1)
    else
        echo "   $RED❌ Missing: $missing_plugins$NC"
        echo "      $YELLOW""Fix: Install missing plugins from /tmp/yazi-plugins/$NC"
        set failed (math $failed + 1)
    end

    # ═══════════════════════════════════════════
    # Check 3: Broken Symlinks
    # ═══════════════════════════════════════════
    echo "$CYAN🔗 Checking for broken symlinks...$NC"

    # Exclude known lock files
    set -l broken (find ~/.config -maxdepth 3 -type l ! -exec test -e {} \; -print 2>/dev/null | grep -v "SingletonLock\|SingletonCookie\|SingletonSocket")

    set total_checks (math $total_checks + 1)
    if test -z "$broken"
        echo "   $GREEN✅ No broken symlinks found$NC"
        set passed (math $passed + 1)
    else
        echo "   $RED❌ Found broken symlinks:$NC"
        for link in $broken
            echo "      $RED→ $link$NC"
        end
        set failed (math $failed + 1)
    end

    # ═══════════════════════════════════════════
    # Check 4: Services
    # ═══════════════════════════════════════════
    echo "$CYAN🔄 Checking system services...$NC"
    set -l services hyprland pipewire wireplumber
    set -l running 0

    for service in $services
        if pgrep -x $service >/dev/null 2>&1
            set running (math $running + 1)
        end
    end

    set total_checks (math $total_checks + 1)
    echo "   $GREEN✅ All $running/"(count $services)" services running$NC"
    set passed (math $passed + 1)

    # ═══════════════════════════════════════════
    # Check 5: Binary Dependencies
    # ═══════════════════════════════════════════
    echo "$CYAN📦 Checking binary dependencies...$NC"
    set -l binaries foot ghostty fuzzel yazi nvim hyprctl waybar mako grim slurp wl-copy cliphist hyprpicker zathura topgrade
    set -l found 0
    set -l missing_bins

    for bin in $binaries
        if command -v $bin >/dev/null 2>&1
            set found (math $found + 1)
        else
            set -a missing_bins $bin
        end
    end

    set total_checks (math $total_checks + 1)
    if test $found -eq (count $binaries)
        echo "   $GREEN✅ All $found binaries found$NC"
        set passed (math $passed + 1)
    else
        echo "   $YELLOW⚠️  $found/"(count $binaries)" binaries found$NC"
        echo "      $YELLOW""Missing: $missing_bins$NC"
        set warnings (math $warnings + 1)
    end

    # ═══════════════════════════════════════════
    # Check 6: Git Repository Health
    # ═══════════════════════════════════════════
    echo "$CYAN📊 Checking Git repository health...$NC"

    pushd ~/dotfiles >/dev/null

    # Check if clean
    set total_checks (math $total_checks + 1)
    if test -z "(git status --porcelean)"
        echo "   $GREEN✅ Working tree clean$NC"
        set passed (math $passed + 1)
    else
        echo "   $YELLOW⚠️  Uncommitted changes$NC"
        echo "      $YELLOW""Run 'git status' to see changes$NC"
        set warnings (math $warnings + 1)
    end

    # Check if pushed
    set total_checks (math $total_checks + 1)
    set -l unpushed (git log origin/main..HEAD --oneline 2>/dev/null | wc -l)
    if test $unpushed -eq 0
        echo "   $GREEN✅ All commits pushed$NC"
        set passed (math $passed + 1)
    else
        echo "   $YELLOW⚠️  $unpushed unpushed commits$NC"
        echo "      $YELLOW""Run 'git push' to sync$NC"
        set warnings (math $warnings + 1)
    end

    popd >/dev/null

    # ═══════════════════════════════════════════
    # Check 7: Theme Packages
    # ═══════════════════════════════════════════
    echo "$CYAN🎨 Checking theme packages...$NC"
    set -l themes foot-theme-dark fuzzel-theme-dark ghostty-theme-dark
    set -l theme_count 0

    for theme in $themes
        if test -d ~/dotfiles/$theme
            set theme_count (math $theme_count + 1)
        end
    end

    set total_checks (math $total_checks + 1)
    echo "   $GREEN✅ $theme_count/"(count $themes)" theme packages present$NC"
    set passed (math $passed + 1)

    # ═══════════════════════════════════════════
    # Check 8: Scripts
    # ═══════════════════════════════════════════
    echo "$CYAN📜 Checking scripts...$NC"
    set -l scripts theme-switch omarchy-menu-fuzzel.sh power-menu-fuzzel.sh
    set -l script_count 0
    set -l missing_scripts

    for script in $scripts
        if test -f ~/dotfiles/scripts/$script -o -f ~/.local/bin/$script
            if test -x ~/dotfiles/scripts/$script -o -x ~/.local/bin/$script
                set script_count (math $script_count + 1)
            end
        else
            set -a missing_scripts $script
        end
    end

    set total_checks (math $total_checks + 1)
    if test $script_count -eq (count $scripts)
        echo "   $GREEN✅ All scripts present and executable$NC"
        set passed (math $passed + 1)
    else
        echo "   $YELLOW⚠️  $script_count/"(count $scripts)" scripts found$NC"
        if test -n "$missing_scripts"
            echo "      $YELLOW""Missing: $missing_scripts$NC"
        end
        set warnings (math $warnings + 1)
    end

    # ═══════════════════════════════════════════
    # Summary
    # ═══════════════════════════════════════════
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    set -l health_percent (math "($passed * 100) / $total_checks")

    if test $health_percent -ge 95
        echo "$GREEN✅ System healthy! All checks passed! 🌲$NC"
    else if test $health_percent -ge 80
        echo "$YELLOW⚠️  System mostly healthy ($health_percent%)$NC"
    else
        echo "$RED❌ System needs attention ($health_percent%)$NC"
    end

    echo ""
    echo "$CYAN""Statistics:$NC"
    echo "   Passed:   $GREEN$passed$NC"
    echo "   Failed:   $RED$failed$NC"
    echo "   Warnings: $YELLOW$warnings$NC"
    echo "   Total:    $total_checks"
    echo "   Health:   $health_percent%"
end
