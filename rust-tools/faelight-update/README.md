# 🌲 Faelight Update Manager v3.0.0

Intelligent, comprehensive update manager for Arch Linux and the Faelight Forest ecosystem.

## ✨ Features

### Better Than Topgrade
- ✅ **No double-run bug** - Smart package manager integration
- ✅ **Health checks** - Before and after updates
- ✅ **0-Core integration** - Workspace-aware updates
- ✅ **Config file support** - Skip packages, auto-yes categories
- ✅ **Smart .pacnew handling** - Interactive merge with pacdiff
- ✅ **AUR rebuild detection** - Warns about outdated dependencies
- ✅ **Auto-updates prompt cache** - Live update counter

### Update Categories
- 📦 **System Packages** (pacman)
- 🔷 **AUR Packages** (paru)
- 🦀 **Cargo Tools** (cargo-update)
- 🦀 **Rust Toolchain** (rustup)
- 📦 **NPM Packages** (npm)
- 🐍 **Python Packages** (pip)
- 🌲 **0-Core Workspace** (workspace rebuild)
- 📝 **Neovim Plugins** (lazy.nvim)
- 📁 **Yazi Packages** (ya pack)
- 🔄 **Git Repositories** (~/repos auto-pull)
- ⚡ **Firmware** (fwupdmgr)
- 📦 **Flatpak** (flatpak)

## 📦 Installation

### Standalone (Any Arch Linux System)
```bash
# Clone the repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/faelight-update

# Build and install
cargo install --path .

# Verify
faelight-update --version
```

### Dependencies
- **Required:** `pacman`, `paru` (or `yay`)
- **Optional:** `cargo-update`, `npm`, `pip`, `nvim`, `fwupdmgr`, `flatpak`

### First Run
```bash
# Dry run to see what would be updated
faelight-update --dry-run

# Interactive mode to select packages
faelight-update --interactive

# Full update
faelight-update
```

## 📚 Examples

### Basic Usage
```bash
# Check for updates (dry run)
faelight-update -n

# Update everything
faelight-update

# Skip health check
faelight-update --skip-health
```

### Scripting
```bash
# Get update count for status bars
faelight-update --count-only

# JSON output for parsing
faelight-update --json
```

### Selective Updates
```bash
# Only update specific categories
faelight-update --only pacman,aur

# Skip specific categories
faelight-update --skip neovim,workspace
```


## 🚀 Usage
```bash
# Standard update
faelight-update

# Dry run (check only)
faelight-update --dry-run

# Interactive selection
faelight-update --interactive

# Skip health check
faelight-update --skip-health

# Only specific categories
faelight-update --only pacman,aur

# Skip categories
faelight-update --skip flatpak,firmware

# JSON output
faelight-update --json
```

## ⚙️ Configuration

Config file: `~/.config/faelight-update/config.toml`
```toml
# Skip specific packages
skip_packages = ["linux", "nvidia"]

# Skip entire categories
skip_categories = ["Flatpak"]

# Auto-confirm these categories (no prompt)
auto_yes_categories = ["Cargo Tools", "NPM Packages"]

# Custom update order
update_order = [
    "System Packages",
    "AUR Packages",
    "0-Core Workspace",
]

# Enable parallel updates (experimental)
parallel_updates = false

# Desktop notifications
notify_on_updates = false
```

## 🎯 Philosophy

- **Manual control over automation** - You decide what updates
- **Understanding over convenience** - Clear feedback at each step
- **Health-first** - System health checks prevent issues
- **Graceful degradation** - Missing tools don't break updates

## 📋 Workflow

1. **Health Check** - Verify system health
2. **Check Updates** - Scan all categories
3. **Show Summary** - Clear overview with counts
4. **Impact Analysis** - Highlight critical updates
5. **Apply Updates** - Execute in optimal order
6. **Cleanup** - Clear caches
7. **Post-Update** - Update prompt, check .pacnew, detect rebuilds
8. **Final Health** - Verify system stability

## 🔧 Integration

Works seamlessly with:
- `doctor` - System health checker
- `intent` - Intent ledger
- `faelight-git` - Git tooling
- Starship prompt - Live update counter

## 📊 Example Output
```
🌲 Faelight Update Manager v3.0.0

🏥  Running health check...
   ✅  System healthy
   
🔍  Checking for updates...
   Checking pacman...
   Checking AUR (paru)...
   Checking cargo-installed tools...
   
📊 Update Summary
──────────────────────────────────────────────────
  📦 System Packages (4 available)
     • lazygit
     • systemd
     • systemd-libs
     • systemd-sysvcompat
     
  🔷 AUR Packages (up to date)
  🦀 Cargo Tools (up to date)
  
──────────────────────────────────────────────────
  4 updates available
  
📊 Impact Analysis
──────────────────────────────────────────────────
  🔴 1 critical system packages
  
✨  Ready to update 4 packages!
Proceed? (Y/n):
```

## 🏗️ Architecture

- **Modular checkers** - Each category has its own module
- **Graceful errors** - Missing tools don't fail entire update
- **Config-driven** - Behavior controlled by config file
- **Workspace integration** - Rebuilds 0-Core after updates

## 🎓 For Linus & Graydon

This is a production-grade update manager demonstrating:
- **Rust best practices** - Modular, typed, error-handled
- **System integration** - Deep Arch Linux knowledge
- **User experience** - Clear, informative, controllable
- **Philosophy** - Manual stewardship over blind automation

Built as part of the 0-Core ecosystem - a complete computing environment built from scratch in Rust.

---

**Part of:** [0-Core](https://github.com/WidkidoneR2/0-Core) - Personal computing environment  
**Author:** Christian  
**License:** MIT
