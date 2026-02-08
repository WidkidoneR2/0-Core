# 🌲 Faelight-Fetch v2.1.0

**Zone-aware system information for Faelight Forest**

Beautiful, minimal system info display with zone detection and 0-Core integration.

## ✨ Features

### Core Information
- **System Version** - Current Faelight Forest version
- **Zone Detection** - Shows current directory zone with icon 🦀🌲📁
- **Profile State** - Active system profile (DEF, gaming, work, etc.)
- **Core Lock Status** - Immutable protection state (🔒/🔓)
- **Health Percentage** - System health from dot-doctor (🟢🟡🔴)

### System Details
- Window Manager
- Terminal Emulator
- Shell
- Kernel Version
- Uptime
- Hostname

### Beautiful Formatting
- Box border header
- Right-aligned labels
- Zone-aware icons
- Color-coded health
- Clean, minimal output

## 📦 Installation
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/faelight-fetch

# Build and install
cargo install --path .

# Verify
faelight-fetch
```

### Dependencies

- `dot-doctor` - Health checking
- `lsattr` - Core lock status
- `faelight-zone` - Zone detection
- `faelight-core` - Core paths

## 🚀 Usage
```bash
# Display system information
faelight-fetch

# Add to shell profile for login display
echo 'faelight-fetch' >> ~/.zshrc
```

## 📊 Output Example
```
╭─────────────────────────────────╮
│ 🌲 Faelight Forest v9.4.0       │
╰─────────────────────────────────╯

      zone  🦀 WORK
   profile  DEF
      core  🔓 unlocked
    health  🟡 89%

        wm  sway
      term  foot
     shell  zsh
    kernel  6.18.7-arch1-1
    uptime  21m
      host  fealight
```

## 🎯 Zone Detection

Shows current directory zone with appropriate icon:

| Zone | Icon | Meaning |
|------|------|---------|
| WORK | 🦀 | Rust projects |
| CORE | 🌲 | 0-Core system files |
| CODE | 💻 | General development |
| DOCS | 📚 | Documentation |
| MEDIA | 🎬 | Media files |
| DOWNLOADS | 📥 | Download directory |
| TMP | ⚡ | Temporary files |
| HOME | 🏠 | Home directory |

## 🎨 Health Icons

Visual health status indicators:

- 🟢 **100%** or **≥90%** - Excellent health
- 🟡 **70-89%** - Needs attention
- 🔴 **<70%** - Critical issues

## 🔒 Core States

Protection status:

- 🔓 **Unlocked** - Core is editable (no immutable flag)
- 🔒 **Locked** - Core is protected (chattr +i active)

## 💡 Philosophy

**"No ASCII art. No bloat. No configuration. Just facts."**

Faelight-fetch provides exactly what matters for 0-Core:
- Zone-aware context
- System health at a glance
- Essential system information
- Clean, professional output

## 🔄 Replaces

Designed as a cleaner, 0-Core-aware alternative to:
- fastfetch
- neofetch
- screenfetch

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 🎯 Use Cases

### Login Display
```bash
# Add to ~/.zshrc
faelight-fetch
```

### Quick System Check
```bash
# See zone + health at a glance
faelight-fetch
```

### Screenshots
```bash
# Professional system info for sharing
faelight-fetch
```

### CI/CD Verification
```bash
# Verify system state in pipelines
faelight-fetch
```

## 🛠️ Implementation

- **Lines of Code:** ~200
- **Binary Size:** ~600KB
- **Dependencies:** Minimal (sysinfo, faelight-zone, faelight-core)
- **Build Time:** Fast (<5s)
- **No Config Required:** Zero configuration needed

## 🌲 Part of 0-Core

Faelight-fetch is a core component of the 0-Core ecosystem, providing zone-aware system information display.

## 📄 License

Intentional Stewardship - Manual control over automation.
