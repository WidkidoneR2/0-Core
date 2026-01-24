# 🌲 Faelight Forest v8.1.0 - Sway Edition

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v8.1.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Sway](https://img.shields.io/badge/Sway-1.11-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Health](https://img.shields.io/badge/Health-100%25-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v8.1.0 Milestone:** Interactive updates, security hardening, ecosystem integration - 31 tools production-ready 🌲🦀

> **v8.0.0 Milestone:** Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture 🌲🦀

---

## 🏆 v8.1.0 Highlights - "The Garden"

**What's New:**
- ✅ **faelight-update v0.2.0** - Interactive update manager better than topgrade
- ✅ **Enhanced Security** - 14-check health monitoring with UFW, fail2ban, Mullvad, SSH hardening
- ✅ **Ecosystem Integration** - 31 tools working together, tools growing with intention
- ✅ **100% System Health** - All checks passing, security hardened
- ✅ **Philosophy-Driven** - "A garden requires attention, not automation"

**Flagship Tools:**
- 🚀 **faelight-update v0.2.0** - Interactive update manager with multi-source detection, TUI selection, health-check-first approach
- 🚀 **bump-system-version v4.0.0** - Complete release automation with CHANGELOG generation, intent tracking, git integration
- 🦀 **faelight-bootstrap v1.0.0** - One-command Arch Linux installation with 7 interactive phases
- 🏥 **dot-doctor v0.6.0** - 14-check health monitoring with security hardening verification
- 🎓 **teach v1.0.0** - Interactive learning system with quiz mode and achievements

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
| OS | Arch Linux | Rolling release, minimal base |
| WM | Sway 1.11 | Wayland compositor, tiling |
| Bar | faelight-bar | Custom Rust bar |
| Terminal | Foot | Fast, Wayland-native |
| Shell | Zsh + Nushell | 188+ aliases, Rust shell option |
| Prompt | Starship | Lock status indicator |
| Editor | Neovim (LazyVim) | Faelight colorscheme, Rust LSP |
| Launcher | faelight-launcher | Custom Rust launcher with fuzzy search |
| Files | Yazi | Terminal file manager |
| Notifications | faelight-notify | Wayland notifications |
| Login | tuigreet | Faelight themed greeter |

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

**Applied to:** Sway, faelight-bar, Foot, Neovim, tuigreet

---

## 🦀 The Rust Toolchain

All **31 core tools** are compiled Rust binaries organized in a workspace - 100% production-ready.

### Core Infrastructure (11 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| dot-doctor | 14-check health monitor | v0.6.0 | ✅ Production |
| faelight-update | Interactive update manager | v0.2.0 | 🚀 Flagship |
| faelight-core | Shared library (config, health, IPC) | v0.1.0 | ✅ Stable |
| core-protect | Immutable filesystem protection | v1.0.1 | ✅ Production |
| safe-update | Smart system updates with snapshots | v1.0.0 | ✅ Production |
| core-diff | Package-aware diff with risk levels | v2.0.0 | ✅ Production |
| dotctl | Central control utility | v2.0.0 | ✅ Production |
| entropy-check | Drift detection system | v1.0.0 | ✅ Production |
| intent-guard | Command safety validation | v1.0.0 | ✅ Production |
| faelight-stow | Package management | v0.3.0 | ✅ Stable |
| faelight-snapshot | BTRFS snapshot manager | v1.0.0 | ✅ Production |

### Faelight Desktop Environment (8 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| faelight-fetch | System info display | v1.0.0 | ✅ Production |
| faelight-bar | Wayland status bar (Sway IPC) | v0.9.0 | ✅ Stable |
| faelight-launcher | XDG app launcher with fuzzy search | v3.3.0 | ✅ Production |
| faelight-dmenu | Wayland dmenu replacement | v2.0.0 | ✅ Production |
| faelight-menu | Power menu (lock/logout/shutdown) | v0.7.0 | ✅ Stable |
| faelight-notify | Notification daemon | v0.9.0 | ✅ Stable |
| faelight-lock | Screen locker | v1.0.0 | ✅ Production |
| faelight-dashboard | System dashboard TUI | v1.0.0 | ✅ Production |

### Development & Workflow (8 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| intent | Intent Ledger management | v2.0.0 | ✅ Production |
| archaeology-0-core | System history explorer | v1.0.0 | ✅ Production |
| workspace-view | Sway workspace intelligence | v1.0.0 | ✅ Production |
| faelight-git | Git workflow automation | v2.1.0 | ✅ Production |
| profile | System profile switching | v1.0.0 | ✅ Production |
| teach | Interactive learning guide | v1.0.0 | ✅ Production |
| faelight | Unified binary interface | v1.0.0 | ✅ Production |
| keyscan | Keybind conflict detection | v1.0.0 | ✅ Production |

### Version Management (4 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| bump-system-version | Complete release automation | v4.0.0 | 🚀 Flagship |
| faelight-bootstrap | One-command system setup | v1.0.0 | 🚀 Flagship |
| get-version | Package version reader | v2.0.0 | ✅ Production |
| latest-update | Recently updated finder | v2.0.0 | ✅ Production |

**Benefits of Rust:**
- ⚡ **Faster** — Compiled binaries vs shell interpretation
- 🔒 **Safer** — Memory safety, no buffer overflows
- ✅ **Type-checked** — Errors caught at compile time
- 🛠️ **Maintainable** — Better error handling, clearer structure
- 🦀 **Modern** — Workspace monorepo with shared dependencies

**Total Lines of Rust:** ~15,000+ across all tools

### Project Scale
```
Code Statistics (as of v8.1.0):
  Rust source code:    102,517 lines  🦀
  Configuration files:   6,949 lines  ⚙️
  Intent documentation:  8,509 lines  🎯
  System guides:        ~2,000 lines  📚
  ────────────────────────────────────────
  Total authored:      ~120,000 lines

31 production-ready tools, 12 stow packages, 72+ documented decisions.
Built from scratch in ~3 months, from catastrophic failure to 100% health.
```

**Philosophy:** Every line intentional. Every decision documented. Every tool understood.

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

### Package Organization (Stow-Based)
```
0-core/
├── stow/                  # 🎯 ALL dotfile packages (GNU Stow managed)
│   ├── wm-sway/          Sway window manager config
│   ├── shell-zsh/        Zsh + aliases (188+)
│   ├── shell-nushell/    Nushell configuration
│   ├── prompt-starship/  Starship prompt
│   ├── term-foot/        Foot terminal
│   ├── editor-nvim/      Neovim + Faelight theme
│   ├── fm-yazi/          Yazi file manager
│   ├── vcs-git/          Git configuration
│   ├── config-faelight/  Typed TOML configs
│   ├── browser-qutebrowser/ Qutebrowser
│   ├── browser-brave/    Brave theming
│   └── tools-topgrade/   System updater config
│
├── rust-tools/           # 🦀 Rust workspace (monorepo - 31 tools)
│   ├── faelight/         Unified CLI
│   ├── dot-doctor/       Health check engine
│   ├── faelight-update/  Interactive update manager
│   ├── bump-system-version/ Complete release automation
│   ├── faelight-bootstrap/  One-command installer
│   ├── faelight-bar/     Status bar
│   ├── faelight-git/     Git governance
│   └── [24 more tools]   All production-ready
│
├── scripts/              # 📜 Compiled binaries (31 tools)
├── docs/                 # 📚 Tool references & guides
├── INTENT/               # 🎯 Intent ledger (78+ decisions)
├── profiles/             # 👤 System profiles
├── packages/             # 📦 Package lists (official/AUR)
└── system/               # ⚙️ System-level configs
```

**Deployment:** `cd ~/0-core/stow && stow -t ~ package-name`

---

## 🛡️ Security

| Layer | Implementation |
|-------|----------------|
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

Profiles are displayed in **faelight-bar**: `DEF` `GAM` `WRK` `LOW`

---

## 🔒 Core Protection

The `0-core` directory can be locked to prevent accidental changes:
```bash
lock-core    # Make immutable (chattr +i)
unlock-core  # Allow editing
```

**Status shown in:**
- **Starship prompt**: 🔒 locked / 🔓 unlocked
- **faelight-bar**: `LCK` / `UNL`
- **Git commits** blocked when locked

---

## 🏥 Health Monitoring
```bash
doctor   # Full 14-check diagnostic (100% health)
```

### 14 Health Checks:
- ✅ **Stow Symlinks** - All 12/12 packages
- ✅ **System Services** - faelight-bar, faelight-notify
- ✅ **Broken Symlinks** - None found
- ✅ **Yazi Plugins** - All 4 installed
- ✅ **Binary Dependencies** - All 15 present
- ✅ **Git Repository** - Clean, all pushed
- ✅ **Theme Packages** - Faelight Forest present
- ✅ **Scripts** - All executable
- ✅ **Package Metadata** - All .dotmeta files
- ✅ **Intent Ledger** - System functional
- ✅ **Profile System** - Working correctly
- ✅ **Faelight Config** - All valid
- ✅ **Sway Keybinds** - No conflicts (116 bindings)
- ✅ **Security Hardening** - UFW, fail2ban, Mullvad VPN, SSH

---

## ⌨️ Key Bindings

| Key | Action |
|-----|--------|
| `Super + Return` | Terminal (Foot) |
| `Super + D` | Launcher |
| `Super + B` | Browser (Brave) |
| `Super + Q` | Close window |
| `Super + V` | Toggle floating |
| `Super + F` | Fullscreen |
| `Super + H/J/K/L` | Focus (vim keys) |
| `Super + Shift + H/J/K/L` | Move window |
| `Super + 1-5` | Workspaces |
| `Super + Shift + E` | Exit Sway |

---

## 🚀 Quick Start

### Installation
```bash
# One-command setup (Arch Linux)
curl -fsSL https://raw.githubusercontent.com/WidkidoneR2/0-Core/main/bootstrap.sh | bash

# Or manual
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core
faelight-bootstrap
```

### Essential Commands
```bash
# System
doctor           # Health check (14 checks)
safe-update      # Update with snapshot
bump-system-version 8.1.0  # Release new version

# Navigation  
core             # cd ~/0-core
v                # nvim
y                # yazi
lg               # lazygit

# Learning
teach --begin    # Interactive tutorial
intent list      # View decisions
```

---

## 📜 Intent Ledger

78+ documented decisions drive this system's evolution.
```bash
intent list              # View all intents
intent show 067          # View specific intent
intent add future "..."  # Document new decision
```

**Categories:** `decisions`, `experiments`, `philosophy`, `future`, `incidents`

**Philosophy in Action:**
- **Intent 001:** Rust migration (v5.0.0)
- **Intent 059:** Monorepo unification
- **Intent 066:** faelight-bar v2.0 architecture
- **Intent 067:** Post-presentation evolution plan

---

## 🔄 Version History

| Version | Date | Milestone |
|---------|------|-----------|
| v8.1.0 | 2026-01-23 | Interactive updates, security hardening, ecosystem integration |
| v8.0.0 | 2026-01-22 | 31 tools production-ready, full audit complete |
| v7.6.5 | 2026-01-19 | Tool audit quick wins |
| v7.6.4 | 2026-01-19 | Release automation complete |
| v7.6.3 | 2026-01-19 | Stow migration complete |
| v7.0.0 | 2026-01-14 | Architectural excellence |
| v6.0.0 | 2026-01-09 | Sway edition |
| v5.0.0 | 2026-01-05 | First Rust tool |

[See full version history](#)

---

## 🌟 Credits

- **Inspiration:** [Omarchy](https://github.com/2nthony/omarchy) — the starting point
- **Philosophy:** Manual control, explicit intent, human comprehension
- **Tools:** Rust, Sway, Neovim, Zsh, Starship

---

## 📄 License

**MIT** — Use freely, learn deeply, configure intentionally.

---

> *"The forest grew its own tools, wrote its own rules, and found a new home."* 🌲🦀
