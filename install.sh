#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🌲 Faelight Forest Dotfiles Installation Script
# Version 2.8.6 - GNU Stow Edition
# Installs all configs with GNU Stow for clean symlink management
# ═══════════════════════════════════════════════════════════
set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║   🌲 FAELIGHT FOREST DOTFILES INSTALLATION v2.6        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

DOTFILES_DIR="$HOME/0-core"

# Check if 0-core directory exists
if [ ! -d "$DOTFILES_DIR" ]; then
    echo "❌ Error: Dotfiles directory not found at $DOTFILES_DIR"
    exit 1
fi

# Check if stow is installed
if ! command -v stow &> /dev/null; then
    echo "❌ Error: GNU Stow is not installed"
    echo "Install with: sudo pacman -S stow"
    exit 1
fi

# Backup existing configs
echo "📦 Backing up existing configs..."
BACKUP_DIR="$HOME/.config_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup function
backup_if_exists() {
    if [ -e "$1" ] && [ ! -L "$1" ]; then
        echo "  Backing up: $1"
        cp -r "$1" "$BACKUP_DIR/"
    fi
}

# Backup existing configs (only if they're not already symlinks)
backup_if_exists "$HOME/.config/fish"
backup_if_exists "$HOME/.config/hypr"
backup_if_exists "$HOME/.config/waybar"
backup_if_exists "$HOME/.config/kitty"
backup_if_exists "$HOME/.config/nvim"
backup_if_exists "$HOME/.config/yazi"
backup_if_exists "$HOME/.config/mako"
backup_if_exists "$HOME/.config/gtk-3.0"
backup_if_exists "$HOME/.config/gtk-4.0"

echo "✅ Backups saved to: $BACKUP_DIR"
echo ""

# Remove old configs (now that they're backed up)
echo "🗑️  Removing old configs to prepare for Stow..."
rm -rf "$HOME/.config/fish"
rm -rf "$HOME/.config/hypr"
rm -rf "$HOME/.config/waybar"
rm -rf "$HOME/.config/kitty"
rm -rf "$HOME/.config/nvim"
rm -rf "$HOME/.config/yazi"
rm -rf "$HOME/.config/mako"
rm -rf "$HOME/.config/gtk-3.0"
rm -rf "$HOME/.config/gtk-4.0"
echo "✅ Ready for Stow installation!"
echo ""

# Change to 0-core directory for stow
cd "$DOTFILES_DIR"

# Install configs using GNU Stow
echo "🔗 Installing configurations with GNU Stow..."

# Stow all packages
echo "  🐠 Installing Fish Shell config..."
stow -v fish

echo "  🖥️  Installing Hyprland configs..."
stow -v hypr

echo "  📊 Installing Waybar configs..."
stow -v waybar

echo "  🐱 Installing Kitty config..."
stow -v kitty

echo "  📁 Installing Yazi config..."
stow -v yazi

echo "  📝 Installing LazyVim configs..."
stow -v nvim

echo "  🔔 Installing Mako notifications..."
stow -v mako

echo "  🎨 Installing GTK themes..."
stow -v gtk

echo "✅ All configurations installed with GNU Stow!"
echo ""

# Set Papirus folder colors
echo "🌅 Setting sunset-themed folder colors..."
if command -v papirus-folders &> /dev/null; then
    papirus-folders -C orange --theme Papirus-Dark
    sudo gtk-update-icon-cache -f -t /usr/share/icons/Papirus-Dark 2>/dev/null || true
    echo "  ✓ Sunset folder colors applied"
else
    echo "  ⚠️  papirus-folders not installed - run: yay -S papirus-icon-theme papirus-folders"
fi

# Install scripts
echo "🔧 Installing scripts..."
mkdir -p "$HOME/.local/bin"
ln -sf "$DOTFILES_DIR/scripts/sys-cleanup" "$HOME/.local/bin/sys-cleanup"
ln -sf "$DOTFILES_DIR/scripts/quick-note" "$HOME/.local/bin/quick-note"
chmod +x "$HOME/.local/bin/sys-cleanup"
chmod +x "$HOME/.local/bin/quick-note"

# Install gitleaks and pre-commit hook
echo "🛡️  Setting up gitleaks secret scanning..."
if ! command -v gitleaks &> /dev/null; then
    echo "  Installing gitleaks..."
    yay -S --noconfirm gitleaks
else
    echo "  ✓ Gitleaks already installed"
fi

echo "  Installing pre-commit hook..."
mkdir -p "$DOTFILES_DIR/.git/hooks"
cp "$DOTFILES_DIR/hooks/pre-commit" "$DOTFILES_DIR/.git/hooks/pre-commit"
chmod +x "$DOTFILES_DIR/.git/hooks/pre-commit"

# Install documentation
echo "📚 Installing documentation..."
mkdir -p "$HOME/faelight-forest-docs"
cp "$DOTFILES_DIR/docs/COMPLETE_GUIDE.md" "$HOME/faelight-forest-docs/"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║   ✅ INSTALLATION COMPLETE!                            ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "🌲 Faelight Forest 0-core installed successfully!"
echo ""
echo "📝 GNU Stow is now managing your symlinks:"
echo "  - To remove a config: stow -D <package>"
echo "  - To reinstall: stow -R <package>"
echo "  - To check links: stow -n <package>"
echo ""
echo "Next steps:"
echo "  1. Reload Fish: exec fish"
echo "  2. Reload Hyprland: hyprctl reload"
echo "  3. Restart Waybar: killall waybar && waybar &"
echo ""

