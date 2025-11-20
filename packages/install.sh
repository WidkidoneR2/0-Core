#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🌲 Faelight Forest - Package Restoration Script
# Installs all packages from saved lists
# ═══════════════════════════════════════════════════════════

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║   🌲 FAELIGHT FOREST PACKAGE RESTORATION               ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check for yay
if ! command -v yay &> /dev/null; then
    echo "❌ yay is not installed. Installing yay first..."
    sudo pacman -S --needed git base-devel
    git clone https://aur.archlinux.org/yay.git /tmp/yay
    cd /tmp/yay
    makepkg -si --noconfirm
    cd -
fi

# Install official packages
echo "📦 Installing official packages..."
if [ -f "$SCRIPT_DIR/official.txt" ]; then
    sudo pacman -S --needed - < "$SCRIPT_DIR/official.txt"
    echo "✅ Official packages installed"
else
    echo "⚠️  No official packages list found"
fi

# Install AUR packages
echo "📦 Installing AUR packages..."
if [ -f "$SCRIPT_DIR/aur.txt" ]; then
    yay -S --needed - < "$SCRIPT_DIR/aur.txt"
    echo "✅ AUR packages installed"
else
    echo "⚠️  No AUR packages list found"
fi

# Install Flatpak apps
if command -v flatpak &> /dev/null && [ -f "$SCRIPT_DIR/flatpak.txt" ]; then
    echo "📦 Installing Flatpak applications..."
    while read -r app; do
        [ -z "$app" ] && continue
        flatpak install -y flathub "$app"
    done < "$SCRIPT_DIR/flatpak.txt"
    echo "✅ Flatpak apps installed"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║        ✅ PACKAGE RESTORATION COMPLETE!                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
