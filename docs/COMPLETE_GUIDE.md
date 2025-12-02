# 🌲 FAELIGHT FOREST - COMPLETE MASTER GUIDE

**The Ultimate Reference for Your Immortal Arch Linux System**

**Version:** 2.7.2 - Security Hardened Edition  
**Last Updated:** November 30, 2025  
**System Status:** IMMORTAL ♾️  
**Snapshots:** 40+  
**Commits:** 80+  
**Hardening Index:** 71/100 🛡️

---

## 📋 Table of Contents

1. [Quick Start](#-quick-start)
2. [System Overview](#-system-overview)
3. [Fish Shell Reference](#-fish-shell-reference)
4. [Hyprland Keybindings](#️-hyprland-keybindings)
5. [Theme System](#-theme-system)
6. [LazyVim Reference](#-lazyvim-reference)
7. [Snapshot System](#-snapshot-system-snapper)
8. [Backup & Sync System](#-backup--sync-system)
9. [Security Features](#️-security-features)
10. [GNU Stow Management](#-gnu-stow-management)
11. [Package Management](#-package-management)
12. [Recovery Procedures](#-recovery-procedures)
13. [Troubleshooting](#-troubleshooting)

---

## 🚀 Quick Start

### Essential Commands
```bash
# Documentation
guide              # This guide (view anytime)
keys               # Quick keybinding reference
colors             # Theme color palette
health             # System health check

# Theme System (v2.7.0+)
theme-dark         # Switch to dark theme
theme-light        # Switch to light theme
theme-toggle       # Toggle between themes
# Or use: SUPER + SHIFT + P

# System Management
safe-update        # Snapshot + update system
sys-cleanup        # Clean caches and orphans
quick-note         # Daily scratchpad

# Security (v2.7.2)
security-check     # Weekly security routine
security-score     # Show hardening index
vuln-check         # High-risk vulnerabilities
audit-full         # Full Lynis security audit
jail-status        # Fail2ban status

# Snapshots
snapshots          # View all BTRFS snapshots
snapshot "desc"    # Create named snapshot

# Backups
dotfiles-sync      # Manual sync to GitHub
save-packages      # Update package lists
auto-sync          # Full automated sync

# Security Audit
audit-secrets      # Scan repo for leaked credentials

# File Management
y                  # Yazi file manager
compare            # Meld visual diff tool

# Git Shortcuts
lg                 # LazyGit
gs                 # Git status
gp                 # Git push
```

---

## 🌲 System Overview

### What Makes This System Special

- 🎨 **BEAUTIFUL** - Faelight Forest theme with dark/light modes
- 📸 **IMMORTAL** - 40+ BTRFS snapshots (never lose data)
- 🔄 **SELF-BACKING** - Auto-sync to GitHub every 6 hours
- 🛡️ **ULTRA-SECURE** - Hardening Index 71/100, zero credential leaks
- 📦 **REPRODUCIBLE** - Recreate exact system anywhere
- ⚡ **OPTIMIZED** - 100+ productivity keybindings
- 🎭 **ADAPTIVE** - Instant theme switching (dark ↔ light)
- 📚 **DOCUMENTED** - Complete guides for everything
- 🔧 **PROFESSIONAL** - GNU Stow dotfile management

### Core Components

| Component | Tool | Notes |
|-----------|------|-------|
| **OS** | Arch Linux | Rolling release |
| **Filesystem** | BTRFS | With subvolumes |
| **Window Manager** | Hyprland | Wayland compositor |
| **Status Bar** | Waybar | Theme-aware, VPN module |
| **Launcher** | Walker | Faelight themed |
| **Terminal** | Kitty | Theme-aware colors |
| **Shell** | Fish | 100+ aliases, theme-aware prompt |
| **Editor** | LazyVim | Faelight theme |
| **File Manager (GUI)** | Thunar | With Meld integration |
| **File Manager (TUI)** | Yazi | Faelight themed |
| **Visual Diff** | Meld | Config verification |
| **Password Manager** | KeePassXC | Local, encrypted |
| **Snapshots** | Snapper + snap-pac | 40+ snapshots |
| **VPN** | Mullvad | Waybar indicator |
| **Backup** | Git + Cron | Automated |
| **Security** | Lynis + arch-audit | Regular scans |
| **Dotfiles** | GNU Stow | Professional management |

### System Statistics (v2.7.2)

- **Packages:** 170+ official, 4+ AUR
- **Snapshots:** 40+ BTRFS snapshots
- **Commits:** 80+ to GitHub
- **Hardening Index:** 71/100
- **Themes:** 2 (Dark + Light)
- **Keybindings:** 100+
- **Fish Aliases:** 100+
- **Security Score:** Enterprise-grade

---

## 🐟 Fish Shell Reference

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
dots               # ~/dotfiles
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

### Theme Commands (v2.7.0+)
```bash
# Theme Switching
theme-dark         # Switch to dark theme
theme-light        # Switch to light theme
theme-toggle       # Toggle between themes

# Theme Info
colors             # Display theme palette
```

### Security Commands (v2.7.2)
```bash
# Weekly Routine
security-check     # Full security audit (update + scan + audit)

# Individual Checks
audit-full         # Full Lynis security audit
audit-quick        # Quick Lynis audit
security-score     # Show hardening index (71/100)
vuln-check         # Show high-risk vulnerabilities

# Fail2ban Monitoring
jail-status        # Check Fail2ban status
ban-list           # Show banned IPs

# Secret Scanning
audit-secrets      # Scan dotfiles for credentials
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
code               # VSCode
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

# Quick Tools
quick-note         # Daily markdown scratchpad
compare            # Meld visual diff

# Snapshots
snapshots          # List all BTRFS snapshots
snapshot "desc"    # Create named snapshot

# Backups
dotfiles-sync      # Sync configs to GitHub
save-packages      # Update package lists
auto-sync          # Full automated sync
safe-update        # Snapshot + system update

# Config Verification
verify-all         # Compare all configs with dotfiles
verify-hypr        # Verify Hyprland configs
verify-waybar      # Verify Waybar configs
verify-kitty       # Verify Kitty configs
verify-fish        # Verify Fish configs
verify-nvim        # Verify Neovim configs

# File Managers
y                  # yazi (TUI, themed)
thunar             # Thunar (GUI)
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
SUPER + /           Keybindings Help (full reference)
```

### Theme Switching (v2.7.0+)
```
SUPER + SHIFT + P   Toggle theme (dark ↔ light)
```

**What switches:**
- Hyprland colors
- Waybar (entire bar + all modules)
- Kitty terminal colors
- Mako notifications
- GTK applications
- Fish prompt colors
- Fish syntax highlighting

### Window Management
```bash
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
SUPER + M           Toggle scratchpad
```

### Workspaces
```
# Switch Workspaces
SUPER + 1-5         Go to workspace 1-5

# Move Windows
SUPER + SHIFT + 1-5 Move window to workspace 1-5

# Workspace Types
💻  Terminal  (WS 1)   Kitty, terminals
🌐 󰈹 Browser   (WS 2)   Brave (auto-opens here!)
📁 󰉋 Files     (WS 3)   Thunar, Yazi
💻  Code      (WS 4)   Neovim, VSCode
🎨 󰖯 Default   (WS 5)   Everything else
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
SUPER + S           Full screen → save
SUPER + SHIFT + S   Area selection → save
SUPER + ALT + S     Area → clipboard
SUPER + CTRL + S    Area → editor (Swappy)
```

### Notifications (Mako)
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
SUPER + SHIFT + R   Reload Hyprland config
SUPER + SHIFT + E   Exit Hyprland
```

### Special
```
SUPER + ALT + Q     Kill window (force)
SUPER + SHIFT + SPACE Toggle floating
```

---

## 🎨 Theme System

### Overview (v2.7.0+)

Faelight Forest now has **two beautiful themes** that switch your entire system instantly!

**Dark Theme:**
- Background: Deep forest green (#0f1c16)
- Accents: Bright lime, cyan, purple
- Perfect for: Night work, low-light environments

**Light Theme:**
- Background: Soft parchment (#ede8e0)
- Accents: Dark teal, bright purple, blue
- Perfect for: Daytime, bright environments

### Switch Themes
```bash
# Fish commands
theme-dark         # Switch to dark theme
theme-light        # Switch to light theme
theme-toggle       # Toggle between themes

# Or use keybinding
SUPER + SHIFT + P  # Toggle instantly!
```

### What Changes

When you switch themes, **everything** updates:

✅ **Hyprland:**
- Window borders
- Blur effects
- Background colors

✅ **Waybar:**
- Entire bar background
- All module colors
- Workspace icons (lime → purple)
- Network icons (cyan → blue)
- All bubbles and borders

✅ **Kitty Terminal:**
- Background/foreground
- All 16 ANSI colors
- Matching Fish prompt

✅ **Fish Shell:**
- Prompt colors (timestamp, folder, ❯)
- Syntax highlighting
- Autosuggestion colors
- Status indicators (✔/✗)

✅ **Mako Notifications:**
- Background colors
- Border colors
- Text colors

✅ **GTK Applications:**
- All GTK3/GTK4 apps
- System dialogs
- File choosers

### Theme Files Location
```
~/dotfiles/themes/
├── faelight-dark/
│   ├── theme.json         # Dark theme config
│   └── terminal.conf      # 16 ANSI colors
└── faelight-light/
    ├── theme.json         # Light theme config
    └── terminal.conf      # 16 ANSI colors

~/dotfiles/waybar/.config/waybar/
├── style-dark.css         # Dark Waybar theme
└── style-light.css        # Light Waybar theme
```

### Color Palettes

**Dark Theme Colors:**
| Element | Color | Hex | Usage |
|---------|-------|-----|-------|
| Background | Dark Forest | `#0f1c16` | Terminal, windows |
| Primary | Bright Teal | `#5bb7a5` | Borders, accents |
| Secondary | Soft Green | `#8ed1a3` | Text, highlights |
| Accent 1 | Lime | `#d4ff00` | Workspaces, folders |
| Accent 2 | Cyan | `#56d4dd` | WiFi, prompt |
| Accent 3 | Purple | `#7400e7` | Timestamp |
| Success | Neon Green | `#00ff00` | Status ✔ |
| Error | Neon Pink | `#ff0066` | Status ✗ |

**Light Theme Colors:**
| Element | Color | Hex | Usage |
|---------|-------|-----|-------|
| Background | Parchment | `#ede8e0` | Terminal, windows |
| Primary | Dark Teal | `#1a6b54` | Borders, accents |
| Secondary | Rich Text | `#0a150f` | Text |
| Accent 1 | Bright Purple | `#7400e7` | Workspaces, timestamp |
| Accent 2 | Bright Blue | `#0066cc` | WiFi, prompt |
| Accent 3 | Magenta | `#c7256e` | Prompt |
| Success | Neon Green | `#00ff00` | Status ✔ |
| Error | Neon Red | `#ff0000` | Status ✗ |

### Current Theme
```bash
# Check current theme
cat ~/dotfiles/themes/current.txt

# View theme colors
colors
```

---

## 📝 LazyVim Reference

### Essential Keybindings (Normal Mode)

**File Operations:**
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

**Navigation:**
```
Ctrl + h/j/k/l     Navigate splits
<leader><leader>   Find files (quick)
<leader>/          Search in current buffer
<leader>,          Switch buffers
<leader>`          Switch to last buffer
```

**Code Operations:**
```
gd                 Go to definition
gr                 Go to references
K                  Hover documentation
<leader>ca         Code actions
<leader>rn         Rename symbol
[d / ]d            Next/previous diagnostic
<leader>xx         Open trouble (diagnostics)
```

**Editing:**
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

# In Telescope:
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

---

## 📸 Snapshot System (Snapper)

### Overview

Your system creates automatic BTRFS snapshots:

- ✅ **Hourly** - Last 5 hours kept
- ✅ **Daily** - Last 7 days kept
- ✅ **Pre/Post** - Before every package install
- ✅ **Manual** - Anytime you want

**Current Status:** 40+ snapshots protecting your system! 🌲

### View Snapshots
```bash
snapshots          # List all snapshots
```

Example output:
```
 # │ Type   │ Date                            │ Description
───┼────────┼─────────────────────────────────┼────────────
38 │ single │ Sat 30 Nov 2025 09:00:00 AM CST │ timeline
39 │ pre    │ Sat 30 Nov 2025 10:30:15 AM CST │ pacman -Syu
40 │ post   │ Sat 30 Nov 2025 10:32:47 AM CST │ pacman -Syu
41 │ single │ Sat 30 Nov 2025 11:00:00 AM CST │ timeline
```

### Create Manual Snapshot
```bash
snapshot "Description of what you're doing"
```

**Examples:**
```bash
snapshot "Before major system update"
snapshot "Before installing NVIDIA drivers"
snapshot "Working configuration backup"
snapshot "Before theme customization"
```

### Snapshot Details
```bash
# View specific snapshot
sudo snapper -c root list | grep 38

# Compare two snapshots
sudo snapper -c root status 38..40

# Check disk space used by snapshots
sudo snapper -c root list | tail -10
```

### Automatic Snapshot Triggers

Snapshots are automatically created when:

- ✅ Installing packages with `pacman` or `yay`
- ✅ Every hour (timeline snapshots)
- ✅ When you run `safe-update`

### Rollback to Snapshot

See [Recovery Procedures](#-recovery-procedures) for detailed rollback instructions.

---

## 🔄 Backup & Sync System

### Auto-Sync Overview

Your system automatically backs up to GitHub:

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

✅ **All dotfiles (via GNU Stow):**
- Fish Shell configuration + functions
- Hyprland configuration (all .conf files)
- Waybar config + both theme styles
- Walker configuration + theme
- Kitty terminal config + theme files
- Yazi file manager + theme
- LazyVim configuration
- Mako notification config
- GTK theme configs

✅ **Theme files:**
- Both dark and light themes
- Terminal color schemes
- Waybar CSS files
- Theme switching script

✅ **System configs:**
- Snapper configuration
- Crontab schedule
- Security hardening configs

✅ **Package tracking:**
- 170+ official packages
- 4+ AUR packages
- Complete version list
- Installation script

✅ **Scripts:**
- safe-update
- theme-switch.sh
- save-packages
- dotfiles-sync
- auto-sync
- sys-cleanup
- quick-note
- VPN status/toggle
- Security verification scripts

✅ **Documentation:**
- This complete guide
- Keybindings reference
- Meld visual diff guide
- Browser theming guide
- Changelog
- Roadmap

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

On a new machine:
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles

# Install packages
cd ~/dotfiles/packages
./install.sh

# Install dotfiles with GNU Stow
cd ~/dotfiles
./install.sh

# Done! Exact system restored! ✅
```

---

## 🛡️ Security Features

### Security Status (v2.7.2)

**Hardening Index:** 71/100 (+3 from v2.7.1)

Your system includes enterprise-grade security:

- ✅ **LUKS2 full disk encryption**
- ✅ **UFW firewall** (DROP default policy)
- ✅ **Fail2ban** intrusion prevention (enabled)
- ✅ **Kernel hardening** (9 critical settings secured)
- ✅ **Mullvad VPN** (Waybar status indicator)
- ✅ **Vulnerability scanning** (arch-audit)
- ✅ **Security auditing** (Lynis)
- ✅ **DNS over TLS** (Cloudflare 1.1.1.1)
- ✅ **Zero credential leaks** (88-line .gitignore + Gitleaks)
- ✅ **KeePassXC** password manager

### Security Aliases (v2.7.2)
```bash
# Weekly Security Routine (Recommended!)
security-check      # Full audit: update + scan vulnerabilities + Lynis

# Individual Checks
audit-full          # Full Lynis security audit (saves score)
audit-quick         # Quick Lynis audit (faster, saves score)
security-score      # Show current hardening index (instant!)
vuln-check          # Show high-risk vulnerabilities only

# Fail2ban Monitoring
jail-status         # Check Fail2ban status
ban-list            # Show banned IPs (if SSH jail active)

# Secret Scanning
audit-secrets       # Scan dotfiles for leaked credentials
```

### Weekly Security Routine

**Every Monday morning (recommended):**
```bash
# Run the full security check
security-check
```

**What it does:**
1. Updates all packages (`pacman` + `yay`)
2. Scans for vulnerable packages (`arch-audit`)
3. Runs security audit (`lynis --quick`)
4. Shows your hardening score

**What to look for:**
- ⚠️ **High risk** vulnerabilities - monitor, wait for patches (normal on Arch)
- 🔴 **Fail2ban warnings** - investigate ban patterns
- 📊 **Score changes** - should stay 70+ or improve

### Understanding Your Security Score

**Hardening Index Scale:**
- **85-100:** Excellent (server-grade hardening)
- **70-84:** Good (strong desktop security) ← **You are here! (71)**
- **55-69:** Moderate (basic protections)
- **40-54:** Weak (improvements needed)
- **0-39:** Critical (major gaps)

**Your score breakdown (71/100):**

✅ **What's protecting you:**
- Kernel hardening (kptr_restrict, dmesg_restrict, BPF hardening)
- Filesystem protections (FIFOs, regular files, SUID)
- Network security (martian logging, redirect blocking)
- Firewall active (UFW + iptables)
- Intrusion prevention (Fail2ban enabled)
- Full disk encryption (LUKS2)
- Package vulnerability scanning (arch-audit)

⚠️ **Why not 100:**
- No MAC framework (AppArmor/SELinux) - overkill for desktop
- No file integrity monitoring (AIDE) - planned for v2.9
- No system auditing (auditd) - planned for v2.9
- Compilers present - needed for development
- Known CVEs - normal for Arch Linux rolling release

### Kernel Hardening Details

**Applied settings** (via `/etc/sysctl.d/99-hardening.conf`):
```ini
# Kernel Security
kernel.kptr_restrict = 2              # Hide kernel pointers
kernel.dmesg_restrict = 1             # Restrict kernel logs
kernel.unprivileged_bpf_disabled = 1  # Disable unprivileged BPF
kernel.sysrq = 0                      # Disable SysRq keys
fs.suid_dumpable = 0                  # Prevent core dumps
fs.protected_fifos = 2                # FIFO hardening
fs.protected_regular = 2              # Regular file hardening

# Network Security
net.ipv4.conf.all.log_martians = 1    # Log suspicious packets
net.ipv4.conf.default.log_martians = 1
net.core.bpf_jit_harden = 2           # BPF JIT hardening
```

**Apply changes:**
```bash
sudo sysctl --system
```

### VPN Status (Waybar Module)

Visual indicators in Waybar:
- 🟢 **Green** = Connected (shows location)
- 🟡 **Yellow** = Connecting (animated)
- 🔴 **Red** = Disconnected

**Click to toggle VPN on/off!**

### Security Audit Commands
```bash
# Firewall status
sudo ufw status

# VPN status
mullvad status

# Fail2ban status
sudo fail2ban-client status

# Check vulnerable packages
arch-audit

# Full security scan
sudo lynis audit system

# Check hardening score
security-score

# DNS over TLS status
resolvectl status | grep DNSOverTLS

# View system services
systemctl list-units --state=running
```

### VPN Commands
```bash
# Connect/disconnect VPN
mullvad connect
mullvad disconnect

# Check status
mullvad status

# Show account
mullvad account

# List locations
mullvad relay list
```

### Secret Protection (Gitleaks)

Automatic secret scanning prevents accidental credential commits.

**Features:**
- Pre-commit hook blocks secrets automatically
- Scans for API keys, tokens, private keys, passwords
- Custom patterns in `.gitleaks.toml`

**Commands:**
```bash
audit-secrets      # Scan current directory
scan-staged        # Check staged git files
```

**What's Protected:**
- AWS keys, GitHub tokens, API keys
- Private keys (.pem, .key files)
- Files matching `*secret*`, `*-secret*`, `*_secret*`
- SSH keys, GPG keys
- VPN credentials
- Browser data (saved passwords)

The pre-commit hook runs automatically - no action needed! ✅

---

## 📦 GNU Stow Management

### What is GNU Stow? (v2.6.0+)

GNU Stow creates symlinks from `~/dotfiles/package/.config/app/` to `~/.config/app/`.

**Benefits:**
- ✅ Clean, standard dotfile structure
- ✅ Easy version control (everything in one repo)
- ✅ Simple to add/remove configs
- ✅ No manual symlink management
- ✅ Professional dotfile organization

### Stow Structure
```
~/dotfiles/
├── fish/.config/fish/              # Fish shell
├── hypr/.config/hypr/              # Hyprland
├── waybar/.config/waybar/          # Waybar (both themes!)
├── walker/.config/walker/          # Walker launcher
├── kitty/.config/kitty/            # Kitty terminal
├── nvim/.config/nvim/              # LazyVim
├── yazi/.config/yazi/              # Yazi file manager
├── mako/.config/mako/              # Mako notifications
├── gtk/.config/gtk-{3.0,4.0}/      # GTK themes
├── scripts/                        # Not stowed
├── packages/                       # Not stowed
├── system/                         # Not stowed
├── docs/                           # Not stowed
└── themes/                         # Not stowed
```

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

### Config Verification (Meld)

Verify symlinks are working correctly:
```bash
# Compare all configs
verify-all

# Or individual components
verify-hypr        # Hyprland
verify-waybar      # Waybar
verify-kitty       # Kitty
verify-fish        # Fish
verify-nvim        # Neovim

# Visual diff tool
compare ~/.config/fish/config.fish ~/dotfiles/fish/.config/fish/config.fish
```

---

## 📦 Package Management

### Package Tracking

Your system tracks all packages for full reproducibility:

- 📦 **170+ official packages** (explicitly installed)
- 📦 **4+ AUR packages**
- 📦 **1000+ total packages** (with dependencies)

### Update Package Lists
```bash
save-packages      # Updates all package lists
```

**This creates:**
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

# Check for vulnerable packages (v2.7.2)
arch-audit
vuln-check         # Show high-risk only
```

### Package Lists Location
```
~/dotfiles/packages/
├── official.txt           # 170+ official packages
├── aur.txt               # 4+ AUR packages
├── all-with-versions.txt # All packages with versions
├── install.sh            # Installation script
└── README.md             # Documentation
```

---

## 🆘 Recovery Procedures

### Emergency Snapshot Rollback

If system breaks after update:

1. **Reboot system**
2. **Boot from GRUB** - Select snapshot from menu (grub-btrfsd)
   - Or boot from Arch USB/live system

3. **Mount BTRFS filesystem:**
```bash
   sudo mount /dev/nvme0n1p2 /mnt  # Adjust your partition
```

4. **List snapshots:**
```bash
   sudo btrfs subvolume list /mnt
```

5. **Find your working snapshot** (e.g., #39)

6. **Rollback:**
```bash
   # Delete current broken @ subvolume
   sudo btrfs subvolume delete /mnt/@
   
   # Create new @ from working snapshot
   sudo btrfs subvolume snapshot /mnt/.snapshots/39/snapshot /mnt/@
   
   # Reboot
   sudo reboot
```

7. **Your system is restored!** ✅

### Fresh System Install

Complete system recreation from scratch:

1. **Install Arch Linux with BTRFS**
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

4. **Install dotfiles with Stow:**
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

7. **Setup security (v2.7.2):**
```bash
   # Install security tools
   sudo pacman -S lynis arch-audit
   
   # Apply kernel hardening
   sudo cp system/security/99-hardening.conf /etc/sysctl.d/
   sudo sysctl --system
   
   # Configure Fail2ban (if needed)
   sudo cp system/security/jail.local /etc/fail2ban/
   sudo systemctl restart fail2ban
```

8. **Start VPN daemon:**
```bash
   sudo systemctl enable --now mullvad-daemon
```

9. **Done! Exact system restored!** 🎉

### Recover Specific Configs
```bash
# Pull latest dotfiles
cd ~/dotfiles
git pull

# Reinstall specific component with Stow
cd ~/dotfiles
stow -R fish        # Reinstall Fish
stow -R waybar      # Reinstall Waybar
stow -R hypr        # Reinstall Hyprland

# Or manually copy configs
cp ~/dotfiles/fish/.config/fish/config.fish ~/.config/fish/
cp ~/dotfiles/hypr/.config/hypr/*.conf ~/.config/hypr/
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

**Wrong theme colors after theme switch:**
```bash
# Reload Waybar
killall waybar

# Switch theme again
theme-dark   # or theme-light

# Verify correct CSS file being used
ls -lh ~/.config/waybar/style-*.css
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

### Theme System Issues

**Theme not switching completely:**
```bash
# Check current theme
cat ~/dotfiles/themes/current.txt

# Manually switch
theme-dark   # or theme-light

# Check logs
journalctl -xe | grep theme

# Verify theme files exist
ls -lh ~/dotfiles/themes/faelight-*/
```

**Terminal colors not updating:**
```bash
# Check Kitty config
cat ~/.config/kitty/current-theme.conf

# Reload Kitty
killall -SIGUSR1 kitty

# Or restart Kitty completely
killall kitty
kitty &
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

# List snapshots
snapshots
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
cd ~/dotfiles
stow -R yazi
```

### Package Issues

**Broken packages after update:**
```bash
# Rollback to snapshot before update
snapshots
# Note the pre-update snapshot number

# See Recovery Procedures for rollback

# Or fix broken packages
sudo pacman -Syu
sudo pacman -S --overwrite '*' package_name
```

### Security Issues

**Fail2ban not working:**
```bash
# Check status
sudo fail2ban-client status

# Check jails
jail-status

# Restart Fail2ban
sudo systemctl restart fail2ban

# View logs
sudo journalctl -u fail2ban -n 50
```

**Vulnerable packages showing:**
```bash
# This is NORMAL for Arch Linux!
vuln-check

# Update system
sudo pacman -Syu

# Check again (some may remain - wait for upstream patches)
vuln-check
```

### Stow Issues

**Symlinks broken:**
```bash
# Check symlinks
ls -la ~/.config/ | grep "\->"

# Reinstall with Stow
cd ~/dotfiles
stow -R fish
stow -R hypr
stow -R waybar
# etc.

# Or use install script
./install.sh
```

---

## 📊 System Status Commands

### Check Everything
```bash
health             # Complete health check
```

**Individual checks:**
```bash
snapshots          # View snapshots
crontab -l         # Check scheduled tasks
systemctl list-timers | grep snapper
git -C ~/dotfiles status
mullvad status
sudo ufw status
security-score     # Hardening index (v2.7.2)
arch-audit         # Vulnerable packages
```

### Logs
```bash
# Auto-sync log
tail -50 ~/.auto-sync.log

# System logs
journalctl -b      # Current boot
journalctl -u fail2ban
journalctl -u mullvad-daemon

# Lynis log
sudo less /var/log/lynis.log
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

**Dark Theme:**
| Type | Color | Hex | Usage |
|------|-------|-----|-------|
| Primary | Bright Teal | `#5bb7a5` | Borders, accents |
| Secondary | Soft Green | `#8ed1a3` | Text, highlights |
| Accent 1 | Lime | `#d4ff00` | Workspaces, folders |
| Accent 2 | Cyan | `#56d4dd` | WiFi, prompt |
| Accent 3 | Purple | `#7400e7` | Timestamp |
| Background | Dark Forest | `#0f1c16` | Terminal, windows |
| Success | Neon Green | `#00ff00` | Status ✔ |
| Error | Neon Pink | `#ff0066` | Status ✗ |

**Light Theme:**
| Type | Color | Hex | Usage |
|------|-------|-----|-------|
| Primary | Dark Teal | `#1a6b54` | Borders, accents |
| Secondary | Rich Text | `#0a150f` | Text |
| Accent 1 | Bright Purple | `#7400e7` | Workspaces, timestamp |
| Accent 2 | Bright Blue | `#0066cc` | WiFi, prompt |
| Accent 3 | Magenta | `#c7256e` | Prompt |
| Background | Parchment | `#ede8e0` | Terminal, windows |
| Success | Neon Green | `#00ff00` | Status ✔ |
| Error | Neon Red | `#ff0000` | Status ✗ |

### View Palette
```bash
colors
```

### Themed Applications

✅ Hyprland (borders, blur, colors)  
✅ Waybar (bar, modules, both themes)  
✅ Walker (launcher)  
✅ Kitty (terminal, 16 ANSI colors)  
✅ Fish (prompt, syntax highlighting)  
✅ LazyVim (via theme)  
✅ Yazi (file manager)  
✅ Mako (notifications)  
✅ GTK (all GTK3/GTK4 apps)  

---

## 🎯 Daily Workflow

### Morning Routine
```bash
# Check system health
health

# View snapshots (peace of mind!)
snapshots

# Check security score
security-score

# Update if needed
safe-update

# Start your day! ☕
```

### Weekly Security Check (Recommended!)

**Every Monday:**
```bash
# Run full security check
security-check

# Review results:
# - System updates
# - Vulnerable packages
# - Security score
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

- ✅ **40+ snapshots** - Every hour, every day, every install
- ✅ **Auto-backup** - GitHub sync every 6 hours
- ✅ **Full tracking** - Every package documented
- ✅ **One-command restore** - Recreate anywhere
- ✅ **Complete docs** - This guide + specialized guides
- ✅ **Security hardened** - 71/100 hardening index, VPN, firewall, encryption
- ✅ **Beautiful themes** - Dark + Light, instant switching
- ✅ **Professional management** - GNU Stow dotfiles
- ✅ **Zero leaks** - 88-line .gitignore + Gitleaks scanning

### Remember:

- Run `guide` anytime to view this
- Run `keys` for quick keybinding reference
- Run `health` to check system status
- Run `security-check` weekly (Mondays!)
- Run `audit-secrets` to check for credential leaks
- Check `snapshots` regularly (peace of mind!)
- Your system backs itself up automatically ✅
- **Theme switching is instant!** `SUPER + SHIFT + P`

### Quick Access
```bash
guide              # This guide
keys               # Keybindings
colors             # Theme colors
health             # System status
security-score     # Security status
snapshots          # Snapshot list
```

---

## 📚 Additional Resources

- **Repository:** https://github.com/WidkidoneR2/dotfiles
- **Keybindings:** `~/dotfiles/docs/KEYBINDINGS.md` or run `keys`
- **Meld Guide:** `~/dotfiles/docs/MELD_GUIDE.md`
- **Browser Theming:** `~/dotfiles/brave/THEMING.md`
- **Changelog:** `~/dotfiles/CHANGELOG.md`
- **Roadmap:** `~/dotfiles/docs/planning/ROADMAP.md`
- **Package Lists:** `~/dotfiles/packages/`
- **System Configs:** `~/dotfiles/system/`

---

## 🎊 Congratulations!

You have one of the most robust, beautiful, and reproducible Linux systems ever created.

**Your Faelight Forest stands eternal.** 🌲

### Never worry about:

- ❌ Breaking your system
- ❌ Losing configurations
- ❌ Forgetting how you set things up
- ❌ Not being able to restore
- ❌ Leaking credentials to GitHub
- ❌ Security vulnerabilities (monitored!)
- ❌ Inconsistent theming

### Always have:

- ✅ 40+ snapshots to roll back to
- ✅ GitHub backup of everything
- ✅ Complete documentation
- ✅ One-command system restoration
- ✅ Zero credential leaks
- ✅ Security monitoring (71/100)
- ✅ Beautiful, switchable themes
- ✅ Professional dotfile management

---

## 🌲 **May your Faelight Forest grow eternal!** 🌲✨

**Version 2.7.2 - Security Hardened Edition**  
**Built with ❤️ by Christian**  
**November 30, 2025**

---

### Changes for Version 2.7.2

**🔒 Security Hardening:**
- Lynis security auditing integrated
- arch-audit vulnerability scanning
- Kernel hardening (9 critical settings)
- Fail2ban jails enabled
- Security aliases and weekly routine
- Hardening Index: 71/100

**🎨 Theme System (v2.7.0-2.7.1):**
- Dark/Light theme instant switching
- Complete system-wide theming
- Terminal color schemes
- Theme-aware Fish prompt and syntax highlighting
- Dual Waybar themes

**📦 GNU Stow (v2.6.0):**
- Professional dotfile management
- Clean symlink structure
- Easy config verification
- Modular package system

**📚 Documentation:**
- Complete guide rewritten for v2.7.2
- Security section expanded
- Theme system documented
- Stow workflow explained
- Updated commands and workflows

---

**🎯 Quick Start Commands (New in v2.7.2):**
```bash
security-check     # Weekly security routine
security-score     # Show hardening index
theme-toggle       # Switch themes (or SUPER + SHIFT + P)
vuln-check         # Check vulnerabilities
guide              # This guide
```

---
