# 📦 Faelight Stow v2.1.0

**Intelligent GNU Stow wrapper with automatic verification and repair**

Manage dotfiles safely with automatic symlink verification, conflict detection, and one-command fixes.

## ✨ Features

### Smart Verification
- 🔍 **Auto-discovery** - Scans stow directory for all packages
- ✅ **Validation** - Verifies symlinks point to correct targets
- 🔧 **Auto-repair** - Fixes broken symlinks with `--fix`
- 📊 **Health reporting** - Shows package status at a glance

### Safe Operations
- 🛡️ **Conflict detection** - Warns before overwriting files
- 📝 **Dry-run mode** - Preview changes before applying
- 🎯 **Selective stowing** - Install individual packages
- 🔕 **Silent mode** - Perfect for scripts (exit codes)

### Integration Ready
- ✅ Works with any stow directory structure
- ✅ Used by system health checkers
- ✅ Perfect for automation scripts
- ✅ Clean exit codes for CI/CD

## 📦 Installation

### Standalone (Any System with GNU Stow)
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/faelight-stow

# Build and install
cargo install --path .

# Verify
faelight-stow --version
```

### Dependencies

- **Required:** `stow` (GNU Stow)
- **Optional:** None!
```bash
# Install stow
# Arch: sudo pacman -S stow
# Debian/Ubuntu: sudo apt install stow
# macOS: brew install stow
```

## 🚀 Usage

### Basic Operations
```bash
# Verify all packages
faelight-stow

# Output:
# 🔗 faelight-stow v2.1.0 - Verifying symlinks...
# ✅ shell-zsh: All symlinks valid
# ✅ nvim: All symlinks valid
# ❌ tmux: Invalid symlink
#
# Summary: 2/3 packages healthy

# Auto-fix issues
faelight-stow --fix
    faelight-stow check shell-zsh wm-sway  # Check for conflicts before stowing

# Check for conflicts before stowing
faelight-stow check shell-zsh wm-sway


# Stow a new package
faelight-stow my-package

# Silent mode (for scripts)
faelight-stow --quiet
echo $?  # 0 = all healthy, 1 = issues found
```

### Examples

#### Daily Verification
```bash
# Check dotfiles health
faelight-stow
```

#### Adding New Package
```bash
# Stow a new dotfile package
faelight-stow git-config
# Verifies after stowing
```

#### Automation / CI
```bash
#!/bin/bash
# Check dotfiles in CI pipeline
if ! faelight-stow --quiet; then
    echo "Dotfiles have issues!"
    faelight-stow  # Show details
    exit 1
fi
```

#### Auto-repair Script
```bash
# Weekly cron job to fix dotfiles
0 0 * * 0 faelight-stow --fix --notify
    faelight-stow check shell-zsh wm-sway  # Check for conflicts before stowing
```

## 🎯 How It Works

### Stow Directory Structure

Expects this structure:
```
~/stow/              # or custom path
├── shell-zsh/
│   └── .zshrc
├── nvim/
│   └── .config/nvim/init.lua
└── git/
    └── .gitconfig
```

### Verification Process

1. **Scans** stow directory for packages
2. **Checks** each symlink target
3. **Validates** symlinks point to stow directory
4. **Reports** status with colors
5. **Repairs** (if `--fix` used)

### What Gets Checked

✅ Symlink exists  
✅ Target file exists in stow package  
✅ Symlink points to correct location  
✅ No broken links  
✅ No conflicts with real files  

## 🔧 Options

| Flag | Description | Example |
|------|-------------|---------|
| `--quiet` | Silent mode (exit codes only) | `faelight-stow --quiet` |
| `--fix` | Auto-repair broken symlinks | `faelight-stow --fix` |
    faelight-stow check shell-zsh wm-sway  # Check for conflicts before stowing
| `--notify` | Send notification on issues | `faelight-stow --notify` |
| `-v, --version` | Show version | `faelight-stow --version` |
| `-h, --help` | Show help | `faelight-stow --help` |

## 💡 Use Cases

### Personal Dotfiles
```bash
# Verify dotfiles after git pull
cd ~/dotfiles
git pull
faelight-stow --fix
    faelight-stow check shell-zsh wm-sway  # Check for conflicts before stowing
```

### Multi-Machine Sync
```bash
# On new machine
git clone <dotfiles-repo>
cd dotfiles
stow */  # Install all packages
faelight-stow  # Verify
```

### System Health
```bash
# Part of health check script
if ! faelight-stow --quiet; then
    echo "⚠️  Dotfile issues detected"
fi
```

## 🤝 Part of 0-Core

Faelight-stow is part of the 0-Core ecosystem but works perfectly standalone with any stow setup!

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 📄 License

Intentional Stewardship - Manual control over automation.
