#!/usr/bin/env bash
# Custom Rofi-based Omarchy Menu (replaces walker)

case "${1:-main}" in
    main)
        # Main menu options
        options="🔄 Update System
🎨 Theme Menu
⚙️  Settings
🔌 Power Menu
📦 Package Manager
🔧 System Tools"
        
        choice=$(echo -e "$options" | rofi -dmenu -p "Omarchy Menu" -i)
        
        case "$choice" in
            "🔄 Update System")
                kitty --title "System Update" -e topgrade ;;
            "🎨 Theme Menu")
                $0 theme ;;
            "⚙️  Settings")
                $0 settings ;;
            "🔌 Power Menu")
                rofi -show power-menu ;;
            "📦 Package Manager")
                kitty --title "Package Manager" -e bash -c "yay; read -p 'Press enter to close...'" ;;
            "🔧 System Tools")
                $0 tools ;;
        esac
        ;;
    
    theme)
        # Theme switching
        options="🌙 Dark Theme
☀️  Light Theme
🎨 Theme from Wallpaper
🖼️  Change Wallpaper
🔙 Back"
        
        choice=$(echo -e "$options" | rofi -dmenu -p "Theme Menu" -i)
        
        case "$choice" in
            "🌙 Dark Theme")
                theme-switch dark ;;
            "☀️  Light Theme")
                theme-switch light ;;
            "🎨 Theme from Wallpaper")
                notify-send "Theme Engine" "v2.8.2+ feature - coming soon!" ;;
            "🖼️  Change Wallpaper")
                # Use rofi file browser or image selector
                notify-send "Wallpaper" "Feature coming soon!" ;;
            "🔙 Back")
                $0 main ;;
        esac
        ;;
    
    settings)
        # Settings menu
        options="🖥️  Display Settings
⌨️  Keyboard Settings
🖱️  Mouse Settings
🔊 Audio Settings
🌐 Network Settings
🔙 Back"
        
        choice=$(echo -e "$options" | rofi -dmenu -p "Settings" -i)
        
        case "$choice" in
            "🖥️  Display Settings")
                kitty --title "Display Settings" -e bash -c "hyprctl monitors; read -p 'Press enter to close...'" ;;
            "⌨️  Keyboard Settings")
                nvim ~/.config/hypr/input.conf ;;
            "🖱️  Mouse Settings")
                nvim ~/.config/hypr/input.conf ;;
            "🔊 Audio Settings")
                pavucontrol ;;
            "🌐 Network Settings")
                kitty --title "Network Manager" -e nmtui ;;
            "🔙 Back")
                $0 main ;;
        esac
        ;;
    
    tools)
        # System tools
        options="🔍 System Monitor (btop)
🐳 Docker (lazydocker)
📊 Disk Usage
🧹 Clean System
🔍 Search Files
🔙 Back"
        
        choice=$(echo -e "$options" | rofi -dmenu -p "System Tools" -i)
        
        case "$choice" in
            "🔍 System Monitor (btop)")
                kitty --title "System Monitor" -e btop ;;
            "🐳 Docker (lazydocker)")
                kitty --title "Docker" -e lazydocker ;;
            "📊 Disk Usage")
                kitty --title "Disk Usage" -e bash -c "df -h; read -p 'Press enter to close...'" ;;
            "🧹 Clean System")
                kitty --title "Clean System" -e bash -c "yay -Sc; read -p 'Press enter to close...'" ;;
            "🔍 Search Files")
                kitty --title "Search" -e bash -c "cd && fzf" ;;
            "🔙 Back")
                $0 main ;;
        esac
        ;;
    
    *)
        echo "Usage: $0 [main|theme|settings|tools]"
        exit 1
        ;;
esac
