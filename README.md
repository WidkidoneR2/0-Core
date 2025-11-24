# 🌲 Faelight Forest - The Immortal Arch Linux System

![License: MIT](https://img.shields.io/badge/License-MIT-green)
![Platform](https://img.shields.io/badge/Platform-Arch_Linux-blue)
![Hyprland](https://img.shields.io/badge/WM-Hyprland-teal)

Faelight Forest is a **fully reproducible, self-healing Arch Linux system** with NixOS-style snapshots, Hyprland workflow, and enterprise-grade security. Your system will never break, and your workflow is fully optimized.  

**Last Updated:** November 23, 2025

---

## 📋 Table of Contents

<details>
<summary>Click to Expand Table of Contents</summary>

1. [What is Faelight Forest?](#what-is-faelight-forest)
2. [Preview](#preview)
3. [System Features](#system-features)
4. [Theme Colors](#theme-colors)
5. [Included Packages & Scripts](#included-packages--scripts)
6. [Installation & Quick Start](#installation--quick-start)
7. [Snapshots](#snapshots)
8. [Automated Backups](#automated-backups)
9. [Security Hardening](#security-hardening)
10. [Hyprland Keybindings](#hyprland-keybindings)
11. [Aliases & Functions](#aliases--functions)
12. [Workspaces & Scratchpad](#workspaces--scratchpad)
13. [System Utilities](#system-utilities)
14. [Git Shortcuts](#git-shortcuts)
15. [Emergency Recovery](#emergency-recovery)
16. [Updating & Maintenance](#updating--maintenance)
17. [Documentation](#documentation)
18. [Credits & License](#credits--license)

</details>

---

## 🌟 What is Faelight Forest?

Faelight Forest combines:

- 🎨 **Beautiful custom theming** — teal/mint/lime palette
- 📸 **NixOS-style snapshots** — BTRFS + Snapper
- 🔄 **Automated GitHub backups** — every 6 hours
- 🛡️ **Enterprise security hardening**
- 📦 **Full system reproducibility**
- 🚀 **Hyprland workflow optimization** — 100+ keybindings

> TL;DR: Your system will never die, break, or lose data. 🔥

---

## 📸 Preview

- **WM:** Hyprland (gradient borders teal → mint)  
- **Bar:** Waybar with icon workspaces & VPN status  
- **Launcher:** Walker (Faelight themed)  
- **Terminal:** Kitty (Faelight colors)  
- **Editor:** LazyVim (100+ productivity keybindings)

---

## 🛠️ System Features

- **Snapshots:** Hourly, daily, and pre-update automatic BTRFS snapshots  
- **Auto-sync:** GitHub backup every 6 hours  
- **Security:** VPN, encrypted DNS, firewall, fail2ban  
- **Recovery:** Boot into any snapshot, full disaster recovery

---

## 🎨 Theme Colors

| Type       | Color | Hex       |
|------------|-------|-----------|
| Primary    | Teal  | `#5bb7a5`|
| Secondary  | Mint  | `#8ed1a3`|
| Accent     | Lime  | `#c7df63`|
| Text       | Mint  | `#e8f5d5`|
| Background | Dark  | `#0f1c16`|
| Surface    | Moss  | `#2e6146`|

---

## 📦 Included Packages & Scripts

- **faelight-forest/**
  - `fish/` - Fish shell (100+ aliases & functions)  
  - `hypr/` - Hyprland configs (keybindings, workspaces)  
  - `waybar/` - Status bar themed modules  
  - `walker/` - Launcher configuration  
  - `kitty/` - Terminal themes  
  - `nvim/` - LazyVim setup  
  - `packages/` - 167 official + 4 AUR packages, install scripts  
  - `scripts/` - Utility scripts (safe-update, auto-sync, sys-cleanup)  
  - `system/` - Snapper and system config backups  
  - `docs/` - Complete documentation  

---

## ⚡ Installation & Quick Start

**Prerequisites:**

- Arch Linux with BTRFS (`@`, `@home`, `@log`, `@pkg`)  
- Git installed  

**Steps:**

```bash
# Clone repository
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles
cd ~/dotfiles

# Install packages
cd packages
./install.sh

# Install dotfiles
cd ~/dotfiles
./install.sh

# Setup Snapper
sudo pacman -S snapper snap-pac grub-btrfs
yay -S inotify-tools
sudo cp system/snapper-root.conf /etc/snapper/configs/root
sudo systemctl enable --now snapper-timeline.timer snapper-cleanup.timer grub-btrfsd

# Setup auto-sync
sudo pacman -S cronie
sudo systemctl enable --now cronie
crontab system/crontab

# Enable Mullvad VPN
sudo systemctl enable --now mullvad-daemon

# Reload Hyprland
hyprctl reload
✅ Done! Your Faelight Forest is complete. 🌲

📸 Snapshots
Automatic BTRFS snapshots:

Before every pacman install

Hourly (last 5 hours)

Daily (last 7 days)

Manual snapshots anytime

Commands:

bash
Copy code
snapshots         # List snapshots
snapshot "desc"   # Create snapshot
Rollback instructions: see Emergency Recovery

🔄 Automated Backups
Auto-sync every 6 hours to GitHub

Syncs dotfiles, updates package lists, commits, logs to ~/.auto-sync.log

Manual Commands:

bash
Copy code
auto-sync         # Full sync
dotfiles-sync     # Dotfiles only
save-packages     # Package lists only
🛡️ Security Hardening
Full disk encryption (LUKS2)

UFW firewall

Mullvad VPN (Waybar indicator)

DNS over TLS (1.1.1.1)

Fail2ban enabled

Disabled unnecessary services (CUPS, Avahi)

Health check:

bash
Copy code
health
sudo ufw status
mullvad status
🔑 Hyprland Keybindings
<details> <summary>Click to Expand Full Keybindings</summary>
🌟 Core Applications (SUPER + Key)
Key	Action
SUPER+RETURN	Terminal (Kitty)
SUPER+B	Browser
SUPER+E	File Manager
SUPER+N	Editor
SUPER+C	VSCode
SUPER CTRL+RETURN	Terminal (Alacritty)

📁 File Managers
Key	Action
SUPER SHIFT+F	File Manager (GUI)
SUPER SHIFT+Y	File Manager (Yazi)

🌐 Browsers & Web
Key	Action
SUPER SHIFT+B	Browser (New)
SUPER SHIFT ALT+B	Browser (Private)

🤖 AI Assistants
Key	Action
SUPER SHIFT ALT+A	Claude
SUPER CTRL+A	Grok

💬 Communication
Key	Action
SUPER SHIFT+G	Signal
SUPER SHIFT+E	Email
SUPER SHIFT+C	Calendar

🎥 Media & Social
Key	Action
SUPER SHIFT+Y	YouTube
SUPER SHIFT+X	X/Twitter
SUPER SHIFT ALT+X	X Post

🛠️ System Utilities
Key	Action
SUPER SHIFT+T	Activity Monitor (btop)
SUPER SHIFT+D	Docker (lazydocker)
SUPER SHIFT+/	Passwords (KeePassXC)

✏️ Productivity Apps
Key	Action
SUPER SHIFT+O	Obsidian
SUPER SHIFT+W	Typora
SUPER SHIFT+N	Neovim

📋 Clipboard
Key	Action
SUPER+P	Clipboard History
SUPER SHIFT+P	Clear Clipboard
SUPER CTRL+P	Clipboard Menu

📸 Screenshots
Key	Action
SUPER+S	Full Screenshot
SUPER SHIFT+S	Area Screenshot
SUPER ALT+S	Clipboard Screenshot
SUPER CTRL+S	Editor Screenshot

🔒 System Controls
Key	Action
SUPER+L	Lock Screen
SUPER SHIFT+L	Logout
SUPER ALT+L	Suspend
SUPER CTRL+L	Hibernate
SUPER+ESC	Power Menu

🔊 Audio
Key	Action
XF86AudioRaiseVolume	Volume +5%
XF86AudioLowerVolume	Volume -5%
XF86AudioMute	Toggle Mute
XF86AudioMicMute	Toggle Mic
XF86AudioPlay	Play/Pause
XF86AudioNext	Next Track
XF86AudioPrev	Previous Track

🔆 Brightness
Key	Action
XF86MonBrightnessUp	+5% Brightness
XF86MonBrightnessDown	-5% Brightness

🪟 Window Management
Focus: SUPER+H/J/K/L or Arrows

Move: SUPER SHIFT+H/J/K/L or Arrows

Resize: SUPER CTRL+H/J/K/L or Arrows

Actions: SUPER+Q/V/F/Z/T/O

🗂️ Workspaces
5 Themed Workspaces: 💻 🌐 📝 💬 🎨

SUPER+[1-5]: Switch

SUPER SHIFT+[1-5]: Move Window

SUPER ALT+[1-5]: Move Window Silent

SUPER+W: Workspace Switcher

🖱️ Mouse
SUPER + Mouse Drag: Move/Resize

SUPER + Scroll: Switch Workspaces

🎮 Groups & Tabs
SUPER+G: Toggle Group

SUPER+TAB: Cycle Forward

SUPER SHIFT+TAB: Cycle Backward

🔧 Hyprland Controls
SUPER ALT+R: Reload WM

SUPER ALT+K: Kill WM

SUPER ALT+W: Restart Waybar

💡 Notifications & Help
SUPER+I: Toggle Notifications

SUPER SHIFT+I: Clear Notifications

SUPER+/: Keybindings Help

</details>
🐟 Aliases & Functions
<details> <summary>Common Aliases</summary>
fish
Copy code
# System
update         sudo pacman -Syu
install        sudo pacman -S
remove         sudo pacman -Rns

# Git
gs             git status
gcmsg          git commit -m
gp             git push

# Navigation
..             cd ..
cdir           cd $1; ls

# Scripts
backup         ~/dotfiles/scripts/backup.sh
syncdot        ~/dotfiles/scripts/dotfiles-sync
safeupdate     ~/dotfiles/scripts/safe-update
</details> <details> <summary>Common Functions</summary>
fish
Copy code
# Snapshot creation
snapshot "Description"

# Quick search history
searchhist "term"

# Launch default apps
omarchy-launch-editor
omarchy-launch-browser
</details>
🪟 Workspaces & Scratchpad
5 Themed Workspaces:

💻 Terminal

🌐 Browser

📝 Editor

💬 Communication

🎨 Creative

Scratchpad:

SUPER+M toggle

SUPER SHIFT+M move

SUPER ALT+M silent move

🔧 System Utilities
Command	Description
safe-update	Snapshot + update system
sys-cleanup	Clean caches, orphans
quick-note	Daily scratchpad
health	System health overview

🐙 Git Shortcuts
Alias	Command
lg	LazyGit
gs	git status
gp	git push

🆘 Emergency Recovery
Boot from Arch USB → mount BTRFS → list snapshots → rollback

Fresh install: clone repo → packages/install.sh → ./install.sh → dotfiles-sync

🔄 Updating & Maintenance
bash
Copy code
safe-update
cd ~/dotfiles
git pull
./install.sh
dotfiles-sync
📚 Documentation
COMPLETE_GUIDE.md — Full system reference

RECOVERY.md — Disaster recovery

packages/README.md — Package management

system/README.md — Config restoration

🙏 Credits & License
Theme: Faelight Forest

WM: Hyprland

Bar: Waybar

Launcher: Walker

Shell: Fish

Editor: LazyVim

Snapshots: Snapper

VPN: Mullvad

License: MIT

🌲 Welcome to Faelight Forest — your system is immortal, secure, and beautifully productive. 🌲✨
