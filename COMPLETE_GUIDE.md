# 🌲 Omarchy-FAELIGHT FOREST - Complete System Guide

> **The definitive guide to understanding, using, and mastering your Omarchy system.**

This guide covers EVERYTHING - from basic concepts to advanced workflows. Whether you're learning Linux or optimizing your setup, this is your reference.

---

## 📑 Table of Contents

1. [Introduction & Philosophy](#introduction--philosophy)
2. [System Architecture](#system-architecture)
3. [Stow-Based Management](#stow-based-management)
4. [Security Hardening](#security-hardening)
5. [Core Tools Deep Dive](#core-tools-deep-dive)
6. [System Utilities](#system-utilities)
7. [Fish Shell Configuration](#fish-shell-configuration)
8. [Theme Engine](#theme-engine)
9. [Daily Workflows](#daily-workflows)
10. [Troubleshooting](#troubleshooting)
11. [Advanced Topics](#advanced-topics)

---

## 🎯 Introduction & Philosophy

### **What is Omarchy?**

Omarchy is a **minimal, native Wayland desktop environment** built on Arch Linux with Hyprland. The name combines "Omni" (all) and "Archy" (Arch), representing a complete, well-ordered system.

### **Core Philosophy: "Tight Ship"**

Every tool serves a purpose. No bloat, no redundancy, only quality native Wayland applications that work flawlessly together.

**Principles:**
1. **Native Wayland First** - No X11 compatibility layers
2. **Minimal Resource Usage** - Fast startup, low memory
3. **Quality Over Features** - Reliability trumps bells and whistles
4. **Purposeful Selection** - Every tool chosen after extensive testing
5. **Maintainability** - Clear structure, well-documented

### **What Makes This Different?**

**Not just another 0-core repo:**
- ✅ Every choice explained (WHY this tool?)
- ✅ Complete health checking (dot-doctor)
- ✅ Auto-documentation (keyscan)
- ✅ Theme generation from wallpapers
- ✅ Battle-tested for daily use

**Built for:**
- Learning Linux systematically
- Understanding how components connect
- Daily productivity without friction
- Long-term maintainability

---

## 🏗️ System Architecture

### **High-Level Overview**
```
┌─────────────────────────────────────────────────────────────┐
│                    USER INTERACTION                         │
│  Keyboard → Hyprland → Applications → Display               │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   HYPRLAND (Compositor)                     │
│  • Window management                                        │
│  • Keybindings                                              │
│  • Workspaces                                               │
│  • Animations                                               │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                        │
│  Foot (terminal) → Fish (shell) → Commands                 │
│  Fuzzel (launcher) → Apps                                   │
│  Yazi (file manager) → Files                                │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      SYSTEM LAYER                           │
│  • Wayland (display protocol)                               │
│  • Pipewire (audio)                                         │
│  • systemd (init)                                           │
│  • Linux kernel                                             │
└─────────────────────────────────────────────────────────────┘
```

### **Directory Structure Explained**
```
~/0-core/                    # Main 0-core repository
│
├── hypr/                      # Hyprland compositor config
│   └── .config/hypr/
│       ├── hyprland.conf      # Main config (sources all others)
│       ├── env.conf           # Environment variables
│       ├── monitors.conf      # Display configuration
│       ├── bindings.conf      # All keybindings (300+ lines!)
│       ├── workspaces.conf    # Workspace rules
│       ├── autostart.conf     # Programs to launch on startup
│       ├── animations.conf    # Animation settings
│       └── windowrules.conf   # Window-specific rules
│
├── foot-theme-dark/           # Primary terminal colors
│   └── .config/foot/
│       └── foot.ini           # Complete Foot configuration
│
├── ghostty-theme-dark/        # Backup terminal
│   └── .config/ghostty/
│       └── config             # Complete Ghostty configuration
│
├── fuzzel-theme-dark/         # App launcher theme
│   └── .config/fuzzel/
│       └── fuzzel.ini         # Launcher config + colors
│
├── waybar/                    # Status bar
│   └── .config/waybar/
│       ├── config             # Bar layout + modules
│       └── style.css          # Styling + colors
│
├── mako/                      # Notification daemon
│   └── .config/mako/
│       └── config             # Notification styling
│
├── yazi/                      # TUI file manager
│   └── .config/yazi/
│       ├── yazi.toml          # Main config
│       ├── keymap.toml        # Keybindings + Meld integration
│       └── theme.toml         # Colors
│
├── fish/                      # Fish shell
│   └── .config/fish/
│       ├── config.fish        # Main config + aliases
│       └── functions/         # Custom functions
│
├── nvim/                      # Neovim (LazyVim)
│   └── .config/nvim/
│       └── ...                # LazyVim configuration
│
├── scripts/                   # System utilities
│   ├── dot-doctor             # System health checker
│   ├── keyscan                # Keybinding documentation
│   ├── theme-switch           # Theme switcher
│   ├── omarchy-menu-fuzzel    # Main system menu
│   ├── power-menu-fuzzel      # Power options
│   ├── 0-core-sync          # Git sync helper
│   ├── quick-note             # Quick note taker
│   ├── safe-update            # Safe system updates
│   └── sys-cleanup            # System maintenance
│
├── pkglist.txt                # Complete package list
├── packages/                  # Detailed package info
│   ├── official.txt           # Official repo packages
│   ├── aur.txt                # AUR packages
│   ├── groups.txt             # Package groups
│   └── all-with-versions.txt  # Full list with versions
│
├── deploy.sh                  # Automated deployment script
├── README.md                  # Quick overview
├── CHANGELOG.md               # Version history
└── COMPLETE_GUIDE.md          # This file!
```

### **How Components Connect**

**Startup Flow:**
```
1. Login → Hyprland starts
2. Hyprland reads hyprland.conf
3. hyprland.conf sources all modular configs:
   - env.conf (sets up environment)
   - monitors.conf (configures displays)
   - bindings.conf (loads keybindings)
   - autostart.conf (launches apps)
   - etc.
4. Autostart launches:
   - Waybar (status bar)
   - Mako (notifications)
   - cliphist (clipboard history)
   - etc.
5. System ready!
```

**Application Launch Flow:**
```
1. Press SUPER+SPACE
2. Hyprland receives keybinding
3. Executes: fuzzel
4. Fuzzel shows app list
5. Select app → Fuzzel launches it
6. App appears in current workspace
```

**Theme Application Flow:**
```
1. Run: theme-switch dark
2. Script unstows old theme packages
3. Script stows new theme packages
4. Symlinks updated → new colors active
5. Reload apps to see changes
```

---

## 🔗 Stow-Based Management

### **What is GNU Stow?**

GNU Stow is a **symlink manager** that makes dotfile management elegant and maintainable.

**Without Stow (traditional):**
```bash
# Manual symlinks - error-prone!
ln -s ~/0-core/hypr/.config/hypr ~/.config/hypr
ln -s ~/0-core/fish/.config/fish ~/.config/fish
# ... repeat for every config
# If you make a mistake, broken configs!
```

**With Stow (elegant):**
```bash
# One command creates all symlinks!
cd ~/0-core
stow hypr fish waybar mako
# Clean, reversible, no manual linking!
```

### **How Stow Works**

**Concept:** Stow treats each directory as a "package" and creates symlinks from the package to your home directory.

**Example:**
```
Before Stow:
~/0-core/hypr/.config/hypr/hyprland.conf  (real file)
~/.config/hypr/                              (doesn't exist)

After "stow hypr":
~/0-core/hypr/.config/hypr/hyprland.conf  (real file)
~/.config/hypr/hyprland.conf                (symlink →)
```

**The symlink points back to the 0-core directory!**

### **Stow Commands Explained**
```bash
# STOW (create symlinks)
stow hypr
# Creates symlinks from ~/0-core/hypr/* to ~/

# UNSTOW (remove symlinks)
stow -D hypr
# Removes all symlinks created by this package

# RESTOW (refresh symlinks)
stow -R hypr
# Unstows then stows again (useful after config changes)

# STOW MULTIPLE
stow hypr fish waybar mako
# Stows multiple packages at once

# CHECK WHAT WOULD HAPPEN (dry-run)
stow -n hypr
# Shows what would be done without doing it

# VERBOSE (see all actions)
stow -v hypr
# Shows every symlink created
```

### **Package Structure Rules**

**Stow requires specific structure:**
```
package/          # Package directory
└── .config/      # Must mirror home directory structure!
    └── app/
        └── config
```

**When stowed, creates:**
```
~/.config/app/config → ~/0-core/package/.config/app/config
```

**Important:** The structure inside the package must match where files go in your home directory!

### **Theme Packages Strategy**

**Why separate theme packages?**
```
Base config:          Theme-specific colors:
foot/                 foot-theme-dark/
└── .config/foot/     └── .config/foot/
    └── (none!)           └── foot.ini

When stowed together, you get both!
```

**Benefits:**
1. Easy theme switching (unstow one, stow another)
2. Base settings don't need duplication
3. Clean separation of concerns
4. Theme engine can modify theme packages without touching base

**Example workflow:**
```bash
# Switch from dark to light theme
stow -D foot-theme-dark fuzzel-theme-dark
stow foot-theme-light fuzzel-theme-light

# Apps reload → new colors!
```

### **Stow Conflicts & Resolution**

**Common issues:**

**Conflict: File already exists**
```bash
stow: ERROR: existing target is neither a link nor a directory
```

**Solution:**
```bash
# Option 1: Remove the existing file
rm ~/.config/app/config
stow app

# Option 2: Adopt the existing file
stow --adopt app  # Moves existing file into package
```

**Conflict: Directory already exists**
```bash
# If ~/.config/app/ already has files
# Stow will merge (not replace)
stow app  # Works fine!
```

### **Why We Use Stow**

**Advantages:**
1. **Version Control** - All configs in one git repo
2. **Reversible** - Unstow anytime, no damage
3. **Clean** - No scattered files across system
4. **Portable** - Clone repo on new machine → stow → done!
5. **Safe** - Stow checks conflicts before acting
6. **Organized** - Each app is a separate package

**Real-world benefit:**
```bash
# New machine setup (5 minutes!)
git clone https://github.com/you/0-core ~/0-core
cd ~/0-core
stow */  # Stow everything!
# Done! All configs deployed!
```

---

## 🔐 Security Hardening

### **Security Philosophy**

**Omarchy implements enterprise-level security for a personal system:**
- Defense in depth (multiple layers)
- Principle of least privilege
- Encryption at rest
- Network-level protection
- Intrusion detection

**Threat model:** Protect against:
- Physical theft (encryption)
- Network attacks (firewall)
- Brute force (fail2ban)
- DNS manipulation (DNS over TLS)
- ISP monitoring (VPN)

### **Layer 1: Full Disk Encryption (LUKS2)**

**What:** LUKS2 encrypts your entire drive. Without the password, data is unreadable.

**Setup during installation:**
```bash
# During Arch installation
cryptsetup luksFormat /dev/sda2
cryptsetup open /dev/sda2 cryptroot
mkfs.ext4 /dev/mapper/cryptroot
```

**How it works:**
```
Boot:
1. GRUB loads
2. Asks for LUKS password
3. Unlocks encrypted partition
4. System boots normally
5. All data encrypted on disk

Shutdown:
1. System powers off
2. Drive automatically locked
3. Data unreadable without password
```

**Verify encryption:**
```bash
lsblk -f
# Look for "crypto_LUKS" type
```

**Why this matters:**
- Laptop stolen? Data is safe
- Drive removed? Unreadable
- Physical access? Still can't get data without password

**Performance impact:** ~5% (modern CPUs have hardware encryption)

### **Layer 2: Firewall (UFW)**

**What:** Uncomplicated Firewall - blocks unwanted network connections.

**Configuration:**
```bash
# Enable firewall
sudo ufw enable

# Default policy: deny all incoming
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow specific services if needed
sudo ufw allow ssh  # If you need SSH

# Check status
sudo ufw status verbose
```

**What it blocks:**
- Incoming connection attempts
- Port scans
- Unsolicited traffic

**What it allows:**
- Your outgoing connections (web, email, etc.)
- Responses to your requests

**Verify:**
```bash
sudo ufw status
# Should show: Status: active
```

### **Layer 3: Intrusion Prevention (fail2ban)**

**What:** Automatically bans IPs that show malicious behavior.

**Configuration:** `/etc/fail2ban/jail.local`
```ini
[DEFAULT]
bantime = 3600        # Ban for 1 hour
findtime = 600        # Look for failures in 10 min window
maxretry = 5          # Allow 5 tries before ban

[sshd]
enabled = true
port = ssh
logpath = /var/log/auth.log
```

**How it works:**
```
1. Someone tries to SSH into your machine
2. Fails 5 times in 10 minutes
3. fail2ban bans their IP for 1 hour
4. They can't try again during ban
```

**Check bans:**
```bash
sudo fail2ban-client status sshd
```

**Why this matters:**
- Stops brute force attacks
- Automatic protection (set and forget)
- Logs suspicious activity

### **Layer 4: DNS over TLS (Quad9)**

**What:** Encrypts DNS queries so your ISP can't see which websites you visit.

**Configuration:** `/etc/systemd/resolved.conf`
```ini
[Resolve]
DNS=9.9.9.9#dns.quad9.net
DNSOverTLS=yes
DNSSEC=yes
```

**How it works:**
```
Without DNS over TLS:
You → ISP → "What's example.com?" (ISP can see!)
ISP → "It's 93.184.216.34"

With DNS over TLS:
You → Encrypted tunnel → Quad9 → Response
ISP only sees: "Encrypted DNS query" (can't read it!)
```

**Verify:**
```bash
resolvectl status
# Should show: DNS over TLS: yes
```

**Why Quad9:**
- Blocks malicious domains
- Privacy-focused (no logging)
- Fast global network
- Supports DNS over TLS

### **Layer 5: VPN (Mullvad)**

**What:** Routes all traffic through encrypted tunnel.

**Configuration:**
```bash
# Install
sudo pacman -S mullvad-vpn

# Connect
mullvad connect

# Check status
mullvad status
```

**What it protects:**
- Your real IP address (hidden)
- All traffic encrypted
- ISP can't see what you're doing
- Bypass geo-restrictions

**Kill switch:**
```bash
# Enable kill switch (block internet if VPN drops)
mullvad lockdown-mode set on
```

**When to use:**
- Public WiFi (always!)
- Torrenting
- Privacy-sensitive activities
- Accessing region-locked content

**Why Mullvad:**
- No email/registration required
- Anonymous accounts
- WireGuard support (fast!)
- Audited & trusted

### **Security Checklist**

**Verify your security setup:**
```bash
# 1. Check encryption
lsblk -f | grep crypto

# 2. Check firewall
sudo ufw status

# 3. Check fail2ban
sudo systemctl status fail2ban

# 4. Check DNS over TLS
resolvectl status | grep "DNS over TLS"

# 5. Check VPN
mullvad status
```

**All should return positive results!**

### **Maintenance**

**Monthly:**
```bash
# Review fail2ban bans
sudo fail2ban-client status

# Check firewall logs
sudo tail /var/log/ufw.log

# Update blocked lists
sudo pacman -Syu
```

**As needed:**
```bash
# Unban an IP (if you banned yourself!)
sudo fail2ban-client set sshd unbanip <IP>

# Temporarily disable firewall
sudo ufw disable  # Re-enable after!
```

---

## 🛠️ Core Tools Deep Dive

### **🦶 Foot - Primary Terminal**

**Why Foot?**

**Speed comparison:**
- Foot: 2-3ms startup
- Kitty: 50-100ms startup
- Alacritty: 30-50ms startup

**Memory comparison:**
- Foot: 5-10MB RAM
- Kitty: 30-50MB RAM
- Alacritty: 15-25MB RAM

**Chosen because:**
1. Native Wayland (built FOR it, not ported)
2. Blazing fast (fastest terminal available)
3. Minimal memory footprint
4. Excellent font rendering
5. Simple configuration
6. Reliable & stable

**Configuration breakdown:**

`foot-theme-dark/.config/foot/foot.ini`:
```ini
[main]
font=JetBrainsMono Nerd Font:size=11  # Font with ligatures
dpi-aware=yes                          # Proper scaling
pad=8x8                                # Inner padding

[scrollback]
lines=10000                            # History buffer

[url]
launch=xdg-open ${url}                 # Click URLs to open

[mouse]
hide-when-typing=yes                   # Auto-hide cursor

[colors]
background=0f1c16                      # Faelight green
foreground=e8f5d5                      # Light text
# ... 16 ANSI colors ...

[tweak]
font-monospace-warn=no                 # Suppress warnings
```

**Usage tips:**
```bash
# Open foot
foot

# With custom title
foot --title "My Terminal"

# With custom working directory
foot --working-directory=/some/path

# Run command and keep open
foot --hold -- some-command
```

**Keybindings (in Foot):**
- `Ctrl+Shift+C` - Copy
- `Ctrl+Shift+V` - Paste
- `Ctrl+Shift++` - Increase font size
- `Ctrl+Shift+-` - Decrease font size
- `Ctrl+Shift+0` - Reset font size
- `Shift+PageUp` - Scroll up
- `Shift+PageDown` - Scroll down

**When to use Foot vs Ghostty:**
- **Foot (99% of time):** Quick commands, daily work
- **Ghostty (1% of time):** When you need advanced features

---

### **👻 Ghostty - Backup Terminal**

**Why Ghostty?**

Ghostty is the backup because it's:
1. Native Wayland
2. Modern & actively developed
3. Feature-rich (splits, tabs, etc.)
4. Good performance
5. Complements Foot well

**When to use:**
- Complex terminal layouts
- Long-running processes
- When Foot isn't enough

**Configuration:** Similar to Foot, in `ghostty-theme-dark/.config/ghostty/config`

---

### **🚀 Fuzzel - Application Launcher**

**Why Fuzzel over Rofi?**

**Problems with Rofi-Wayland:**
- Not native (ported from X11)
- Click handling issues (had to click multiple times!)
- Focus stealing bugs
- Slower response

**Fuzzel benefits:**
1. Native Wayland (built for it)
2. Instant response (no lag)
3. Clean, minimal interface
4. No click issues
5. Smaller (~300KB vs 1MB)

**Configuration explained:**

`fuzzel-theme-dark/.config/fuzzel/fuzzel.ini`:
```ini
[main]
terminal=foot                    # Launch terminal apps in Foot
layer=overlay                    # Always on top
width=50                         # Window width (characters)
lines=8                          # Show 8 results
line-height=28                   # Spacing between items
font=Hack Nerd Font Mono:size=12 # Big readable text
horizontal-pad=24                # Side padding
vertical-pad=16                  # Top/bottom padding
inner-pad=12                     # Padding around text

[colors]
background=0f1c16ee              # Dark green + transparency
text=e8f5d5ff                    # Light text
match=7fb069ff                   # Green for matched letters
selection=7fb069ff               # Green selection
selection-text=0f1c16ff          # Dark text on selection
border=7fb069ff                  # Green border

[border]
width=2                          # 2px border
radius=12                        # Rounded corners

[dmenu]
exit-immediately-if-empty=yes    # Close if no results
```

**Usage:**
```bash
# Launch apps
fuzzel

# As dmenu replacement
echo -e "Option 1\nOption 2" | fuzzel --dmenu

# With custom prompt
fuzzel --prompt="Select: "
```

**How it works:**
1. Press SUPER+SPACE
2. Fuzzel scans .desktop files
3. Shows matching apps as you type
4. Arrow keys to navigate
5. Enter to launch

**Customization:**
- Width, height, colors all in fuzzel.ini
- Theme engine can generate configs
- Icons from icon theme (Papirus-Dark)

---

### **📁 Yazi - Primary File Manager**

**Why Yazi?**

**Comparison:**
| Feature | Yazi | Thunar | PCManFM |
|---------|------|--------|---------|
| Interface | TUI | GUI | GUI |
| Speed | Instant | Fast | Fast |
| Memory | ~5MB | ~30MB | ~10MB |
| Keyboard | Full | Partial | Partial |
| Customizable | Extensive | Limited | Limited |
| Dependencies | Minimal | Many | Some |

**Chosen because:**
1. Blazing fast (TUI, no GUI overhead)
2. Powerful (vim-style, highly customizable)
3. Keyboard-driven (never need mouse!)
4. Minimal resource usage
5. Modern (written in Rust)
6. Extensible (plugins, scripts)

**Configuration breakdown:**

**yazi.toml** - Main settings
**keymap.toml** - Keybindings (including Meld integration!)
**theme.toml** - Colors

**Key features configured:**

**1. Vim-style navigation:**
- `h/j/k/l` - Move left/down/up/right
- `gg` - Top of list
- `G` - Bottom of list
- `/` - Search
- `n` - Next search result

**2. File operations:**
- `Space` - Select file
- `a` - Select all
- `y` - Copy (yank)
- `d` - Cut
- `p` - Paste
- `D` - Delete

**3. Meld integration (custom):**
```toml
# In keymap.toml
[[manager.prepend_keymap]]
on = [ "SUPER ALT+c", "m" ]
run = 'shell "meld $@" --block --confirm'
desc = "Compare with Meld"
```

**Usage:**
1. Select 2 files (Space)
2. Press SUPER ALT+cm
3. Meld opens with diff!

**Why this is powerful:**
- No GUI file manager needed
- Professional diff tool available
- All keyboard-driven
- Zero bloat!

**Daily workflow:**
```bash
# Open yazi
yazi

# Navigate to directory
# (use hjkl or arrow keys)

# Preview files
# (automatic in right pane)

# Search
/filename

# Copy files
Space (mark)
Space (mark another)
y (copy)

# Navigate to destination
# (hjkl)
p (paste)

# Compare files
Space (mark 2 files)
SUPER ALT+cm (opens Meld)
```

---

### **🖥️ PCManFM-Qt - GUI File Manager**

**Why PCManFM-Qt?**

**When GUI is needed:**
- Learning file system paths
- Visual browsing
- Drag-and-drop operations
- Showing others your files

**Why PCManFM-Qt specifically:**
1. Lightweight (3MB vs Thunar's 20MB!)
2. Qt-based (lighter than GTK)
3. Fast & responsive
4. Works great on Wayland
5. Minimal dependencies

**When to use:**
- Yazi (99% of time) - Daily work
- PCManFM-Qt (1% of time) - When visual is needed

**Launch:**
```bash
pcmanfm-qt
```

**Or via keybinding:** `SUPER+F` (if configured)

---

### **📋 Cliphist - Clipboard History**

**Why essential:**

Ever copy something, then copy something else, and lose the first thing? **Never again!**

**How it works:**
```
1. Autostart: wl-paste watches clipboard
2. You copy text/image → Cliphist stores it
3. Press SUPER+V → See history
4. Select item → Copied to clipboard again!
```

**Configuration:**

In `hypr/.config/hypr/autostart.conf`:
```bash
# Clipboard history
exec-once = wl-paste --type text --watch cliphist store
exec-once = wl-paste --type image --watch cliphist store
```

In `hypr/.config/hypr/bindings.conf`:
```bash
bind = SUPER, V, exec, cliphist list | fuzzel --dmenu | cliphist decode | wl-copy
```

**Usage:**
1. Copy stuff (Ctrl+C, etc.) - automatically saved
2. Press SUPER+V
3. Fuzzel shows history
4. Select what you want
5. It's copied!

**Database location:** `~/.local/share/cliphist/db`

**Clear history:**
```bash
rm ~/.local/share/cliphist/db
```

---

### **🎨 Hyprpicker - Color Picker**

**What it does:**
- Click any pixel on screen
- Color copied to clipboard (hex format)
- Perfect for theming!

**Usage:**
```bash
# Press SUPER SHIFT+C
# Or run manually:
hyprpicker -a  # -a = auto-copy to clipboard

# Click a color
# Result: #7fb069 copied!
```

**Why useful:**
- Extract colors from wallpapers
- Match colors across apps
- Design consistent themes
- Quick color reference

---

### **📄 Zathura - PDF Viewer**

**Why Zathura over Evince?**

| Feature | Zathura | Evince |
|---------|---------|--------|
| Size | 5MB | 40MB |
| Interface | Minimal | Full GUI |
| Keybindings | Vim-style | Click-heavy |
| Speed | Instant | Slower |
| Dependencies | Few | GNOME stack |

**Chosen because:**
1. Minimal (fits tight ship philosophy)
2. Keyboard-driven (vim keybindings!)
3. Fast PDF rendering
4. Lightweight
5. Customizable

**Key bindings:**
- `j/k` - Scroll down/up
- `h/l` - Previous/next page
- `gg` - First page
- `G` - Last page
- `+/-` - Zoom in/out
- `=` - Reset zoom
- `r` - Rotate
- `i` - Invert colors (dark mode!)
- `/` - Search
- `q` - Quit

**Default PDF viewer:**
```bash
xdg-mime default org.pwmt.zathura.desktop application/pdf
```

**Now all PDFs open in Zathura!**

---

## 🔧 System Utilities

### **🏥 dot-doctor - System Health Checker**

**What it does:**
Validates EVERYTHING in your system - like a comprehensive health check.

**Checks performed:**

**1. Package Installation:**
```bash
# Reads pkglist.txt
# Checks each package installed
# Reports missing packages
```

**2. Stow Deployments:**
```bash
# Checks if configs are stowed
# Verifies symlinks exist
# Detects broken links
```

**3. Config Files:**
```bash
# Validates critical configs exist
# Checks file permissions
# Ensures executables are executable
```

**4. Scripts:**
```bash
# Checks all scripts in scripts/
# Verifies execute permissions
# Tests if they're in PATH
```

**5. Services:**
```bash
# Checks systemd services
# Verifies enabled/running state
```

**Usage:**
```bash
dot-doctor

# Output example:
🏥 System Health Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Packages (156/156 installed)
✅ Stow packages (12/12 deployed)
⚠️  Missing script: ~/.local/bin/theme-switch
   Fix: cp ~/0-core/scripts/theme-switch ~/.local/bin/

✅ Hyprland config valid
✅ All services running

Overall: 95% healthy
```

**How it works:**

1. **Loads pkglist.txt**
2. **Queries pacman** for each package
3. **Checks symlinks** in ~/.config
4. **Validates files** exist
5. **Reports** with colored output
6. **Suggests fixes** for issues

**Why this is essential:**

After making changes, run dot-doctor to ensure nothing broke!

**Example workflow:**
```bash
# Make changes
nvim ~/0-core/hypr/.config/hypr/bindings.conf

# Restow
cd ~/0-core
stow -R hypr

# Validate
dot-doctor

# If issues, fix them!
```

**Source code:** `~/0-core/scripts/dot-doctor`

---

### **🔍 keyscan - Keybinding Documentation**

**What it does:**
Auto-generates documentation of ALL your keybindings!

**How it works:**
```
1. Reads hypr/.config/hypr/bindings.conf
2. Parses all bind statements
3. Extracts:
   - Modifier keys (SUPER, ALT, SHIFT, etc.)
   - Key pressed
   - Action performed
   - Description (if present)
4. Formats as clean markdown
5. Outputs to terminal or file
```

**Usage:**
```bash
# Print to terminal
keyscan

# Save to file
keyscan > keybindings.md

# View in less
keyscan | less
```

**Output example:**
```markdown
# Hyprland Keybindings

## System
- `SUPER+Q` - Close active window
- `SUPER+M` - Exit Hyprland
- `SUPER+ESCAPE` - Power menu

## Applications
- `SUPER+RETURN` - Terminal (Foot)
- `SUPER+SPACE` - App launcher (Fuzzel)
- `SUPER+E` - File manager (Yazi)

## Workspaces
- `SUPER+1` - Switch to workspace 1
- `SUPER+2` - Switch to workspace 2
...
```

**Why this is powerful:**

**Before keyscan:**
- Forget keybindings? → Dig through bindings.conf
- New user? → No reference available
- Changed bindings? → Manual documentation update

**With keyscan:**
- Run keyscan → Instant reference!
- Always up-to-date (reads live config)
- Share with others (markdown output)
- Print for desk reference

**Learning workflow:**
```bash
# Generate keybindings reference
keyscan > ~/keybindings.md

# Open in your editor
nvim ~/keybindings.md

# Or print it
keyscan | lpr
```

**Source code:** `~/0-core/scripts/keyscan`

---

### **🔄 topgrade - System Updater**

**What is Topgrade?**

**ONE command to update EVERYTHING:**
- Arch packages (pacman)
- AUR packages (yay/paru)
- Flatpaks
- Rust binaries (cargo)
- Node packages (npm)
- System firmware
- And more!

**Why better than manual updates:**

**Manual way (tedious):**
```bash
sudo pacman -Syu      # Update official
yay -Syu              # Update AUR
flatpak update        # Update flatpak
cargo install-update -a  # Update Rust
npm update -g         # Update npm
# ... etc for every package manager!
```

**Topgrade way (simple):**
```bash
topgrade
# Done! Everything updated!
```

**How it works:**
```
1. Detects installed package managers
2. Runs update for each:
   - pacman -Syu
   - yay -Syu
   - flatpak update
   - cargo install-update
   - npm update
   - etc.
3. Shows combined progress
4. Reports what was updated
5. Asks for confirmation before breaking changes
```

**Configuration:** `~/.config/topgrade.toml`
```toml
[misc]
# Don't ask for confirmation (careful!)
assume_yes = false

# Cleanup after updates
cleanup = true

# Show commands being run
verbose = true

[linux]
# Update arch packages
arch_package_manager = "yay"

# Update AUR
yay_arguments = "--devel"

[git]
# Update git repos in specific directories
repos = [
    "~/0-core",
    "~/projects/*",
]
```

**Usage:**
```bash
# Update everything
topgrade

# Update specific things only
topgrade --only pacman
topgrade --only aur

# Dry run (see what would update)
topgrade --dry-run

# Skip confirmations (be careful!)
topgrade --yes
```

**Via Omarchy menu:**
```
Press SUPER ALT+SPACE
Select "🔄 Update System"
Topgrade runs in terminal
```

**Best practices:**

**Weekly:**
```bash
topgrade  # Keep system current
```

**Before major work:**
```bash
topgrade  # Ensure latest versions
```

**After fresh install:**
```bash
topgrade  # Update everything first
```

**Why this matters:**
- Security updates applied
- Bug fixes installed
- New features available
- ONE command (no forgetting updates!)

---

### **🎨 theme-switch - Theme Switcher**

**What it does:**
Instantly switch between dark/light themes system-wide!

**How it works:**
```bash
theme-switch dark

# Internally:
1. Unstows old theme packages:
   - stow -D foot-theme-light
   - stow -D fuzzel-theme-light
   - stow -D ghostty-theme-light

2. Stows new theme packages:
   - stow foot-theme-dark
   - stow fuzzel-theme-dark
   - stow ghostty-theme-dark

3. Updates Hyprland:
   - Changes border colors
   - Reloads config

4. Updates GTK:
   - Changes GTK theme
   - Updates icon theme

5. Reloads apps:
   - Sends reload signal to Waybar
   - Restarts Mako
```

**Configuration:** `~/0-core/scripts/theme-switch`

**Source code walkthrough:**
```bash
#!/usr/bin/env bash
# theme-switch - System-wide theme switcher

# Color codes for output
GREEN='\033[0;32m'
NC='\033[0m'

# Get theme argument
theme=$1

# Validate
if [[ "$theme" != "dark" && "$theme" != "light" ]]; then
    echo "Usage: theme-switch {dark|light}"
    exit 1
fi

# Apply theme function
apply_theme() {
    local theme=$1
    
    # Unstow old themes
    stow -D foot-theme-dark foot-theme-light 2>/dev/null || true
    stow -D fuzzel-theme-dark fuzzel-theme-light 2>/dev/null || true
    stow -D ghostty-theme-dark ghostty-theme-light 2>/dev/null || true
    
    # Stow new theme
    stow foot-theme-$theme 2>/dev/null || true
    stow fuzzel-theme-$theme 2>/dev/null || true
    stow ghostty-theme-$theme 2>/dev/null || true
    
    # Update Hyprland borders
    if [[ "$theme" == "dark" ]]; then
        hyprctl keyword general:col.active_border "rgb(7fb069)"
    else
        hyprctl keyword general:col.active_border "rgb(4a90e2)"
    fi
    
    # Update GTK theme
    gsettings set org.gnome.desktop.interface gtk-theme "Adwaita-$theme"
    
    # Reload Waybar
    killall waybar && waybar &
    
    # Restart Mako
    killall mako && mako &
}

# Execute
echo -e "${GREEN}🌲 Switching to Faelight Forest $theme...${NC}"
cd ~/0-core
apply_theme $theme
echo -e "${GREEN}✅ Theme switched to $theme!${NC}"
```

**Usage examples:**
```bash
# Switch to dark theme
theme-switch dark

# Switch to light theme
theme-switch light

# Via Omarchy menu
SUPER ALT+SPACE → Theme Menu → Dark/Light Theme
```

**What gets themed:**
- ✅ Foot terminal colors
- ✅ Ghostty terminal colors
- ✅ Fuzzel launcher colors
- ✅ Hyprland borders
- ✅ Waybar (status bar)
- ✅ Mako notifications
- ✅ GTK apps
- ✅ (Future: More apps via theme engine!)

**Why this is great:**
- ONE command changes everything
- Consistent look across all apps
- Easy to try both themes
- Perfect for day/night preferences

---

---

## 🐚 Fish Shell Configuration

### **Why Fish over Bash/Zsh?**

**Fish Philosophy:** "Friendly Interactive Shell"

**Advantages over Bash:**
- Out-of-the-box features (no plugins needed!)
- Autosuggestions (from history)
- Syntax highlighting (see errors before running!)
- Better tab completion
- Simpler scripting syntax
- Web-based configuration

**Comparison:**

| Feature | Fish | Bash | Zsh |
|---------|------|------|-----|
| Autosuggestions | ✅ Built-in | ❌ Need plugin | ❌ Need plugin |
| Syntax highlight | ✅ Built-in | ❌ Need plugin | ❌ Need plugin |
| Config | Simple | Complex | Very complex |
| Speed | Fast | Fast | Slower |
| POSIX compliant | ❌ No | ✅ Yes | ✅ Yes |

**Trade-off:** Fish is NOT POSIX compliant (some bash scripts won't work)

**Solution:** Use bash for scripts, fish for interactive use!

### **Configuration Structure**

**Main config:** `fish/.config/fish/config.fish`
```
fish/
└── .config/fish/
    ├── config.fish           # Main configuration
    ├── fish_variables        # Fish internal state
    └── functions/            # Custom functions (if any)
        └── *.fish
```

### **config.fish Breakdown**

**Full file with explanations:**
```fish
# ═══════════════════════════════════════════════════════════
# 🐠 FISH SHELL CONFIGURATION
# Location: ~/.config/fish/config.fish
# Purpose: Main shell configuration with aliases and functions
# ═══════════════════════════════════════════════════════════

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 1: ENVIRONMENT VARIABLES                        │
# └─────────────────────────────────────────────────────────┘

# Default editor (for git, cron, etc.)
set -gx EDITOR nvim

# Default visual editor
set -gx VISUAL nvim

# XDG Base Directory specification
# Keeps configs organized in ~/.config, data in ~/.local/share
set -gx XDG_CONFIG_HOME $HOME/.config
set -gx XDG_DATA_HOME $HOME/.local/share
set -gx XDG_CACHE_HOME $HOME/.cache

# Add custom scripts to PATH
# This makes all scripts in ~/.local/bin available system-wide
fish_add_path $HOME/.local/bin

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 2: ALIASES - SYSTEM INFORMATION                │
# └─────────────────────────────────────────────────────────┘

# Quick system info
alias sysinfo='inxi -Fxxxz'          # Full system information
alias temp='sensors'                  # CPU/GPU temperatures
alias ports='sudo netstat -tulanp'    # Open ports

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 3: ALIASES - FILE OPERATIONS                   │
# └─────────────────────────────────────────────────────────┘

# Modern replacements for traditional commands
alias ls='exa --icons --group-directories-first'
# exa: Modern ls replacement with colors and icons
# --icons: Show file type icons
# --group-directories-first: Directories at top

alias ll='exa -la --icons --group-directories-first'
# -l: Long format (permissions, size, date)
# -a: Show hidden files

alias tree='exa --tree --icons'
# --tree: Show directory structure as tree

alias cat='bat --style=plain'
# bat: Modern cat with syntax highlighting
# --style=plain: No line numbers or git markers

alias catn='bat --style=numbers'
# Same as cat but with line numbers

# File finding
alias fd='fd --hidden --exclude .git'
# fd: Modern find replacement
# --hidden: Search hidden files
# --exclude .git: Skip .git directories

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 4: ALIASES - NAVIGATION                        │
# └─────────────────────────────────────────────────────────┘

# Quick directory jumps
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias .....='cd ../../../..'

# Recent directories
alias z='zoxide'              # Jump to frecent directories
alias zi='zoxide query -i'    # Interactive directory selection

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 5: ALIASES - PACKAGE MANAGEMENT                │
# └─────────────────────────────────────────────────────────┘

# Pacman shortcuts
alias pacs='sudo pacman -S'           # Install package
alias pacr='sudo pacman -R'           # Remove package
alias pacu='sudo pacman -Syu'         # Update system
alias pacq='pacman -Qq'               # List installed packages
alias pacss='pacman -Ss'              # Search packages

# AUR helper (yay)
alias yays='yay -S'                   # Install AUR package
alias yayr='yay -R'                   # Remove AUR package
alias yayu='yay -Syu'                 # Update AUR + official

# System maintenance
alias cleanup='sudo pacman -Rns (pacman -Qtdq)'  # Remove orphans
alias mirror='sudo reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist'

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 6: ALIASES - GIT                               │
# └─────────────────────────────────────────────────────────┘

# Git shortcuts
alias g='git'
alias gs='git status'
alias ga='git add'
alias gaa='git add --all'
alias gc='git commit -m'
alias gp='git push'
alias gpl='git pull'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
alias gl='git log --oneline --graph --decorate'
alias glog='git log --oneline --graph --all --decorate'

# Dotfiles management
alias dots='cd ~/0-core'
alias dotstat='cd ~/0-core && git status'
alias dotsync='cd ~/0-core && git pull && git add -A && git commit -m "sync" && git push'

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 7: ALIASES - DOCKER                            │
# └─────────────────────────────────────────────────────────┘

alias d='docker'
alias dc='docker-compose'
alias dps='docker ps'
alias di='docker images'
alias dprune='docker system prune -a'    # Clean up Docker

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 8: ALIASES - SYSTEM UTILITIES                  │
# └─────────────────────────────────────────────────────────┘

# Process management
alias psg='ps aux | grep -v grep | grep -i -e VSZ -e'  # Search processes
alias kill9='kill -9'                     # Force kill

# Disk usage
alias du='dust'                           # Modern du replacement
alias df='duf'                            # Modern df replacement

# Network
alias myip='curl ifconfig.me'             # Public IP
alias localip='ip addr show'              # Local IP

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 9: FUNCTIONS - ADVANCED                        │
# └─────────────────────────────────────────────────────────┘

# Create directory and cd into it
function mkcd
    mkdir -p $argv
    and cd $argv
end

# Extract archives (any format)
function extract
    if test -f $argv[1]
        switch $argv[1]
            case '*.tar.bz2'
                tar xjf $argv[1]
            case '*.tar.gz'
                tar xzf $argv[1]
            case '*.bz2'
                bunzip2 $argv[1]
            case '*.rar'
                unrar x $argv[1]
            case '*.gz'
                gunzip $argv[1]
            case '*.tar'
                tar xf $argv[1]
            case '*.tbz2'
                tar xjf $argv[1]
            case '*.tgz'
                tar xzf $argv[1]
            case '*.zip'
                unzip $argv[1]
            case '*.Z'
                uncompress $argv[1]
            case '*.7z'
                7z x $argv[1]
            case '*'
                echo "'$argv[1]' cannot be extracted"
        end
    else
        echo "'$argv[1]' is not a valid file"
    end
end

# Backup file with timestamp
function backup
    cp $argv[1] $argv[1].backup-(date +%Y%m%d-%H%M%S)
end

# Quick note taking
function note
    echo (date +"%Y-%m-%d %H:%M:%S") - $argv >> ~/notes.txt
end

# Search command history
function hg
    history | grep $argv
end

# PCManFM in background (GUI file manager)
function pcmanfm
    command pcmanfm-qt $argv &>/dev/null &
end

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 10: TOOL INITIALIZATION                        │
# └─────────────────────────────────────────────────────────┘

# Zoxide (smart cd)
# Tracks your most used directories
# Usage: z <partial_name>
if type -q zoxide
    zoxide init fish | source
end

# Starship prompt (if installed)
# Modern, fast, customizable prompt
if type -q starship
    starship init fish | source
end

# ┌─────────────────────────────────────────────────────────┐
# │ SECTION 11: FISH-SPECIFIC SETTINGS                     │
# └─────────────────────────────────────────────────────────┘

# Disable greeting message
set fish_greeting

# Vi keybindings (optional - uncomment if you want)
# fish_vi_key_bindings

# Enable colored man pages
set -gx LESS_TERMCAP_mb \e'[1;32m'     # Begin blink
set -gx LESS_TERMCAP_md \e'[1;32m'     # Begin bold
set -gx LESS_TERMCAP_me \e'[0m'        # End mode
set -gx LESS_TERMCAP_se \e'[0m'        # End standout-mode
set -gx LESS_TERMCAP_so \e'[1;33m'     # Begin standout-mode
set -gx LESS_TERMCAP_ue \e'[0m'        # End underline
set -gx LESS_TERMCAP_us \e'[1;4;31m'   # Begin underline

# ═══════════════════════════════════════════════════════════
# END OF FISH CONFIGURATION
# 
# To reload this config: source ~/.config/fish/config.fish
# To edit: nvim ~/.config/fish/config.fish
# ═══════════════════════════════════════════════════════════
```

### **Understanding Fish Syntax**

**Variables:**
```fish
# Set local variable
set my_var "value"

# Set global variable (available to child processes)
set -gx MY_VAR "value"

# Use variable
echo $my_var
```

**Functions:**
```fish
# Define function
function my_function
    echo "Hello $argv[1]"
end

# Call it
my_function World  # Outputs: Hello World
```

**Conditionals:**
```fish
# If statement
if test -f file.txt
    echo "File exists"
else
    echo "File doesn't exist"
end

# Shorter syntax
test -f file.txt; and echo "Exists"
```

**Loops:**
```fish
# For loop
for file in *.txt
    echo $file
end

# While loop
set count 1
while test $count -le 5
    echo $count
    set count (math $count + 1)
end
```

### **Key Aliases Explained**

**Modern CLI Tools:**

**exa vs ls:**
```bash
# Traditional ls
ls -la
# Output: Plain text, hard to read

# Modern exa
exa -la --icons
# Output: Colored, icons, readable sizes
```

**bat vs cat:**
```bash
# Traditional cat
cat file.py
# Output: Plain text

# Modern bat
bat file.py
# Output: Syntax highlighting, line numbers, git markers
```

**fd vs find:**
```bash
# Traditional find
find . -name "*.txt"
# Complex syntax, slow

# Modern fd
fd txt
# Simple syntax, blazing fast, ignores .git automatically
```

**Why these tools matter:**
- Faster (written in Rust)
- Better defaults (colors, icons)
- Easier to use (simpler syntax)
- More intuitive (readable output)

### **Custom Functions Explained**

**mkcd - Make and Enter Directory:**
```fish
function mkcd
    mkdir -p $argv
    and cd $argv
end

# Usage:
mkcd ~/projects/new-project
# Creates directory AND enters it!
```

**extract - Universal Archive Extractor:**
```fish
# Instead of remembering:
tar xzf file.tar.gz
unzip file.zip
7z x file.7z

# Just use:
extract file.tar.gz
extract file.zip
extract file.7z
# Works with any format!
```

**backup - Timestamped Backups:**
```fish
# Instead of:
cp important.conf important.conf.backup

# Use:
backup important.conf
# Creates: important.conf.backup-20241209-153045
```

**note - Quick Note Taking:**
```fish
# Quick thoughts
note "Remember to buy milk"
note "Bug in line 42 of script"

# View notes
cat ~/notes.txt
# Output:
# 2024-12-09 15:30:45 - Remember to buy milk
# 2024-12-09 15:31:12 - Bug in line 42 of script
```

### **Zoxide - Smart Directory Jumping**

**What it does:**
Tracks your most-used directories and lets you jump to them with partial names!

**Traditional navigation:**
```bash
cd ~/projects/work/client/website/src/components
# So much typing!
```

**With zoxide:**
```bash
z components
# Jumps to most frequently used directory matching "components"!
```

**How it works:**
1. Tracks every `cd` you do
2. Builds frequency database
3. When you `z <query>`, jumps to best match

**Usage examples:**
```fish
# Jump to most used directory matching "dots"
z dots
# → ~/0-core

# Interactive selection (multiple matches)
zi proj
# Shows menu:
# 1. ~/projects/work
# 2. ~/projects/personal
# Select number

# Jump up directories
z ..    # Same as cd ..
z ...   # Same as cd ../..
```

**Why this is game-changing:**
- No more long paths!
- Muscle memory for frecent directories
- Saves SO much time daily

### **Fish Tips & Tricks**

**Autosuggestions:**
```
Type: cd ~/proj
Fish suggests: cd ~/projects/personal/website

Press → (right arrow) to accept suggestion!
```

**Tab Completion:**
```
Type: git ch
Press Tab → Shows: checkout, cherry, cherry-pick
Type: o → Completes to: checkout
```

**History Search:**
```
Type: git commit
Press ↑ (up arrow) → Cycles through git commit commands in history
```

**Command Substitution:**
```fish
# Bash way:
echo "Today is $(date)"

# Fish way:
echo "Today is "(date)
```

**Piping:**
```fish
# Works like bash
cat file.txt | grep pattern | sort | uniq
```

### **Customization**

**Change prompt:**
```fish
# Option 1: Use Starship (recommended)
# Already configured in config.fish if installed

# Option 2: Custom Fish prompt
function fish_prompt
    echo (set_color cyan)(prompt_pwd)(set_color normal) '❯ '
end
```

**Add your own aliases:**
```fish
# Edit config
nvim ~/.config/fish/config.fish

# Add alias
alias myalias='my command'

# Reload config
source ~/.config/fish/config.fish
```

**Persistent variables:**
```fish
# Set universal variable (persists across sessions)
set -U my_var "value"

# View all universal variables
set -U
```

---

## 🎨 Theme Engine

### **What is the Faelight Theme Engine?**

**Problem:**
- Want to theme your system from a wallpaper
- Manually extracting colors is tedious
- Manually updating configs is error-prone
- Hard to keep everything coordinated

**Solution:**
Automated pipeline that extracts colors from images and generates complete, validated theme configs for all your applications!

**Location:** `~/faelight-theme-engine/`

### **Architecture Overview**
```
┌──────────────────────────────────────────────────────────────┐
│                        INPUT                                 │
│                   wallpaper.jpg                              │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                  COLOR EXTRACTION                            │
│  • Uses colorgram.py                                         │
│  • Extracts 8 dominant colors                                │
│  • Detects dark/light theme automatically                    │
│  • Calculates color brightness                               │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                 PALETTE GENERATION                           │
│  • Maps colors to ANSI codes (0-15)                          │
│  • Hue-based intelligent assignment                          │
│  • Creates bright variants (+30% brightness)                 │
│  • Selects optimal accent color                              │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                CONTRAST VALIDATION                           │
│  • WCAG AAA standards (7:1 for text)                         │
│  • WCAG AA standards (4.5:1 for accents)                     │
│  • Auto-adjusts until standards met                          │
│  • Gamma correction for accuracy                             │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│               TEMPLATE RENDERING                             │
│  • Jinja2 templates for each app                             │
│  • Generates complete configs                                │
│  • Foot, Ghostty, Fuzzel, Waybar                             │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                DOTFILES INTEGRATION                          │
│  • Applies to theme packages                                 │
│  • Preserves base configurations                             │
│  • Ready to stow and use!                                    │
└──────────────────────────────────────────────────────────────┘
```

### **Module Breakdown**

**1. extract.py - Color Extraction**

**What it does:**
- Takes image as input
- Extracts dominant colors using colorgram.py
- Analyzes color properties (hue, saturation, brightness)
- Determines if image is dark or light themed

**Key functions:**
```python
extract_colors(image_path, num_colors=8)
# Returns: List of (r, g, b, proportion) tuples

detect_theme(colors)
# Returns: "dark" or "light" based on brightness
# Threshold: <128 = dark, ≥128 = light

calculate_brightness(r, g, b)
# Uses weighted formula: 0.299*R + 0.587*G + 0.114*B
# This matches human perception of brightness
```

**Example:**
```bash
faelight-theme extract ~/wallpapers/forest.jpg --output colors.json

# Output (colors.json):
{
  "colors": [
    {"r": 15, "g": 28, "h": 22, "proportion": 0.518},
    {"r": 127, "g": 176, "b": 105, "proportion": 0.245},
    ...
  ],
  "theme": "dark"
}
```

**2. palette.py - ANSI Color Generation**

**What it does:**
- Takes extracted colors
- Maps to 16 ANSI color codes
- Creates harmonious palette
- Generates bright variants

**Color mapping strategy:**
```
Hue-based assignment:
- Red (0°):     Finds closest red-ish color
- Yellow (60°): Finds closest yellow-ish color
- Green (120°): Finds closest green-ish color
- Cyan (180°):  Finds closest cyan-ish color
- Blue (240°):  Finds closest blue-ish color
- Magenta (300°): Finds closest magenta-ish color

Brightness variants:
- Colors 0-7:  Normal brightness
- Colors 8-15: +30% brighter versions
```

**Key functions:**
```python
generate_palette(colors, theme)
# Returns complete 16-color ANSI palette

find_closest_hue(colors, target_hue, tolerance=60)
# Finds color closest to target hue within tolerance

adjust_brightness(color, factor)
# Makes color brighter/darker by factor (e.g., 1.3 = 30% brighter)
```

**3. contrast.py - WCAG Validation**

**What it does:**
- Validates color contrast ratios
- Ensures readability (WCAG standards)
- Auto-adjusts colors if needed

**Standards:**
```
WCAG AAA (highest):
- Text on background: 7:1 ratio
- Large text: 4.5:1 ratio

WCAG AA (standard):
- Text on background: 4.5:1 ratio
- Large text: 3:1 ratio
```

**Key functions:**
```python
contrast_ratio(color1, color2)
# Returns ratio from 1 (no contrast) to 21 (max contrast)
# Formula: (L1 + 0.05) / (L2 + 0.05)
# Where L = relative luminance with gamma correction

validate_palette(palette)
# Checks all important combinations:
# - Background vs Foreground
# - Background vs Accent
# - Background vs ANSI colors
# Returns validation report

auto_adjust_contrast(palette, min_ratio=7.0)
# Iteratively adjusts colors until min_ratio achieved
# Brightens light colors, darkens dark colors
```

**Why this matters:**
- Poor contrast = hard to read
- Automatic fixing = always readable
- WCAG compliance = accessible to everyone

**4. render.py - Template System**

**What it does:**
- Takes palette JSON
- Renders Jinja2 templates
- Generates complete configs

**Template example (foot.ini.j2):**
```jinja2
[colors]
background={{ background[1:] }}
foreground={{ foreground[1:] }}

regular0={{ colors.color0[1:] }}
regular1={{ colors.color1[1:] }}
...

# [1:] strips the # from hex codes
# Ghostty needs colors without #
```

**Key functions:**
```python
render_template(template_name, palette)
# Returns rendered config as string

render_all(palette, output_dir)
# Renders all templates to directory
# Returns dict of {template: output_path}
```

**Available templates:**
- `foot.ini.j2` - Foot terminal
- `ghostty.conf.j2` - Ghostty terminal
- `fuzzel.ini.j2` - Fuzzel launcher
- `waybar.css.j2` - Waybar status bar

**5. apply-to-0-core.sh - Integration Script**

**What it does:**
- Renders all templates
- Copies to 0-core packages
- Preserves base configurations
- Prepares for stow

**Workflow:**
```bash
./scripts/apply-to-0-core.sh my-palette.json

# Internally:
1. Determines theme type (dark/light from JSON)
2. Renders templates to /tmp
3. Copies to 0-core packages:
   - foot-theme-dark/.config/foot/foot.ini
   - ghostty-theme-dark/.config/ghostty/config
   - fuzzel-theme-dark/.config/fuzzel/fuzzel.ini
4. Reports success
5. Suggests next steps (stow, theme-switch)
```

### **Complete Workflow Example**

**Step 1: Extract colors from wallpaper**
```bash
cd ~/faelight-theme-engine

faelight-theme extract ~/wallpapers/sunset.jpg --output sunset-palette.json

# Output:
# Extracted 8 colors
# Theme: dark
# Dominant: #1a3c5a (dark blue)
# Accent: #ff6b35 (orange)
```

**Step 2: Validate palette (optional)**
```bash
faelight-theme validate sunset-palette.json

# Output:
# ✅ Background/Foreground contrast: 9.2:1 (AAA)
# ✅ Background/Accent contrast: 5.8:1 (AA+)
# ✅ All ANSI colors validated
# Palette is ready to use!
```

**Step 3: Render templates**
```bash
faelight-theme render sunset-palette.json --all --output-dir test-output

# Output:
# ✅ Rendered foot.ini
# ✅ Rendered ghostty.conf
# ✅ Rendered fuzzel.ini
# ✅ Rendered waybar-style.css
# All files saved to test-output/
```

**Step 4: Preview configs**
```bash
# Check generated configs
bat test-output/foot.ini
bat test-output/fuzzel.ini

# Verify colors look good
```

**Step 5: Apply to 0-core**
```bash
./scripts/apply-to-0-core.sh sunset-palette.json

# Output:
# 📝 Rendering templates...
# ✅ Rendered 4 templates
# 📦 Applying to 0-core packages...
#   ✓ Foot theme
#   ✓ Ghostty theme
#   ✓ Fuzzel theme
#   ✓ Waybar theme
# ✅ Theme applied to 0-core!
```

**Step 6: Deploy theme**
```bash
cd ~/0-core

# Check what changed
git diff foot-theme-dark/
git diff fuzzel-theme-dark/

# If you like it, commit
git add foot-theme-dark/ ghostty-theme-dark/ fuzzel-theme-dark/
git commit -m "theme: Sunset theme from wallpaper"

# Apply theme
theme-switch dark

# Apps reload with new colors!
```

### **Advanced Usage**

**Generate light theme:**
```bash
# Use a light wallpaper
faelight-theme extract ~/wallpapers/beach.jpg --output beach-palette.json

# Renders to light theme
./scripts/apply-to-0-core.sh beach-palette.json
# Creates configs in foot-theme-light/, etc.

# Switch to light theme
theme-switch light
```

**Customize palette before rendering:**
```bash
# Extract base palette
faelight-theme extract wallpaper.jpg --output base.json

# Edit JSON manually
nvim base.json
# Adjust colors, tweak values

# Render with customized palette
faelight-theme render base.json --all
```

**Batch processing:**
```bash
# Generate themes from multiple wallpapers
for wallpaper in ~/wallpapers/*.jpg; do
    name=$(basename "$wallpaper" .jpg)
    faelight-theme extract "$wallpaper" --output "$name-palette.json"
    faelight-theme render "$name-palette.json" --all --output-dir "themes/$name"
done

# Now you have themes/ directory with all variations!
```

### **Understanding the Color Science**

**Why certain colors for certain roles?**

**Background:**
- Darkest/lightest color from image
- Covers most of screen → dominant presence
- Must have good contrast with foreground

**Foreground:**
- High contrast with background
- Used for text → must be readable
- Automatically adjusted if contrast too low

**Accent:**
- Most saturated color
- Used for highlights, selections
- Needs moderate contrast (not as high as text)

**ANSI Colors:**
- Mapped by hue similarity
- Maintains color harmony
- Bright variants for emphasis

**Color temperature:**
- Warm colors (red, orange, yellow) → energetic
- Cool colors (blue, green, cyan) → calming
- Engine preserves temperature of original image

### **Troubleshooting Theme Engine**

**"Not enough distinct colors"**
```bash
# Try fewer colors
faelight-theme extract image.jpg --num-colors 6

# Or use different image (more varied)
```

**"Contrast too low"**
```bash
# Engine auto-adjusts, but you can force:
# Edit palette.json manually
# Increase brightness of foreground
# Or use validation to see exact ratios:
faelight-theme validate palette.json
```

**"Colors don't match wallpaper"**
```bash
# Possible reasons:
# 1. Image has subtle colors (extract picks dominant)
# 2. You're noticing accent colors (not dominant)

# Solution:
# Extract more colors to get full range
faelight-theme extract image.jpg --num-colors 12
```

**"Generated theme looks washed out"**
```bash
# Edit palette JSON
# Increase saturation:
# For HSV colors, increase S value (0-1)
# For RGB, push values toward extremes

# Or choose different source image
# (high saturation photos work best)
```

---

## 📋 Daily Workflows

### **Morning Routine**

**Start your day right:**
```bash
# 1. Check system health
dot-doctor

# 2. Update system (if needed)
topgrade

# 3. Check 0-core status
cd ~/0-core && git status

# 4. Open workspace
# Press SUPER+1 through SUPER+9 for different tasks
```

### **Development Workflow**

**Project work:**
```bash
# Navigate to project
z myproject  # Zoxide jump

# Open editor (LazyVim)
nvim .

# Terminal in same directory
SUPER+RETURN  # Opens Foot in current directory

# File management
yazi  # Browse, select, operate

# Git operations
gs    # Status
ga .  # Add all
gc "commit message"  # Commit
gp    # Push
```

**Workspace organization:**
- Workspace 1: Terminals
- Workspace 2: Editor
- Workspace 3: File manager / Browser
- Workspace 4: Communication
- Workspace 5: Media

**Quick switching:**
```
SUPER+1  # Terminal workspace
SUPER+2  # Editor workspace
SUPER SHIFT+2  # Move window to workspace 2
```

### **File Management Workflow**

**With Yazi:**
```bash
# Open Yazi
yazi

# Navigate (vim keys)
h/j/k/l

# Operations
Space  # Select
y      # Copy
d      # Cut
p      # Paste
D      # Delete
/      # Search

# Compare files
Space (select 2 files)
SUPER ALT+cm  # Opens Meld

# Bulk rename
Space (select multiple)
r  # Rename (opens editor)
```

**With PCManFM-Qt (when visual needed):**
```bash
# Launch
SUPER+F  # Or pcmanfm

# Drag-and-drop
# Visual browsing
# Good for showing others
```

### **System Maintenance Workflow**

**Weekly:**
```bash
# Update everything
topgrade

# Check system health
dot-doctor

# Clean up
yay -Sc  # Clear package cache
docker system prune  # Clean Docker (if used)

# Check disk space
df
dust  # Visual disk usage
```

**Monthly:**
```bash
# Review installed packages
pacman -Qq | wc -l  # Count packages
pacman -Qdt  # List orphans
cleanup  # Remove orphans

# Update mirror list
mirror

# Check logs
journalctl -p 3 -xb  # Errors this boot

# Review fail2ban
sudo fail2ban-client status
```

### **Theme Switching Workflow**

**Dynamic theming:**
```bash
# Morning (bright environment) → Light theme
theme-switch light

# Evening (dim environment) → Dark theme
theme-switch dark

# Generate theme from new wallpaper
cd ~/faelight-theme-engine
faelight-theme extract ~/wallpapers/new.jpg --output new-palette.json
./scripts/apply-to-0-core.sh new-palette.json
cd ~/0-core
theme-switch dark
```

### **Backup Workflow**

**Dotfiles (automatic with git):**
```bash
# Regular backups
cd ~/0-core
git add -A
git commit -m "backup: $(date +%Y-%m-%d)"
git push

# Quick alias
dotsync  # Defined in fish config
```

**Important files:**
```bash
# Backup before changes
backup important-file.conf
# Creates: important-file.conf.backup-20241209-153045

# Backup entire directory
tar czf backup-$(date +%Y%m%d).tar.gz ~/important-dir/
```

### **Clipboard Management Workflow**

**With Cliphist:**
```bash
# Copy multiple items
Ctrl+C (item 1)
Ctrl+C (item 2)
Ctrl+C (item 3)

# Retrieve old item
SUPER+V  # Opens history in Fuzzel
# Select item → It's copied!

# Paste
Ctrl+V
```

**Power user tip:**
```bash
# Copy command output to clipboard
ls -la | wl-copy

# Paste clipboard to file
wl-paste > file.txt

# Screenshot to clipboard
grim -g "$(slurp)" - | wl-copy
```

### **Color Picking Workflow**

**For theming:**
```bash
# Pick color from screen
SUPER SHIFT+C  # Launches hyprpicker
# Click pixel
# Color copied to clipboard!

# Use in config
nvim config.fish
# Paste color: Ctrl+Shift+V

# Or check clipboard
wl-paste  # Shows: #7fb069
```

### **Learning Workflow**

**Exploring your system:**
```bash
# See all keybindings
keyscan | less

# Check system health
dot-doctor

# List all aliases
alias

# Find a command
hg <search term>  # Searches history

# Learn package
pacman -Qi package-name  # Detailed info
```

---

## 🔧 Troubleshooting

### **Hyprland Issues**

**Symptom: Keybindings not working**
```bash
# Check if Hyprland sees keybinding
hyprctl binds | grep SUPER

# Reload config
hyprctl reload

# Check for syntax errors
hyprland  # Will show errors on startup

# Verify config loads
cat ~/.config/hypr/hyprland.conf
# Should source bindings.conf
```

**Symptom: App not launching from keybinding**
```bash
# Test app manually
foot  # Does it work?

# Check keybinding definition
grep "foot" ~/.config/hypr/bindings.conf

# Check if app is installed
which foot

# Check Hyprland logs
journalctl --user -u hyprland -n 50
```

**Symptom: Monitor not detected**
```bash
# List monitors
hyprctl monitors

# Edit monitor config
nvim ~/.config/hypr/monitors.conf

# Force detect
hyprctl reload
```

### **Stow Issues**

**Symptom: "Stow would cause conflicts"**
```bash
# See what's conflicting
stow -n -v hypr  # Dry run, verbose

# Two solutions:

# Option 1: Remove existing file
rm ~/.config/hypr/hyprland.conf
stow hypr

# Option 2: Adopt existing file
stow --adopt hypr
# Moves existing file into 0-core/wm-hypr/
# Then you can manage it with git
```

**Symptom: "Target already exists"**
```bash
# Existing directory not empty
# Solution: Unstow first, then restow
stow -D hypr
stow hypr
```

**Symptom: Changes not reflected**
```bash
# Restow to update symlinks
stow -R hypr

# Or unstow and stow again
stow -D hypr && stow hypr
```

### **Fish Shell Issues**

**Symptom: Alias not working**
```bash
# Reload config
source ~/.config/fish/config.fish

# Check if alias exists
alias | grep myalias

# Check for typos
cat ~/.config/fish/config.fish | grep myalias
```

**Symptom: Command not found**
```bash
# Check if in PATH
echo $PATH

# Check if script exists
ls -la ~/.local/bin/script-name

# Make executable if needed
chmod +x ~/.local/bin/script-name
```

**Symptom: Syntax error in config**
```bash
# Check syntax
fish -n ~/.config/fish/config.fish
# Will show error line if syntax wrong

# Common issues:
# - Missing 'end' in function
# - Unclosed quotes
# - Wrong variable syntax ($var not var)
```

### **Theme Issues**

**Symptom: Theme switch doesn't work**
```bash
# Check if script exists
which theme-switch

# Run manually with verbose output
bash -x ~/.local/bin/theme-switch dark

# Check if theme packages exist
ls ~/0-core/ | grep theme

# Verify stow status
cd ~/0-core
stow -D foot-theme-light foot-theme-dark
stow foot-theme-dark
```

**Symptom: Colors wrong after theme switch**
```bash
# Reload apps
# Terminals: Close and reopen
# Fuzzel: Automatically reloads
# Waybar: killall waybar && waybar &

# Check if correct theme stowed
ls -la ~/.config/foot/foot.ini
# Should point to foot-theme-dark

# Verify config contents
cat ~/.config/foot/foot.ini | grep background
```

### **Application Issues**

**Symptom: Foot/Ghostty not launching**
```bash
# Check if installed
pacman -Q foot ghostty

# Test manually
foot
ghostty

# Check for errors
foot --help
ghostty --version

# Check config syntax
foot --check-config
```

**Symptom: Fuzzel empty/no apps**
```bash
# Check if apps have .desktop files
ls /usr/share/applications/

# Regenerate desktop database
update-desktop-database ~/.local/share/applications/

# Clear cache
rm -rf ~/.cache/fuzzel/
```

**Symptom: Yazi not showing files**
```bash
# Check permissions
ls -la current-directory

# Check if yazi installed correctly
yazi --version

# Reset yazi config
mv ~/.config/yazi/yazi.toml ~/.config/yazi/yazi.toml.backup
# Restart yazi
```

### **System Issues**

**Symptom: High CPU usage**
```bash
# Check processes
btop  # Interactive process viewer

# Find culprit
ps aux --sort=-%cpu | head -10

# Kill if needed
kill -9 PID
```

**Symptom: High memory usage**
```bash
# Check memory
free -h

# See what's using memory
ps aux --sort=-%mem | head -10

# Clear cache if needed
sudo sync; echo 3 | sudo tee /proc/sys/vm/drop_caches
```

**Symptom: No internet connection**
```bash
# Check interface
ip addr show

# Check if connected
ping 1.1.1.1

# If DNS issue
ping 1.1.1.1  # Works?
ping google.com  # Doesn't work?
# = DNS problem

# Check DNS
cat /etc/resolv.conf

# Restart network
sudo systemctl restart NetworkManager
```

### **Package Issues**

**Symptom: Package not found**
```bash
# Update package database
sudo pacman -Sy

# Search for package
pacman -Ss package-name

# Check AUR
yay -Ss package-name
```

**Symptom: Dependency conflict**
```bash
# See what's conflicting
sudo pacman -S package-name
# Read error message

# Force reinstall if needed
sudo pacman -S --overwrite package-name

# Or remove conflicting package first
sudo pacman -R conflicting-package
```

**Symptom: Broken packages after update**
```bash
# Reinstall package
sudo pacman -S package-name

# Check for partial upgrades
sudo pacman -Qk

# Downgrade if needed
sudo pacman -U /var/cache/pacman/pkg/package-old-version.pkg.tar.zst
```

### **Getting Help**

**Where to look:**

1. **Documentation:**
   - Arch Wiki: `https://wiki.archlinux.org/`
   - Hyprland Wiki: `https://wiki.hyprland.org/`
   - Application man pages: `man foot`

2. **Logs:**
   - Hyprland: `journalctl --user -u hyprland`
   - System: `journalctl -xe`
   - dmesg: `sudo dmesg | tail -50`

3. **Community:**
   - Arch Forums
   - Hyprland Discord
   - Reddit r/archlinux

4. **This guide!**
   - Read relevant section
   - Try troubleshooting steps
   - Check examples

---

## 🚀 Advanced Topics

### **Custom Keybinding Creation**

**Add new keybinding:**
```bash
# Edit bindings
nvim ~/0-core/hypr/.config/hypr/bindings.conf

# Add binding
bind = SUPER SHIFT, N, exec, my-script.sh

# Restow
cd ~/0-core
stow -R hypr

# Reload Hyprland
hyprctl reload

# Update documentation
keyscan > ~/keybindings.md
```

### **Creating Custom Scripts**

**Template:**
```bash
#!/usr/bin/env bash
# Script name and description
# Location: ~/.local/bin/script-name

# Exit on error
set -e

# Your code here
echo "Hello World"

# Make executable
chmod +x ~/.local/bin/script-name

# Add to PATH (already done if in ~/.local/bin)
```

### **Creating Custom Fish Functions**

**Add to config.fish:**
```fish
function my_function
    # Function code
    echo "Doing something with $argv[1]"
end

# Or create separate file
# ~/.config/fish/functions/my_function.fish
function my_function
    # Code here
end
```

### **Custom Theme Creation**

**From scratch:**
```bash
# Create theme package
mkdir -p ~/0-core/foot-theme-custom/.config/foot

# Create config
nvim ~/0-core/foot-theme-custom/.config/foot/foot.ini

# Add colors
[colors]
background=your-color
foreground=your-color
...

# Stow it
cd ~/0-core
stow foot-theme-custom

# Test
foot
```

### **Hyprland Window Rules**

**Add custom rules:**
```bash
nvim ~/0-core/hypr/.config/hypr/windowrules.conf

# Examples:
windowrulev2 = float, class:^(floating-app)$
windowrulev2 = workspace 3, class:^(my-app)$
windowrulev2 = opacity 0.9, class:^(transparent-app)$
```

### **Automation with Systemd**

**Create user service:**
```bash
# Create service file
nvim ~/.config/systemd/user/my-service.service

[Unit]
Description=My Custom Service

[Service]
ExecStart=/path/to/script

[Install]
WantedBy=default.target

# Enable and start
systemctl --user enable my-service
systemctl --user start my-service
```

---

## 📚 Resources & Learning

### **Essential Reading**

- [Arch Wiki](https://wiki.archlinux.org/) - Complete Linux reference
- [Hyprland Wiki](https://wiki.hyprland.org/) - Compositor documentation
- [Fish Tutorial](https://fishshell.com/docs/current/tutorial.html) - Shell guide

### **Tool Documentation**

- Foot: `man foot`
- Yazi: `https://yazi-rs.github.io/`
- Fuzzel: `man fuzzel`
- Stow: `info stow`

### **Community**

- r/archlinux - Arch Linux subreddit
- r/unixporn - Desktop customization
- Hyprland Discord - Live help

---

## 🎯 Conclusion

**You now have:**
- ✅ Complete understanding of Omarchy system
- ✅ Knowledge of every tool and why it's chosen
- ✅ Workflows for daily tasks
- ✅ Troubleshooting skills
- ✅ Advanced customization knowledge

**Remember:**
- **dot-doctor** - Regular health checks
- **keyscan** - Reference your keybindings
- **topgrade** - Keep system updated
- **This guide** - Your complete reference

**Philosophy:**
> "A tight ship with quality tools. No bloat, only purpose."

---


🌲 May your Faelight Forest grow eternal! 🌲

Version 2.8.6 - Immortal Edition
Built with ❤️ by Christian
December 09, 2025
