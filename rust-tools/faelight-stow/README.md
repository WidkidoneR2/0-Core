# faelight-stow v3.0.0

🌟 **LEGENDARY Dotfile Manager** - Backup, rollback, groups, and intelligent health monitoring.

## 🎉 What Makes v3.0.0 Legendary

### 🏆 Tier 1 - Critical Features

**1. Backup & Rollback System**
```bash
faelight-stow --backup stow shell-zsh   # Auto-backup before stowing
faelight-stow rollback                  # List available backups
faelight-stow rollback 20260211_230000  # Restore specific backup
```

**2. Package Groups**
```bash
faelight-stow group dev      # Stow: editor-nvim, vcs-git, wm-sway
faelight-stow group minimal  # Stow: shell-zsh, vcs-git
faelight-stow group full     # Stow: complete environment
```

**3. Dry-Run Preview**
```bash
faelight-stow --dry-run stow shell-zsh    # Preview without changes
faelight-stow --dry-run group dev         # Preview group stow
```

### 🎯 Tier 2 - Power Features

**4. Health Scoring**
```bash
faelight-stow health
# Output:
# 📊 Health Score: 85%
#    Stowed: 11/13 packages
```

**5. Conflict Checking**
```bash
faelight-stow check shell-zsh editor-nvim
# Shows what conflicts BEFORE stowing
```

**6. Diff Mode (Coming Soon)**
```bash
faelight-stow diff shell-zsh
# Will show differences between package and current dotfiles
```

## Usage

### Basic Commands

**Verify All Packages:**
```bash
faelight-stow verify          # Check all packages
faelight-stow --quiet verify  # Only show issues
faelight-stow --fix verify    # Auto-fix issues
```

**Stow Packages:**
```bash
faelight-stow stow shell-zsh
faelight-stow stow shell-zsh editor-nvim  # Multiple packages
faelight-stow --backup stow shell-zsh     # With backup
```

**Check Health:**
```bash
faelight-stow health
```

## Philosophy

> "Every dotfile change should be intentional, backed up, and reversible."

Features:
- ✅ **Auto-discover packages** from stow directory
- ✅ **Verify symlink integrity** automatically
- ✅ **Backup system** with rollback capability
- ✅ **Package groups** for common workflows
- ✅ **Health monitoring** (0-100% score)
- ✅ **Dry-run mode** for safe previews
- ✅ **Smart error messages** with helpful hints
- 🚧 **Conflict detection** (advanced - coming soon)
- 🚧 **Diff mode** (coming soon)

## Installation
```bash
cd ~/0-core
cargo build --release -p faelight-stow
cargo install --path rust-tools/faelight-stow
```

Binary installs to: `~/.cargo/bin/faelight-stow`

## Package Groups

Predefined groups for common setups:

| Group | Packages |
|-------|----------|
| **dev** | editor-nvim, vcs-git, wm-sway |
| **minimal** | shell-zsh, vcs-git |
| **full** | shell-zsh, editor-nvim, fm-yazi, vcs-git, wm-sway, term-foot |

Add custom groups by editing `src/main.rs` (config file coming soon).

## Advanced Usage

**Backup Workflow:**
```bash
# Create backup before major changes
faelight-stow --backup stow shell-zsh

# Make changes...

# Rollback if needed
faelight-stow rollback
```

**CI/CD Integration:**
```bash
# Verify dotfiles in pipeline
faelight-stow --quiet verify || exit 1
```

**Health Monitoring:**
```bash
# Check health score
if [ $(faelight-stow health | grep "Health Score" | awk '{print $4}' | tr -d '%') -lt 80 ]; then
    echo "⚠️  Dotfile health below 80%!"
fi
```

## Part of 0-Core

One of 40 Rust tools in the Faelight Forest ecosystem.

**Philosophy:** Manual control over automation, intentional changes, complete reversibility.

See: https://github.com/WidkidoneR2/0-Core

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
