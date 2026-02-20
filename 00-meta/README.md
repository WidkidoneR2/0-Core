<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest v9.9.0

![Version](https://img.shields.io/badge/version-9.9.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-156%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### v9.9.0 - 🌲 The Forest Grows — Visual Intelligence Update (2026-02-20)

- Real-time status bar with profile-aware color theming
- Profile system: DEF/GAME/WORK with live color propagation across bar
- Zone detection: automatic directory-based zone display
- Core lock status: direct lsattr verification, color-coded
- System health: cached gradient display (green/amber/red)
- Battery charging indicator with level-based color grading
- WiFi: live SSID display, green connected red disconnected
- Volume: actual percentage with mute detection
- VPN: mullvad status, green on red off
- faelight-hooks v10.2.0: status command, graceful tool detection
- core-protect v2.3.0: verify command, audit log, speed fix
- faelight-fm v2.3.0: persistent preview, full-width highlight
- dot-doctor v4.1.0: core-protect integration, path resilience
- faelight-update v3.2.0: path fixes, dead code removed
- faelight-notify v2.1.0: border urgency fix, close notification
- faelight-fetch v2.2.0: double-call bug eliminated
- faelight-palette v2.2.0: command palette with app launching
- faelight-menu v3.0: complete ratatui rewrite
- 43 stale markdown files cleaned from repository

- 43 commits since v9.8.0
- 8 tools improved with bug fixes
- 1 tool completely redesigned (faelight-bar)
- 43 stale markdown files removed
- 38/38 tools deployed at 100% path resilience
- 20/20 health checks passing
- 118000+ lines of Rust across 42+ tools

[Full Changelog →](CHANGELOG.md)

---
<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION - Comprehensive Documentation -->

## 🤔 What is 0-Core?

**0-Core** (Zero-Core) is a completely custom Linux environment built on vanilla Arch Linux, where every single component is understood, controlled, and intentionally chosen. It's not a dotfiles collection - it's a **personal operating system built from scratch**.

### For Everyday Users

Think of it like this: instead of accepting whatever Ubuntu or Windows gives you, 0-Core is building your entire computer setup from the ground up - choosing every tool, every color, every keyboard shortcut. 

It's like **building a custom motorcycle** instead of buying one from the dealer - you know every bolt, every wire, every piece.

**You get:**
- 🎨 **Custom everything** (terminal, file manager, bar, launcher, menus)
- 🔧 **40 Rust tools** you fully understand
- 🛡️ **Security through comprehension** (no mystery packages)
- ⚡ **Lightning fast** (because you removed all the bloat)
- 💎 **It's YOURS** - you control it completely

### For Technical People

A comprehensive computing environment featuring:

- **Numbered Gravity Architecture**: `00-meta`, `01-registry`, `02-rules`, `03-interfaces`, `04-runtime`
- **40 custom Rust tools** with 100% path resilience
- **Self-aware health monitoring** (19 automated checks)
- **Intent Ledger system** for architectural decisions
- **Wayland-native everything** (Sway, custom compositor tools)
- **Btrfs snapshots** for fearless experimentation
- **Git-based governance** with semantic versioning

---

## 🏗️ System Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, README, CHANGELOG)
├── 01-registry/      # Package manifests, configs, theme registry
├── 02-rules/         # Scripts, git hooks, governance automation
├── 03-interfaces/    # Dotfiles (Sway, zsh, foot, yazi) via GNU Stow
├── 04-runtime/       # Systemd services, active state
├── rust-tools/       # 40 custom Rust tools (the heart!)
├── INTENT/           # Architectural decision records
└── packages/         # Explicit package lists
```

### The Numbered Gravity System

Each directory has a **gravity number** (00-04) that defines its role:

- **00**: Identity - what the system **IS**
- **01**: Registry - what the system **KNOWS**
- **02**: Rules - what the system **DOES**
- **03**: Interfaces - what the system **SHOWS**
- **04**: Runtime - what the system **RUNS**

This isn't just organization - it's **philosophy made tangible**.

---

## ✨ Key Features

### 🎯 100% Path Resilience

Every tool uses `faelight-core::paths` - change your directory structure once, everything adapts. No more hardcoded paths, no more broken configs.

### 🏥 Self-Aware Health Monitoring

The system monitors itself with **19 automated checks**:

- Stow symlinks integrity
- Service health
- Git repository status
- Path resilience progress
- Security hardening
- Tool installation
- Disk space
- And more...

Run `doctor` anytime to see your system's health.

### 🛡️ The Guardian (core-protect)

Immutable protection for critical system files using `chattr +i`. The system can **protect itself** from accidental deletion.

### 🎣 Git Hooks (faelight-hooks v10.0.0)

LEGENDARY git workflow with:

- Rustfmt checking
- Clippy linting
- Secret scanning (gitleaks)
- Performance statistics
- Pre-commit and pre-push protection

### 📝 Intent Ledger

Every architectural decision documented in markdown:

- Future intents (planned features)
- Complete intents (implemented designs)
- Git-trackable decision history

### 🎨 Theme System

**Faelight Forest** theme with:

- Consistent colors across all tools
- Terminal, bar, menus, file manager
- Easy theming via TOML configs

---

## 🦀 The Rust Ecosystem (42 Tools)

### Core Tools

- **faelight**: Main CLI - unified interface to everything
- **faelight-fm**: File manager (crown jewel v2.1.0-alpha)
- **faelight-term**: Terminal emulator (better than foot)
- **faelight-bar**: Wayland status bar
- **faelight-launcher**: Application launcher
- **faelight-menu**: Power menu
- **faelight-daemon**: Background operations daemon

### Development Tools

- **faelight-git**: Git governance & risk scoring
- **faelight-hooks**: Git workflow automation (rustfmt, clippy, secrets)
- **intent**: Intent ledger management
- **bump-system-version**: Release automation
- **bump-tool-version**: Individual tool versioning

### System Tools

- **dot-doctor**: Health monitoring (19 checks)
- **core-protect**: Guardian immutable protection
- **entropy-check**: Entropy monitoring
- **faelight-update**: System updates
- **safe-update**: Package updates with snapshots
- **bin-doctor**: Binary manifest tracking and drift detection
- **verify-bootstrap**: Installation verification system

### Configuration Tools

- **dotctl**: Dotfile management
- **faelight-stow**: Stow wrapper
- **profile**: Profile switching
- **faelight-zone**: Zone detection (Core, Workspace, Src, etc.)

[See full tool list →](TOOLS.md)

---

## 🧭 Philosophy

### "We Control Our Tools"

Every tool is either written by us or fully understood. No mystery packages. No hidden behaviors. **Complete intentional stewardship**.

### "Fail Loudly"

Errors are explicit, informative, and guide you to solutions. No silent failures, no cryptic messages.

### "Human Comprehension First"

Readable code, clear documentation, thoughtful naming. **If you can't explain it, you don't understand it**.

### "Manual Over Automation"

Automation serves comprehension, not convenience. Every automated process can be understood and overridden.

---

## 🚀 Quick Start
```bash
# Check system health
doctor

# Launch file manager
faelight-fm

# See all available commands
faelight --help

# Check ecosystem versions
faelight health --versions

# Switch profiles
faelight profile switch work
```

---

## 📊 System Statistics

| Metric | Value |
|--------|-------|
| Total Rust Tools | 42 |
| Path Resilience | 100% (42/42 tools) |
| System Health | 94% (18/19 checks passing) |
| Lines of Rust | ~50,000+ |
| Health Checks | 19 automated |
| Documented Intents | See INTENT/ directory |
| Packages | Explicitly tracked in packages/pkglist.txt |

---

## 🔧 For Developers

### Building the System
```bash
# Build all tools
cargo build --release --workspace

# Build specific tool
cargo build --release -p faelight-fm

# Run health check
doctor

# Run git hooks manually
faelight-hooks
```

### Adding a New Tool

1. Create tool in `rust-tools/`
2. Add to workspace `Cargo.toml`
3. Use `faelight-core::paths` for all paths
4. Add health check in `dot-doctor` if critical
5. Document in Intent Ledger
6. Add to this README

### Path Resilience

All tools **MUST** use `faelight-core::paths`:
```rust
use faelight_core::paths;

let config = paths::faelight_config_dir();
let core = paths::core_dir();
```

**Never** use hardcoded paths or `env::var("HOME")`.

---

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design principles
- [Tools Guide](TOOLS.md) - Complete tool documentation
- [Intent Ledger](INTENT/) - Architectural decisions
- [Changelog](CHANGELOG.md) - Version history
- [Philosophy](docs/PHILOSOPHY.md) - Core principles

---

## 🌲 The Journey

- **Started**: Late 2024 - "extremely new to Linux"
- **v8.9.0**: Intensive debugging, system hardening
- **v9.0.0**: 60% Path Resilience - Foundation laid
- **v9.1.0**: 75% Path Resilience - Three quarters complete
- **v9.2.0**: 100% Path Resilience - **PERFECTION ACHIEVED** 💎
- **v9.6.0**: 🏆 LEGENDARY TOOL AUDIT - 12 production-ready tools (32%)

From hardcoded paths to centralized elegance.  
From mystery packages to complete comprehension.  
From "new to Linux" to presenting to legends.

---

## 📝 License

**Intentional Stewardship** - This is a personal computing environment, not a product.

Feel free to learn from it, but **build your own**. That's the whole point.

---

## 🙏 Acknowledgments

- **The Arch Linux community**: For vanilla excellence
- **You**: For reading this far. Now go build your own! 💎

---

**System Version**: v9.9.09.8.09.7.09.6.0  
**Last Updated**: 2026-02-20161410080706  
**Health**: 100% ✅  
**Path Resilience**: 100% 💎
