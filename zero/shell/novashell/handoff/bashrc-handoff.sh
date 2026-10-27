if [[ $- == *i* ]] && [[ -z "$NSH_ACTIVE" ]] && [[ -x "$HOME/.local/bin/nsh" ]]; then
    if [[ -e "$HOME/.config/nsh/off" ]]; then
        echo "nsh handoff is off (rm ~/.config/nsh/off to turn it on); staying in bash." >&2
    else
        export NSH_ACTIVE=1
        "$HOME/.local/bin/nsh"
        nsh_status=$?
        case $nsh_status in
            101) nsh_why="panic" ;;
            132) nsh_why="SIGILL" ;;
            134) nsh_why="SIGABRT" ;;
            135) nsh_why="SIGBUS" ;;
            139) nsh_why="SIGSEGV" ;;
            *) exit "$nsh_status" ;;
        esac
        echo "nsh crashed ($nsh_why, status $nsh_status); staying in bash. touch ~/.config/nsh/off to stop the handoff." >&2
        unset nsh_status nsh_why
    fi
fi
