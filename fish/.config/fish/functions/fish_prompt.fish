function fish_prompt
    set_color -o "#d3d37c"
    echo -n "["(date "+%H:%M:%S")"] "

    set_color -o "#5bb7a5"
    echo -n "📁 "(basename (pwd))" "

    if test $status -eq 0
        set_color -o "#5bb77a"
        echo -n "✔ "
    else
        set_color -o "#c94c4c"
        echo -n "✗ "
    end

    set_color -o "#e8f5d5"
    echo -n "❯ "
    set_color normal
end
