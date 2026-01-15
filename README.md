# 🌲 Faelight Forest v7.1.0 - Sway Edition

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v7.1.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Sway](https://img.shields.io/badge/Sway-1.11-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Health](https://img.shields.io/badge/Health-100%25-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v7.1.0 Milestone:** Security - gitleaks pre-commit integration, enhanced token detection, login polish. 🌲🔒

---

## 🎯 The Philosophy

0-Core is more than dotfiles — it's a position on how personal computing should work.

**Core principles:**
- **Manual control over automation** — YOU decide when things run
- **Intent over convention** — Every decision documented
- **Understanding over convenience** — Know your system
- **Recovery over perfection** — Plan for failure

Read the full manifesto: [docs/THEORY_OF_OPERATION.md](docs/THEORY_OF_OPERATION.md)

---

## 🖥️ The Stack

| Component | Choice | Notes |
|-----------|--------|-------|
| **OS** | Arch Linux | Rolling release, minimal base |
| **WM** | Sway 1.11 | Wayland compositor, tiling |
| **Bar** | faelight-bar | Custom Rust bar |
| **Terminal** | Foot | Fast, Wayland-native |
| **Shell** | Zsh + Nushell | 188+ aliases, Rust shell option |
| **Prompt** | Starship | Lock status indicator |
| **Editor** | Neovim (LazyVim) | Faelight colorscheme, Rust LSP |
| **Launcher** | faelight-launcher | Custom Rust launcher with fuzzy search |
| **Files** | Yazi | Terminal file manager |
| **Notifications** | faelight-notify | Wayland notifications |
| **Login** | tuigreet | Faelight themed greeter |

---

## 🎨 Faelight Forest Theme

A cohesive visual identity across the entire system:

| Color | Hex | Usage |
|-------|-----|-------|
| Forest Night | `#0f1411` | Base background |
| Faelight Green | `#6be3a3` | Primary accent |
| Faelight Blue | `#5cc8ff` | Secondary accent |
| Amber Leaf | `#f5c177` | Warnings |
| Fog White | `#d7e0da` | Text |

Applied to: Sway, faelight-bar, Foot, Neovim, Fuzzel, Mako, tuigreet

---

## 🦀 The Rust Toolchain

**All 14 core tools are compiled Rust binaries:**

| Tool | Purpose | Complexity |
|------|---------|------------|
| `dot-doctor` | 12-check health monitor | Hard |
| `faelight-bar` | Wayland status bar (Sway IPC) | Hard |
| `core-protect` | Immutable filesystem protection | Hard |
| `safe-update` | Smart system updates with snapshots | Hard |
| `core-diff` | Package-aware diff with risk levels | Medium |
| `dotctl` | Central control utility | Medium |
| `intent` | Intent Ledger management | Medium |
| `profile` | System profile switching | Medium |
| `teach` | Interactive learning guide | Medium |
| `theme-switch` | Dark/light theme switcher | Medium |
| `bump-system-version` | System version management | Medium |
| `bump-version` | Package version bumper | Medium |
| `get-version` | Package version reader | Simple |
| `latest-update` | Recently updated finder | Simple |

**Benefits:**
- ⚡ **Faster** — Compiled binaries vs shell interpretation
- 🔒 **Safer** — Memory safety, no buffer overflows
- ✅ **Type-checked** — Errors caught at compile time
- 🛠️ **Maintainable** — Better error handling, clearer structure

---

## 🏗️ Directory Structure

### Numbered Hierarchy
```
~/0-core/     🔒 Configuration (this repo) - MOST CRITICAL
~/1-src/      📁 Source code & projects
~/2-projects/ 💼 Active work
~/3-archive/  💎 Completed/archived
~/4-media/    🎬 Media files
~/secrets/    🔐 Never committed
```

### Package Organization
```
0-core/
├── 🖥️ Desktop Environment
│   ├── wm-sway/           Sway window manager
│   └── faelight-notify/   Custom notification daemon
│
├── 💻 Shell & Terminal
│   ├── shell-zsh/         Zsh configuration (188+ aliases)
│   ├── prompt-starship/   Starship prompt
│   └── term-foot/         Foot terminal
│
├── 🛠️ Development
│   ├── editor-nvim/       Neovim + Faelight colorscheme
│   ├── fm-yazi/           File manager
│   └── vcs-git/           Git configuration
│
├── 🦀 Rust Tools (9 binaries)
│   └── rust-tools/
│       ├── faelight/          Unified CLI
│       ├── dot-doctor/        Health engine
│       ├── faelight-bar/      Status bar
│       ├── faelight-git/      Git governance
│       ├── faelight-notify/   Notifications
│       ├── faelight-launcher/ App launcher
│       ├── faelight-menu/     Power menu
│       ├── faelight-lock/     Screen locker
│       └── bump-system-version/
│
├── ⚙️ Configuration
│   └── config-faelight/   Typed TOML configs
│
├── 📜 Scripts
│   └── scripts/           Compiled binaries + shell scripts
│
├── 📚 Documentation
│   └── docs/              Tool references & guides
│
└── 📜 Intent Ledger
    └── INTENT/            Decision documentation
```

---
## 🛡️ Security

| Layer | Implementation |
|-------|---------------|
| Disk | Btrfs (LUKS recommended) |
| Firewall | UFW (deny incoming) |
| Intrusion | fail2ban (sshd jail) |
| DNS | DNSOverTLS (Quad9) |
| VPN | Mullvad (auto-connect, clickable in bar) |
| Secrets | Gitleaks pre-commit scanning |
| Kernel | 99-hardening.conf (sysctl) |
| Core | Immutable protection (chattr +i) |

---

## 🎮 Profile System

Switch between system states:
```bash
profile list      # Show available profiles
profile gaming    # Maximum GPU performance
profile work      # Focus mode with VPN
profile low-power # Battery optimization
profile default   # Balanced daily driver
```

Profiles are displayed in faelight-bar: `DEF` `GAM` `WRK` `LOW`

---

## 🔒 Core Protection

The 0-core directory can be locked to prevent accidental changes:
```bash
lock-core    # Make immutable (chattr +i)
unlock-core  # Allow editing
```

Status shown in:
- Starship prompt: 🔒 locked / 🔓 unlocked
- faelight-bar: `LCK` / `UNL`
- Git commits blocked when locked

---

## 🏥 Health Monitoring
```bash
dot-doctor   # Full 12-check diagnostic
```

Checks:
- ✅ Stow symlinks
- ✅ Yazi plugins
- ✅ Broken symlinks
- ✅ System services
- ✅ Binary dependencies
- ✅ Git repository health
- ✅ Theme packages
- ✅ Scripts executable
- ✅ Config aging
- ✅ Intentional defaults
- ✅ Intent Ledger
- ✅ Profile System

---

## ⌨️ Key Bindings

| Key | Action |
|-----|--------|
| `Super + Return` | Terminal (Foot) |
| `Super + D` | Launcher (Fuzzel) |
| `Super + B` | Browser (Brave) |
| `Super + Q` | Close window |
| `Super + V` | Toggle floating |
| `Super + F` | Fullscreen |
| `Super + H/J/K/L` | Focus (vim keys) |
| `Super + Shift + H/J/K/L` | Move window |
| `Super + 1-5` | Workspaces |
| `Super + Shift + E` | Exit Sway |

---

## 🚀 Quick Commands
```bash
# System
dot-doctor        # Health check
safe-update       # Update with snapshot
topgrade          # Full system upgrade

# Navigation
core              # cd ~/0-core
src               # cd ~/1-src

# Development
v                 # nvim
lg                # lazygit
y                 # yazi

# Info
intent list       # Show intents
latest-update     # Recent changes
get-version       # Package version
```

---

## 📜 Intent Ledger

Document decisions, not just configurations:
```bash
intent list              # View all intents
intent show 001          # View specific intent
intent add future "..."  # Add new intent
```

Categories: `decisions`, `experiments`, `philosophy`, `future`, `incidents`

---

## 🔄 Version History
| Version | Date | Milestone |
|---------|------|-----------|
| v7.1.0 | 2026-01-15 | Rust Hygiene - Code quality improvements |
| v7.0.1 | 2026-01-15 | Security - gitleaks integration, login polish |
| v7.0.0 | 2026-01-09 | Sway Edition - Complete migration |
| v6.9.1 | 2026-01-14 | dot-doctor v0.3, keybind detection, security verification |
| v6.4.0 | 2026-01-10 | Unified CLI, typed configs, health engine |
| v6.3.0 | 2026-01-10 | faelight-menu, faelight-lock |
| v6.0.0 | 2026-01-09 | Sway Edition - Complete migration |
| v5.1.0 | 2026-01-06 | faelight-bar - Custom Rust bar |
| v5.0.0 | 2026-01-06 | Complete Rust transition |
| v4.0.0 | 2025-12 | Profile system |
| v3.0.0 | 2025-12 | Intent Ledger |

See [CHANGELOG.md](CHANGELOG.md) for full history.

---

## 🌟 Credits

- **Inspiration:** [Omarchy](https://github.com/omarchy) — the starting point
- **Philosophy:** Manual control, explicit intent, human comprehension
- **Tools:** Rust, Sway, Neovim, Zsh, Starship

---

## 📄 License

MIT — Use freely, learn deeply, configure intentionally.

---

> *"The forest grew its own tools, wrote its own rules, and found a new home."* 🌲🦀
