<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest vv10.8.0

![Version](https://img.shields.io/badge/version-v10.8.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### vv10.8.0 - 🌲 The Forest Between Worlds (2026-03-12)

- 118 — doctor facelift — cockpit-style health dashboard
- 119 — core security advise — judgment layer for security decisions
- 121 — faelight-readme — auto-update README dynamic sections on release
- 123 — faelight-audit — Tool Intelligence Layer (core audit domain)
- 124 — faelight-sandbox v2 — Forest-Aware Isolation Environment
- faelight-shell v0.2.0 — Phase 2 complete, full data pipeline, 26 commands (INT-120)
- faelight-shell — ht, ct, domains, select fix, 26 commands (INT-120)
- faelight-shell — fix select pipe, multi-stage pipeline working (INT-120)
- faelight-shell — decisions-table, count pipe, 22 commands total (INT-120)
- faelight-sandbox v3 Phase 2 — policy enforcement wired, network isolation active, policy logged to ledger (INT-125)

- Commits: 1438
- Tools: 50 deployed
- Health: 95%
- Intents: 77 complete

[Full Changelog →](CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION - Comprehensive Documentation -->
## 🤔 What is 0-Core?

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch**.

### For Everyday Users
Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, login screen, notifications, compositor)
- 🦀 52 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat, no hidden automation)
- 💎 Complete ownership and control
- 🧭 A system that remembers its own decisions and advises you

### For Technical People
- **`core` v3.0.0** — single orchestrator binary with 15+ native Rust domains
- **Capability-gated dispatch** — every command checks permissions before executing
- **Core v6 — The Judgment Layer** — decision ledger, outcome tracking, judgment assist, heuristics engine, scenario simulation
- **faelight-compositor v0.1.0** — Rust Wayland compositor on Smithay, events flowing to ledger
- **faelight-release** — intelligent release system with generation model, rollback, learning layer
- **22-check health monitoring** system
- **Intent Ledger** — 115+ architectural decisions, fully documented

---
## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── engine/           # core v3.0.0 — single orchestrator binary (Rust)
│   └── src/domains/  # 15+ native Rust domains
├── rust-tools/       # 52 custom Rust tools
├── 03-interfaces/    # Dotfiles (Niri, zsh, foot, yazi) via GNU Stow
├── scripts/          # Thin wrappers → core + compiled binaries
├── intents/          # Architectural decision records (115+ intents)
├── runtime/          # SQLite state, capability logs, checkpoints, decisions
└── registry/         # Zero-logic TOML declarations
```

### The `core` Orchestrator
```
core <domain> <command>
Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update, checkpoint, simulate, plugins,
         decisions (Core v6)
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

### 🧭 Core v6 — The Judgment Layer
The forest now remembers its own decisions and advises you based on history.
```bash
core decide "upgrade smithay"     # Record decision with risk assessment + context fingerprint
core decision outcome DEC-001 success  # Close the loop
core advise                       # Judgment advisory for current state
core advise "planned action"      # Advisory for a specific decision
core hindsight                    # Success rate across all decisions
core heuristics                   # Auto-derived lessons (3+ observations)
core lessons                      # Human-readable wisdom summary
core story                        # 30-day narrative of your computing life
core simulate scenario "..."      # Risk simulation using decision history
```
Every decision stores a **context fingerprint** (CTX-XXXX) — health, git churn, active intents.
Similar contexts surface historical patterns. The forest advises. You decide.

### 🖥️ faelight-compositor — The Last Sibling
```bash
fc            # Launch faelight-compositor (nested in Niri for development)
```
A Rust Wayland compositor on Smithay — the only compositor that knows it's part of a forest.
- Emits `window.open`, `window.focus`, `workspace.switch` events to `state.db`
- Integrates with `core`'s capability model
- `doctor` monitors compositor health
- Built from scratch on Smithay — same foundation as COSMIC and Niri

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
lock-core      # Immutable flag on ~/0-core
unlock-core    # Remove for editing
```

### 🔒 Capability Model
Every domain declares required capabilities at dispatch time:
```
doctor    → OrchestratorAccess, FilesystemReadHome
security  → OrchestratorAccess, FilesystemReadHome, NetworkQuery
git       → FilesystemReadHome, FilesystemWriteHome, SpawnProcess
lock      → ControlWM
update    → FilesystemReadHome, ExecutePacman, SpawnProcess
decisions → FilesystemReadHome, FilesystemWriteHome
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
intent show 109                  # Specific intent
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
## 🦀 The Rust Ecosystem (52 Tools)

| Category | Tools |
|---|---|
| **Orchestrator** | `core` (v3.0.0 — 15+ domains + Core v6 judgment layer) |
| **Compositor** | `faelight-compositor` (Smithay, Wayland-native, ledger participant) |
| **UI** | `faelight-bar`, `faelight-term`, `faelight-palette`, `faelight-menu`, `faelight-notify`, `faelight-login` |
| **Clipboard** | `faelight-clipboard` |
| **Intelligence** | `faelight-forecast`, `faelight-pulse`, `faelight-niri-bridge`, `faelight-idle` |
| **Browser** | `faelight-browser` |
| **Git** | `faelight-git`, `faelight-hooks` |
| **System** | `core-protect`, `faelight-update`, `safe-update`, `faelight-sandbox`, `faelight-snapshot`, `faelight-wallpaper` |
| **Dev** | `faelight-release`, `bump-tool-version`, `get-version`, `core-diff` |
| **Shell** | `dotctl`, `profile`, `intent`, `faelight-zone`, `faelight-link`, `faelight-lock`, `faelight-fetch` |
| **Search** | `faelight-search` |
| **Audit** | `alias-audit`, `bin-doctor`, `entropy-check`, `archaeology-0-core` |
| **Bootstrap** | `faelight-bootstrap`, `faelight-daemon`, `faelight-cleanup`, `keyscan`, `teach`, `intent-guard`, `workspace-view`, `verify-bootstrap` |

> `faelight-compositor` — the last sibling comes home. The only compositor that writes to the forest ledger.
> `faelight-login` replaces tuigreet — the forest greets you in Rust.
> `faelight-release` replaces `bump-system-version` — the forest publishes itself.

---
## 🧭 Philosophy

**"We Control Our Tools"** — Every tool written or fully understood. No mystery packages.

**"Fail Loudly"** — Errors are explicit and guide you to solutions. No silent failures.

**"Manual Over Automation"** — Automation serves comprehension, not convenience.

**"Understanding Over Convenience"** — If you can't explain it, you don't own it.

**"Design for Recovery"** — `rm -rf runtime/` is always safe. Full reset, no data loss.

**"The Forest Knows Itself"** — Health, history, intent, and judgment are first-class citizens.

**"The Forest Remembers"** — Every decision recorded. Every outcome tracked. Wisdom earned.

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
advise                          # Judgment advisory
story                           # 30-day forest narrative
fc                              # Launch faelight-compositor
faelight-release history        # Release generation history
0c                              # cd ~/0-core
lock-core                       # Lock before shutdown
```

---
## 🔧 For Developers

### Building
```bash
cargo build --release -p core                    # Build orchestrator
cargo build --release -p faelight-compositor     # Build compositor
cargo build --release --workspace                # Build all 52 tools
doctor                                           # Verify health
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
| v10.6.0 | **The Judgment Layer — Core v5 complete, 5 phases, ledger foundation** ⚖️ |
| v10.7.0 | **The Forest Remembers — faelight-compositor + Core v6 complete** 🌲 |

From hardcoded paths to centralized elegance.
From 40 separate binaries to one orchestrator.
From "new to Linux" to a self-aware system that thinks, remembers, and forecasts.
From tuigreet to faelight-login — the forest greets you first.
From manual releases to faelight-release — the forest now publishes itself.
From borrowed substrate to faelight-compositor — the compositor finally came home.
From events to decisions — the forest now remembers what worked.


