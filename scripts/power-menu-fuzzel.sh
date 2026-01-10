#!/usr/bin/env bash
# Fuzzel Power Menu - Faelight Forest

# Power menu options
options="⏻  Shutdown
🔁  Reboot
💤  Suspend
🔒  Lock
🚪  Logout"

# Show menu with fuzzel
chosen=$(echo -e "$options" | fuzzel --dmenu \
    --prompt="Power: " \
    --width=30 \
    --lines=5 \
    --line-height=32 \
    --font="Hack Nerd Font Mono:size=14")

# Execute based on choice
case "$chosen" in
    *"Shutdown"*) systemctl poweroff ;;
    *"Reboot"*) systemctl reboot ;;
    *"Suspend"*) systemctl suspend ;;
    *"Lock"*) swaylock ;;
    *"Logout"*) swaymsg exit ;;
esac
