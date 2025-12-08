#!/usr/bin/env bash
# Colorful Rofi Power Menu - Faelight Forest

# Power menu options with colored icons
options="⏻  Shutdown
🔁  Reboot
💤  Suspend
🔒  Lock
🚪  Logout"

# Show menu with dedicated theme
chosen=$(echo -e "$options" | rofi -dmenu -i -p "Power Menu" -theme power-menu)

# Execute based on choice
case "$chosen" in
    *"Shutdown"*) systemctl poweroff ;;
    *"Reboot"*) systemctl reboot ;;
    *"Suspend"*) systemctl suspend ;;
    *"Lock"*) swaylock ;;
    *"Logout"*) hyprctl dispatch exit ;;
esac
