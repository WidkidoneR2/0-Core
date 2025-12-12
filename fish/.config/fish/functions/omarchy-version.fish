function omarchy-version --description "Show Omarchy system version"
    set -l version_file ~/dotfiles/VERSION

    if test -f $version_file
        set -l omarchy_version (cat $version_file)
        echo "🌲 Omarchy v$omarchy_version"

        pushd ~/dotfiles >/dev/null
        set -l last_commit (git log -1 --format="%h - %s" 2>/dev/null)
        if test -n "$last_commit"
            echo "   Last update: $last_commit"
        end
        popd >/dev/null
    else
        echo "❌ VERSION file not found"
        return 1
    end
end
