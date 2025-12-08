function dot-doctor --description "Health check for Faelight Forest dotfiles"
    echo "🏥 Dotfile Health Check - Faelight Forest"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    set -l errors 0
    set -l warnings 0

    # Check 1: Stow Symlinks
    echo "🔗 Checking Stow symlinks..."
    _check_stow_symlinks
    if test $status -ne 0
        set errors (math $errors + 1)
    end
    echo ""

    # Check 2: Services
    echo "🔄 Checking system services..."
    _check_services
    if test $status -ne 0
        set warnings (math $warnings + 1)
    end
    echo ""

    # Check 3: Binary Dependencies
    echo "📦 Checking binary dependencies..."
    _check_binaries
    if test $status -ne 0
        set warnings (math $warnings + 1)
    end
    echo ""

    # Final report
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if test $errors -eq 0 -a $warnings -eq 0
        echo "✅ All checks passed! System healthy! 🌲"
        return 0
    else
        echo "⚠️  Found $errors errors and $warnings warnings"
        return 1
    end
end

# Helper function: Check Stow symlinks
function _check_stow_symlinks
    set -l has_errors 0

    # Expected directory symlinks (whole directory)
    set -l dir_symlinks \
        ~/.config/hypr \
        ~/.config/waybar \
        ~/.config/nvim \
        ~/.config/yazi \
        ~/.config/mako

    # Individual file symlinks (check directory exists)
    set -l file_based \
        ~/.config/fish \
        ~/.config/kitty
    ~/.config/rofi

    set -l valid 0
    set -l broken 0

    # Check directory symlinks
    for link in $dir_symlinks
        if test -L $link
            set target (readlink $link)
            if string match -q "*dotfiles*" $target
                set valid (math $valid + 1)
            else
                echo "   ❌ $link points to wrong location"
                set broken (math $broken + 1)
                set has_errors 1
            end
        else if test -e $link
            echo "   ❌ $link exists but is not a symlink"
            set broken (math $broken + 1)
            set has_errors 1
        else
            echo "   ❌ $link is missing"
            set broken (math $broken + 1)
            set has_errors 1
        end
    end

    # Check file-based directories (Fish, Kitty)
    for dir in $file_based
        if test -d $dir
            set valid (math $valid + 1)
        else
            echo "   ❌ $dir is missing"
            set broken (math $broken + 1)
            set has_errors 1
        end
    end

    if test $has_errors -eq 0
        echo "   ✅ All $valid packages properly stowed"
    else
        echo "   ⚠️  $valid valid, $broken issues"
    end

    return $has_errors
end

# Helper function: Check system services
function _check_services
    set -l has_warnings 0

    # Services to check
    set -l services \
        mullvad-daemon \
        fail2ban \
        cronie

    set -l running 0
    set -l stopped 0

    for service in $services
        if systemctl is-active --quiet $service
            set running (math $running + 1)
        else
            echo "   ⚠️  $service is not running"
            set stopped (math $stopped + 1)
            set has_warnings 1
        end
    end

    if test $has_warnings -eq 0
        echo "   ✅ All $running services running"
    else
        echo "   ⚠️  $running running, $stopped stopped"
    end

    return $has_warnings
end

# Helper function: Check binary dependencies
function _check_binaries
    set -l has_warnings 0

    # Important binaries to check
    set -l binaries \
        kitty \
        waybar \
        hyprctl \
        rofi \
        nvim \
        yazi \
        fish \
        brave \
        mullvad \
        fail2ban-client \
        btop \
        eza \
        bat \
        fd \
        rg \
        git \
        stow

    set -l found 0
    set -l missing 0

    for binary in $binaries
        if command -v $binary >/dev/null 2>&1
            set found (math $found + 1)
        else
            echo "   ⚠️  $binary not found in PATH"
            set missing (math $missing + 1)
            set has_warnings 1
        end
    end

    if test $has_warnings -eq 0
        echo "   ✅ All $found binaries found"
    else
        echo "   ⚠️  $found found, $missing missing"
    end

    return $has_warnings
end
