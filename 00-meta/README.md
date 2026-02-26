<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest v10.2.0

![Version](https://img.shields.io/badge/version-10.2.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-85%25-green?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### v10.2.0 - 🌲 Two Tools Out of WIP — faelight-term & faelight-browser production ready (2026-02-26)

- ENDfaelight-term v10.3.0 — OUT OF WIP
- - vte crate parser (same as Alacritty/Zellij)
- - Alternate screen buffer (nvim/fm full size)
- - Dynamic resize via TIOCSWINSZ
- - Backspace, delete, escape, return key handling
- - DSR cursor position response (atuin inline history)
- - Production ready — actively replacing foot
- faelight-browser v0.4.0 — OUT OF WIP
- - w3m-style inline link navigation
- - Forward/back navigation (Shift+F/B)
- - Reader mode strips nav/ads (Shift+R)
- - In-page search Ctrl+F with match count
- - Unicode panic fixes
- - Brave web search integration
- System
- - 9 new tools: atuin, tokei, hyperfine, tealdeer, ouch, difftastic, cargo-flamegraph, bottom, onefetch
- - Removed: qutebrowser, fuzzel, mako (replaced by faelight tools)
- - Added helix to packages
- - Fixed infinite loop in update domain
- - Intents 094/095 marked complete
- - 19 commits since v10.1.0

- Tools: 43 active (2 out of WIP this release)
- Commits: 1260 total (19 since v10.1.0)
- Intents: 62 complete
- Packages: 97 system packages
- Health: 95%
- Lines of Rust: 30,976+ across 205 files

[Full Changelog →](CHANGELOG.md)

---
<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION - Comprehensive Documentation -->

## 🤔 What is 0-Core?

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch**.

### For Everyday Users

Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, file manager)
- 🦀 34 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (3ms cold start, no bloat)
- 💎 Complete ownership and control

### For Technical People

- **`core` v2.0.0** — single orchestrator binary with 15 native Rust domains
- **Capability-gated dispatch** — every command checks permissions before executing
- **Runtime locking** and JSONL capability audit logging
- **22-check health monitoring** system
- **Intent Ledger** for all architectural decisions
- **Wayland-native** (Sway, custom compositor tools)

---

## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── engine/           # core v2.0.0 — single orchestrator binary (Rust)
│   └── src/domains/  # 15 native Rust domains
├── rust-tools/       # 34 custom Rust tools
├── 03-interfaces/    # Dotfiles (Sway, zsh, foot, yazi) via GNU Stow
├── scripts/          # Thin wrappers → core v2 + compiled binaries
├── intents/          # Architectural decision records
├── runtime/          # SQLite state, capability logs, process locks
└── registry/         # Zero-logic TOML declarations
```

### The `core` Orchestrator
```
core <domain> <command>

Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update
```

Every domain call is capability-gated, logged, and lock-protected.

### Layer Model

| Layer | Name | Purpose |
|---|---|---|
| 0 | Substrate | Kernel, Wayland, Sway — untouched |
| 1 | Core Engine | `core` binary — single surface |
| 2 | Registry | Zero-logic TOML declarations |
| 3 | Policy | Constraints, no execution |
| 4 | Runtime | All mutable state — gitignored |
| 5 | Adapters | Thin translation only |

---

## ✨ Key Features

### 🏥 Self-Aware Health Monitoring (22 Checks)
```bash
doctor        # Run all 22 health checks
```
Checks: stow symlinks, services, broken symlinks, yazi plugins, binary deps,
git status, themes, scripts, intents, profiles, config files, keybinds,
security hardening, security audit, alias coverage, rust toolchain,
disk space, tool installation, path resilience, archaeology, core protection.

### 🛡️ Core Protection
```bash
core-protect lock      # Immutable flag on ~/0-core
core-protect unlock    # Remove for editing
core-protect status    # Check current state
```

### 🔒 Capability Model
Every domain declares required capabilities at dispatch time:
```
doctor    → OrchestratorAccess, FilesystemReadHome
security  → OrchestratorAccess, FilesystemReadHome, NetworkQuery
git       → FilesystemReadHome, FilesystemWriteHome, SpawnProcess
lock      → ControlSway
update    → FilesystemReadHome, ExecutePacman, SpawnProcess
```
All capability usage logged to `runtime/logs/capabilities.jsonl`.

### 🎣 Git Governance
Pre-commit hooks: rustfmt, clippy, secret scanning, merge conflict detection.
```bash
faelight-git commit    # Intent-aware commits
faelight-git risk      # Risk score before pushing
fg sync                # Pull + push workflow
```

### 📝 Intent Ledger
```bash
intent list            # All intents
intent list --active   # In-progress
intent show 092        # Specific intent
intent stats           # Ledger overview
```

### 🌡️ Configuration Drift Detection
```bash
core doctor entropy --baseline   # Snapshot current state
core doctor entropy              # Check for drift
core doctor entropy --trends     # 30-day history
```

---

## 🦀 The Rust Ecosystem (34 Tools)

| Category | Tools |
|---|---|
| **Orchestrator** | `core` (v2.0.0 — 15 domains) |
| **UI** | `faelight-bar`, `faelight-fm`, `faelight-term`, `faelight-palette`, `faelight-menu` |
| **Git** | `faelight-git`, `faelight-hooks` |
| **System** | `core-protect`, `faelight-update`, `safe-update`, `faelight-sandbox`, `faelight-snapshot` |
| **Dev** | `bump-system-version`, `bump-tool-version`, `get-version`, `core-diff` |
| **Shell** | `dotctl`, `profile`, `intent`, `faelight-zone`, `faelight-link`, `faelight-lock`, `faelight-notify`, `faelight-fetch` |
| **Audit** | `alias-audit`, `bin-doctor`, `entropy-check`, `archaeology-0-core` |
| **Bootstrap** | `faelight-bootstrap`, `faelight-daemon`, `faelight-cleanup`, `keyscan`, `teach`, `intent-guard`, `workspace-view`, `verify-bootstrap` |

> `alias-audit`, `bin-doctor`, `entropy-check` are also natively absorbed into `core doctor` — standalone binaries kept for direct use.

[See full tool list →](TOOLS.md)

---

## 🧭 Philosophy

**"We Control Our Tools"** — Every tool written or fully understood. No mystery packages.

**"Fail Loudly"** — Errors are explicit and guide you to solutions. No silent failures.

**"Manual Over Automation"** — Automation serves comprehension, not convenience.

**"Understanding Over Convenience"** — If you can't explain it, you don't own it.

**"Design for Recovery"** — `rm -rf runtime/` is always safe. Full reset, no data loss.

---

## 🚀 Quick Reference
```bash
doctor                          # System health (22 checks)
core security scan              # Security audit
core git status                 # Git + risk score
core workspace recent today     # Recent files
core intent list --active       # Active intents
core doctor entropy             # Config drift check
0c                              # cd ~/0-core
core-protect lock               # Lock before shutdown
```

---

## 🔧 For Developers

### Building
```bash
cargo build --release -p core       # Build orchestrator
cargo build --release --workspace   # Build all tools
doctor                              # Verify health
```

### Adding a Domain

1. Add module to `engine/src/domains/`
2. Wire in `engine/src/app/dispatcher.rs` with capability gate
3. Add CLI commands to `engine/src/cli/`
4. Create thin wrapper in `scripts/`
5. Document in Intent Ledger

---

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Tools Guide](TOOLS.md)
- [Intent Ledger](intents/)
- [Changelog](CHANGELOG.md)
- [Philosophy](docs/PHILOSOPHY.md)

---

## 🌲 The Journey

| Version | Milestone |
|---|---|
| v1.0.0 | "Extremely new to Linux" — first Arch install |
| v8.0.0 | Stow symlink fix, system hardening |
| v9.0.0 | 60% path resilience |
| v9.2.0 | 100% path resilience — 40 tools |
| v9.6.0 | Legendary tool audit — production-ready |
| v9.9.0 | The Forest Grows — Visual Intelligence Update |
| v10.0.0 | **core v2.0.0 — migration complete** 🏛️ |
| v10.1.0 | **The Forest Matures — all 6 phases done** 🌲 |

From hardcoded paths to centralized elegance.
From 40 separate binaries to one orchestrator.
From "new to Linux" to presenting to legends.

---

## 📝 License

**Intentional Stewardship** — This is a personal computing environment, not a product.
Feel free to learn from it, but **build your own**. That's the whole point.

---

## 🙏 Acknowledgments

- **The Arch Linux community**: For vanilla excellence
- **You**: For reading this far. Now go build your own! 💎

---
