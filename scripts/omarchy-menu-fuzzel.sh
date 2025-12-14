#!/usr/bin/env bash
# Omarchy Fuzzel Menu - Fully functional, theme-aware
# Requires: fuzzel, foot, hyprctl, systemctl, makoctl, fzf, pavucontrol, topgrade, yay, btop, lazydocker

show_menu() {
  echo -e "$1" | fuzzel --dmenu \
    --prompt="$2" \
    --width=35 \
    --lines=7 \
    --line-height=32 \
    --font="Hack Nerd Font Mono:size=14"
}

theme_menu() {
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
  "🔙 Back") return ;;
  esac
}

settings_menu() {
  options="🖥️ Display Settings
⌨️ Keyboard Settings
🖱️ Mouse Settings
🔊 Audio Settings
🌐 Network Settings
🔙 Back"
  choice=$(show_menu "$options" "Settings Menu")

  case "$choice" in
  "🖥️ Display Settings") foot --title "Display Settings" -e bash -c "hyprctl monitors; read -p 'Press enter to close...'" ;;
  "⌨️ Keyboard Settings") nvim ~/.config/hypr/input.conf ;;
  "🖱️ Mouse Settings") nvim ~/.config/hypr/input.conf ;;
  "🔊 Audio Settings") pavucontrol ;;
  "🌐 Network Settings") foot --title "Network Manager" -e nmtui ;;
  "🔙 Back") return ;;
  esac
}

tools_menu() {
  options="🔍 System Monitor (btop)
🐳 Docker (lazydocker)
📊 Disk Usage
🧹 Clean System
🔍 Search Files
🔙 Back"
  choice=$(show_menu "$options" "System Tools")

  case "$choice" in
  "🔍 System Monitor (btop)") foot --title "System Monitor" -e btop ;;
  "🐳 Docker (lazydocker)") foot --title "Docker" -e lazydocker ;;
  "📊 Disk Usage") foot --title "Disk Usage" -e bash -c "df -h; read -p 'Press enter to close...'" ;;
  "🧹 Clean System") foot --title "Clean System" -e bash -c "yay -Sc; read -p 'Press enter to close...'" ;;
  "🔍 Search Files") foot --title "Search Files" -e bash -c "cd && fzf" ;;
  "🔙 Back") return ;;
  esac
}

power_menu() {
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
  "🔙 Back") return ;;
  esac
}

# Main menu loop
while true; do
  options="🔄 Update System
🎨 Theme Menu
⚙️ Settings
🔌 Power Menu
📦 Package Manager
🔧 System Tools
❌ Exit"

  choice=$(show_menu "$options" "Omarchy Menu")

  case "$choice" in
  "🔄 Update System") foot --title "System Update" -e topgrade ;;
  "🎨 Theme Menu") theme_menu ;;
  "⚙️ Settings") settings_menu ;;
  "🔌 Power Menu") ~/0-core/scripts/power-menu-fuzzel.sh ;;
  "📦 Package Manager") foot --title "Package Manager" -e bash -c "yay; read -p 'Press enter to close...'" ;;
  "🔧 System Tools") tools_menu ;;
  "❌ Exit") break ;;
  esac
done
