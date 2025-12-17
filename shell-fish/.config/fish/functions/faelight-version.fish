function faelight-version --description "Show Faelight Forest 0-core version"
    set -l version_file ~/0-core/VERSION

    if test -f $version_file
        set -l ff_version (cat $version_file)
        echo "🌲 Faelight Forest v$ff_version"

        pushd ~/0-core >/dev/null
        set -l last_commit (git log -1 --format="%s" 2>/dev/null)

        if test -n "$last_commit"
            echo "   Dotfiles: $last_commit"
        end
        popd >/dev/null
    else
        echo "❌ VERSION file not found"
        return 1
    end
end
