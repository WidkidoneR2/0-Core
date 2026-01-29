# 🔗 faelight-link v0.2.0

**Zone-aware symlink manager for Faelight Forest**

Rust-based stow replacement with conflict detection and ecosystem integration.

## ✨ Features

### v0.2.0 - Conflict Detection
- ✅ Auto-discover packages from `~/0-core/stow/`
- ✅ List packages with file counts
- ✅ Show link status
- ✅ Stow packages (create symlinks)
- ✅ **Conflict detection** (detects existing files)
- ✅ **Interactive resolution** (backup/skip/overwrite/quit)
- ✅ **Automatic backups** (timestamped in ~/.local/share/faelight-link/backups/)
- ✅ Beautiful colored output

## 🚀 Usage
```bash
# List all packages
faelight-link list

# Show status of links
faelight-link status

# Stow a package (with conflict detection!)
faelight-link stow editor-nvim

# Stow without prompt
faelight-link stow editor-nvim --force
```

## 🎯 Conflict Resolution

When a file already exists, you'll be prompted:
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

Backups are stored in: `~/.local/share/faelight-link/backups/`

## 📋 Roadmap

### ✅ v0.1.0 - Prototype
- [x] Package discovery
- [x] Link creation
- [x] Status checking

### ✅ v0.2.0 - Conflict Handling
- [x] Detect existing files
- [x] Interactive conflict resolution
- [x] Backup/skip/overwrite options
- [x] Timestamped backups

### v0.3.0 - Zone Awareness (Next)
- [ ] Zone detection
- [ ] Lock enforcement
- [ ] Health checks

### v0.4.0 - Ecosystem
- [ ] Intent tracking
- [ ] Daemon integration
- [ ] Canonical link formatting

### v1.0.0 - Production
- [ ] Unstow command
- [ ] Clean/audit commands
- [ ] Full stow replacement

## 🏗️ Built In

- **v0.1.0:** ~1 hour (prototype)
- **v0.2.0:** ~1 hour (conflict detection)

**Total:** Working stow replacement in 2 hours! 🚀

---

**Part of the forest.** 🌲
