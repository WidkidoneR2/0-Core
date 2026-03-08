<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest v10.6.0

![Version](https://img.shields.io/badge/version-10.6.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### v10.6.0 - 🌲 The Judgment Layer (2026-03-08)

- 104 — faelight-wallpaper — Rust Wallpaper Daemon
- 105 — core intent dashboard — Terminal Intent Overview
- 107 — faelight-search — Unified Rust Search
- 110 — core why visual — Workspace Topology in Event Ledger
- 111 — faelight-bar — Fractional Scaling Support (wp_fractional_scale_v1)
- 113 — Core v5 — The Intelligent System
- core: Core v5 complete — all 5 phases, ledger foundation, forecasting, causality, patterns, compositor intelligence
- core: Core v5 Phase 4 — pattern recognition, correlate domains, suggest based on learned history
- core: Core v5 Phase 3 — causality engine, why health-since, causal domain analysis, causal chain
- core: Core v5 Phase 2 — forecast line integrated into core doctor output
- core: Core v5 Phase 1 — ledger foundation, indexed queries, stats/query/export commands

- Commits: 1363
- Tools: 50 deployed
- Health: 95%
- Intents: 69 complete

[Full Changelog →](CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION - Comprehensive Documentation -->

## 🤔 What is 0-Core?

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch**.

### For Everyday Users

Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, login screen, notifications)
- 🦀 47 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat, no hidden automation)
- 💎 Complete ownership and control

### For Technical People

- **`core` v3.0.0** — single orchestrator binary with 15+ native Rust domains
- **Capability-gated dispatch** — every command checks permissions before executing
- **Core v4** — checkpoint/restore, intent discipline, security debt tracking, analytics
- **faelight-release** — intelligent release system with generation model, rollback, learning layer
- **22-check health monitoring** system
- **Intent Ledger** — 114 architectural decisions, fully documented
- **Wayland-native** — Niri compositor, custom Rust toolchain all the way down

---

## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── engine/           # core v3.0.0 — single orchestrator binary (Rust)
│   └── src/domains/  # 15+ native Rust domains
├── rust-tools/       # 47 custom Rust tools
├── 03-interfaces/    # Dotfiles (Niri, Sway, zsh, foot, yazi) via GNU Stow
├── scripts/          # Thin wrappers → core + compiled binaries
├── intents/          # Architectural decision records (114 intents)
├── runtime/          # SQLite state, capability logs, checkpoints
└── registry/         # Zero-logic TOML declarations
```

### The `core` Orchestrator
```
core <domain> <command>

Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update, checkpoint, simulate, plugins
```

Every domain call is capability-gated, logged, and lock-protected.

### Layer Model

| Layer | Name | Purpose |
|---|---|---|
| 0 | Substrate | Kernel, Wayland, Niri — understood, not opaque |
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

### 📸 Core v4 — Checkpoint & Recovery
```bash
cpc <name>                    # Create checkpoint with health snapshot
core checkpoint list          # List all checkpoints
core checkpoint diff <name>   # Compare checkpoint to current state
core checkpoint restore       # Advisory restore guidance
```

### 🎯 Intent Discipline
```bash
cistart 101                   # Start intent — auto-checkpoint
cicomplete 101                # Mark complete — log outcome
cis                           # Current intent status
cilist                        # Full intent ledger
```

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
lock      → ControlWM
update    → FilesystemReadHome, ExecutePacman, SpawnProcess
```
All capability usage logged to `runtime/logs/capabilities.jsonl`.

### 🎣 Git Governance
Pre-commit hooks: rustfmt, clippy, secret scanning, merge conflict detection.
```bash
faelight-git commit    # Intent-aware commits with risk scoring
faelight-git risk      # Risk score before pushing
fg sync                # Pull + push workflow
```

### 📝 Intent Ledger
```bash
cilist                           # All intents
intent show 099                  # Specific intent
intent stats                     # Ledger overview + velocity
core intent burndown             # Visual progress chart
```

### 🌡️ Configuration Drift Detection
```bash
core doctor entropy --baseline   # Snapshot current state
core doctor entropy              # Check for drift
core doctor entropy --trends     # 30-day history
```

---

## 🦀 The Rust Ecosystem (47 Tools)

| Category | Tools |
|---|---|
| **Orchestrator** | `core` (v3.0.0 — 15+ domains) |
| **UI** | `faelight-bar`, `faelight-term`, `faelight-palette`, `faelight-menu`, `faelight-notify`, `faelight-login` |
| **Clipboard** | `faelight-clipboard` |
| **Intelligence** | `faelight-forecast`, `faelight-pulse`, `faelight-niri-bridge` |
| **Browser** | `faelight-browser` |
| **Git** | `faelight-git`, `faelight-hooks` |
| **System** | `core-protect`, `faelight-update`, `safe-update`, `faelight-sandbox`, `faelight-snapshot` |
| **Dev** | `faelight-release`, `bump-tool-version`, `get-version`, `core-diff` |
| **Shell** | `dotctl`, `profile`, `intent`, `faelight-zone`, `faelight-link`, `faelight-lock`, `faelight-fetch` |
| **Audit** | `alias-audit`, `bin-doctor`, `entropy-check`, `archaeology-0-core` |
| **Bootstrap** | `faelight-bootstrap`, `faelight-daemon`, `faelight-cleanup`, `keyscan`, `teach`, `intent-guard`, `workspace-view`, `verify-bootstrap` |

> `faelight-login` replaces tuigreet — the forest now greets you in Rust.
> `faelight-release` replaces `bump-system-version` — the forest now publishes itself.
> `alias-audit`, `bin-doctor`, `entropy-check` are also natively absorbed into `core doctor` — standalone binaries kept for direct use.

[See full tool list →](TOOLS.md)

---

## 🧭 Philosophy

**"We Control Our Tools"** — Every tool written or fully understood. No mystery packages.

**"Fail Loudly"** — Errors are explicit and guide you to solutions. No silent failures.

**"Manual Over Automation"** — Automation serves comprehension, not convenience.

**"Understanding Over Convenience"** — If you can't explain it, you don't own it.

**"Design for Recovery"** — `rm -rf runtime/` is always safe. Full reset, no data loss.

**"The Forest Knows Itself"** — Health, history, and intent are first-class citizens.

---

## 🚀 Quick Reference
```bash
doctor                          # System health (22 checks)
core security scan              # Security audit
core git status                 # Git + risk score
core workspace recent today     # Recent files
cilist                          # Intent ledger
core doctor entropy             # Config drift check
cpc <name>                      # Create checkpoint
forecast                        # 7-day health projection
pulse                           # Live event stream
faelight-release history        # Release generation history
0c                              # cd ~/0-core
core-protect lock               # Lock before shutdown
```

---

## 🔧 For Developers

### Building
```bash
cargo build --release -p core       # Build orchestrator
cargo build --release --workspace   # Build all 47 tools
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
| v10.0.0 | **core v2.0.0 — migration complete** 🏛️ |
| v10.1.0 | **The Forest Matures — all 6 phases done** 🌲 |
| v10.3.0 | **Core v3 — Event Bus, Plugin Registry, Health Forecasting** 🧠 |
| v10.4.0 | **Core v4 + Niri Version — checkpoint system, Rust greeter, compositor migration** 🔒🌲 |
| v10.5.0 | **The Intelligent Forest — faelight-release, forecast, pulse, niri-bridge** 🧠 |

From hardcoded paths to centralized elegance.
From 40 separate binaries to one orchestrator.
From "new to Linux" to a self-aware system that thinks, remembers, and forecasts.
From tuigreet to faelight-login — the forest greets you first.
From manual releases to faelight-release — the forest now publishes itself.

---

## 📝 License

**Intentional Stewardship** — This is a personal computing environment, not a product.
Feel free to learn from it, but **build your own**. That's the whole point.

---

## 🙏 Acknowledgments

- **The Arch Linux community**: For vanilla excellence
- **The Niri project**: For proving a single developer can build a production Rust compositor
- **The Smithay project**: The foundation faelight-compositor will be built on
- **You**: For reading this far. Now go build your own! 💎

---
