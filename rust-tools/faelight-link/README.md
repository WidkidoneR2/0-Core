# 🔗 faelight-link v0.1.0

**Zone-aware symlink manager for Faelight Forest**

Rust-based stow replacement with ecosystem integration.

## ✨ Features (v0.1.0 Prototype)

- ✅ Auto-discover packages from `~/0-core/stow/`
- ✅ List packages with file counts
- ✅ Show link status
- ✅ Stow packages (create symlinks)
- ✅ Interactive confirmation prompts
- ✅ Absolute symlinks (for now)

## 🚀 Usage
```bash
# List all packages
faelight-link list

# Show status of links
faelight-link status

# Stow a package
faelight-link stow editor-nvim

# Stow without prompt
faelight-link stow editor-nvim --force
```

## 📋 Roadmap

### v0.2.0 - Conflict Handling
- [ ] Detect existing files
- [ ] Interactive conflict resolution
- [ ] Backup/skip/overwrite options

### v0.3.0 - Zone Awareness
- [ ] Zone detection
- [ ] Lock enforcement
- [ ] Health checks

### v0.4.0 - Ecosystem
- [ ] Intent tracking
- [ ] Daemon integration
- [ ] Canonical link formatting

### v1.0.0 - Production
- [ ] Replace stow completely
- [ ] Unstow command
- [ ] Clean/audit commands

## 🏗️ Architecture

Built in ~1 hour as working prototype!

**Learned from:** lnko algorithms  
**Built in:** Pure Rust  
**Integrated with:** Faelight ecosystem

---

**Part of the forest.** 🌲
