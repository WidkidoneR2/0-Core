function faelight-version --description "Show Faelight Forest dotfiles version"
    set -l version_file ~/dotfiles/VERSION

    if test -f $version_file
        set -l ff_version (cat $version_file)
        echo "🌲 Faelight Forest v$ff_version"

        pushd ~/dotfiles >/dev/null
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
