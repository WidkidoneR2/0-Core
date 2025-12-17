function omarchy-version --description "Show complete Omarchy version info"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌲 Omarchy System Information"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Base system
    echo "Base OS:     Omarchy v3.2.2"

    # Faelight Forest 0-core
    set -l version_file ~/0-core/VERSION
    if test -f $version_file
        set -l ff_version (cat $version_file)
        echo "Dotfiles:    Faelight Forest v$ff_version"
    else
        echo "Dotfiles:    VERSION file not found"
    end

    # Last config update
    pushd ~/0-core >/dev/null
    set -l last_commit (git log -1 --format="%s" 2>/dev/null)

    if test -n "$last_commit"
        echo "Last update: $last_commit"
    end
    popd >/dev/null

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
end
