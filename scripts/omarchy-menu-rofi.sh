#!/usr/bin/env bash
# Omarchy Rofi Menu - Fully functional, theme-aware
# Requires: rofi, kitty, hyprctl, systemctl, makoctl, fzf, pavucontrol, topgrade, yay, btop, lazydocker

ROFI_THEME="$HOME/.config/rofi/faelight-forest.rasi"

# Helper to show a menu
show_menu() {
    echo -e "$1" | rofi -dmenu -p "$2" -i -theme "$ROFI_THEME"
}

case "${1:-main}" in
    main)
        options="🔄 Update System
🎨 Theme Menu
⚙️ Settings
🔌 Power Menu
📦 Package Manager
🔧 System Tools"
        choice=$(show_menu "$options" "Omarchy Menu")

        case "$choice" in
            "🔄 Update System") kitty --title "System Update" -e topgrade ;;
            "🎨 Theme Menu") $0 theme ;;
            "⚙️ Settings") $0 settings ;;
            "🔌 Power Menu") $0 power ;;
            "📦 Package Manager") kitty --title "Package Manager" -e bash -c "yay; read -p 'Press enter to close...'" ;;
            "🔧 System Tools") $0 tools ;;
        esac
        ;;
    
    theme)
        options="🌙 Dark Theme
☀️ Light Theme
🎨 Theme from Wallpaper
🖼️ Change Wallpaper
🔙 Back"
        choice=$(show_menu "$options" "Theme Menu")

        case "$choice" in
            "🌙 Dark Theme") theme-switch dark ;;
            "☀️ Light Theme") theme-switch light ;;
            "🎨 Theme from Wallpaper") notify-send "Theme Engine" "Feature coming soon!" ;;
            "🖼️ Change Wallpaper") notify-send "Wallpaper" "Feature coming soon!" ;;
            "🔙 Back") $0 main ;;
        esac
        ;;

    settings)
        options="🖥️ Display Settings
⌨️ Keyboard Settings
🖱️ Mouse Settings
🔊 Audio Settings
🌐 Network Settings
🔙 Back"
        choice=$(show_menu "$options" "Settings Menu")

        case "$choice" in
            "🖥️ Display Settings") kitty --title "Display Settings" -e bash -c "hyprctl monitors; read -p 'Press enter to close...'" ;;
            "⌨️ Keyboard Settings") nvim ~/.config/hypr/input.conf ;;
            "🖱️ Mouse Settings") nvim ~/.config/hypr/input.conf ;;
            "🔊 Audio Settings") pavucontrol ;;
            "🌐 Network Settings") kitty --title "Network Manager" -e nmtui ;;
            "🔙 Back") $0 main ;;
        esac
        ;;

    tools)
        options="🔍 System Monitor (btop)
🐳 Docker (lazydocker)
📊 Disk Usage
🧹 Clean System
🔍 Search Files
🔙 Back"
        choice=$(show_menu "$options" "System Tools")

        case "$choice" in
            "🔍 System Monitor (btop)") kitty --title "System Monitor" -e btop ;;
            "🐳 Docker (lazydocker)") kitty --title "Docker" -e lazydocker ;;
            "📊 Disk Usage") kitty --title "Disk Usage" -e bash -c "df -h; read -p 'Press enter to close...'" ;;
            "🧹 Clean System") kitty --title "Clean System" -e bash -c "yay -Sc; read -p 'Press enter to close...'" ;;
            "🔍 Search Files") kitty --title "Search Files" -e bash -c "cd && fzf" ;;
            "🔙 Back") $0 main ;;
        esac
        ;;

    power)
        options="🔒 Lock
💤 Suspend
🔁 Reboot
⏻ Shutdown
🔙 Back"
        choice=$(show_menu "$options" "Power Menu")

        case "$choice" in
            "🔒 Lock") swaylock ;;
            "💤 Suspend") systemctl suspend ;;
            "🔁 Reboot") systemctl reboot ;;
            "⏻ Shutdown") systemctl poweroff ;;
            "🔙 Back") $0 main ;;
        esac
        ;;

    *)
        echo "Usage: $0 [main|theme|settings|tools|power]"
        exit 1
        ;;
esac

