#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# 🌲 Faelight Forest Dotfiles Installation Script
# Installs all configs with proper symlinks
# ═══════════════════════════════════════════════════════════

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║   🌲 FAELIGHT FOREST DOTFILES INSTALLATION             ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

DOTFILES_DIR="$HOME/dotfiles"

# Check if dotfiles directory exists
if [ ! -d "$DOTFILES_DIR" ]; then
    echo "❌ Error: Dotfiles directory not found at $DOTFILES_DIR"
    exit 1
fi

# Backup existing configs
echo "📦 Backing up existing configs..."
BACKUP_DIR="$HOME/.config_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup function
backup_if_exists() {
    if [ -e "$1" ]; then
        echo "  Backing up: $1"
        cp -r "$1" "$BACKUP_DIR/"
    fi
}

# Backup existing configs
backup_if_exists "$HOME/.config/fish"
backup_if_exists "$HOME/.config/hypr"
backup_if_exists "$HOME/.config/waybar"
backup_if_exists "$HOME/.config/kitty"
backup_if_exists "$HOME/.config/nvim"

echo "✅ Backups saved to: $BACKUP_DIR"
echo ""

# Install Fish config
echo "🐠 Installing Fish Shell config..."
mkdir -p "$HOME/.config/fish"
ln -sf "$DOTFILES_DIR/fish/config.fish" "$HOME/.config/fish/config.fish"
ln -sf "$DOTFILES_DIR/fish/functions" "$HOME/.config/fish/functions"

# Install Hyprland configs
echo "🖥️  Installing Hyprland configs..."
mkdir -p "$HOME/.config/hypr"
ln -sf "$DOTFILES_DIR/hypr/"*.conf "$HOME/.config/hypr/"

# Install Waybar configs
echo "📊 Installing Waybar configs..."
mkdir -p "$HOME/.config/waybar"
ln -sf "$DOTFILES_DIR/waybar/config.jsonc" "$HOME/.config/waybar/config.jsonc"
ln -sf "$DOTFILES_DIR/waybar/style.css" "$HOME/.config/waybar/style.css"

# Install Kitty config
echo "🐱 Installing Kitty config..."
mkdir -p "$HOME/.config/kitty"
ln -sf "$DOTFILES_DIR/kitty/kitty.conf" "$HOME/.config/kitty/kitty.conf"

# Install Yazi config
echo "📁 Installing Yazi config..."
mkdir -p "$HOME/.config/yazi"
ln -sf "$DOTFILES_DIR/yazi/yazi.toml" "$HOME/.config/yazi/yazi.toml"
ln -sf "$DOTFILES_DIR/yazi/theme.toml" "$HOME/.config/yazi/theme.toml"

# Install GTK configs
echo "🎨 Installing GTK theme configs..."
mkdir -p "$HOME/.config/gtk-3.0"
mkdir -p "$HOME/.config/gtk-4.0"
ln -sf "$DOTFILES_DIR/gtk-3.0/settings.ini" "$HOME/.config/gtk-3.0/settings.ini"
ln -sf "$DOTFILES_DIR/gtk-4.0/settings.ini" "$HOME/.config/gtk-4.0/settings.ini"

# Install Mako notification config
echo "🔔 Installing Mako notification config..."
mkdir -p "$HOME/.config/mako"
ln -sf "$DOTFILES_DIR/mako/config" "$HOME/.config/mako/config"

# Set Papirus folder colors
echo "🌅 Setting sunset-themed folder colors..."
if command -v papirus-folders &> /dev/null; then
    papirus-folders -C orange --theme Papirus-Dark
    sudo gtk-update-icon-cache -f -t /usr/share/icons/Papirus-Dark 2>/dev/null || true
    echo "  ✓ Sunset folder colors applied"
else
    echo "  ⚠️  papirus-folders not installed - run: yay -S papirus-icon-theme papirus-folders"
fi

# Install LazyVim configs
echo "📝 Installing LazyVim configs..."
mkdir -p "$HOME/.config/nvim"
cp -r "$DOTFILES_DIR/nvim/lua" "$HOME/.config/nvim/"

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
echo "🌲 Faelight Forest dotfiles installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Reload Fish: exec fish"
echo "  2. Reload Hyprland: hyprctl reload"
echo "  3. Restart Waybar: killall waybar && waybar &"
echo ""

