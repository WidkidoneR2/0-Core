# 🌲 FAELIGHT FOREST - COMPLETE MASTER GUIDEw

**The Ultimate Reference for Your Immortal Arch Linux System**

**Version:** 2.0 - Legendary Edition  
**Last Updated:** November 20, 2025  
**System Status:** IMMORTAL ♾️  
**Snapshots:** 34+  
**Commits:** 13

---

## 📋 Table of Contents

1. [Quick Start](#quick-start)
2. [System Overview](#system-overview)
3. [Fish Shell Reference](#fish-shell-reference)
4. [Hyprland Keybindings](#hyprland-keybindings)
5. [LazyVim Reference](#lazyvim-reference)
6. [Snapshot System](#snapshot-system)
7. [Backup & Sync System](#backup-sync-system)
8. [Security Features](#security-features)
9. [Package Management](#package-management)
10. [Recovery Procedures](#recovery-procedures)
11. [Troubleshooting](#troubleshooting)

---

## 🚀 Quick Start

### Essential Commands
```bash
# Documentation
guide              # This guide (view anytime)
keys               # Quick keybinding reference
colors             # Theme color palette
health             # System health check

# System Management
safe-update        # Snapshot + update system
sys-cleanup        # Clean caches and orphans
quick-note         # Daily scratchpad

# Snapshots
snapshots          # View all BTRFS snapshots
snapshot "desc"    # Create named snapshot

# Backups
dotfiles-sync      # Manual sync to GitHub
save-packages      # Update package lists
auto-sync          # Full automated sync

# Security
audit-secrets      # Scan repo for leaked credentials

# File Management
y                  # Yazi file manager

# Git Shortcuts
lg                 # LazyGit
gs                 # Git status
gp                 # Git push
```

---

## 🌲 System Overview

### What Makes This System Special
```
🎨 BEAUTIFUL       - Faelight Forest theme everywhere
📸 IMMORTAL        - 34+ BTRFS snapshots (never lose data)
🔄 SELF-BACKING    - Auto-sync to GitHub every 6 hours
🛡️ ULTRA-SECURE    - 88-line .gitignore, zero credential leaks
📦 REPRODUCIBLE    - Recreate exact system anywhere
⚡ OPTIMIZED       - 100+ productivity keybindings
📚 DOCUMENTED      - Complete guides for everything
```

### Core Components

- **OS**: Arch Linux
- **Filesystem**: BTRFS with subvolumes
- **Window Manager**: Hyprland
- **Status Bar**: Waybar (with VPN module)
- **Launcher**: Walker (Faelight themed)
- **Terminal**: Kitty
- **Shell**: Fish (100+ aliases)
- **Editor**: LazyVim
- **File Manager**: Yazi (themed)
- **Password Manager**: KeePassXC
- **Snapshots**: Snapper + snap-pac
- **VPN**: Mullvad with Waybar indicator
- **Backup**: Git + Cron automation

---

## 🐠 Fish Shell Reference

### Navigation Aliases
```bash
# Directory Navigation
..                 # cd ..
...                # cd ../..
....               # cd ../../..
~                  # cd ~
-                  # cd -

# Quick Access
dl                 # ~/Downloads
docs               # ~/Documents
pics               # ~/Pictures
vids               # ~/Videos
conf               # ~/.config
```

### File Operations (Modern Tools)
```bash
ls                 # eza (colored, icons)
ll                 # eza -lah (detailed list)
la                 # eza -a (show hidden)
lt                 # eza --tree (tree view)
tree               # eza --tree --level=2

cat                # bat (syntax highlighting)
grep               # rg (ripgrep - faster)
find               # fd (faster, simpler)
```

### System Management
```bash
# Package Management
pacu               # sudo pacman -Syu (update)
paci               # sudo pacman -S (install)
pacr               # sudo pacman -Rns (remove)
pacs               # pacman -Ss (search)
pacq               # pacman -Q | grep (query)
yup                # yay -Syu (AUR update)
yins               # yay -S (AUR install)

# System Info
ff                 # fastfetch (system info)
neo                # neofetch
dsize              # du -sh * (directory sizes)
ports              # netstat -tulanp (open ports)

# System Maintenance
sys-cleanup        # Full system cleanup
orphans            # List orphan packages
clean-cache        # Clear package cache
```

### Git Shortcuts
```bash
lg                 # lazygit (TUI)
gs                 # git status
ga                 # git add
gc                 # git commit -m
gp                 # git push
gpl                # git pull
gd                 # git diff
glog               # git log --oneline --graph
```

### Development
```bash
v                  # nvim
vim                # nvim
python             # python3
py                 # python3
```

### Faelight Forest Specific
```bash
# Documentation
guide              # View this complete guide
keys               # Quick keybinding reference  
colors             # Display theme colors
health             # System health check

# Quick Access
quick-note         # Daily markdown scratchpad

# Snapshots
snapshots          # List all BTRFS snapshots
snapshot "desc"    # Create named snapshot

# Backups
dotfiles-sync      # Sync configs to GitHub
save-packages      # Update package lists
auto-sync          # Full automated sync
safe-update        # Snapshot + system update

# Security
audit-secrets      # Audit dotfiles for credentials

# File Manager
y                  # yazi (themed file manager)
```

---

## ⌨️ Hyprland Keybindings

### Essential (Learn These First!)
```
SUPER + SPACE       Launcher (Walker)
SUPER + RETURN      Terminal (Kitty)
SUPER + B           Browser (Firefox)
SUPER + E           File Manager
SUPER + 1-5         Switch workspace
SUPER + Q           Close window
SUPER + L           Lock screen
SUPER + SHIFT + E   Exit Hyprland
```

### Window Management
```
# Focus Navigation
SUPER + H/J/K/L     Move focus (vim-style)
SUPER + Arrow Keys  Move focus (arrows)

# Move Windows
SUPER + SHIFT + H/J/K/L    Move window (vim-style)
SUPER + SHIFT + Arrow Keys Move window (arrows)

# Resize Windows
SUPER + R           Enter resize mode
  Then: H/J/K/L     Resize (vim-style)
  ESC or SUPER + R  Exit resize mode

# Window States
SUPER + F           Fullscreen toggle
SUPER + V           Floating toggle
SUPER + P           Pin window
SUPER + S           Special workspace (scratchpad)
```

### Workspaces
```
# Switch Workspaces
SUPER + 1-5         Go to workspace 1-5

# Move Windows
SUPER + SHIFT + 1-5 Move window to workspace 1-5

# Workspace Types
 Terminal  (WS 1)   Kitty, terminals
󰈹 Browser   (WS 2)   Firefox, Brave
󰉋 Files     (WS 3)   Nautilus, Yazi
 Code      (WS 4)   Neovim, VSCode
󰖯 Default   (WS 5)   Everything else
```

### Applications
```
SUPER + SPACE       Walker launcher
SUPER + RETURN      Kitty terminal
SUPER + B           Browser
SUPER + E           File manager
SUPER + N           Neovim
```

### Screenshots
```
PRINT               Full screenshot
SUPER + PRINT       Select area
SUPER + SHIFT + S   Select area (alt)
```

### Media Keys
```
Volume Up/Down      Adjust volume
Mute                Toggle mute
Brightness Up/Down  Adjust brightness
Media Play/Pause    Control playback
Media Next/Previous Track control
```

### System
```
SUPER + L           Lock screen
SUPER + SHIFT + R   Reload Hyprland config
SUPER + SHIFT + E   Exit Hyprland
SUPER + ALT + W     Restart Waybar
```

### Special
```
SUPER + M           Toggle Waybar
SUPER + ALT + Q     Kill window (force)
SUPER + SHIFT + SPACE Toggle floating
```

---

## 📝 LazyVim Reference

### Essential Keybindings (Normal Mode)

#### File Operations
```
<leader>ff         Find files (Telescope)
<leader>fg         Find text (grep)
<leader>fb         Find buffers
<leader>fr         Recent files
<leader>e          Toggle file tree (Neo-tree)
<leader>w          Save file
<leader>q          Quit
<leader>Q          Quit without saving
```

#### Navigation
```
Ctrl + h/j/k/l     Navigate splits
<leader><leader>   Find files (quick)
<leader>/          Search in current buffer
<leader>,          Switch buffers
<leader>`          Switch to last buffer
```

#### Code Operations
```
gd                 Go to definition
gr                 Go to references
K                  Hover documentation
<leader>ca         Code actions
<leader>rn         Rename symbol
[d / ]d            Next/previous diagnostic
<leader>xx         Open trouble (diagnostics)
```

#### Editing
```
gcc                Comment line
gc                 Comment selection (visual)
<leader>gg         LazyGit
<leader>ut         Toggle terminal
```

### Neo-tree (File Explorer)
```
<leader>e          Toggle Neo-tree
a                  Add file/folder
d                  Delete
r                  Rename
c                  Copy
x                  Cut
p                  Paste
```

### Telescope (Fuzzy Finder)
```
<leader>ff         Find files
<leader>fg         Live grep
<leader>fb         Buffers
<leader>fh         Help tags
<leader>fr         Recent files
<leader>fs         Git status
Ctrl + j/k         Navigate results
Enter              Open file
Ctrl + x           Open in split
Ctrl + v           Open in vsplit
```

### Terminal (ToggleTerm)
```
<C-\>              Toggle terminal
<leader>ut         Toggle terminal
<Esc><Esc>         Exit terminal mode
<C-h/j/k/l>        Navigate from terminal
```

### LazyGit Integration
```
<leader>gg         Open LazyGit
<leader>gf         LazyGit current file history
q                  Close LazyGit
```

### Hidden Files Configuration

LazyVim shows hidden files by default in this setup! ✅

---

## 📸 Snapshot System (Snapper)

### Overview

Your system creates **automatic BTRFS snapshots**:
- ✅ **Hourly** - Last 5 hours kept
- ✅ **Daily** - Last 7 days kept
- ✅ **Pre/Post** - Before every package install
- ✅ **Manual** - Anytime you want

**Current Status:** 34+ snapshots and growing! 🌲

### View Snapshots
```bash
snapshots          # List all snapshots
```

Example output:
```
 # │ Type   │ Date                            │ Description
───┼────────┼─────────────────────────────────┼────────────
25 │ single │ Thu 20 Nov 2025 09:53:20 AM CST │ Blueprint Complete
26 │ single │ Thu 20 Nov 2025 10:00:02 AM CST │ timeline
27 │ single │ Thu 20 Nov 2025 10:01:00 AM CST │ timeline
28 │ single │ Thu 20 Nov 2025 10:03:25 AM CST │ LEGENDARY STATUS
```

### Create Manual Snapshot
```bash
snapshot "Description of what you're doing"
```

Examples:
```bash
snapshot "Before major system update"
snapshot "Before installing NVIDIA drivers"
snapshot "Working configuration backup"
```

### Snapshot Details
```bash
# View specific snapshot
sudo snapper -c root list | grep 25

# Compare two snapshots
sudo snapper -c root status 25..28

# Check disk space used by snapshots
sudo snapper -c root list | tail -10
```

### Automatic Snapshot Triggers

Snapshots are **automatically created** when:
1. ✅ Installing packages with `pacman` or `yay`
2. ✅ Every hour (timeline snapshots)
3. ✅ When you run `safe-update`

### Rollback to Snapshot

See [Recovery Procedures](#recovery-procedures) for detailed rollback instructions.

---

## 🔄 Backup & Sync System

### Auto-Sync Overview

Your system **automatically backs up** to GitHub:
- ⏰ **Every 6 hours** - Full dotfiles sync
- ⏰ **Daily at 11 PM** - Package lists update
- 📊 **Auto-commit** - Changes logged with timestamps
- 🔄 **Auto-push** - Pushed to GitHub automatically

**Repository:** https://github.com/WidkidoneR2/dotfiles

### Manual Sync Commands
```bash
# Full sync (configs + packages + commit + push)
auto-sync

# Sync dotfiles only
dotfiles-sync

# Update package lists only
save-packages
```

### What Gets Backed Up

✅ **All dotfiles:**
- Fish Shell configuration + functions
- Hyprland configuration (all .conf files)
- Waybar config + styling
- Walker configuration + theme
- Kitty terminal config
- Yazi file manager + theme
- LazyVim configuration

✅ **System configs:**
- Snapper configuration
- Crontab schedule

✅ **Package tracking:**
- 167 official packages
- 4 AUR packages  
- Complete version list
- Installation script

✅ **Scripts:**
- safe-update
- save-packages
- dotfiles-sync
- auto-sync
- sys-cleanup
- quick-note
- VPN status/toggle

✅ **Documentation:**
- This complete guide
- Recovery guide
- README

### Check Sync Status
```bash
# View auto-sync log
tail -50 ~/.auto-sync.log

# Check cron jobs
crontab -l

# View GitHub commits
cd ~/dotfiles
git log --oneline -10

# Check repository status
cd ~/dotfiles
git status
```

### Restore from Backup
```bash
# On new machine:
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles
cd ~/dotfiles

# Install packages
cd packages
./install.sh

# Install dotfiles
cd ~/dotfiles
./install.sh

# Done! Exact system restored! ✅
```

---

## 🛡️ Security Features

### Active Security Layers
```
✅ Full Disk Encryption (LUKS2)
✅ UFW Firewall (active)
✅ Mullvad VPN (with Waybar indicator)
✅ DNS over TLS (Cloudflare 1.1.1.1)
✅ Fail2ban (intrusion prevention)
✅ Ultra-secure .gitignore (88 lines, zero credential leaks)
✅ KeePassXC password manager
```

### VPN Status (Waybar Module)

**Visual indicators:**
- 🟢 **Green** = Connected (shows location)
- 🟡 **Yellow** = Connecting (animated)
- 🔴 **Red** = Disconnected

**Click to toggle** VPN on/off!

### Security Audit
```bash
# Scan dotfiles for leaked credentials
audit-secrets

# Should show: ✅ No secrets found!
```

### Check Security Status
```bash
# Firewall
sudo ufw status

# VPN
mullvad status

# Fail2ban
sudo fail2ban-client status

# DNS over TLS
resolvectl status | grep DNSOverTLS

# Services
systemctl list-units --state=running
```

### Security Commands
```bash
# Connect/disconnect VPN
mullvad connect
mullvad disconnect

# Firewall management
sudo ufw status
sudo ufw enable
sudo ufw disable

# View fail2ban logs
sudo journalctl -u fail2ban -n 50
```

---

## 📦 Package Management

### Package Tracking

Your system tracks **all packages** for full reproducibility:
- 📦 **167 official packages** (explicitly installed)
- 📦 **4 AUR packages** 
- 📦 **925 total packages** (with dependencies)

### Update Package Lists
```bash
save-packages      # Updates all package lists
```

This creates:
- `official.txt` - Official Arch packages
- `aur.txt` - AUR packages
- `all-with-versions.txt` - All packages with versions
- `install.sh` - One-command restore script

### Safe System Update
```bash
safe-update        # Creates snapshot + updates system
```

**What it does:**
1. ✅ Checks for updates
2. ✅ Shows disk space
3. ✅ Creates pre-update snapshot
4. ✅ Saves package list
5. ✅ Runs system update (pacman + yay)
6. ✅ Checks for .pacnew files
7. ✅ Offers to reboot if kernel updated
8. ✅ Runs cleanup if desired

### Manual Package Management
```bash
# Update system
sudo pacman -Syu    # Official packages
yay -Syu            # AUR packages

# Install packages
sudo pacman -S package_name
yay -S aur_package

# Remove packages
sudo pacman -Rns package_name

# Search packages
pacman -Ss search_term
yay -Ss aur_search

# View package info
pacman -Si package_name
yay -Si aur_package
```

### Package Lists Location
```
~/dotfiles/packages/
├── official.txt           # 167 official packages
├── aur.txt               # 4 AUR packages
├── all-with-versions.txt # All packages with versions
├── flatpak.txt           # Flatpak apps (if any)
├── groups.txt            # Package groups
├── install.sh            # Installation script
└── README.md             # Documentation
```

## 🛡️ Secret Protection with Gitleaks

**Automatic secret scanning prevents accidental credential commits.**

### Features
- Pre-commit hook blocks secrets automatically
- Scans for API keys, tokens, private keys, passwords
- Custom patterns in `.gitleaks.toml`

### Quick Commands
```fish
scan-secrets     # Scan current directory
scan-staged      # Check staged git files
```

### What's Protected
- AWS keys, GitHub tokens, API keys
- Private keys (.pem, .key files)
- Files matching `*secret*`, `*-secret*`, `*_secret*`

The pre-commit hook runs automatically - no action needed!
---

## 🆘 Recovery Procedures

### Emergency Snapshot Rollback

**If system breaks after update:**

1. **Reboot system**
2. **Boot from Arch USB/live system**

3. **Mount BTRFS filesystem:**
```bash
sudo mount /dev/nvme0n1p2 /mnt  # Adjust your partition
```

4. **List snapshots:**
```bash
sudo btrfs subvolume list /mnt
```

5. **Find your working snapshot** (e.g., #25)

6. **Rollback:**
```bash
# Delete current broken @ subvolume
sudo btrfs subvolume delete /mnt/@

# Create new @ from working snapshot
sudo btrfs subvolume snapshot /mnt/.snapshots/25/snapshot /mnt/@

# Reboot
sudo reboot
```

**Your system is restored!** ✅

### Fresh System Install

**Complete system recreation from scratch:**

1. **Install Arch Linux** with BTRFS
   - Create subvolumes: `@`, `@home`, `@log`, `@pkg`

2. **Clone dotfiles:**
```bash
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles
cd ~/dotfiles
```

3. **Install all packages:**
```bash
cd packages
./install.sh
```

4. **Install dotfiles:**
```bash
cd ~/dotfiles
./install.sh
```

5. **Setup Snapper:**
```bash
sudo pacman -S snapper snap-pac grub-btrfs
yay -S inotify-tools
sudo cp system/snapper-root.conf /etc/snapper/configs/root
sudo systemctl enable --now snapper-timeline.timer
sudo systemctl enable --now snapper-cleanup.timer
sudo systemctl enable --now grub-btrfsd
```

6. **Setup cron:**
```bash
sudo pacman -S cronie
sudo systemctl enable --now cronie
crontab system/crontab
```

7. **Start VPN daemon:**
```bash
sudo systemctl enable --now mullvad-daemon
```

**Done! Exact system restored!** 🎉

### Recover Specific Configs
```bash
# Pull latest dotfiles
cd ~/dotfiles
git pull

# Reinstall specific component
./install.sh

# Or manually copy configs
cp ~/dotfiles/fish/config.fish ~/.config/fish/
cp ~/dotfiles/hypr/*.conf ~/.config/hypr/
# etc.
```

---

## 🔧 Troubleshooting

### Waybar Issues

**Waybar not showing:**
```bash
# Check if running
pgrep waybar

# Restart Waybar
killall waybar && waybar &

# Check for errors
waybar 2>&1 | head -20

# Reload Hyprland (auto-starts Waybar)
hyprctl reload
```

**VPN module not working:**
```bash
# Check Mullvad daemon
systemctl status mullvad-daemon

# Start daemon
sudo systemctl start mullvad-daemon

# Test VPN scripts
~/.local/bin/vpn-status
mullvad status
```

### Walker Issues

**Walker not opening:**
```bash
# Check if service is running
pgrep -a walker

# Start Walker service
walker --gapplication-service &

# Test Walker
walker
```

### Snapshot Issues

**Snapshots not creating:**
```bash
# Check Snapper timers
systemctl status snapper-timeline.timer
systemctl status snapper-cleanup.timer

# Check Snapper config
sudo cat /etc/snapper/configs/root

# Manually create snapshot to test
sudo snapper -c root create --description "Test snapshot"
```

### Sync Issues

**Auto-sync not working:**
```bash
# Check cron service
systemctl status cronie

# Check crontab
crontab -l

# View auto-sync log
tail -50 ~/.auto-sync.log

# Test manually
auto-sync
```

### Yazi Issues

**Hidden files not showing:**
```bash
# Check config
cat ~/.config/yazi/yazi.toml

# Toggle hidden files (in Yazi)
# Press: zh

# Reinstall config
cp ~/dotfiles/yazi/yazi.toml ~/.config/yazi/
```

### Package Issues

**Broken packages after update:**
```bash
# Rollback to snapshot before update
snapshots
# Note the pre-update snapshot number
# See Recovery Procedures above

# Or fix broken packages
sudo pacman -Syu
sudo pacman -S --overwrite '*' package_name
```

---

## 📊 System Status Commands

### Check Everything
```bash
health             # Complete health check

# Individual checks:
snapshots          # View snapshots
crontab -l         # Check scheduled tasks
systemctl list-timers | grep snapper
git -C ~/dotfiles status
mullvad status
sudo ufw status
```

### Logs
```bash
# Auto-sync log
tail -50 ~/.auto-sync.log

# System logs
journalctl -b      # Current boot
journalctl -u fail2ban
journalctl -u mullvad-daemon
```

### Disk Usage
```bash
# Snapshot space usage
sudo btrfs filesystem df /

# Package cache
du -sh /var/cache/pacman/pkg

# Home directory
du -sh ~/

# Clean up
sys-cleanup
```

---

## 🎨 Theme Reference

### Faelight Forest Colors
```
Primary:    #5bb7a5  (Bright teal)
Secondary:  #8ed1a3  (Mint green)
Accent:     #c7df63  (Lime green)
Text:       #e8f5d5  (Soft mint)
Background: #0f1c16  (Dark forest)
Surface:    #2e6146  (Moss green)
```

**View palette:**
```bash
colors
```

### Themed Applications

- ✅ Hyprland (borders, blur)
- ✅ Waybar (bar, modules)
- ✅ Walker (launcher)
- ✅ Kitty (terminal)
- ✅ Fish (prompt)
- ✅ LazyVim (via theme)
- ✅ Yazi (file manager)

---

## 🎯 Daily Workflow

### Morning Routine
```bash
# Check system health
health

# View snapshots (peace of mind!)
snapshots

# Update if needed
safe-update

# Start your day! ☕
```

### Before Major Changes
```bash
# Create snapshot before big changes
snapshot "Before installing X"

# Do your thing...

# If something breaks:
snapshots          # Find working snapshot
# See Recovery Procedures for rollback
```

### End of Day
```bash
# Quick note
quick-note

# Manual sync if you made lots of changes
dotfiles-sync

# (Or just let auto-sync handle it!) ✅
```

---

## 🌲 Final Notes

### Your System is Immortal Because:

1. ✅ **34+ snapshots** - Every hour, every day, every install
2. ✅ **Auto-backup** - GitHub sync every 6 hours
3. ✅ **Full tracking** - Every package documented
4. ✅ **One-command restore** - Recreate anywhere
5. ✅ **Complete docs** - This guide + recovery guide
6. ✅ **Security hardened** - VPN, firewall, encryption, zero leaks
7. ✅ **Beautiful theme** - Faelight Forest everywhere

### Remember:

- Run `guide` anytime to view this
- Run `keys` for quick keybinding reference
- Run `health` to check system status
- Run `audit-secrets` to check for credential leaks
- Check `snapshots` regularly (peace of mind!)
- Your system backs itself up automatically ✅

---

## 📚 Additional Resources

- **Repository:** https://github.com/WidkidoneR2/dotfiles
- **Recovery Guide:** See RECOVERY.md in dotfiles
- **Package Lists:** ~/dotfiles/packages/
- **System Configs:** ~/dotfiles/system/

---

## 🎊 Congratulations!

You have one of the most **robust, beautiful, and reproducible** Linux systems ever created.

**Your Faelight Forest stands eternal.** 🌲

**Never worry about:**
- ❌ Breaking your system
- ❌ Losing configurations  
- ❌ Forgetting how you set things up
- ❌ Not being able to restore
- ❌ Leaking credentials to GitHub

**Always have:**
- ✅ 34+ snapshots to roll back to
- ✅ GitHub backup of everything
- ✅ Complete documentation
- ✅ One-command system restoration
- ✅ Zero credential leaks

---

**🌲 May your Faelight Forest grow eternal! 🌲**

*Version 2.0 - Legendary Edition*  
*Built with ❤️ by Christian*  
*November 20, 2025*  
*Snapshots: 34+ | Commits: 13 | Status: LEGENDARY ♾️*
