# 🌲 Faelight Forest - The Immortal Arch Linux System
![Version](https://img.shields.io/badge/Version-2.7.2-brightgreen)
![License: MIT](https://img.shields.io/badge/License-MIT-green)
![Platform](https://img.shields.io/badge/Platform-Arch_Linux-blue)
![Hyprland](https://img.shields.io/badge/WM-Hyprland-teal)


Faelight Forest is a **fully reproducible, self-healing Arch Linux system** with NixOS-style snapshots, professional dotfile management, and enterprise-grade security. Your system will never break, and your workflow is fully optimized.

---
## ✨ What's New in Version 2.7.2 (11-30-25)

### 🔒 Security Hardening Sprint
- **Hardening Index: 71/100** (improved from 68 - +3 points!)
- **Fail2ban jails enabled** - Critical fix (was completely disabled)
- **Kernel hardening applied** - 9 security settings hardened via sysctl
- **Vulnerability scanning** - arch-audit installed and monitoring
- **Security auditing** - Lynis integrated for weekly scans

### 🛡️ Security Improvements
- Kernel pointer obfuscation (`kptr_restrict = 2`)
- Restricted kernel message access (`dmesg_restrict = 1`)
- Disabled unprivileged BPF (`unprivileged_bpf_disabled = 1`)
- Hardened file system protections (FIFOs, regular files)
- Network security logging (martian packets)
- BPF JIT hardening
- Intrusion prevention active (Fail2ban)

### 🔧 New Security Aliases
```bash
security-check    # Full weekly security audit
vuln-check        # Show high-risk vulnerabilities
security-score    # Display current hardening index (71/100)
audit-full        # Complete Lynis security scan
jail-status       # Check Fail2ban status
```

### 📝 Documentation
- **COMPLETE_GUIDE.md** - Complete rewrite for v2.7.2 with security section
- **system/security/** - Security configuration documentation
- Weekly security routine documented

### Previous Updates (Version 2.7.1 - 11-29-25)
```

### Previous Updates (Version 2.7.1 - 11-29-25)

### 🎨 Complete Theme Refinement
- **Theme-Aware Terminal Colors** - 16 ANSI colors perfectly matched to each theme
- **Fish Prompt Polish** - Purple timestamps, lime/teal folders, cyan/magenta prompts
- **Fish Syntax Highlighting** - Orange commands, lime paths, purple functions in dark theme
- **Waybar Light Theme** - Bright purple workspace icons, blue WiFi
- **Neon Status Indicators** - Bright green ✔ and red ✗ in both themes
- **Perfect Color Coherence** - Every component matches the Faelight aesthetic

### 📁 New Files
```
themes/
├── faelight-dark/terminal.conf    # Complete 16-color ANSI palette
└── faelight-light/terminal.conf   # Light theme terminal colors

fish/.config/fish/functions/
└── set_fish_colors.fish           # Theme-aware syntax highlighting

waybar/.config/waybar/
├── style-dark.css                 # Dark Waybar theme
└── style-light.css                # Light Waybar theme (NEW!)
```

### Previous Updates (Version 2.7 - 11-27-25)

### 🎨 Theme Toggle System
- **Instant Theme Switching** - `SUPER + SHIFT + P` toggles your entire system
- **Light & Dark Themes** - Beautifully designed Faelight Forest variants
- **System-Wide** - Changes Hyprland, Kitty, Mako, GTK apps simultaneously
- **Fish Aliases** - `theme-dark`, `theme-light`, `theme-toggle`

### 📁 Professional Theme Structure
```
~/dotfiles/themes/
├── faelight-dark/theme.json    # Your current dark forest theme
├── faelight-light/theme.json   # New light variant
└── current.txt                 # Tracks active theme
```

### ✨ Enhanced Keybindings
- All keybinds now use `bindd` format
- Show descriptions in Walker (`SUPER + W`)
- Better discoverability for all shortcuts
- Fixed keybind conflicts

**📖 See full changelog:** [CHANGELOG.md](CHANGELOG.md)

---

### Security Features (v2.7.2)
- **Hardening Index: 71/100** ✅
- **Lynis security auditing** - Regular system scans
- **arch-audit** - Vulnerability tracking
- **Kernel hardening** - 9 critical settings secured
- **Fail2ban active** - Intrusion prevention enabled
- **Weekly audit routine** - Automated security checks

### Version 2.6.1 - Fish Shell Enhancements
- Fixed `colors` alias to display Faelight Forest palette
- Fixed `keys` alias to show keybindings reference
- Added bash/sh protection warnings
- Better error handling and command-not-found messages
- All 411 lines of Fish config preserved and enhanced

### Version 2.6.0 - GNU Stow Migration
- **Professional Dotfile Management** with GNU Stow
- Declarative symlink management (`stow`, `stow -D`, `stow -R`)
- Package-based structure for modular configs
- Easy to add/remove individual configs
- Better conflict handling during installation

---

## 🌟 Previous Features (Version 2.5)

### 🎨 Visual Polish
- **Brave Browser Theming** - Custom Faelight Forest CSS for new tabs
- **Mako Notifications** - Beautiful forest-themed popups with urgency colors
- **Papirus Icons** - Sunset-colored folders matching the aesthetic
- **GTK Theming** - Consistent look across all applications

### 📚 Enhanced Documentation
- **Meld Visual Diff Guide** - Complete workflows for config verification
- **Keybindings Reference** - 100+ shortcuts organized by category
- **Browser Theming Guide** - Stylus CSS customization

### 🔧 New Tools
- **Thunar** - GUI file manager with Meld integration
- **Meld** - Visual diff tool for comparing configs
- **Verification Aliases** - `verify-hypr`, `verify-waybar`, etc.
- **Notification Controls** - `SUPER + I` for Do Not Disturb

**📖 Full version history:** [CHANGELOG.md](CHANGELOG.md)

---

## 📋 Table of Contents

<details>
<summary>Click to Expand Table of Contents</summary>

1. [What is Faelight Forest?](#what-is-faelight-forest)
2. [Preview](#preview)
3. [System Features](#system-features)
4. [Theme Colors](#theme-colors)
5. [Included Components](#included-components)
6. [Installation & Quick Start](#installation--quick-start)
7. [GNU Stow Management](#gnu-stow-management)
8. [Snapshots](#snapshots)
9. [Automated Backups](#automated-backups)
10. [Security Hardening](#security-hardening)
11. [Hyprland Keybindings](#hyprland-keybindings)
12. [Aliases & Functions](#aliases--functions)
13. [Workspaces](#workspaces)
14. [System Utilities](#system-utilities)
15. [Emergency Recovery](#emergency-recovery)
16. [Updating & Maintenance](#updating--maintenance)
17. [Documentation](#documentation)
18. [Credits & License](#credits--license)

</details>

---

## 🌟 What is Faelight Forest?

Faelight Forest combines:
- 🎨 **Beautiful custom theming** — teal/mint/lime palette everywhere
- 📦 **GNU Stow dotfile management** — Professional, declarative configs
- 📸 **NixOS-style snapshots** — BTRFS + Snapper (40+ snapshots)
- 🔄 **Automated GitHub backups** — every 6 hours
- 🛡️ **Enterprise security hardening** — VPN, firewall, encryption
- 🔧 **Full system reproducibility** — 170+ packages tracked
- 🚀 **Hyprland workflow optimization** — 100+ productivity keybindings
- 🔔 **Themed notifications** — Even popups match the forest aesthetic
- 🔍 **Visual config verification** — Meld integration

> TL;DR: Your system will never die, break, or lose data. 🔥

---

## 📸 Preview

### Core Setup
- **WM:** Hyprland (gradient borders teal → mint)
- **Bar:** Waybar with icon workspaces & VPN status
- **Launcher:** Walker (Faelight themed)
- **Terminal:** Kitty (Faelight colors)
- **Editor:** LazyVim (Faelight theme, 100+ keybindings)
- **File Managers:** Thunar (GUI) + Yazi (TUI)
- **Browser:** Brave (custom themed new tabs)
- **Notifications:** Mako (forest green with cyan borders)
- **Icons:** Papirus-Dark with sunset-colored folders

### Visual Features
- 🌲 Forest green backgrounds (#0f1c16)
- 💎 Cyan/teal accent colors
- 🌿 Mint green highlights
- 🍋 Lime green alerts
- 🌅 Sunset orange folders

---

## 🛠️ System Features

### Core Features
- **GNU Stow:** Professional dotfile management with declarative symlinks
- **Snapshots:** Hourly, daily, and pre-update automatic BTRFS snapshots (40+)
- **Auto-sync:** GitHub backup every 6 hours with full commit history
- **Security:** VPN (Mullvad), encrypted DNS, UFW firewall, fail2ban
- **Recovery:** Boot into any snapshot, full disaster recovery ready
- **Reproducibility:** One-command system restoration anywhere

### Version 2.6 Highlights
- **Stow-based configs:** Clean package structure, easy maintenance
- **Perfect theme coherence:** Nvim background matches Kitty/Waybar
- **LazyVim restored:** Full functionality with proper init.lua
- **Modular management:** Add/remove configs independently

---

## 🎨 Theme Colors

| Type       | Color | Hex       | Usage |
|------------|-------|-----------|-------|
| Primary    | Teal  | `#5bb7a5` | Borders, accents |
| Secondary  | Mint  | `#8ed1a3` | Text, highlights |
| Accent     | Lime  | `#c7df63` | Alerts, focus |
| Text       | Mint  | `#e8f5d5` | Primary text |
| Background | Dark  | `#0f1c16` | Window backgrounds |
| Surface    | Moss  | `#2e6146` | UI surfaces |

**View your palette:** Run `colors` in terminal

---

## 📦 Included Components

### GNU Stow Package Structure
```
~/dotfiles/
├── fish/.config/fish/              # Fish shell (100+ aliases)
├── hypr/.config/hypr/              # Hyprland configs
├── waybar/.config/waybar/          # Status bar
├── walker/.config/walker/          # Launcher
├── kitty/.config/kitty/            # Terminal
├── nvim/.config/nvim/              # LazyVim with init.lua
├── yazi/.config/yazi/              # File manager TUI
├── mako/.config/mako/              # Notifications
├── gtk/.config/gtk-{3.0,4.0}/      # GTK theming
├── packages/                       # 170+ tracked packages
├── scripts/                        # Utility scripts
├── system/                         # Snapper config
├── brave/                          # Browser theming
└── docs/                           # Complete documentation
    ├── COMPLETE_GUIDE.md
    ├── KEYBINDINGS.md
    ├── MELD_GUIDE.md
    └── CHANGELOG.md
```

### Key Statistics
- **170+ official packages** explicitly installed
- **4+ AUR packages** tracked
- **40+ BTRFS snapshots** maintained
- **100+ keybindings** optimized
- **100+ Fish aliases** for productivity
- **Auto-sync every 6 hours** to GitHub
- **GNU Stow packages** for clean config management

---

## ⚡ Installation & Quick Start

### Prerequisites
```bash
# Required
sudo pacman -S stow git

# System requirements
- Arch Linux with BTRFS (@, @home, @log, @pkg subvolumes)
```

### Installation Steps
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles
cd ~/dotfiles

# Install packages (170+ packages)
cd packages
./install.sh

# Install dotfiles with GNU Stow
cd ~/dotfiles
./install.sh

# What install.sh does:
# - Backs up existing configs
# - Uses GNU Stow to create symlinks
# - Installs all packages (fish, hypr, waybar, etc.)
# - Sets up Papirus folder colors
# - Configures scripts and documentation

# Setup Snapper (automatic snapshots)
sudo pacman -S snapper snap-pac grub-btrfs
yay -S inotify-tools
sudo cp system/snapper-root.conf /etc/snapper/configs/root
sudo systemctl enable --now snapper-timeline.timer snapper-cleanup.timer grub-btrfsd

# Setup automated backups
sudo pacman -S cronie
sudo systemctl enable --now cronie
crontab system/crontab

# Enable Mullvad VPN daemon
sudo systemctl enable --now mullvad-daemon

# Start Mako notifications
mako &

# Reload Hyprland
hyprctl reload
```

**✅ Done! Your Faelight Forest is complete.** 🌲

### Quick Access Commands
```bash
guide              # View complete system guide
keys               # Quick keybinding reference (or press SUPER + /)
colors             # Display theme palette
health             # System health check
snapshots          # List all BTRFS snapshots
verify-all         # Compare all configs with dotfiles
```

---

## 📦 GNU Stow Management

### What is Stow?
GNU Stow creates symlinks from `~/dotfiles/package/.config/app/` to `~/.config/app/`. This gives you:
- ✅ Clean, standard dotfile structure
- ✅ Easy version control (everything in one repo)
- ✅ Simple to add/remove configs
- ✅ No manual symlink management

### Stow Commands
```bash
# Install a package (create symlinks)
cd ~/dotfiles
stow fish         # Install Fish config only
stow hypr         # Install Hyprland config only

# Remove a package (delete symlinks)
stow -D fish      # Remove Fish symlinks

# Reinstall a package
stow -R waybar    # Re-create Waybar symlinks

# Install everything (recommended)
./install.sh      # Automated install with backups

# Check what stow would do (dry run)
stow -n fish      # Show what would happen
```

### Verify Symlinks
```bash
# Check if configs are properly linked
ls -la ~/.config/ | grep "\->"

# Should show symlinks like:
# fish -> ../dotfiles/fish/.config/fish
# hypr -> ../dotfiles/hypr/.config/hypr
# waybar -> ../dotfiles/waybar/.config/waybar
```

### Editing Configs
When you edit a config in `~/.config/`, you're actually editing the file in `~/dotfiles/` thanks to symlinks!
```bash
# Edit Hyprland config
nvim ~/.config/hypr/hyprland.conf

# This actually edits:
# ~/dotfiles/hypr/.config/hypr/hyprland.conf

# Check git status
cd ~/dotfiles
git status
# Shows: modified: hypr/.config/hypr/hyprland.conf
```

**💡 Pro Tip:** Your configs automatically update in the dotfiles repo!

---

## 📸 Snapshots

### Automatic BTRFS Snapshots
Your system automatically creates snapshots:
- ✅ **Before every pacman install** (pre/post hooks)
- ✅ **Hourly** (last 5 hours kept)
- ✅ **Daily** (last 7 days kept)
- ✅ **Manual** anytime you want

**Current Status:** 40+ snapshots protecting your system

### Commands
```bash
snapshots           # List all snapshots
snapshot "desc"     # Create manual snapshot
```

### Example Usage
```bash
# Before major changes
snapshot "Before GNU Stow migration"

# View all snapshots
snapshots

# Rollback instructions: see docs/COMPLETE_GUIDE.md
```

---

## 🔄 Automated Backups

### GitHub Auto-Sync
Your system automatically backs up to GitHub:
- ⏰ **Every 6 hours** - Full dotfiles sync
- ⏰ **Daily at 11 PM** - Package lists update
- 📊 **Auto-commit** - Changes logged with timestamps
- 🔄 **Auto-push** - Pushed to repository automatically

### What Gets Backed Up
✅ All configuration files (via Stow structure)  
✅ Fish shell setup and functions  
✅ Hyprland, Waybar, Walker configs  
✅ LazyVim, Kitty, Yazi themes  
✅ Mako notifications, GTK themes  
✅ Browser theming files  
✅ Package lists (170+ packages)  
✅ System scripts and utilities  
✅ Complete documentation  

### Manual Commands
```bash
auto-sync         # Full sync (configs + packages + commit + push)
dotfiles-sync     # Sync dotfiles only
save-packages     # Update package lists only
```

**View sync logs:** `tail -50 ~/.auto-sync.log`

---

## 🛡️ Security Hardening

### Active Security Layers
```
✅ Full Disk Encryption (LUKS2)
✅ UFW Firewall (configured and active)
✅ Mullvad VPN (with Waybar status indicator)
✅ DNS over TLS (Cloudflare 1.1.1.1)
✅ Fail2ban (intrusion prevention)
✅ Gitleaks pre-commit hook (prevents credential leaks)
✅ 88-line .gitignore (zero credential exposure)
✅ KeePassXC password manager
✅ Disabled unnecessary services
```

### VPN Status Indicator
**Waybar VPN module shows:**
- 🟢 **Green** = Connected (shows location)
- 🟡 **Yellow** = Connecting (animated)
- 🔴 **Red** = Disconnected

**Click to toggle** VPN on/off directly from Waybar!

### Security Commands
```bash
health            # System health overview
audit-secrets     # Scan for leaked credentials
sudo ufw status   # Check firewall
mullvad status    # Check VPN connection
```

---

## 🔑 Hyprland Keybindings

### Essential Shortcuts
```
SUPER + SPACE       Launcher (Walker)
SUPER + RETURN      Terminal (Kitty)
SUPER + B           Browser (Brave)
SUPER + E           File Manager (Thunar)
SUPER + /           Keybindings Help (opens full reference!)
SUPER + 1-5         Switch workspace
SUPER + Q           Close window
SUPER + L           Lock screen
SUPER + I           Toggle Do Not Disturb
```

### Quick Reference Categories
- **🚀 Applications** - Launch programs
- **🪟 Window Management** - Focus, move, resize (Vim keys!)
- **🗂️ Workspaces** - 5 themed workspaces with icons
- **📸 Screenshots** - Multiple capture modes
- **🔔 Notifications** - DND and clearing
- **🔊 Media** - Volume, brightness, playback
- **🔒 Power** - Lock, logout, suspend, hibernate

**📖 Complete list:** Press `SUPER + /` or see [KEYBINDINGS.md](docs/KEYBINDINGS.md)

<details>
<summary>Click to Expand Core Keybindings</summary>

### Applications (SUPER + Key)
| Key | Action |
|-----|--------|
| `SUPER + RETURN` | Terminal (Kitty) |
| `SUPER + B` | Browser (Brave) |
| `SUPER + E` | File Manager (Thunar) |
| `SUPER + N` | Editor (Neovim) |
| `SUPER + C` | VSCode |
| `SUPER + SHIFT + Y` | Yazi (TUI file manager) |

### Window Management
| Key | Action |
|-----|--------|
| `SUPER + H/J/K/L` | Focus (Vim keys) |
| `SUPER + SHIFT + H/J/K/L` | Move window |
| `SUPER + CTRL + H/J/K/L` | Resize window |
| `SUPER + Q` | Close window |
| `SUPER + V` | Toggle floating |
| `SUPER + F` | Fullscreen |

### Workspaces
| Key | Workspace |
|-----|-----------|
| `SUPER + 1` |  Terminal |
| `SUPER + 2` | 󰈹 Browser (Brave auto-opens here!) |
| `SUPER + 3` | 󰉋 Files |
| `SUPER + 4` |  Code |
| `SUPER + 5` | 󰖯 Default |

### Screenshots
| Key | Action |
|-----|--------|
| `SUPER + S` | Full screen → save |
| `SUPER + SHIFT + S` | Area selection → save |
| `SUPER + ALT + S` | Area → clipboard |
| `SUPER + CTRL + S` | Area → editor (Swappy) |

### Notifications
| Key | Action |
|-----|--------|
| `SUPER + I` | Toggle Do Not Disturb |
| `SUPER + SHIFT + I` | Clear all notifications |

</details>

---

## 🐟 Aliases & Functions

### System Management
```bash
# Package Management
pacu               # sudo pacman -Syu (update)
paci               # sudo pacman -S (install)
pacr               # sudo pacman -Rns (remove)
yup                # yay -Syu (AUR update)

# System Maintenance
safe-update        # Snapshot + update system
sys-cleanup        # Clean caches and orphans
health             # System health check
```

### File Operations (Modern Tools)
```bash
ls                 # eza (colored, icons)
ll                 # eza -lah (detailed list)
cat                # bat (syntax highlighting)
grep               # rg (ripgrep)
find               # fd (faster find)
```

### Git Shortcuts
```bash
lg                 # LazyGit (TUI)
gs                 # git status
ga                 # git add
gc                 # git commit -m
gp                 # git push
```

### Faelight Forest Specific
```bash
# Documentation
guide              # View complete guide
keys               # Keybindings reference
colors             # Theme palette

# Snapshots & Backup
snapshots          # List BTRFS snapshots
snapshot "desc"    # Create snapshot
auto-sync          # Full GitHub sync
dotfiles-sync      # Sync configs only

# File Managers
y                  # Yazi (TUI)
thunar             # Thunar (GUI)

# Visual Verification
verify-hypr        # Compare Hypr configs
verify-waybar      # Compare Waybar configs
verify-kitty       # Compare Kitty configs
verify-fish        # Compare Fish configs
verify-nvim        # Compare Neovim configs
verify-all         # Compare everything
compare            # meld (visual diff tool)
```

---

## 🗂️ Workspaces

### 5 Themed Workspaces
1. **💻  Terminal** (WS 1) - Kitty, terminals, system monitors
2. **🌐 󰈹 Browser** (WS 2) - Brave (auto-opens here!)
3. **📁 󰉋 Files** (WS 3) - Thunar, Yazi, file management
4. **💻  Code** (WS 4) - Neovim, VSCode, editors
5. **🎨 󰖯 Default** (WS 5) - Everything else (Discord, Signal, media)

### Scratchpad (Special Workspace)
Floating workspace for quick access windows:
```
SUPER + M           Toggle scratchpad
SUPER + SHIFT + M   Move window to scratchpad
SUPER + ALT + M     Move silently to scratchpad
```

Perfect for: Calculator, KeePassXC, system monitors

---

## 🔧 System Utilities

### Core Scripts
| Command | Description |
|---------|-------------|
| `safe-update` | Create snapshot → update system → check for issues |
| `sys-cleanup` | Clean caches, orphans, and temporary files |
| `quick-note` | Daily markdown scratchpad |
| `health` | Complete system health overview |

### Backup & Sync
| Command | Description |
|---------|-------------|
| `auto-sync` | Full sync (configs + packages + commit + push) |
| `dotfiles-sync` | Sync dotfiles only |
| `save-packages` | Update package lists |

### Security
| Command | Description |
|---------|-------------|
| `audit-secrets` | Scan for leaked credentials |
| `scan-secrets` | Gitleaks scan current directory |

---

## 🆘 Emergency Recovery

### Quick Recovery
**If system breaks after update:**
1. **Reboot** into GRUB
2. **Select snapshot** from GRUB menu (grub-btrfsd)
3. **Boot into working snapshot**
4. System restored! ✅

### Full Recovery (Manual)
```bash
# Boot from Arch USB
sudo mount /dev/nvme0n1p2 /mnt

# List snapshots
sudo btrfs subvolume list /mnt

# Rollback to working snapshot
sudo btrfs subvolume delete /mnt/@
sudo btrfs subvolume snapshot /mnt/.snapshots/36/snapshot /mnt/@
sudo reboot
```

### Fresh Install
**Complete system recreation:**
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles

# Install everything
cd ~/dotfiles/packages && ./install.sh
cd ~/dotfiles && ./install.sh

# Done! Exact system restored.
```

---

## 🔄 Updating & Maintenance

### Safe System Update
```bash
safe-update        # Snapshot + update + verify
```

**What it does:**
1. Creates pre-update snapshot
2. Updates all packages (pacman + yay)
3. Saves new package list
4. Checks for .pacnew files
5. Offers cleanup and reboot

### Update Dotfiles
```bash
cd ~/dotfiles
git pull
./install.sh       # Reinstalls with Stow
dotfiles-sync      # Sync your changes back
```

### Stow Package Updates
```bash
# Reinstall specific package
cd ~/dotfiles
stow -R fish       # Reinstall Fish config

# Check for conflicts
stow -n waybar     # Dry run

# Remove and reinstall
stow -D hypr && stow hypr
```

### Maintenance Schedule
- **Daily:** Auto-sync runs every 6 hours
- **Weekly:** Run `safe-update` and check `snapshots`
- **Monthly:** Run `sys-cleanup` and review `health`
- **As needed:** `verify-all` to check config drift

---

## 📚 Documentation

### Complete Guides
- **[COMPLETE_GUIDE.md](docs/COMPLETE_GUIDE.md)** - Full system reference
  - All commands, workflows, and features
  - Troubleshooting and recovery
  - 🎯 Run `guide` to view anytime
  
- **[KEYBINDINGS.md](docs/KEYBINDINGS.md)** - 100+ keyboard shortcuts
  - Organized by category with tables
  - Pro tips and workflow patterns
  - 🎯 Press `SUPER + /` or run `keys`
  
- **[MELD_GUIDE.md](docs/MELD_GUIDE.md)** - Visual diff workflows
  - File and directory comparison
  - Verification alias usage
  - Thunar integration
  - Real-world examples
  
- **[brave/THEMING.md](brave/THEMING.md)** - Browser customization
  - Faelight Forest CSS theme
  - Color palette reference
  - Stylus installation guide
  
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
  - All updates and changes
  - Version 2.6 GNU Stow migration

### Package Documentation
- **[packages/README.md](packages/README.md)** - Package management
- **[system/README.md](system/README.md)** - System config restoration

### Quick Access
```bash
guide              # Open complete guide
keys               # Open keybindings reference
colors             # Show theme colors
health             # System health check
```

---

## 🙏 Credits & License

### Core Technologies
- **Theme:** Faelight Forest (custom)
- **Dotfile Manager:** [GNU Stow](https://www.gnu.org/software/stow/)
- **OS:** Arch Linux
- **WM:** [Hyprland](https://hyprland.org/)
- **Bar:** [Waybar](https://github.com/Alexays/Waybar)
- **Launcher:** [Walker](https://github.com/abenz1267/walker)
- **Shell:** [Fish](https://fishshell.com/)
- **Terminal:** [Kitty](https://sw.kovidgoyal.net/kitty/)
- **Editor:** [LazyVim](https://www.lazyvim.org/)
- **File Managers:** [Thunar](https://docs.xfce.org/xfce/thunar/start), [Yazi](https://yazi-rs.github.io/)
- **Visual Diff:** [Meld](https://meldmerge.org/)
- **Notifications:** [Mako](https://github.com/emersion/mako)
- **Icons:** [Papirus](https://github.com/PapirusDevelopmentTeam/papirus-icon-theme)
- **Snapshots:** [Snapper](http://snapper.io/)
- **VPN:** [Mullvad](https://mullvad.net/)

### License
This project is licensed under the **MIT License**.

---

## 🌲 Welcome to Faelight Forest

Your system is **immortal, secure, and beautifully productive**.

### You'll Never Worry About:
- ❌ Breaking your system
- ❌ Losing configurations
- ❌ Manual symlink management
- ❌ Being unable to restore
- ❌ Leaking credentials
- ❌ Config drift
- ❌ Ugly interfaces

### You'll Always Have:
- ✅ 40+ snapshots to roll back to
- ✅ GitHub backup of everything
- ✅ GNU Stow managing symlinks
- ✅ Complete documentation
- ✅ One-command restoration
- ✅ Zero credential leaks
- ✅ Visual config verification
- ✅ Beautiful themed everything
- ✅ 100+ productivity shortcuts

---

**🌲 May your Faelight Forest grow eternal! 🌲✨**

*Version 2.7.2 - Security Hardened Edition*  
*Built with ❤️ by Christian*  
*November 30, 2025*

**Repository:** [github.com/WidkidoneR2/dotfiles](https://github.com/WidkidoneR2/dotfiles)
