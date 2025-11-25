# 🌲 FAELIGHT FOREST - COMPLETE MASTER GUIDE
**The Ultimate Reference for Your Immortal Arch Linux System**

**Version:** 2.5 - The Theming & Documentation Edition  
**Last Updated:** November 25, 2025  
**System Status:** IMMORTAL ♾️  
**Snapshots:** 40+  
**Commits:** 25+

---

## 📋 Table of Contents

1. [Quick Start](#quick-start)
2. [What's New in Version 2.5](#whats-new-in-version-25)
3. [System Overview](#system-overview)
4. [Fish Shell Reference](#fish-shell-reference)
5. [Hyprland Keybindings](#hyprland-keybindings)
6. [LazyVim Reference](#lazyvim-reference)
7. [Snapshot System](#snapshot-system)
8. [Backup & Sync System](#backup-sync-system)
9. [Security Features](#security-features)
10. [Package Management](#package-management)
11. [Recovery Procedures](#recovery-procedures)
12. [Troubleshooting](#troubleshooting)

---

## 🚀 Quick Start

### Essential Commands
```bash
# Documentation
guide              # This guide (view anytime)
keys               # Quick keybinding reference (SUPER + /)
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
thunar             # Thunar GUI file manager

# Visual Diff
verify-hypr        # Compare Hypr configs
verify-waybar      # Compare Waybar configs
verify-kitty       # Compare Kitty configs
verify-all         # Compare all configs

# Git Shortcuts
lg                 # LazyGit
gs                 # Git status
gp                 # Git push
```

---

## 🎉 What's New in Version 2.5

### 🎨 Theming & Visual Polish

- **🌐 Brave Browser Theming** - Custom Faelight Forest CSS for new tab pages
  - Forest green gradient backgrounds
  - Cyan borders and mint text
  - Lime green highlights
  - See: `~/dotfiles/brave/THEMING.md`

- **🔔 Mako Notifications** - Beautiful themed notification popups
  - Forest green backgrounds with cyan borders
  - Urgency-based colors (info/normal/critical)
  - Rounded corners and modern spacing
  - Integrated with Papirus icons

- **📁 Papirus Icons** - Sunset-colored folders matching your theme
  - Replaces default Yaru icons
  - Orange folders for tropical sunset vibes
  - High-quality icon set with 48px support

### 📚 Enhanced Documentation

- **🔍 Meld Visual Diff Guide** (`MELD_GUIDE.md`)
  - Complete workflows for visual file comparison
  - Verification aliases usage
  - Thunar integration
  - Real-world examples

- **⌨️ Keybindings Reference** (`KEYBINDINGS.md`)
  - All 100+ shortcuts organized by category
  - Quick reference tables with legends
  - Pro tips and workflow patterns
  - Access with `SUPER + /`

- **🎨 Browser Theming Guide** (`brave/THEMING.md`)
  - Stylus CSS installation
  - Color palette reference
  - Backup and restore instructions

### 🔧 New Tools & Workflows

- **Notification Controls:**
  - `SUPER + I` - Toggle Do Not Disturb mode
  - `SUPER + SHIFT + I` - Clear all notifications

- **Thunar + Meld Integration:**
  - Visual file comparison from file manager
  - Right-click → Compare with Meld
  - Perfect for config verification

- **Verification Aliases:**
```bash
  verify-hypr      # Compare Hypr configs
  verify-waybar    # Compare Waybar configs
  verify-kitty     # Compare Kitty configs
  verify-fish      # Compare Fish configs
  verify-nvim      # Compare Neovim configs
  verify-all       # Compare everything
```

- **Browser Workspace Fix:**
  - Brave now correctly opens in workspace 2
  - Silent window rules prevent focus stealing

### 🛠️ System Improvements

- **GTK Theming** - Consistent look across all apps
- **nwg-look** - GUI theme switcher for Wayland
- **Improved Documentation Structure** - All guides in `/docs`

📖 **See detailed guides:**
- Visual Diff: `~/dotfiles/docs/MELD_GUIDE.md`
- Keybindings: `~/dotfiles/docs/KEYBINDINGS.md` (or press `SUPER + /`)
- Browser Theme: `~/dotfiles/brave/THEMING.md`

---

## 🌲 System Overview

### What Makes This System Special
```
🎨 BEAUTIFUL       - Faelight Forest theme everywhere (even browser!)
📸 IMMORTAL        - 40+ BTRFS snapshots (never lose data)
🔄 SELF-BACKING    - Auto-sync to GitHub every 6 hours
🛡️ ULTRA-SECURE    - 88-line .gitignore, zero credential leaks
📦 REPRODUCIBLE    - Recreate exact system anywhere
⚡ OPTIMIZED       - 100+ productivity keybindings
📚 DOCUMENTED      - Complete guides for everything
🔔 THEMED NOTIFS   - Even popups match the forest aesthetic
🔍 VISUAL DIFFS    - Meld integration for config verification
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
- **File Managers**: 
  - Thunar (GUI, Papirus icons)
  - Yazi (TUI, themed)
- **Visual Diff**: Meld
- **Browser**: Brave (custom themed)
- **Notifications**: Mako (Faelight themed)
- **GTK Theme**: Adwaita-dark
- **Icons**: Papirus-Dark (sunset folders)
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

# File Managers
y                  # Yazi (TUI, themed)
thunar             # Thunar (GUI, Papirus icons)

# Visual Diff & Verification
compare            # meld (alias)
verify-hypr        # Compare Hypr configs
verify-waybar      # Compare Waybar configs
verify-kitty       # Compare Kitty configs
verify-fish        # Compare Fish configs
verify-nvim        # Compare Neovim configs
verify-all         # Compare all configs
```

---

## ⌨️ Hyprland Keybindings

### Essential (Learn These First!)
```
SUPER + SPACE       Launcher (Walker)
SUPER + RETURN      Terminal (Kitty)
SUPER + B           Browser (Brave)
SUPER + E           File Manager (Thunar)
SUPER + 1-5         Switch workspace
SUPER + Q           Close window
SUPER + L           Lock screen
SUPER + SHIFT + E   Exit Hyprland
SUPER + /           Keybindings Help (this list!)
```

### Window Management
```
# Focus Navigation
SUPER + H/J/K/L     Move focus (vim-style)
SUPER + Arrow Keys  Move focus (arrows)
SUPER + TAB         Cycle next window
SUPER + SHIFT + TAB Cycle previous window

# Move Windows
SUPER + SHIFT + H/J/K/L    Move window (vim-style)
SUPER + SHIFT + Arrow Keys Move window (arrows)

# Resize Windows
SUPER + CTRL + H/J/K/L     Resize (vim-style)
SUPER + CTRL + Arrow Keys  Resize (arrows)

# Window States
SUPER + F           Fullscreen toggle
SUPER + SHIFT + F   Fullscreen (no bar)
SUPER + V           Floating toggle
SUPER + Z           Pin window
SUPER + T           Toggle split
SUPER + O           Center window
```

### Workspaces
```
# Switch Workspaces
SUPER + 1-5         Go to workspace 1-5

# Move Windows
SUPER + SHIFT + 1-5 Move window to workspace 1-5
SUPER + ALT + 1-5   Move window silently

# Workspace Types
 Terminal  (WS 1)   Kitty, terminals
󰈹 Browser   (WS 2)   Brave (auto-opens here!)
󰉋 Files     (WS 3)   Thunar, Yazi
 Code      (WS 4)   Neovim, VSCode
󰖯 Default   (WS 5)   Everything else

# Navigation
SUPER + ]           Next workspace
SUPER + [           Previous workspace
SUPER + `           Last workspace (toggle)
SUPER + W           Workspace switcher
```

### Applications
```
SUPER + SPACE       Walker launcher
SUPER + RETURN      Kitty terminal
SUPER + B           Brave browser
SUPER + E           Thunar file manager
SUPER + N           Neovim
SUPER + C           VSCode
SUPER + SHIFT + Y   Yazi (TUI file manager)
```

### Screenshots
```
SUPER + S           Full screenshot
SUPER + SHIFT + S   Area selection → save
SUPER + ALT + S     Area → clipboard
SUPER + CTRL + S    Area → editor (Swappy)
```

### Notifications (NEW in 2.5!)
```
SUPER + I           Toggle Do Not Disturb
SUPER + SHIFT + I   Clear all notifications
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
SUPER + SHIFT + L   Logout
SUPER + ALT + L     Suspend
SUPER + CTRL + L    Hibernate
SUPER + ESCAPE      Power menu
SUPER + ALT + R     Reload Hyprland
SUPER + ALT + W     Restart Waybar
```

### Scratchpad
```
SUPER + M           Toggle scratchpad
SUPER + SHIFT + M   Move to scratchpad
SUPER + ALT + M     Move to scratchpad (silent)
```

### Window Groups (Tabbed)
```
SUPER + G           Toggle group (tab windows)
SUPER + TAB         Next tab
SUPER + SHIFT + G   Previous tab
```

### Help
```
SUPER + /           Open keybindings reference!
```

**📖 For complete keybinding reference, press `SUPER + /` or see:**
`~/dotfiles/docs/KEYBINDINGS.md`

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

**Current Status:** 40+ snapshots and growing! 🌲

### View Snapshots
```bash
snapshots          # List all snapshots
```

Example output:
```
 # │ Type   │ Date                            │ Description
───┼────────┼─────────────────────────────────┼────────────
35 │ single │ Mon 25 Nov 2025 09:00:00 AM CST │ timeline
36 │ single │ Mon 25 Nov 2025 10:00:00 AM CST │ Version 2.5 Complete
37 │ single │ Mon 25 Nov 2025 11:00:00 AM CST │ timeline
38 │ single │ Mon 25 Nov 2025 12:00:00 PM CST │ Browser theming added
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
snapshot "Version 2.5 complete"
```

### Snapshot Details
```bash
# View specific snapshot
sudo snapper -c root list | grep 36

# Compare two snapshots
sudo snapper -c root status 35..38

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
- **Mako notification config** (NEW!)
- **GTK 3.0/4.0 configs** (NEW!)

✅ **Browser theming:**
- Brave Stylus CSS themes
- Color palette reference
- Installation guide

✅ **System configs:**
- Snapper configuration
- Crontab schedule

✅ **Package tracking:**
- 170+ official packages
- 4+ AUR packages  
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
- Complete master guide (this file!)
- Meld visual diff guide
- Keybindings reference
- Browser theming guide
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
✅ Gitleaks pre-commit hook (prevents credential commits)
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

- 📦 **170+ official packages** (explicitly installed)
- 📦 **4+ AUR packages** 
- 📦 **950+ total packages** (with dependencies)

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
├── official.txt           # 170+ official packages
├── aur.txt               # 4+ AUR packages
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

5. **Find your working snapshot** (e.g., #36)

6. **Rollback:**
```bash
# Delete current broken @ subvolume
sudo btrfs subvolume delete /mnt/@

# Create new @ from working snapshot
sudo btrfs subvolume snapshot /mnt/.snapshots/36/snapshot /mnt/@

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

### Mako (Notifications) Issues

**Notifications not appearing:**
```bash
# Check if Mako is running
pgrep mako

# Start Mako
mako &

# Test notification
notify-send "Test" "Hello from Faelight Forest!"

# Check logs
journalctl --user -u mako -n 20

# Reload config
makoctl reload
```

**Do Not Disturb not working:**
```bash
# Check current mode
makoctl mode

# Toggle DND
makoctl mode -t do-not-disturb

# Or use keybind: SUPER + I
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

### Thunar Issues

**Thunar not opening:**
```bash
# Test Thunar directly
thunar

# Check if process exists
pgrep thunar

# Reinstall if needed
sudo pacman -S thunar
```

**Meld integration not working:**
```bash
# Test Meld directly
meld file1 file2

# Check custom actions
# Thunar → Edit → Configure custom actions

# Verify command: meld "$@"
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

### Browser Theming Issues

**Stylus theme not applying:**
```bash
# Check if Stylus is installed
# brave://extensions/

# Check theme is enabled
# Stylus → Manage → Check if enabled

# Verify URL pattern: chrome://newtab

# Hard refresh: Ctrl + Shift + R
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
makoctl mode       # Check notification mode
```

### Logs
```bash
# Auto-sync log
tail -50 ~/.auto-sync.log

# System logs
journalctl -b      # Current boot
journalctl -u fail2ban
journalctl -u mullvad-daemon
journalctl --user -u mako  # Notification logs
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
Primary:    #5bb7a5  (Bright teal/cyan)
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
- ✅ **Mako (notifications)** - NEW!
- ✅ **Brave (browser)** - NEW!
- ✅ **Thunar (GTK)** - NEW!
- ✅ **Papirus Icons** - NEW!

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

### Config Verification
```bash
# Check if configs drifted from dotfiles
verify-all

# Or check specific configs
verify-hypr
verify-waybar
verify-kitty

# Use Meld to see differences visually!
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

## 📚 Additional Guides

Your system includes comprehensive guides for every major component:

### Core Guides

- **This Guide** - `~/dotfiles/docs/COMPLETE_GUIDE.md`
  - Complete system reference
  - All commands and workflows
  - Access with: `guide`

- **Keybindings Reference** - `~/dotfiles/docs/KEYBINDINGS.md`
  - All 100+ keyboard shortcuts
  - Organized by category
  - Pro tips and patterns
  - Access with: `SUPER + /` or `keys`

- **Meld Visual Diff Guide** - `~/dotfiles/docs/MELD_GUIDE.md`
  - Visual file comparison workflows
  - Verification alias usage
  - Thunar integration
  - Real-world examples

- **Browser Theming** - `~/dotfiles/brave/THEMING.md`
  - Faelight Forest Stylus CSS
  - Color palette reference
  - Installation and backup

### Quick Reference
```bash
# View any guide
guide              # This complete guide
keys               # Keybindings (or SUPER + /)
colors             # Color palette

# Open in editor
nvim ~/dotfiles/docs/COMPLETE_GUIDE.md
nvim ~/dotfiles/docs/KEYBINDINGS.md
nvim ~/dotfiles/docs/MELD_GUIDE.md
nvim ~/dotfiles/brave/THEMING.md
```

---

## 🌲 Final Notes

### Your System is Immortal Because:

1. ✅ **40+ snapshots** - Every hour, every day, every install
2. ✅ **Auto-backup** - GitHub sync every 6 hours
3. ✅ **Full tracking** - Every package documented
4. ✅ **One-command restore** - Recreate anywhere
5. ✅ **Complete docs** - Guides for everything
6. ✅ **Security hardened** - VPN, firewall, encryption, zero leaks
7. ✅ **Beautiful theme** - Faelight Forest everywhere (even popups!)
8. ✅ **Visual verification** - Meld integration for config checking

### Remember:

- Run `guide` anytime to view this
- Run `keys` or press `SUPER + /` for keybindings
- Run `health` to check system status
- Run `audit-secrets` to check for credential leaks
- Run `verify-all` to check config drift
- Check `snapshots` regularly (peace of mind!)
- Your system backs itself up automatically ✅

---

## 📚 Additional Resources

- **Repository:** https://github.com/WidkidoneR2/dotfiles
- **Recovery Guide:** See RECOVERY.md in dotfiles
- **Package Lists:** ~/dotfiles/packages/
- **System Configs:** ~/dotfiles/system/
- **Documentation:** ~/dotfiles/docs/

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
- ❌ Config drift
- ❌ Ugly notifications

**Always have:**
- ✅ 40+ snapshots to roll back to
- ✅ GitHub backup of everything
- ✅ Complete documentation with guides
- ✅ One-command system restoration
- ✅ Zero credential leaks
- ✅ Visual config verification
- ✅ Beautiful themed everything

---

**🌲 May your Faelight Forest grow eternal! 🌲**

*Version 2.5 - The Theming & Documentation Edition*  
*Built with ❤️ by Christian*  
*November 25, 2025*

---

## 📝 Changes for Version 2.5

### Major Features Added

**🎨 Visual Enhancements:**
- Brave browser custom Faelight Forest CSS theme
- Mako notification system with forest theming
- Papirus-Dark icons with sunset-colored folders
- GTK 3.0/4.0 theming for consistent look
- nwg-look GUI theme manager

**📚 Documentation:**
- MELD_GUIDE.md - Complete visual diff workflows
- KEYBINDINGS.md - 100+ shortcuts organized by category
- brave/THEMING.md - Browser customization guide
- Updated COMPLETE_GUIDE.md with all new features

**🔧 Tools & Workflows:**
- Thunar file manager (replaced Nautilus)
- Meld visual diff tool with Thunar integration
- Verification aliases (verify-hypr, verify-waybar, etc.)
- Notification controls (SUPER + I for DND)
- Help keybind (SUPER + / opens keybindings)

**🐛 Fixes:**
- Browser workspace rules (Brave opens in workspace 2)
- Fixed Brave class name matching
- Silent window rules prevent focus stealing
- Improved configuration organization

### System Improvements

- Increased snapshot count (40+)
- Enhanced backup system
- Better documentation structure
- Improved theme consistency
- More productivity aliases
