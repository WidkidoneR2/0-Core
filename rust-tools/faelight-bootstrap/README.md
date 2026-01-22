# faelight-bootstrap v1.0.0

**One-Command 0-Core Setup** - Automated installation for Faelight Forest on Arch Linux.

🌲 **Faelight Forest** - Philosophy-driven desktop environment

---

## 🎯 Purpose

Provides a complete, reproducible installation of the Faelight Forest ecosystem on a fresh Arch Linux system. Handles dependencies, dotfiles, and tool compilation in one command.

## 🚀 Usage
```bash
# Full installation (interactive)
faelight-bootstrap

# Preview without changes
faelight-bootstrap --dry-run

# Show version
faelight-bootstrap --version

# Show help
faelight-bootstrap --help

# Health check
faelight-bootstrap --health
```

## 📋 Installation Phases

### **Phase 1: Pre-Flight Checks** (Automated)
- ✅ Verifies Arch Linux
- ✅ Checks internet connection
- ✅ Validates not running as root
- ✅ Checks available disk space

### **Phase 2: System Dependencies** (Interactive)
- ✅ Updates package database
- ✅ Installs ~40 packages via pacman:
  - Sway desktop environment
  - Terminal (Foot), Shell (Zsh, Nushell)
  - Editor (Neovim), File manager (Yazi)
  - Rust toolchain
  - Fonts, audio, browsers
  - Security tools (UFW, Mullvad VPN, Snapper)

### **Phase 3: Clone Repository** (Interactive)
- ✅ Backs up existing ~/0-core if present
- ✅ Clones 0-core from GitHub
- ✅ Sets up directory structure

### **Phase 4: Stow Packages** (Interactive)
- ✅ Stows all 12 dotfile packages:
  - wm-sway, shell-zsh, shell-nushell
  - prompt-starship, term-foot, editor-nvim
  - fm-yazi, vcs-git, config-faelight
  - browser-qutebrowser, browser-brave
  - tools-topgrade

### **Phase 5: Build Rust Tools** (Interactive)
- ✅ Builds entire workspace (30 tools)
- ✅ Compiles optimized release binaries
- ✅ Makes tools available in PATH

### **Phase 6: Health Verification** (Automated)
- ✅ Runs dot-doctor health check
- ✅ Verifies installation success

### **Phase 7: Post-Install Tutorial** (Interactive)
- ✅ Shows essential keybindings
- ✅ Explains quick commands
- ✅ Provides next steps

## 🏗️ Architecture

**Technology:**
- Rust with `colored` for terminal output
- Interactive prompts for human control
- Shell command integration
- Workspace-aware Cargo builds

**Philosophy:**
- **Manual Control** - Asks before major actions
- **Safety First** - Backs up existing installations
- **Transparency** - Shows what will happen (dry-run mode)
- **Reproducible** - Same installation every time

**Installation Time:**
- Pre-flight: <1 minute
- Dependencies: 5-10 minutes
- Clone: <1 minute
- Stow: <1 minute
- Rust build: 2-5 minutes
- **Total: ~10-20 minutes**

## 📜 Philosophy

**"Automation serves installation, but human controls choices."**

This tool embodies the 0-Core philosophy by:
- Automating tedious package installation
- Stowing dotfiles with one command
- Building all tools from source
- **But prompting before every major action**

You control what gets installed. The tool eliminates manual drudgery.

## 🔧 Configuration

**Requirements:**
- Arch Linux (or Arch-based distro)
- Internet connection
- Sudo access
- ~2GB free disk space
- ~10-20 minutes

**Optional:**
- GPU for hardware acceleration
- Mullvad VPN account
- GitHub account for repository access

## 🐛 Troubleshooting

**"Not Arch Linux" error:**
- This installer is Arch-specific
- For other distros, manual installation required

**"No internet connection" error:**
- Verify network: `ping archlinux.org`
- Check DNS settings

**Package installation fails:**
- Update system first: `sudo pacman -Syu`
- Check mirrors: `sudo pacman-mirrors --fasttrack`

**Stow conflicts:**
- Existing dotfiles may conflict
- Backup and remove conflicting files
- Or skip stow and manually review packages

**Rust build fails:**
- Ensure rustup installed: `rustup --version`
- Check disk space: `df -h`
- Try manual build: `cd ~/0-core && cargo build --release`

**Dry-run shows what to expect:**
- Run `faelight-bootstrap --dry-run` first
- Preview all actions before installation

## 🎓 Learning

This tool demonstrates:
- **Interactive CLI Design** - Balancing automation with control
- **Phased Installation** - Clear progress through complex setup
- **Error Handling** - Graceful failures with helpful messages
- **Workspace Builds** - Compiling monorepo efficiently
- **Package Management** - Stow-based dotfile deployment

## 🔄 Version History

- **v1.0.0** (2026-01-21) - Linus Edition
  - Added `--help`, `--version`, `--health`, `--dry-run` flags
  - 7-phase interactive installation workflow
  - Updated to current stow structure (12 packages)
  - Workspace build (all 30 tools at once)
  - Intelligent existing installation detection
  - Backup before reinstall
  - Beautiful phased progress output
  - Post-install tutorial system
  - Comprehensive README with philosophy

- **v0.1.0** (2026-01-09) - Initial implementation
  - Basic installation automation
  - Hardcoded package list

---

**Part of the Faelight Forest ecosystem** 🌲🦀

**"From chaos to order. From generic to intentional."**
