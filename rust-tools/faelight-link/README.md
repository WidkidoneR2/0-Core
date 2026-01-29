# 🔗 faelight-link v1.0.0
**Zone-aware symlink manager for Faelight Forest**

![Production Ready](https://img.shields.io/badge/status-production-brightgreen)
![Version](https://img.shields.io/badge/version-1.0.0-blue)

Rust-based stow replacement with conflict detection, health auditing, and ecosystem integration.

## ✨ Features

### Core Operations
- ✅ **stow** - Create symlinks with conflict detection
- ✅ **unstow** - Remove symlinks cleanly  
- ✅ **list** - Show all available packages
- ✅ **status** - Display link counts per package
- ✅ **audit** - Comprehensive health check (broken/orphaned links)
- ✅ **clean** - Remove broken and orphaned links

### Smart Features
- 🔍 Auto-discover packages from `~/0-core/stow/`
- ⚠️ Interactive conflict resolution (backup/skip/overwrite)
- 💾 Automatic timestamped backups
- 🏥 Health monitoring (100% link health tracking)
- 🎨 Beautiful colored output
- 🛡️ Safe operations with confirmation prompts

## 🚀 Usage
```bash
# List all packages
faelight-link list

# Show status of all links
faelight-link status

# Audit link health (check for broken links)
faelight-link audit

# Stow a package (with conflict detection)
faelight-link stow editor-nvim

# Stow without prompts
faelight-link stow editor-nvim --force

# Unstow a package (remove symlinks)
faelight-link unstow editor-nvim

# Clean up broken links
faelight-link clean

# Clean without confirmation
faelight-link clean --force
```

## 🏥 Health Monitoring
```bash
$ faelight-link audit

  Scanning all symlinks...

  ✓ browser-brave (1 links)
  ✓ editor-nvim (3 links)
  ✓ shell-zsh (5 links)
  
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  📊 Summary:
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    Valid links:   28
    Broken links:  0

  ✅ All links are healthy!

  Overall health: 100%
```

## 🎯 Conflict Resolution

When a file already exists during stow:
```
⚠️  Conflict detected!
  Target: ~/.config/nvim/init.lua
  Existing: file (regular file)

How to resolve?
  > Backup   (saves to backups/ with timestamp)
    Skip     (leaves existing file)
    Overwrite (replaces existing file)
    Quit     (stops operation)
```

Backups stored in: `~/.local/share/faelight-link/backups/`

## 📋 Version History

### ✅ v1.0.0 - Production Ready! (2026-01-29)
- ✨ Added **unstow** command (full implementation)
- ✨ Added **audit** command (health checks)
- ✨ Added **clean** command (broken link removal)
- 🎯 Recursive symlink discovery
- 🏥 Health percentage tracking
- 💾 Safe removal with backups
- 🎊 **OUT OF BETA!**

### ✅ v0.2.0 - Conflict Detection (2026-01-28)
- ✨ Conflict detection system
- 🎯 Interactive resolution
- 💾 Timestamped backups
- 🎨 Enhanced output

### ✅ v0.1.0 - Prototype (2026-01-28)
- ✨ Package discovery
- ✨ Basic stow functionality
- ✨ Status checking

## 🏗️ Development

- **v0.1.0:** ~1 hour (prototype)
- **v0.2.0:** ~1 hour (conflict detection)
- **v1.0.0:** ~1.5 hours (unstow, audit, clean)

**Total:** Full stow replacement in 3.5 hours! 🚀

## 🎯 Philosophy

*"Understanding over convenience"* - every operation is transparent, reversible, and requires user confirmation. No hidden automation, complete manual control.

---

**Part of the forest.** 🌲 **Production ready.**
