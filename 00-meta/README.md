# 🌲 Faelight Forest - 0-Core Configuration System

**A production-grade Linux environment built with 40 Rust tools and intentional computing principles.**

[![Version](https://img.shields.io/badge/version-8.8.0-green.svg)](https://github.com/WidkidoneR2/0-Core)
[![Health](https://img.shields.io/badge/health-100%25-brightgreen.svg)](https://github.com/WidkidoneR2/0-Core)
[![Rust](https://img.shields.io/badge/rust-109k%20lines-orange.svg)](https://github.com/WidkidoneR2/0-Core)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **v9.1.0 Milestone:** 75% Path Resilience - 10 tools migrated, faelight-term font fixes 🌲🦀

**v9.0.0 Milestone:** PATH RESILIENCE FOUNDATION - 60% of tools migrated to centralized path management. Enhanced monitoring with 19 health checks including self-aware Path Resilience tracking. The Guardian (core-protect) deployed. System watches its own evolution. 🌲🦀

**v8.9.0 Milestone:** Numbered Gravity Path Hardening - Critical shutdown fix, 4 tools hardened with paths modules, Prompt 2.0 🌲🦀

---

## 🎯 What is 0-Core?

0-Core (Faelight Forest) is a **numbered gravity system** for managing Linux configurations with 40 production Rust tools. Built on vanilla Arch + Sway, it emphasizes manual control, intentional decisions, and comprehensive health monitoring.

**Presented to Linus Torvalds, January 2026** - Feedback incorporated into numbered gravity architecture.

---

## 🏗️ Numbered Gravity Architecture

The repository follows a **directional growth pattern** where numbered directories (00-04) represent operational structure:
```
0-core/
├── 00-meta/              # Identity & Lineage
│   ├── VERSION           # System version (semver)
│   ├── CHANGELOG.md      # Complete history
│   ├── README.md         # This file
│   ├── PHILOSOPHY.md     # Core principles
│   └── TOOLS.md          # Tool reference
│
├── 01-registry/          # Canonical Lists (Source of Truth)
│   ├── tools.toml        # 40 Rust tools registry
│   ├── aliases.toml      # 301 shell aliases
│   ├── zones.toml        # Zone definitions
│   ├── profiles.toml     # System profiles
│   └── packages.txt      # System packages
│
├── 02-rules/             # Enforcement & Safety
│   ├── hooks/            # Git hooks (gitleaks, conflict detection)
│   ├── security/         # Hardening configs
│   └── doctor/           # Health check definitions
│
├── 03-interfaces/        # Human-Editable Surfaces
│   ├── stow/             # 12 config packages (GNU Stow)
│   ├── profiles/         # Profile configs
│   └── systemd-user/     # User services
│
├── 04-runtime/           # Ephemeral Data
│   ├── logs/             # System logs
│   ├── backups/          # BTRFS snapshots
│   └── target/           # Rust build artifacts
│
├── intents/              # Intent Ledger (8,780 lines)
│   ├── complete/         # 28 completed intents
│   ├── decisions/        # Architectural decisions
│   ├── future/           # Planned work
│   └── incidents/        # Problem tracking
│
├── docs/                 # Documentation (7,269 lines)
├── rust-tools/           # 40 Production Tools (109k lines)
├── scripts/              # Compiled binaries
├── Cargo.toml            # Workspace root
└── Cargo.lock
```

**Key Principles:**
- **Reading order is obvious** (00 → 04)
- **Growth pressure is directional** (numbered hierarchy)
- **Authority vs Interface separation** (rules vs surfaces)
- **No archives polluting present** (clean structure)

---

## 🦀 Production Tools (40 Total)

### Core Infrastructure
- **faelight** - Unified CLI frontend
- **faelight-core** - Shared library
- **dot-doctor** - Health engine (15 checks, 100% health)
- **intent** - Intent ledger management

### Desktop Environment
- **faelight-bar** - Custom Wayland bar (70-90% less CPU than Waybar)
- **faelight-launcher** - Fuzzy search app launcher
- **faelight-lock** - Screen locker
- **faelight-menu** - Power menu
- **faelight-fm** - File manager (better than yazi)
- **faelight-term** - Custom terminal emulator

### Automation & Updates
- **faelight-update** - Better than topgrade (9 package sources)
- **faelight-bootstrap** - One-command system setup
- **faelight-snapshot** - BTRFS snapshot management
- **safe-update** - Safe update wrapper

### Development
- **faelight-git** - Git workflow automation + risk scoring
- **faelight-hooks** - Git hook manager
- **alias-audit** - Alias coverage checker (301 aliases)
- **archaeology-0-core** - System evolution analysis

### Versioning
- **bump-system-version** - System version incrementer
- **bump-tool-version** - Tool version incrementer
- **get-version** - Version retriever

### Management
- **faelight-link** - Zone-aware symlink manager
- **faelight-zone** - Zone management
- **profile** - System profile switcher (4 profiles)

[See complete tool list in 01-registry/tools.toml]

---

## 📊 System Health
```bash
$ doctor
🏥 0-Core Health Check - Faelight Forest v9.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Stow Symlinks: All 12/12 packages properly stowed
✅ System Services: All 2/2 services running
✅ Broken Symlinks: No broken symlinks found
✅ Yazi Plugins: All 4 plugins installed
✅ Binary Dependencies: All 15 binaries found
✅ Git Repository: Working tree clean, all commits pushed
✅ Theme Packages: 1/1 theme packages present
✅ Scripts: All scripts present and executable
✅ Intent Ledger: 28 intents (11 complete, 6 planned)
✅ Profile System: Profile system OK (current: default)
✅ Faelight Config: All config files valid
✅ Sway Keybinds: 116 unique keybindings, no conflicts
✅ Security Hardening: 3 protections active
✅ Alias Coverage: All 37 tools have aliases (301 total)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ System healthy! All checks passed! 🌲
Health: 100%
```

**15 automated checks** covering:
- Configuration integrity
- Service health
- Security hardening
- Git repository state
- Package consistency

---

## 🚀 Quick Start
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core

# Bootstrap system (one command!)
./scripts/faelight-bootstrap

# Check health
doctor

# Update everything (better than topgrade)
faelight-update
```

---

## 🎨 Philosophy

**Manual Control Over Automation**
- Every action requires explicit confirmation
- Pre-flight dashboards show what will happen
- Health checks verify system state

**Intentional Computing**
- All decisions documented in Intent Ledger
- Architectural choices have traceable rationale
- System evolution is deliberate, not accidental

**Understanding Over Convenience**
- Build from source when possible
- Custom tools > pre-built solutions
- 40 Rust tools written to understand systems deeply

[Read full philosophy in PHILOSOPHY.md]

---

## 📈 Statistics

- **40 Rust Tools** (109,000 lines of code)
- **301 Shell Aliases** (100% coverage)
- **28 Completed Intents** (8,780 lines documented)
- **15 Health Checks** (100% passing)
- **116 Keybindings** (0 conflicts)
- **12 Stow Packages** (all properly linked)
- **4 System Profiles** (default, work, gaming, low-power)

---

## 🏆 Recent Achievements

### v9.1.0 (Three Quarters Complete) 🎊
- **75% Path Resilience Achieved!** (30/40 tools)
- Migrated 10 tools in Session 7: keyscan, faelight-stow, teach, faelight-bootstrap, faelight-dmenu, faelight-lock
- **faelight-term v10.1.0:** Fixed font baseline rendering - better than foot!
- Fixed scrollback copy/paste in terminal
- System self-awareness: tracks own evolution (75%)
- **THREE QUARTERS COMPLETE** 💎

### v9.0.0 (Path Resilience Foundation) 🎊
- **MAJOR MILESTONE: 60% Path Resilience Achieved!**
- Enhanced monitoring: 19 health checks (was 15) with self-aware tracking
- **NEW:** Path Resilience check - system tracks its own evolution (24/40 tools)
- **NEW:** Rust Toolchain check - build environment verification
- **NEW:** Disk Space check - high-severity warnings at 90%
- **NEW:** Tool Installation check - key tools presence verification
- Deployed The Guardian (core-protect v2.0.0) - immutable protection
- Migrated 8 more tools: dotctl, profile, faelight-zone, safe-update, workspace-view, entropy-check, recent-files, core-protect
- System health: 94% (honest tracking with Path Resilience warning)
- **THE SYSTEM WATCHES ITSELF EVOLVE** 💎
## 🌲 Zone System
```
~/
├── 0-core/     (Core)   🔒 Protected, version controlled
├── 1-src/      (SRC)    🌲 Source code, projects in development
├── 2-projects/ (Proj)   📁 Active work, client projects
├── 3-archive/  (ARCH)   📦 Completed work, historical data
└── Downloads/  (SCR)    💾 Temporary, scratch space
```

Zones provide **spatial organization** with protection levels and semantic meaning.

---

## 📚 Documentation

- [ARCHITECTURE.md](../docs/ARCHITECTURE.md) - System design
- [PHILOSOPHY.md](PHILOSOPHY.md) - Core principles
- [TOOLS.md](TOOLS.md) - Tool reference
- [WORKFLOWS.md](../docs/WORKFLOWS.md) - Common tasks
- [BUILD.md](../docs/BUILD.md) - Building from source

---


## 📜 License

MIT License - See LICENSE file for details

---

**Built with intentionality. Powered by Rust. Guided by philosophy.** 🌲

