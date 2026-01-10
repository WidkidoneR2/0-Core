# 🛠️ Manual Installation Guide - Faelight Forest v6.0.0

This guide covers manual installation and management of 0-Core on Sway.

---

## 📋 Prerequisites

- Arch Linux (or Arch-based distro)
- Sway installed
- Git installed
- GNU Stow installed
```bash
sudo pacman -S sway foot fuzzel mako grim slurp wl-clipboard git stow
```

---

## 📦 Package Structure
0-core/
├── wm-sway/           (Sway window manager)
├── shell-zsh/         (Zsh configuration)
├── prompt-starship/   (Starship prompt)
├── editor-nvim/       (Neovim + LazyVim)
├── fm-yazi/           (Yazi file manager)
├── vcs-git/           (Git configuration)
└── rust-tools/        (Custom Rust binaries)

Each package has a `.dotmeta` file:
```bash
cat wm-sway/.dotmeta
```

---

## 🔄 Backup Existing Configs

Before installing, backup your current configs:
```bash
BACKUP_DIR=~/config-backup-$(date +%Y%m%d)
mkdir -p "$BACKUP_DIR"

[[ -d ~/.config/sway ]] && cp -r ~/.config/sway "$BACKUP_DIR/"
[[ -d ~/.config/foot ]] && cp -r ~/.config/foot "$BACKUP_DIR/"
[[ -d ~/.config/nvim ]] && cp -r ~/.config/nvim "$BACKUP_DIR/"
[[ -f ~/.zshrc ]] && cp ~/.zshrc "$BACKUP_DIR/"

echo "Backed up to: $BACKUP_DIR"
```

---

## 🗑️ Clear Existing Symlinks

Remove existing configs that would conflict:
```bash
rm -rf ~/.config/sway
rm -rf ~/.config/foot
rm -rf ~/.config/fuzzel
rm -rf ~/.config/mako
```

---

## 📥 Clone Repository
```bash
cd ~
git clone https://github.com/YOUR-USERNAME/0-core.git
cd 0-core
```

---

## 🔗 Stow Packages

Use GNU Stow to create symlinks:
```bash
cd ~/0-core

# Desktop Environment
stow wm-sway

# Shell
stow shell-zsh
stow prompt-starship

# Editor
stow editor-nvim

# File Manager
stow fm-yazi

# Git
stow vcs-git
```

---

## ✅ Verify Installation

Check symlinks were created:
```bash
ls -la ~/.config/sway
ls -la ~/.config/zsh

# Verify symlink targets
readlink ~/.config/sway/config
# Should show: ../0-core/wm-sway/.config/sway/config
```

---

## 🧪 Test Configuration

### Test Sway Config
```bash
sway -C  # Check config syntax
```

### Test Sway
```bash
swaymsg reload  # Reload if already running
```

### Test faelight-bar
```bash
pkill faelight-bar
~/0-core/scripts/faelight-bar &
```

---

## 🔧 Making Changes

### The Safe Way

1. Unlock core:
```bash
   unlock-core
```

2. Edit the source file in 0-core:
```bash
   nvim ~/0-core/wm-sway/.config/sway/config
```

3. Reload:
```bash
   swaymsg reload
```

4. Commit:
```bash
   cd ~/0-core
   git add wm-sway/
   git commit -m "Update sway config"
   git push
```

5. Lock:
```bash
   lock-core
```

---

## 🔄 Re-stowing After Changes

If you need to re-stow a package:
```bash
cd ~/0-core
stow -R wm-sway  # Re-stow (restow)
```

If there are conflicts:
```bash
stow --adopt wm-sway  # Adopt existing files into repo
```

---

## 🗑️ Removing a Package

To unlink a package:
```bash
cd ~/0-core
stow -D wm-sway  # Delete symlinks
```

Verify:
```bash
ls -la ~/.config/sway  # Should not exist
```

---

## 🔨 Troubleshooting

### Stow Conflicts

If stow reports conflicts:
```bash
# Check what exists
ls -la ~/.config/sway

# Remove if it's a regular file/dir (not symlink)
rm -rf ~/.config/sway

# Try again
stow wm-sway
```

### Sway Not Starting
```bash
# Check logs
journalctl --user -xe | grep sway

# Validate config
sway -C

# Start with debug
sway -d 2>&1 | tee sway.log
```

### Symlink Points Wrong
```bash
# Remove and re-stow
stow -D wm-sway
stow wm-sway
```

---

## 📊 Health Check

After installation, run:
```bash
dot-doctor
```

Should show 100% health with all checks passing.

---

_Last Updated: January 9, 2026 (v6.0.0)_  
_Part of Faelight Forest 0-Core - Sway Edition_
