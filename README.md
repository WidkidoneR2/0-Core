# 🌲 Faelight Forest v8.6.0 - Sway Edition

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v8.6.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Sway](https://img.shields.io/badge/Sway-1.11-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Health](https://img.shields.io/badge/Health-100%25-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v8.6.0 Milestone:** faelight-link v1.0.0 and faelight-fm v1.0.0 - Production Ready 🌲🦀



---

## 🏆 v8.6.0 — Two Tools Graduate to Production

### ✨ What's New

**🎊 Production Graduations**

Two flagship tools transition from beta to production-ready status:

**🔗 faelight-link v1.0.0 (Beta → Production)**
- Complete stow replacement with zone awareness
- Six core commands: stow, unstow, list, status, audit, clean
- Comprehensive health monitoring (100% link health tracking)
- Interactive conflict resolution (backup/skip/overwrite)
- Safe removal with confirmation prompts
- Timestamped automatic backups

**🌲 faelight-fm v1.0.0 (Beta → Production)**
- File operations: copy (y), cut (d), paste (v)
- Real-time status message system
- Zone protection (locked Core enforcement)
- Better than yazi in every way
- Daemon integration for universal backend
- Intent-aware file management

**🚀 bump-system-version v6.0.0 - The Confidence Release**
- Auto-increment flags (--minor, --patch, --major)
- Automatic version calculation removes mental math
- Clear explanations for each increment type
- Both manual and auto-increment modes supported
- Enhanced help with comprehensive examples
- "So easy a 5-year-old could use it"

### 📐 Philosophy Realized

**"From beta to production - the forest matures with intention."**

This release demonstrates:
- Auto-increment removes decision fatigue
- Status messages provide real-time feedback
- Zone protection prevents accidents
- Confirmation prompts maintain control

### 📚 Documentation Excellence

- Updated faelight-link README with production status
- Updated faelight-fm README with production status
- Added production badges to both tools
- "Better than yazi" comparison table
- Comprehensive usage examples

### ✅ 100% System Health

All 14 checks passing — **38 Rust tools in production** (2 new!)

---

## 🌲 Flagship Tools

**🚀 faelight-bar v2.0.0**
Hybrid Wayland bar with integrated application launcher using keyboard mode switching.
Revolutionary single-process architecture with transparent dropdown overlay.

**🔗 faelight-link v1.0.0** ⭐ NEW!
Zone-aware symlink manager with health monitoring. Complete stow replacement.
Six commands: stow, unstow, list, status, audit, clean.

**🌲 faelight-fm v1.0.0** ⭐ NEW!
Semantic file manager with file operations, zone protection, and daemon integration.
Better than yazi: Intent tracking, spatial awareness, universal backend.

**🦀 faelight-term v9.0.0 (Beta / WIP)**
Terminal emulator with color emoji, copy/paste, and mouse selection.
Actively developed — APIs and behavior may change.

**🏥 dot-doctor v0.5.0**
System health monitoring with auto-fixes and time-traveling history (--history).

**📦 bump-system-version v6.0.0** ⭐ UPGRADED!
Auto-increment version bumping with pre-flight dashboard and calm releases.
First tool to use: --minor, --patch, --major flags.

**🔄 faelight-update v0.4.0**
Impact analysis for critical package updates.

---

## 🦀 The Rust Toolchain

All **38** core tools are compiled Rust binaries organized in a workspace - **100% production-ready**.

### Core Infrastructure (11 tools)

Tool | Purpose | Version | Status
-----|---------|---------|-------
dot-doctor | 14-check health monitor | v0.5.0 | ✅ Production
faelight-update | Interactive update manager | v0.4.0 | 🚀 Flagship
faelight-core | Shared library (config, health, IPC) | v0.1.0 | ✅ Stable
core-protect | Immutable filesystem protection | v1.0.1 | ✅ Production
safe-update | Smart system updates with snapshots | v1.0.0 | ✅ Production
core-diff | Package-aware diff with risk levels | v2.0.0 | ✅ Production
dotctl | Central control utility | v2.0.0 | ✅ Production
entropy-check | Drift detection system | v1.0.0 | ✅ Production
intent-guard | Command safety validation | v1.0.0 | ✅ Production
faelight-stow | Package management | v0.3.0 | ✅ Stable
faelight-snapshot | BTRFS snapshot manager | v1.0.0 | ✅ Production

### Faelight Desktop Environment (9 tools)

Tool | Purpose | Version | Status
-----|---------|---------|-------
faelight-fetch | System info display | v1.0.0 | ✅ Production
faelight-bar | Hybrid Wayland bar with integrated launcher | v2.0.0 | 🚀 Flagship
faelight-launcher | XDG app launcher with fuzzy search | v3.3.0 | ✅ Production
faelight-dmenu | Wayland dmenu replacement | v2.0.0 | ✅ Production
faelight-menu | Power menu (lock/logout/shutdown) | v0.7.0 | ✅ Stable
faelight-notify | Notification daemon | v0.9.0 | ✅ Stable
faelight-lock | Screen locker | v1.0.0 | ✅ Production
faelight-dashboard | System dashboard TUI | v1.0.0 | ✅ Production
faelight-term | Terminal emulator with color emoji | v9.0.0 | ⚠️ Beta/WIP

### Development & Workflow (14 tools) ⭐ +2 NEW!

Tool | Purpose | Version | Status
-----|---------|---------|-------
intent | Intent Ledger management | v2.0.0 | ✅ Production
archaeology-0-core | System history explorer | v1.0.0 | ✅ Production
workspace-view | Sway workspace intelligence | v1.0.0 | ✅ Production
faelight-git | Git workflow automation | v3.0.0 | ✅ Production
faelight-hooks | Git hooks manager (secrets, conflicts) | v1.0.0 | ✅ Production
recent-files | Time-based file discovery dashboard | v0.2.0 | ✅ Production
profile | System profile switching | v1.0.0 | ✅ Production
teach | Interactive learning guide | v1.0.0 | ✅ Production
faelight | Unified binary interface | v1.0.0 | ✅ Production
keyscan | Keybind conflict detection | v1.0.0 | ✅ Production
faelight-zone | Filesystem spatial awareness | v1.1.0 | ✅ Production
**faelight-fm** | **Semantic file manager** | **v1.0.0** | **✅ Production** ⭐ NEW!
**faelight-link** | **Zone-aware symlink manager** | **v1.0.0** | **✅ Production** ⭐ NEW!
faelight-daemon | Universal RPC backend | v0.1.0 | ✅ Stable

### Version Management (4 tools)

Tool | Purpose | Version | Status
-----|---------|---------|-------
**bump-system-version** | **Auto-increment release automation** | **v6.0.0** | **🚀 Flagship** ⭐ UPGRADED!
faelight-bootstrap | One-command system setup | v1.0.0 | 🚀 Flagship
get-version | Package version reader | v2.0.0 | ✅ Production
latest-update | Recently updated finder | v2.0.0 | ✅ Production

**Benefits of Rust:**
- ⚡ **Faster** — Compiled binaries vs shell interpretation
- 🔒 **Safer** — Memory safety, no buffer overflows
- ✅ **Type-checked** — Errors caught at compile time
- 🛠️ **Maintainable** — Better error handling, clearer structure
- 🦀 **Modern** — Workspace monorepo with shared dependencies

**Total Lines of Rust:** ~15,800+ across all tools

---

## 📊 Project Scale

**Code Statistics (as of v8.6.0):**
```
  Rust source code:    108,300 lines  🦀 (+1,100)
  Configuration files:   1,061 lines  ⚙️
  Intent documentation:  8,780 lines  🎯
  System guides:         7,269 lines  📚
  ────────────────────────────────────────────
  Total authored:      ~125,400 lines
```

**Philosophy:** Every line intentional. Every decision documented. Every tool understood.

---

## 🔄 Version History

Version | Date | Milestone
--------|------|----------
**v8.6.0** | **2026-01-29** | **bump v6.0.0, faelight-link & fm v1.0.0 production**
v8.5.0 | 2026-01-26 | Hybrid bar architecture, integrated launcher, keyboard mode switching
v8.4.0 | 2026-01-26 | Git hooks management + source-first architecture
v8.3.0 | 2026-01-25 | Tool upgrades, terminal perfection
v8.2.0 | 2026-01-24 | Spatial awareness, operational dashboard, faelight-term foundation
v8.1.0 | 2026-01-23 | Interactive updates, security hardening, ecosystem integration
v8.0.0 | 2026-01-22 | 31 tools production-ready, full audit complete
v7.6.5 | 2026-01-19 | Tool audit quick wins
v7.6.4 | 2026-01-19 | Release automation complete
v7.6.3 | 2026-01-19 | Stow migration complete
v7.0.0 | 2026-01-14 | Architectural excellence

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
