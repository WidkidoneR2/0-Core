# 🛠️ Manual Installation Guide - Faelight Forest v8.4.0

This guide covers manual installation and management of 0-Core on vanilla Arch Linux with Sway.

> **Note:** For automated installation, use `faelight-bootstrap` instead.

---

## 📋 Prerequisites

- **Arch Linux** (vanilla, not Omarchy)
- **Sway** window manager
- **Git** and **GNU Stow**
- **Rust toolchain** (for building tools)
```bash
sudo pacman -S sway foot git stow rustup base-devel
rustup default stable
```

---

## 📦 Repository Structure
```
0-core/
├── stow/                    # 12 GNU Stow packages
│   ├── wm-sway/            # Sway window manager
│   ├── shell-zsh/          # Zsh + 188+ aliases
│   ├── shell-nushell/      # Nushell configuration
│   ├── prompt-starship/    # Starship prompt
│   ├── term-foot/          # Foot terminal
│   ├── editor-nvim/        # Neovim + Faelight theme
│   ├── fm-yazi/            # Yazi file manager
│   ├── vcs-git/            # Git configuration
│   ├── config-faelight/    # Faelight typed configs
│   ├── browser-qutebrowser/
│   ├── browser-brave/
│   └── tools-topgrade/
├── rust-tools/             # 34 Rust tools (monorepo)
├── scripts/                # Compiled binaries (gitignored)
├── docs/                   # Documentation
├── INTENT/                 # Intent Ledger
└── Cargo.toml             # Workspace manifest
```

---

## 🔄 Backup Existing Configs
```bash
BACKUP_DIR=~/config-backup-$(date +%Y%m%d-%H%M%S)
mkdir -p "$BACKUP_DIR"

# Backup configs
[[ -d ~/.config/sway ]] && cp -r ~/.config/sway "$BACKUP_DIR/"
[[ -d ~/.config/foot ]] && cp -r ~/.config/foot "$BACKUP_DIR/"
[[ -d ~/.config/nvim ]] && cp -r ~/.config/nvim "$BACKUP_DIR/"
[[ -d ~/.config/yazi ]] && cp -r ~/.config/yazi "$BACKUP_DIR/"
[[ -f ~/.zshrc ]] && cp ~/.zshrc "$BACKUP_DIR/"
[[ -f ~/.gitconfig ]] && cp ~/.gitconfig "$BACKUP_DIR/"

echo "✅ Backed up to: $BACKUP_DIR"
```

---

## 📥 Clone Repository
```bash
cd ~
git clone https://github.com/WidkidoneR2/0-Core.git 0-core
cd 0-core
```

---

## 🔨 Build Rust Tools
```bash
cd ~/0-core
cargo build --release

# Copy binaries to scripts/
mkdir -p scripts
cp target/release/faelight-* scripts/
cp target/release/dot-doctor scripts/
cp target/release/intent scripts/
# ... (or use a build script)
```

---

## 🔗 Stow Packages
```bash
cd ~/0-core/stow

# Core packages
stow -t ~ wm-sway
stow -t ~ shell-zsh
stow -t ~ prompt-starship
stow -t ~ term-foot
stow -t ~ editor-nvim
stow -t ~ fm-yazi
stow -t ~ vcs-git
stow -t ~ config-faelight

# Optional packages
stow -t ~ browser-qutebrowser
stow -t ~ browser-brave
stow -t ~ tools-topgrade
```

---

## ✅ Verify Installation
```bash
# Check symlinks
ls -la ~/.config/sway
ls -la ~/.config/faelight
readlink ~/.config/sway/config

# Run health check
doctor

# Should show 100% health with 14/14 checks passing
```

---

## 🧪 Test Configuration

### Test Sway Config
```bash
sway -C  # Validate syntax
```

### Test Services
```bash
# faelight-bar starts automatically via Sway
pgrep -f faelight-bar

# faelight-notify starts automatically
pgrep -f faelight-notify
```

### Test Tools
```bash
# Run automated test suite
~/0-core/scripts/test-all-tools

# Should show 34/34 passing
```

---

## 🔧 Making Changes

### The Safe Workflow

1. **Unlock core:**
```bash
   unlock-core
```

2. **Edit source files:**
```bash
   nvim ~/0-core/stow/wm-sway/.config/sway/config
```

3. **Reload Sway:**
```bash
   swaymsg reload
```

4. **Commit changes:**
```bash
   cd ~/0-core
   git add stow/wm-sway/
   git commit -m "fix(sway): Update keybinding"
   git push
```

5. **Lock core:**
```bash
   lock-core
```

---

## 🔄 Re-stowing After Changes
```bash
cd ~/0-core/stow

# Re-stow a package
stow -R wm-sway

# Adopt existing files into repo (careful!)
stow --adopt wm-sway
```

---

## 🗑️ Removing a Package
```bash
cd ~/0-core/stow
stow -D -t ~ wm-sway  # Delete symlinks

# Verify removal
ls -la ~/.config/sway  # Should not exist
```

---

## 🔨 Troubleshooting

### Stow Conflicts
```bash
# Check what exists
ls -la ~/.config/sway

# Remove conflicting file/dir
rm -rf ~/.config/sway

# Try again
stow -t ~ wm-sway
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

### Binary Not Found
```bash
# Ensure scripts/ is in PATH
echo $PATH | grep 0-core/scripts

# Add to ~/.zshrc if missing:
export PATH="$HOME/0-core/scripts:$PATH"
```

### Git Hooks Not Working
```bash
# Install hooks
cd ~/0-core
faelight-hooks install

# Verify
ls -la .git/hooks/
```

---

## 📊 Health Check

After installation, verify everything:
```bash
doctor --explain
```

Expected output:
- ✅ 14/14 checks passing
- ✅ 100% system health
- ✅ All symlinks active
- ✅ All services running

---

## 🚀 Post-Installation

### Update System
```bash
faelight-update --dry-run  # Check for updates
faelight-update            # Apply updates
```

### Learn the System
```bash
teach                      # Interactive learning guide
intent list                # View decision history
```

### Explore Intent Ledger
```bash
cd ~/0-core/INTENT
ls -la decisions/
```

---

## 📚 Additional Resources

- **Architecture:** See `docs/ARCHITECTURE.md`
- **Build Guide:** See `docs/BUILD.md`
- **Tool Reference:** See `docs/TOOL_REFERENCE.md`
- **Intent Ledger:** Browse `INTENT/` directory

---

## ⚠️ Important Notes

1. **Source-First Architecture:** The `scripts/` directory is gitignored. Only source code is committed.
2. **Workspace Build:** All Rust tools build together via the workspace `Cargo.toml`.
3. **Health Monitoring:** Run `doctor` regularly to verify system integrity.
4. **Core Protection:** Use `lock-core`/`unlock-core` to prevent accidental changes.

---

**Manual control over automation. Understanding over convenience.** 🌲
