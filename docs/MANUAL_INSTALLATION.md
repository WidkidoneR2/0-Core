# Manual Installation Guide - Faelight Forest v3.4.2+

**Philosophy:** Manual control over automation. Understand each step.

---

## Prerequisites

### Required Software

```bash
# Arch Linux with base-devel
sudo pacman -S git stow

# Verify installation
git --version
stow --version
```

### System Requirements

- Arch Linux (or Arch-based distro)
- Wayland-compatible system
- AMD graphics (or modify for your GPU)
- Basic familiarity with terminal

---

## Installation Steps

### 1. Clone the Repository

```bash
# Clone to home directory
cd ~
git clone https://github.com/YOUR-USERNAME/0-Core.git
cd 0-Core

# Verify structure
ls -la
```

---

### 2. Review Package Structure

**Before installing anything, understand what you're getting:**

```bash
# List all packages
ls -d */

# Expected packages:
# - wm-hypr/          (Hyprland window manager)
# - bar-waybar/       (Status bar)
# - shell-zsh/        (Zsh shell)
# - editor-nvim/      (Neovim editor)
# - notif-mako/       (Notifications)
# - theme-gtk/        (GTK themes)
# - fm-yazi/          (File manager)
# - prompt-starship/  (Shell prompt)
```

**Review each package's .dotmeta:**

```bash
# See what each package does and its blast radius
cat wm-hypr/.dotmeta
cat shell-zsh/.dotmeta
# ... review all packages
```

---

### 3. Backup Existing Configs

**CRITICAL: Backup before proceeding!**

```bash
# Create timestamped backup
BACKUP_DIR="$HOME/.config_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup existing configs (if they exist)
[[ -d ~/.config/hypr ]] && cp -r ~/.config/hypr "$BACKUP_DIR/"
[[ -d ~/.config/waybar ]] && cp -r ~/.config/waybar "$BACKUP_DIR/"
[[ -d ~/.config/zsh ]] && cp -r ~/.config/zsh "$BACKUP_DIR/"
[[ -d ~/.config/nvim ]] && cp -r ~/.config/nvim "$BACKUP_DIR/"
[[ -d ~/.config/mako ]] && cp -r ~/.config/mako "$BACKUP_DIR/"
[[ -d ~/.config/yazi ]] && cp -r ~/.config/yazi "$BACKUP_DIR/"

echo "✅ Backups saved to: $BACKUP_DIR"
```

---

### 4. Remove Conflicting Configs

**Stow won't work if configs already exist (and aren't symlinks).**

```bash
# Remove existing configs (now that they're backed up)
rm -rf ~/.config/hypr
rm -rf ~/.config/waybar
rm -rf ~/.config/zsh
rm -rf ~/.config/nvim
rm -rf ~/.config/mako
rm -rf ~/.config/yazi
rm -rf ~/.config/gtk-3.0
rm -rf ~/.config/gtk-4.0
rm -rf ~/.config/starship.toml

echo "✅ Ready for stow installation"
```

---

### 5. Install Packages with Stow

**Install ONE package at a time. Test between each.**

```bash
cd ~/0-Core

# Start with critical packages in order:

# 1. Shell (high-risk)
stow shell-zsh
source ~/.config/zsh/.zshrc  # Test immediately

# 2. Window manager (CRITICAL - test carefully!)
stow wm-hypr
# Don't reload yet - test in nested session first

# 3. Status bar
stow bar-waybar

# 4. Notifications
stow notif-mako

# 5. GTK themes
stow theme-gtk

# 6. Editor
stow editor-nvim

# 7. File manager
stow fm-yazi

# 8. Shell prompt
stow prompt-starship
```

**After each stow, verify:**

```bash
# Check symlinks were created
ls -la ~/.config/hypr
ls -la ~/.config/waybar
# etc.

# Verify they point to 0-Core
readlink ~/.config/hypr/hyprland.conf
# Should show: /home/YOUR-USER/0-Core/wm-hypr/.config/hypr/hyprland.conf
```

---

### 6. Install Scripts

```bash
# Create local bin directory
mkdir -p ~/.local/bin

# Link scripts manually
cd ~/0-Core/scripts

# Link each script individually (review first!)
ln -sf "$(pwd)/core-diff" ~/.local/bin/core-diff
ln -sf "$(pwd)/dot-doctor" ~/.local/bin/dot-doctor
ln -sf "$(pwd)/safe-update" ~/.local/bin/safe-update
ln -sf "$(pwd)/core-protect" ~/.local/bin/core-protect
ln -sf "$(pwd)/dotctl" ~/.local/bin/dotctl

# Verify
ls -la ~/.local/bin/
which core-diff dot-doctor safe-update
```

---

### 7. Test Configuration

**BEFORE reloading Hyprland, test in a safe environment!**

```bash
# Option A: Test in nested Hyprland session
Hyprland  # Launches nested session
# Test keybindings, functionality
# Exit if broken: SUPER+M (or your configured exit key)

# Option B: Test config syntax
hyprland --config ~/.config/hypr/hyprland.conf --check

# If everything works in nested session, reload main session:
hyprctl reload
```

**Test each component:**

```bash
# Test shell
echo $SHELL  # Should show /usr/bin/zsh

# Test waybar
killall waybar && waybar &  # Should appear

# Test mako
notify-send "Test" "Notification working"

# Test nvim
nvim --version

# Test yazi
yazi --version
```

---

### 8. Enable Git Hooks (Optional)

**For secret scanning:**

```bash
cd ~/0-Core

# Install gitleaks (if not installed)
yay -S gitleaks

# Enable pre-commit hook
cp hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Test
git add .
git commit -m "test"  # Should scan for secrets
```

---

### 9. Enable Core Protection (Optional)

**Make 0-Core immutable:**

```bash
# Lock the core
lock-core

# Verify
lsattr -d ~/0-Core  # Should show 'i' flag

# To edit later
unlock-core
# ... make changes ...
lock-core
```

---

## Post-Installation

### Verify Health

```bash
# Run health check
dot-doctor

# Should show 100% or close to it
```

### Learn the System

```bash
# Read documentation
cat ~/0-Core/README.md
cat ~/0-Core/docs/THEORY_OF_OPERATION.md
cat ~/0-Core/docs/WORKFLOWS.md

# Check what changed
core-diff
```

### Customize

```bash
# Review configs BEFORE changing
cat ~/.config/hypr/hyprland.conf
cat ~/.config/waybar/config.jsonc

# Make changes in 0-Core (not in ~/.config)
unlock-core
cd ~/0-Core
nvim wm-hypr/.config/hypr/hyprland.conf

# Test and commit
hyprctl reload
git add wm-hypr/
git commit -m "feat: Custom keybinding"
lock-core
```

---

## Troubleshooting

### Stow Conflicts

```bash
# Error: "existing target is not a symlink"
# Solution: Remove the file first
rm ~/.config/problematic-file
stow wm-hypr

# Or use --adopt (moves existing file into 0-Core)
stow --adopt wm-hypr
```

### Broken Symlinks

```bash
# Find broken symlinks
find ~/.config -xtype l

# Remove them
find ~/.config -xtype l -delete

# Re-stow
cd ~/0-Core
stow wm-hypr
```

### Hyprland Won't Start

```bash
# Check logs
journalctl --user -u hyprland -n 50

# Check config syntax
hyprland --config ~/.config/hypr/hyprland.conf --check

# Restore backup
cp -r "$BACKUP_DIR/hypr" ~/.config/
```

### Shell Issues

```bash
# Reset shell
chsh -s /bin/bash  # Temporarily use bash
# Fix zsh config
# Then: chsh -s /usr/bin/zsh
```

---

## Uninstallation

### Remove Specific Package

```bash
cd ~/0-Core

# Unstow package
stow -D wm-hypr

# Verify symlinks removed
ls -la ~/.config/hypr  # Should not exist or be restored
```

### Complete Removal

```bash
# Unstow all packages
cd ~/0-Core
stow -D */

# Restore backups
cp -r "$BACKUP_DIR/"* ~/.config/

# Remove repository
rm -rf ~/0-Core
```

---

## Philosophy

**Why Manual Installation?**

1. **Understanding** - You see what each package does
2. **Control** - You choose what to install
3. **Safety** - Test each step before proceeding
4. **Learning** - Build knowledge, not dependencies
5. **Intentionality** - Every action is deliberate

**No automation. No surprises. Full control.** 🌲

---

## Next Steps

- Read [THEORY_OF_OPERATION.md](THEORY_OF_OPERATION.md)
- Review [WORKFLOWS.md](WORKFLOWS.md)
- Understand [PHILOSOPHY.md](PHILOSOPHY.md)
- Set up [safe-update](TOOLS.md#safe-update)

**Questions?** Read the docs or open an issue on GitHub.

**Welcome to Faelight Forest!** 🌲

---

_Last Updated: December 26, 2025 (v3.4.3)_  
_Manual installation preserves 0-Core philosophy: intent over automation._
