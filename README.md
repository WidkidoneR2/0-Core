# 🌲 Omarchy-FAELIGHT FOREST - Arch Linux 

> **A minimal, native Wayland system built for speed, reliability, and beauty.**

**Philosophy:** "Tight ship" - Every tool serves a purpose. No bloat, no redundancy, only quality native Wayland applications that work flawlessly together.

![Hyprland](https://img.shields.io/badge/Hyprland-Latest-blue)
![Wayland](https://img.shields.io/badge/Wayland-Native-green)
![License](https://img.shields.io/badge/License-MIT-yellow)

---

## 🎯 Quick Start
```bash
# Clone dotfiles
git clone https://github.com/WidkidoneR2/dotfiles.git ~/dotfiles
cd ~/dotfiles

# Install packages
sudo pacman -S --needed - < pkglist.txt

# Deploy with stow
./deploy.sh

# Run system health check
dot-doctor

# Reload Hyprland
hyprctl reload
```

---

## 📁 System Structure
```
dotfiles/
├── hypr/              # Hyprland config (modular!)
│   └── .config/hypr/
│       ├── hyprland.conf       # Main config (sources all others)
│       ├── env.conf            # Environment variables
│       ├── monitors.conf       # Display setup
│       ├── bindings.conf       # Keybindings
│       ├── workspaces.conf     # Workspace rules
│       ├── autostart.conf      # Startup applications
│       └── animations.conf     # Animation settings
│
├── foot-theme-dark/   # Foot terminal colors
├── ghostty-theme-dark/# Ghostty backup terminal
├── fuzzel-theme-dark/ # Fuzzel launcher theme
├── waybar/            # Status bar
├── mako/              # Notifications
├── yazi/              # TUI file manager
├── fish/              # Fish shell
├── nvim/              # Neovim config
│
├── scripts/           # Utility scripts
│   ├── dot-doctor          # System health checker
│   ├── keyscan             # Keybinding documentation
│   ├── theme-switch        # Theme switcher
│   ├── omarchy-menu-fuzzel # Main menu
│   └── power-menu-fuzzel   # Power options
│
└── pkglist.txt        # Complete package list
```

### 🎨 **Stow-Based Management**

This setup uses [GNU Stow](https://www.gnu.org/software/stow/) for symlink management:
```bash
# Stow a package (creates symlinks)
stow hypr

# Unstow (removes symlinks)
stow -D hypr

# Restow (refresh symlinks)
stow -R hypr

# Deploy everything
./deploy.sh
```

**Why Stow?**
- ✅ Clean separation (each app is a "package")
- ✅ Easy rollback (unstow anytime)
- ✅ Version control friendly
- ✅ No manual symlink management

---

## 🛠️ Core Tools

### **Why These Specific Tools?**

Every tool was chosen after extensive testing for **native Wayland support**, **minimal resource usage**, and **reliability**. No X11 compatibility layers, no bloat.

### 🦶 **Foot** - Primary Terminal
- **Why:** Native Wayland, 2ms startup (vs Kitty's 50ms), 5-10MB RAM
- **Perfect for:** Quick commands, daily terminal work
- **Config:** `foot-theme-dark/.config/foot/foot.ini`

### 👻 **Ghostty** - Backup Terminal  
- **Why:** Modern, native Wayland, feature-rich when needed
- **Use case:** Complex workflows, advanced features
- **Config:** `ghostty-theme-dark/.config/ghostty/config`

### 🚀 **Fuzzel** - Application Launcher
- **Why:** Native Wayland, instant response, no click issues (Rofi had problems)
- **Keybind:** `SUPER+SPACE`
- **Config:** `fuzzel-theme-dark/.config/fuzzel/fuzzel.ini`

### 📁 **Yazi** - Primary File Manager (TUI)
- **Why:** Blazing fast, keyboard-driven, powerful
- **Keybind:** `SUPER+E` or just type `yazi`
- **Features:**
  - Vim-style navigation
  - Integrated Meld for diffs (`SUPER ALT+cm`)
  - No GUI bloat needed!

### 🖥️ **PCManFM-Qt** - GUI File Manager (Backup)
- **Why:** Only 3MB, Qt-based, perfect for visual browsing when learning
- **Use case:** When you need to see things visually
- **Alternative to:** Thunar (20MB, removed)

### 📋 **Cliphist** - Clipboard History
- **Why:** Native Wayland, lightweight, essential productivity tool
- **Keybind:** `SUPER+V`
- **Usage:** Never lose clipboard content again!

### 🎨 **Hyprpicker** - Color Picker
- **Why:** Built for Hyprland, instant color grabbing
- **Keybind:** `SUPER SHIFT+C`
- **Usage:** Click any pixel, color copied to clipboard

### 📄 **Zathura** - PDF Viewer
- **Why:** Minimal, vim-like, 5MB vs Evince's 40MB
- **Default:** Opens all PDFs automatically

### 🔄 **Topgrade** - System Updater
- **Why:** Updates everything (pacman, AUR, flatpak, etc.) in one command
- **Usage:** `topgrade` or via Omarchy menu

---

## 🔧 Utilities & Scripts

### 🏥 **dot-doctor** - System Health Checker

Validates your entire system setup:
```bash
dot-doctor
```

**Checks:**
- ✅ All required packages installed
- ✅ Stow packages deployed correctly
- ✅ Config files exist
- ✅ Scripts are executable
- ✅ No broken symlinks

**Output:** Clear report with issues and fixes

---

### 🔍 **keyscan** - Keybinding Documentation

Auto-generates keybinding reference from your config:
```bash
keyscan
```

**Features:**
- Scans `bindings.conf`
- Extracts all keybindings
- Formats as readable markdown
- Perfect for learning your own system!

---

### 🎨 **theme-switch** - Theme Switcher

Switch between dark/light themes instantly:
```bash
theme-switch dark
theme-switch light
```

**Updates:**
- Foot colors
- Ghostty theme
- Fuzzel theme
- Hyprland borders
- GTK theme
- Mako notifications

---

### 🎯 **omarchy-menu-fuzzel** - Main System Menu

Central hub for all system operations:

**Keybind:** `SUPER ALT+SPACE`

**Features:**
- 🔄 Update System (topgrade)
- 🎨 Theme Menu
- ⚙️ Settings
- 🔌 Power Menu
- 📦 Package Manager
- 🔧 System Tools (btop, lazydocker, etc.)

---

### ⚡ **power-menu-fuzzel** - Power Options

Quick power management:

**Keybind:** `SUPER+ESCAPE`

**Options:**
- ⏻ Shutdown
- 🔁 Reboot
- 💤 Suspend
- 🔒 Lock
- 🚪 Logout

---

## 🎨 Faelight Theme Engine

**NEW in v2.8.x!** Generate terminal themes from any wallpaper.

### **What It Does:**
1. Extracts dominant colors from images
2. Generates color palettes with proper contrast
3. Creates configs for Foot, Ghostty, Fuzzel, Kitty
4. Applies to your system instantly

### **Usage:**
```bash
# Extract colors from wallpaper
faelight-theme extract wallpaper.jpg --output palette.json

# Generate configs
faelight-theme render palette.json --all --output-dir themes/

# Apply to dotfiles
cd ~/faelight-theme-engine
./scripts/apply-to-dotfiles.sh palette.json

# Switch theme
cd ~/dotfiles
theme-switch dark
```

### **How It Works:**
- Uses colorgram.py for color extraction
- Generates 16 ANSI colors with proper brightness
- Validates contrast ratios (WCAG AAA standards)
- Jinja2 templates for each application

**Location:** `~/faelight-theme-engine/`

---

## ⌨️ Key Keybindings

### **System**
- `SUPER+Q` - Close window
- `SUPER+M` - Exit Hyprland
- `SUPER+ALT+SPACE` - Omarchy menu
- `SUPER+ESCAPE` - Power menu

### **Applications**
- `SUPER+RETURN` - Terminal (Foot)
- `SUPER+SPACE` - App launcher (Fuzzel)
- `SUPER+E` - File manager (Yazi)
- `SUPER+B` - Browser (Brave)

### **Utilities**
- `SUPER+V` - Clipboard history
- `SUPER SHIFT+C` - Color picker
- `SUPER SHIFT+S` - Screenshot (grim + slurp)

### **Workspaces**
- `SUPER+[1-9]` - Switch workspace
- `SUPER SHIFT+[1-9]` - Move window to workspace

**Full list:** Run `keyscan` for complete documentation!

---

## 🚀 Installation

### **Prerequisites**
- Arch Linux (or Arch-based distro)
- Basic familiarity with terminal

### **Step 1: Clone Repository**
```bash
git clone https://github.com/YOUR_USERNAME/dotfiles.git ~/dotfiles
cd ~/dotfiles
```

### **Step 2: Install Packages**
```bash
# Install from package list
sudo pacman -S --needed - < pkglist.txt

# Install AUR helper (if not installed)
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si
```

### **Step 3: Deploy Dotfiles**
```bash
cd ~/dotfiles

# Deploy all packages
./deploy.sh

# Or manually with stow
stow hypr waybar mako foot-theme-dark fuzzel-theme-dark fish nvim yazi
```

### **Step 4: Validate Setup**
```bash
dot-doctor
```

Fix any issues reported!

### **Step 5: Reload Hyprland**
```bash
hyprctl reload
```

---

## 🎯 Features

### ✨ **Highlights**

- **Native Wayland:** Zero X11 dependencies
- **Minimal Bloat:** ~106MB freed vs previous setup
- **Theme Engine:** Generate themes from wallpapers
- **Modular Config:** Easy to understand and modify
- **Health Checks:** dot-doctor validates everything
- **Auto Documentation:** keyscan generates keybinding reference

### 🔐 **Security Hardened**

- LUKS2 full disk encryption
- UFW firewall configured
- fail2ban intrusion prevention
- DNSOverTLS with Quad9
- Mullvad VPN integration

### 🎨 **Theming**

- Complete Faelight Forest aesthetic
- Coordinated colors across all apps
- Easy theme switching (dark/light)
- Wallpaper-based theme generation

### ⚡ **Performance**

- Foot: 2ms terminal startup
- Fuzzel: Instant app launching
- Native Wayland: Smooth animations
- Minimal RAM usage across all tools

---

## 📝 Package Philosophy

### **What Was Removed (v2.8.x Cleanup)**

| Tool | Why Removed | Replacement | Space Saved |
|------|-------------|-------------|-------------|
| Kitty | X11-based, heavier | Foot | 60MB |
| Rofi | Wayland port, click issues | Fuzzel | 1MB |
| Thunar | GUI bloat | Yazi + PCManFM-Qt | 20MB |
| Walker | Flaky, unmaintained | Fuzzel | 335MB |
| Evince | GNOME bloat | Zathura | 15MB |
| gvfs | Unnecessary for TUI workflow | Removed | 6MB |

**Total freed:** ~437MB of bloat!

### **Why These Replacements?**

**Native Wayland First:**
- Foot, Ghostty, Fuzzel all built FOR Wayland
- No X11 compatibility layers
- Better performance, fewer bugs

**Minimal By Design:**
- Zathura: 5MB vs Evince 40MB
- Foot: 5-10MB RAM vs Kitty 30-50MB
- PCManFM-Qt: 3MB vs Thunar 20MB

**Quality Over Features:**
- We don't need every feature
- We need features that WORK
- Reliability > bells and whistles

---

## 🤝 Contributing

Improvements welcome! Please:
1. Test changes thoroughly
2. Run `dot-doctor` before committing
3. Update documentation
4. Follow existing code style

---

## 📜 License

MIT License - See LICENSE file

---

## 🙏 Acknowledgments

- Hyprland community
- Arch Linux community  
- All the tool developers
- Everyone who values quality over quantity

---

## 📚 Resources

- [Hyprland Wiki](https://wiki.hyprland.org/)
- [Arch Wiki](https://wiki.archlinux.org/)
- [GNU Stow Manual](https://www.gnu.org/software/stow/manual/)
- [Faelight Theme Engine](https://github.com/YOUR_USERNAME/faelight-theme-engine)

---

**Built with ❤️ and a commitment to quality.**

**No bloat. No compromises. Just tools that work.**

🌲 **Omarchy** - *Order through simplicity*
