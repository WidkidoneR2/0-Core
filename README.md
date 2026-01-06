# 🌲 Faelight Forest v5.1.0 - 0-Core Hybrid Architecture

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v5.1.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Wayland](https://img.shields.io/badge/Wayland-Native-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Security](https://img.shields.io/badge/Lynis-73%25-orange)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v5.1.0 Milestone:** faelight-bar — Custom Rust Wayland status bar replacing Waybar. Intent-aware, clickable, built from scratch. 🦀🌲

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

## 🦀 The Rust Toolchain

**v5.1.0 marks the complete transition from Bash to Rust.**

All 14 core tools are now compiled Rust binaries:

| Tool                  | Purpose                             | Complexity |
| --------------------- | ----------------------------------- | ---------- |
| `dot-doctor`          | 12-check health monitor             | Hard       |
| `core-protect`        | Immutable filesystem protection     | Hard       |
| `safe-update`         | Smart system updates with snapshots | Hard       |
| `core-diff`           | Package-aware diff with risk levels | Medium     |
| `dotctl`              | Central control utility             | Medium     |
| `intent`              | Intent Ledger management            | Medium     |
| `profile`             | System profile switching            | Medium     |
| `teach`               | Interactive learning guide          | Medium     |
| `bump-system-version` | System version management           | Medium     |
| `bump-version`        | Package version bumper              | Medium     |
| `get-version`         | Package version reader              | Simple     |
| `latest-update`       | Recently updated finder             | Simple     |
| `faelight-bar`        | Wayland status bar (daily driver)   | Hard       |
| `theme-switch`        | Dark/light theme switcher           | Medium     |

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
│   ├── wm-hypr/          Hyprland window manager
│   ├── bar-waybar/       Status bar
│   └── notif-mako/       Notifications
│
├── 💻 Shell & Terminal
│   ├── shell-zsh/        Zsh configuration
│   ├── prompt-starship/  Starship prompt
│   └── theme-term-*/     Terminal themes
│
├── 🛠️ Development
│   ├── editor-nvim/      Neovim (LazyVim)
│   ├── fm-yazi/          File manager
│   └── vcs-git/          Git configuration
│
├── 🦀 Rust Tools
│   └── rust-tools/       All 12 Rust binaries
│
├── 📜 Scripts & Profiles
│   ├── scripts/          Compiled Rust tools
│   └── profiles/         System profiles
│
├── 📚 Documentation
│   ├── docs/             Guides & references
│   └── INTENT/           Decision ledger
│
└── 🎨 Themes
    └── theme-*/          GTK, terminal, launcher
```

---

## ✨ Key Features

### 🔒 Immutable Core Protection

```bash
lock-core              # 🔒 Make 0-core immutable
unlock-core            # 🔓 Unlock for editing
core-protect status    # 📊 Check protection state
core-protect edit pkg  # 📝 Edit with blast radius warning
```

### 🎮 System Profiles

```bash
profile list           # See available profiles
profile gaming         # Switch to gaming mode
profile work           # VPN on, focus mode
profile status         # Current state
```

### 🏥 Health Monitoring

```bash
dot-doctor             # 12-check system health
```

Validates: Stow symlinks, plugins, services, binaries, git health, themes, scripts, config aging, intent ledger, and more.

### 🔄 Safe Updates

```bash
safe-update            # Snapshots → Update → Health check
```

Pre/post Btrfs snapshots, yay auto-recovery, .pacnew detection.

### 📜 Intent Ledger

```bash
intent list            # See all decisions & plans
intent show 001        # View specific intent
intent add             # Document new decision
intent search rust     # Find related intents
```

### 📚 Teaching Mode

```bash
teach                  # Interactive 8-lesson guide
```

---

## 🚀 Quick Start

### Prerequisites

- Arch Linux (or Arch-based)
- Git, Stow, Rust toolchain
- See [docs/MANUAL_INSTALLATION.md](docs/MANUAL_INSTALLATION.md)

### Installation

```bash
# Clone
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core

# Deploy packages (one at a time, test each!)
cd ~/0-core
stow shell-zsh
stow wm-hypr
stow bar-waybar
# ... etc

# Verify
dot-doctor
```

---

## 🔐 Security

**73% Lynis Score** — Enterprise-grade for a desktop:

- ✅ LUKS2 full disk encryption
- ✅ UFW firewall
- ✅ fail2ban intrusion prevention
- ✅ DNSOverTLS (Quad9)
- ✅ Mullvad VPN integration
- ✅ Gitleaks secret scanning
- ✅ Manual-only updates (no boot automation)

---

## 📚 Documentation

| Document                                   | Purpose                   |
| ------------------------------------------ | ------------------------- |
| [COMPLETE_GUIDE.md](COMPLETE_GUIDE.md)     | Full system documentation |
| [docs/TOOLS.md](docs/TOOLS.md)             | Tool reference            |
| [docs/KEYBINDINGS.md](docs/KEYBINDINGS.md) | Keyboard shortcuts        |
| [docs/WORKFLOWS.md](docs/WORKFLOWS.md)     | Daily workflows           |
| [docs/ALIASES.md](docs/ALIASES.md)         | Shell aliases             |
| [INTENT/](INTENT/)                         | Decision ledger           |

---

## 🎨 The Faelight Forest Theme

A cohesive visual identity across the entire system:

- **Forest Night** `#0f1411` — Base background
- **Faelight Green** `#6be3a3` — Primary accent
- **Faelight Blue** `#5cc8ff` — Secondary accent
- **Amber Leaf** `#f5c177` — Warnings
- **Fog White** `#d7e0da` — Text

Applied to: Hyprland, Waybar, terminals, Neovim, notifications.

---

## 📊 Stats

```
Version:          v5.1.0
Packages:         19 stow packages
Rust Tools:       12 (100% coverage)
Health Checks:    12 automated
Intents:          18 documented
Profiles:         4 system modes
Shell Aliases:    188+
Lynis Score:      73%
```

---

## 🌲 The Journey

| Version  | Milestone                                                  |
| -------- | ---------------------------------------------------------- |
| v1-2.x   | The "dotfiles" era — generic, chaotic                      |
| v3.0     | Foundation — cleanup, Tokyo Night                          |
| v3.1     | Great Transformation — numbered structure, semantic naming |
| v3.2     | Smart Systems — safe-update, recovery                      |
| v3.5     | Git Guardrails — protected commits                         |
| v3.6     | Intent Ledger — documented decisions                       |
| v4.0     | System Profiles — one-command switching                    |
| v4.1     | Teaching Mode — interactive learning                       |
| v4.2     | Profile Sharing — export/import                            |
| **v5.0** | **Complete Rust Transition** 🦀                            |

---

## 🤝 Contributing

Personal configuration, but improvements welcome!

1. Test thoroughly
2. Run `dot-doctor` before committing
3. Follow semantic naming
4. Document decisions in INTENT/

---

## 📜 License

MIT License — See [LICENSE](LICENSE)

---

**Made with 🌲 by Christian**

_"The forest speaks Rust now."_ 🦀🌲
