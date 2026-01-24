# 🌲 Faelight Forest v8.1.0 - Sway Edition

> **From chaos to order. From generic to intentional. From dotfiles to 0-core.**

A revolutionary approach to Linux configuration management built on **numbered priority**, **semantic clarity**, and **manual control**.

![Version](https://img.shields.io/badge/Version-v8.1.0-brightgreen)
![Arch](https://img.shields.io/badge/Arch-Linux-blue)
![Sway](https://img.shields.io/badge/Sway-1.11-green)
![Rust](https://img.shields.io/badge/Tools-100%25%20Rust-orange)
![Health](https://img.shields.io/badge/Health-100%25-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

> **v8.1.0 Milestone:** The Garden - Tools growing together: faelight-update v0.2.0, enhanced security, ecosystem integration 🌲🦀

**v8.0.0 Milestone:** Complete tool audit - 30 production-ready Rust tools, 100% system health, philosophy-driven architecture 🌲🦀

---

🏆 v8.1.0 Highlights - "The Garden"
What's New:

✅ faelight-update v0.2.0 - Interactive update manager better than topgrade
✅ Enhanced Security - 14-check health monitoring with UFW, fail2ban, Mullvad, SSH hardening
✅ Ecosystem Integration - 31 tools working together, tools growing with intention
✅ 100% System Health - All checks passing, security hardened
✅ Philosophy-Driven - "A garden requires attention, not automation"

Flagship Tools:

🚀 faelight-update v0.2.0 - Interactive update manager with multi-source detection, TUI selection, health-check-first approach
🚀 bump-system-version v4.0.0 - Complete release automation with CHANGELOG generation, intent tracking, git integration
🦀 faelight-bootstrap v1.0.0 - One-command Arch Linux installation with 7 interactive phases
🏥 dot-doctor v0.6.0 - 14-check health monitoring with security hardening verification
🎓 teach v1.0.0 - Interactive learning system with quiz mode and achievements


🦀 The Rust Toolchain
All 31 core tools are compiled Rust binaries organized in a workspace - 100% production-ready.
Core Infrastructure (11 tools)
ToolPurposeVersionStatusdot-doctor14-check health monitorv0.6.0✅ Productionfaelight-updateInteractive update managerv0.2.0🚀 Flagshipfaelight-coreShared library (config, health, IPC)v0.1.0✅ Stablecore-protectImmutable filesystem protectionv1.0.1✅ Productionsafe-updateSmart system updates with snapshotsv1.0.0✅ Productioncore-diffPackage-aware diff with risk levelsv2.0.0✅ ProductiondotctlCentral control utilityv2.0.0✅ Productionentropy-checkDrift detection systemv1.0.0✅ Productionintent-guardCommand safety validationv1.0.0✅ Productionfaelight-stowPackage managementv0.3.0✅ Stablefaelight-snapshotBTRFS snapshot managerv1.0.0✅ Production
Faelight Desktop Environment (8 tools)
ToolPurposeVersionStatusfaelight-fetchSystem info displayv1.0.0✅ Productionfaelight-barWayland status bar (Sway IPC)v0.9.0✅ Stablefaelight-launcherXDG app launcher with fuzzy searchv3.3.0✅ Productionfaelight-dmenuWayland dmenu replacementv2.0.0✅ Productionfaelight-menuPower menu (lock/logout/shutdown)v0.7.0✅ Stablefaelight-notifyNotification daemonv0.9.0✅ Stablefaelight-lockScreen lockerv1.0.0✅ Productionfaelight-dashboardSystem dashboard TUIv1.0.0✅ Production
Development & Workflow (8 tools)
ToolPurposeVersionStatusintentIntent Ledger managementv2.0.0✅ Productionarchaeology-0-coreSystem history explorerv1.0.0✅ Productionworkspace-viewSway workspace intelligencev1.0.0✅ Productionfaelight-gitGit workflow automationv2.1.0✅ ProductionprofileSystem profile switchingv1.0.0✅ ProductionteachInteractive learning guidev1.0.0✅ ProductionfaelightUnified binary interfacev1.0.0✅ ProductionkeyscanKeybind conflict detectionv1.0.0✅ Production
Version Management (4 tools)
ToolPurposeVersionStatusbump-system-versionComplete release automationv4.0.0🚀 Flagshipfaelight-bootstrapOne-command system setupv1.0.0🚀 Flagshipget-versionPackage version readerv2.0.0✅ Productionlatest-updateRecently updated finderv2.0.0✅ Production
Benefits of Rust:

⚡ Faster — Compiled binaries vs shell interpretation
🔒 Safer — Memory safety, no buffer overflows
✅ Type-checked — Errors caught at compile time
🛠️ Maintainable — Better error handling, clearer structure
🦀 Modern — Workspace monorepo with shared dependencies

Total Lines of Rust: ~15,000+ across all tools

🏗️ Directory Structure
Numbered Hierarchy
~/0-core/     🔒 Configuration (this repo) - MOST CRITICAL
~/1-src/      📁 Source code & projects
~/2-projects/ 💼 Active work
~/3-archive/  💎 Completed/archived
~/4-media/    🎬 Media files
~/secrets/    🔐 Never committed
Package Organization (Stow-Based)
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
Deployment: cd ~/0-core/stow && stow -t ~ package-name

🏥 Health Monitoring
bashdoctor   # Full 14-check diagnostic (100% health)
14 Health Checks:

✅ Stow Symlinks - All 12/12 packages
✅ System Services - faelight-bar, faelight-notify
✅ Broken Symlinks - None found
✅ Yazi Plugins - All 4 installed
✅ Binary Dependencies - All 15 present
✅ Git Repository - Clean, all pushed
✅ Theme Packages - Faelight Forest present
✅ Scripts - All executable
✅ Package Metadata - All .dotmeta files
✅ Intent Ledger - System functional
✅ Profile System - Working correctly
✅ Faelight Config - All valid
✅ Sway Keybinds - No conflicts (116 bindings)
✅ Security Hardening - UFW, fail2ban, Mullvad VPN, SSH


🔄 Version History
VersionDateMilestonev8.1.02026-01-23Interactive updates, security hardening, ecosystem integrationv8.0.02026-01-2230 tools production-ready, full audit completev7.6.52026-01-19Tool audit quick winsv7.6.42026-01-19Release automation completev7.6.32026-01-19Stow migration completev7.0.02026-01-14Architectural excellencev6.0.02026-01-09Sway editionv5.0.02026-01-05First Rust tool
See full version history
---

## 🌟 Credits

- **Inspiration:** [Omarchy](https://github.com/omarchy) — the starting point
- **Philosophy:** Manual control, explicit intent, human comprehension
- **Tools:** Rust, Sway, Neovim, Zsh, Starship

---

## 📄 License

MIT — Use freely, learn deeply, configure intentionally.

---

> *"The audit is complete. Every tool documented, tested, and production-ready."* 🌲🦀

