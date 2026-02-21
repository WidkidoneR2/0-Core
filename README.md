# 🌲 Faelight Forest v10.0.0

> *One binary. Five layers. Zero ambiguity.*

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch**.

---

## 🤔 What is 0-Core?

### For Everyday Users

Instead of accepting whatever your OS gives you, 0-Core is building your entire computing environment from the ground up — every tool, every color, every keyboard shortcut, every security decision.

Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, file manager)
- 🦀 36 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat)
- 💎 Complete ownership and control

### For Technical People

A comprehensive computing environment featuring:
- **`core` v2.0.0** — single orchestrator binary replacing 40+ individual tools
- **15 native Rust domains** with capability-gated dispatch
- **36 custom Rust tools** at 100% path resilience
- **21-check health monitoring** system
- **Intent Ledger** for all architectural decisions
- **Wayland-native** (Sway, custom compositor tools)
- **Runtime locking** and JSONL capability audit logging

---

## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── 01-registry/      # Package manifests, alias registry, zone definitions
├── 02-rules/         # Git hooks, security policies, doctor rules
├── 03-interfaces/    # Dotfiles (Sway, zsh, foot, yazi) via GNU Stow
├── engine/           # core v2.0.0 — single orchestrator binary (Rust)
├── rust-tools/       # 36 custom Rust tools
├── scripts/          # Thin wrappers → core v2 + system scripts
├── intents/          # Architectural decision records
├── runtime/          # SQLite state, capability logs, process locks
└── status-blocks/    # Faelight bar status block scripts
```

### The Numbered Gravity System

Each directory has a gravity number defining its role:
- **00** — Identity: what the system **IS**
- **01** — Registry: what the system **KNOWS**
- **02** — Rules: what the system **DOES**
- **03** — Interfaces: what the system **SHOWS**
- `runtime/` — State: what the system **REMEMBERS**

### The `core` Orchestrator

The heart of v10.0.0 is a single binary that replaced all individual v1 tool delegation:
```
core <domain> <command>

Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update
```

Every domain call is:
1. **Capability-gated** — checked against granted permissions
2. **Logged** — recorded in `runtime/logs/capabilities.jsonl`
3. **Lock-protected** — runtime mutex prevents concurrent writes

---

## ✨ Key Features

### 🏥 Self-Aware Health Monitoring (21 Checks)
```bash
doctor        # Run all 21 health checks
```
Checks: stow symlinks, services, broken symlinks, yazi plugins,
binary deps, git status, themes, scripts, intents, profiles,
config files, keybinds, security hardening, security audit,
alias coverage, rust toolchain, disk space, tool installation,
path resilience, core protection.

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
git       → FilesystemReadHome, SpawnProcess
lock      → ControlSway
update    → SpawnProcess, ElevatedPrivilege
```

### 🎣 Git Governance
```bash
faelight-git commit    # Intent-aware commits
faelight-git risk      # Risk score before pushing
faelight-git sync      # Pull + push workflow
```
Pre-commit hooks: rustfmt, clippy, secret scanning, merge conflict detection.

### 📝 Intent Ledger
```bash
intent list            # All intents
intent list --active   # In-progress
intent show 092        # Specific intent
intent stats           # Ledger overview
```

---

## 🦀 The Rust Ecosystem (36 Tools)

| Category | Tools |
|---|---|
| **Orchestrator** | `core` (v2.0.0 — 15 domains) |
| **UI** | `faelight-bar`, `faelight-fm`, `faelight-term`, `faelight-launcher`, `faelight-dmenu`, `faelight-palette`, `faelight-menu` |
| **Git** | `faelight-git`, `faelight-hooks` |
| **System** | `core-protect`, `faelight-update`, `safe-update`, `faelight-sandbox`, `faelight-snapshot` |
| **Dev** | `bump-system-version`, `bump-tool-version`, `get-version`, `core-diff` |
| **Shell** | `dotctl`, `profile`, `intent`, `faelight-zone`, `faelight-link`, `faelight-lock`, `faelight-notify`, `faelight-fetch` |
| **Audit** | `alias-audit`, `bin-doctor`, `entropy-check`, `archaeology-0-core` |
| **Bootstrap** | `faelight-bootstrap`, `faelight-daemon`, `faelight-cleanup`, `keyscan`, `teach` |

---

## 🧭 Philosophy

**"We Control Our Tools"** — Every tool written or fully understood. No mystery packages.

**"Fail Loudly"** — Errors are explicit and guide you to solutions. No silent failures.

**"Manual Over Automation"** — Automation serves comprehension, not convenience.

**"Understanding Over Convenience"** — If you can't explain it, you don't own it.

---

## 🚀 Quick Reference
```bash
doctor                          # System health (21 checks)
core security scan              # Security audit
core git status                 # Git + risk score
core workspace recent today     # Recent files
core intent list --active       # Active intents
0c                              # cd ~/0-core
core-protect lock               # Lock before shutdown
```

---

## 📊 System Statistics

| Metric | Value |
|---|---|
| System Version | v10.0.0 |
| Orchestrator | core v2.0.0 (15 domains) |
| Rust Tools | 36 |
| Path Resilience | 100% (36/36) |
| Health Checks | 21 automated |
| Aliases | 318 |
| Intents | 92 (55 complete, 8 planned) |
| Architecture | Phases 1–6 complete |

---

## 🔧 For Developers

### Building
```bash
cargo build --release -p core     # Build orchestrator
cargo build --release --workspace # Build all tools
doctor                            # Verify health
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
| v9.9.0 | Presented to Linus Torvalds |
| v10.0.0 | **core v2.0.0 — migration complete** 🏛️ |

From hardcoded paths to centralized elegance.
From 40 separate binaries to one orchestrator.
From "new to Linux" to presenting to legends.

---

**System Version**: v10.0.0
**Last Updated**: 2026-02-21
**Health**: 95% locked / 90% unlocked
**Path Resilience**: 100% 💎
