# 🌲 Faelight Forest v8.7.0 - Sway Edition

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v8.7.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Sway](https://img.shields.io/badge/Sway-1.11-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Health](https://img.shields.io/badge/Health-100%25-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v8.7.0 Milestone:** alias-audit v1.0.0, bump-tool-version v1.0.0, and Starship v2.0 🌲🦀


---

## 🏆 v8.7.0 — Version Management & Alias Excellence

### ✨ What's New

**🆕 New Production Tools**

Two powerful new tools join the ecosystem:

**🔍 alias-audit v1.0.0**
- Check for duplicate alias definitions
- Verify all 40 tools have proper aliases
- Detect excessive aliasing patterns
- Beautiful colored output with doctor integration
- Commands: `audit`, `duplicates`, `missing`, `tools`
- Result: 301 total aliases, 100% coverage (37/37 active tools)

**🔧 bump-tool-version v1.0.0**
- Individual tool version management with auto-increment
- Auto-increment flags (`--major`, `--minor`, `--patch`)
- Beautiful pre-flight dashboard (like bump-system-version)
- Handles workspace versions (converts to explicit)
- Updates Cargo.toml + README.md automatically
- Creates tool-specific git tags (e.g., `faelight-link-v1.0.1`)
- Companion to bump-system-version for granular control

**✨ Starship Prompt v2.0**
- Smart path display (no duplication with zone names)
- Git diff stats (±files, insertions/deletions)
- Enhanced git status with counts (`!2`, `+1`, `?3`, etc.)
- Profile icons: 💼 WORK, 🎮 GAMING, 🔋 LOW-POWER, 🛠️ DEV
- Conflict indicator: ⚔️ for merge conflicts
- Fixed path handling to prevent "0-CORE 0-core" duplication

**📋 Alias System Overhaul**
- Hybrid pattern: short aliases (`fm`, `fl`) + f-prefix (`f-fm`, `f-link`)
- Fixed conflicts: `fm` (yazi→faelight-fm), `fl` (faelight→faelight-link)
- Added missing tool aliases for complete coverage
- Profile icon support (💼 🎮 🔋 🛠️)
- Updated from v8.1.0 to v8.7.0
- Total: 301 aliases covering 37/37 active tools (100%)

### 📐 Philosophy Realized

*"Tools to tend the forest, eyes to see its paths - the ecosystem grows with intention."*

This release demonstrates:
- **Versioning granularity**: System-wide OR individual tool bumps
- **Alias excellence**: Every tool accessible, every shortcut intentional
- **Visual clarity**: Prompts that inform without overwhelming
- **Manual control**: Auto-increment available, confirmation required

### ✅ 100% System Health

All **15 checks** passing — **40 Rust tools** in production (+2 new!)

---

## 🏗️ System Architecture

### 📁 Directory Structure
```
~/0-core/                          # System root (immutable when locked)
├── rust-tools/                    # 40 production Rust tools (monorepo workspace)
│   ├── dot-doctor/               # Health monitoring
│   ├── faelight-bar/             # Wayland status bar
│   ├── faelight-link/            # Symlink manager
│   ├── alias-audit/              # Alias health checker
│   └── ... (36 more tools)
│
├── stow/                          # GNU Stow configuration packages
│   ├── shell-zsh/                # Zsh config + 301 aliases
│   ├── sway-wm/                  # Sway window manager
│   ├── prompt-starship/          # Starship prompt config
│   ├── foot-terminal/            # Foot terminal config
│   └── ... (8 more packages)
│
├── scripts/                       # Compiled binaries (gitignored)
│   ├── doctor                    # Health checker
│   ├── bump-system-version       # Release automation
│   └── ... (38 more binaries)
│
├── docs/                          # Documentation & guides
│   ├── intents/                  # Intent Ledger (8,780 lines)
│   ├── guides/                   # System guides (7,269 lines)
│   └── architecture/             # Design decisions
│
├── Cargo.toml                    # Workspace root (all 40 tools)
├── VERSION                       # Current version (8.7.0)
├── CHANGELOG.md                  # Release history
└── README.md                     # This file
```

### 🎯 Design Principles

1. **Numbered Priority** - `0-core` comes first (alphabetically, logically, intentionally)
2. **Manual Control** - Auto-increment available, confirmation always required
3. **Spatial Awareness** - Zones guide workflows (`0-core`, `1-src`, `2-projects`)
4. **Explicit Intent** - Every decision documented in Intent Ledger
5. **Health Monitoring** - 15 automated checks, 100% transparency

### 🔄 Workflow Integration
```
User Action → Zone Detection → Tool Selection → Intent Logging → Health Check
     ↓              ↓                 ↓               ↓              ↓
  cd 0-core    🔒 0-CORE         doctor           intent         ✅ 100%
  Edit file      Zone:          Check #1-15      Document       15/15 Pass
  Commit         Protected      Real-time        Decision       Auto-fix
```

### 🌲 Zone System
```
~
├── 0-core/          🔒 System configuration (immutable when locked)
├── 1-src/           🌲 Source code exploration
├── 2-projects/      📁 Active development
├── 3-archive/       📦 Completed work
└── Downloads/       💾 Temporary files
```

**faelight-zone** provides spatial awareness:
- Current zone displayed in prompt
- Zone-specific behavior (e.g., core protection)
- Intent tracking per zone

---

## 🌲 Flagship Tools

**🚀 faelight-bar v2.0.0**  
Hybrid Wayland bar with integrated application launcher using keyboard mode switching. Revolutionary single-process architecture with transparent dropdown overlay.

**🔗 faelight-link v1.0.0**  
Zone-aware symlink manager with health monitoring. Complete stow replacement. Six commands: stow, unstow, list, status, audit, clean.

**🌲 faelight-fm v1.0.0**  
Semantic file manager with file operations, zone protection, and daemon integration. Better than yazi: Intent tracking, spatial awareness, universal backend.

**🔍 alias-audit v1.0.0** ⭐ NEW!  
Alias health checker - ensures all 40 tools have proper aliases. Commands: audit, duplicates, missing, tools. Result: 301 aliases, 100% coverage.

**🔧 bump-tool-version v1.0.0** ⭐ NEW!  
Individual tool version management with auto-increment (`--major/--minor/--patch`). Beautiful pre-flight dashboard, handles workspace versions, creates tool-specific tags.

**🦀 faelight-term v9.0.0** (Beta / WIP)  
Terminal emulator with color emoji, copy/paste, and mouse selection. Actively developed — APIs and behavior may change.

**🏥 dot-doctor v0.5.0**  
System health monitoring with auto-fixes and time-traveling history (`--history`). **15 automated checks**.

**📦 bump-system-version v6.0.0** ⭐ UPGRADED!  
Auto-increment version bumping with pre-flight dashboard and calm releases. Supports `--minor`, `--patch`, `--major` flags.

**🔄 faelight-update v0.4.0**  
Impact analysis for critical package updates.

---

## 🦀 The Rust Toolchain

All **40** core tools are compiled Rust binaries organized in a workspace monorepo - 100% production-ready.

### Core Infrastructure (11 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| dot-doctor | 15-check health monitor | v0.5.0 | ✅ Production |
| faelight-update | Interactive update manager | v0.4.0 | 🚀 Flagship |
| faelight-core | Shared library (config, health, IPC) | v0.1.0 | ✅ Stable |
| core-protect | Immutable filesystem protection | v1.0.1 | ✅ Production |
| safe-update | Smart system updates with snapshots | v1.0.0 | ✅ Production |
| core-diff | Package-aware diff with risk levels | v2.0.0 | ✅ Production |
| dotctl | Central control utility | v2.0.0 | ✅ Production |
| entropy-check | Drift detection system | v1.0.0 | ✅ Production |
| intent-guard | Command safety validation | v1.0.0 | ✅ Production |
| faelight-stow | Package management | v0.3.0 | ✅ Stable |
| faelight-snapshot | BTRFS snapshot manager | v1.0.0 | ✅ Production |

### Faelight Desktop Environment (9 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| faelight-fetch | System info display | v1.0.0 | ✅ Production |
| faelight-bar | Hybrid Wayland bar with integrated launcher | v2.0.0 | 🚀 Flagship |
| faelight-launcher | XDG app launcher with fuzzy search | v3.3.0 | ✅ Production |
| faelight-dmenu | Wayland dmenu replacement | v2.0.0 | ✅ Production |
| faelight-menu | Power menu (lock/logout/shutdown) | v0.7.0 | ✅ Stable |
| faelight-notify | Notification daemon | v0.9.0 | ✅ Stable |
| faelight-lock | Screen locker | v1.0.0 | ✅ Production |
| faelight-dashboard | System dashboard TUI | v1.0.0 | ✅ Production |
| faelight-term | Terminal emulator with color emoji | v9.0.0 | ⚠️ Beta/WIP |

### Development & Workflow (16 tools) ⭐ +2 NEW!

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| intent | Intent Ledger management | v2.0.0 | ✅ Production |
| archaeology-0-core | System history explorer | v1.0.0 | ✅ Production |
| workspace-view | Sway workspace intelligence | v1.0.0 | ✅ Production |
| faelight-git | Git workflow automation | v3.0.0 | ✅ Production |
| faelight-hooks | Git hooks manager (secrets, conflicts) | v1.0.0 | ✅ Production |
| recent-files | Time-based file discovery dashboard | v0.2.0 | ✅ Production |
| profile | System profile switching | v1.0.0 | ✅ Production |
| teach | Interactive learning guide | v1.0.0 | ✅ Production |
| faelight | Unified binary interface | v1.0.0 | ✅ Production |
| keyscan | Keybind conflict detection | v1.0.0 | ✅ Production |
| faelight-zone | Filesystem spatial awareness | v1.1.0 | ✅ Production |
| faelight-fm | Semantic file manager | v1.0.0 | ✅ Production |
| faelight-link | Zone-aware symlink manager | v1.0.0 | ✅ Production |
| faelight-daemon | Universal RPC backend | v0.1.0 | ✅ Stable |
| **alias-audit** | **Alias health checker** | **v1.0.0** | **✅ Production** ⭐ NEW! |
| **bump-tool-version** | **Individual tool version management** | **v1.0.0** | **✅ Production** ⭐ NEW! |

### Version Management (4 tools)

| Tool | Purpose | Version | Status |
|------|---------|---------|--------|
| bump-system-version | Auto-increment release automation | v6.0.0 | 🚀 Flagship ⭐ UPGRADED! |
| faelight-bootstrap | One-command system setup | v1.0.0 | 🚀 Flagship |
| get-version | Package version reader | v2.0.0 | ✅ Production |
| latest-update | Recently updated finder | v2.0.0 | ✅ Production |

### Benefits of Rust:
- ⚡ **Faster** — Compiled binaries vs shell interpretation
- 🔒 **Safer** — Memory safety, no buffer overflows
- ✅ **Type-checked** — Errors caught at compile time
- 🛠️ **Maintainable** — Better error handling, clearer structure
- 🦀 **Modern** — Workspace monorepo with shared dependencies

**Total Lines of Rust:** ~109,000 across all tools

---

## 📊 Project Scale

**Code Statistics (as of v8.7.0):**
```
  Rust source code:    109,000 lines  🦀 (+700)
  Configuration files:   1,061 lines  ⚙️
  Intent documentation:  8,780 lines  🎯
  System guides:         7,269 lines  📚
  ────────────────────────────────────────────
  Total authored:      ~126,100 lines
```

**Philosophy:** Every line intentional. Every decision documented. Every tool understood.

---

## 🔄 Version History

| Version | Date | Milestone |
|---------|------|-----------|
| **v8.7.0** | **2026-01-29** | **alias-audit, bump-tool-version, Starship v2.0** |
| v8.6.0 | 2026-01-29 | bump v6.0.0, faelight-link & fm v1.0.0 production |
| v8.5.0 | 2026-01-26 | Hybrid bar architecture, integrated launcher |
| v8.4.0 | 2026-01-26 | Git hooks management + source-first architecture |
| v8.3.0 | 2026-01-25 | Tool upgrades, terminal perfection |
| v8.2.0 | 2026-01-24 | Spatial awareness, operational dashboard |
| v8.1.0 | 2026-01-23 | Interactive updates, security hardening |
| v8.0.0 | 2026-01-22 | 31 tools production-ready, full audit complete |

[See full version history](CHANGELOG.md)

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
