<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest v10.1.0

![Version](https://img.shields.io/badge/version-10.1.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-90%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### v10.1.0 - 🌲 The Forest Matures — 0-Core v2 Complete (2026-02-22)

- END0-Core v2 architecture migration complete — all 6 phases done
- Phase 5: alias-audit, bin-doctor, entropy-check absorbed into doctor domain
- Phase 6: capability model enforced across all 15 domains — every operation logged
- faelight-menu v4.0.0 — forest palette aesthetic matching faelight-palette
- faelight-palette v3 — split layout, real 0-Core stats, health cache integration
- faelight-fm — rich preview panel with zone/git/intent metadata
- faelight-link v3 — adopt command, GNU Stow dir-folding awareness
- Health cache — single source of truth across bar, palette, prompt, doctor
- All hardcoded paths removed from engine — fully portable
- Cold start: 3ms

- Tools: 34/34 deployed (100%)
- Aliases: 318 total
- Commits: 1240
- Health: 95%
- Domains: 15 native Rust domains
- Sub-tools absorbed: 3 (alias-audit, bin-doctor, entropy-check)
- Cold start: 3ms

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
- 🦀 36 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat)
- 💎 Complete ownership and control

### For Technical People

- **`core` v2.0.0** — single orchestrator binary with 15 native Rust domains
- **Capability-gated dispatch** — every command checks permissions before executing
- **Runtime locking** and JSONL capability audit logging
- **21-check health monitoring** system
- **Intent Ledger** for all architectural decisions
- **Wayland-native** (Sway, custom compositor tools)

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

- **00** — Identity: what the system **IS**
- **01** — Registry: what the system **KNOWS**
- **02** — Rules: what the system **DOES**
- **03** — Interfaces: what the system **SHOWS**
- `runtime/` — State: what the system **REMEMBERS**

### The `core` Orchestrator
```
core <domain> <command>

Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update
```

Every domain call is capability-gated, logged, and lock-protected.

---

## ✨ Key Features

### 🏥 Self-Aware Health Monitoring (21 Checks)
```bash
doctor        # Run all 21 health checks
```
Checks: stow symlinks, services, broken symlinks, yazi plugins, binary deps,
git status, themes, scripts, intents, profiles, config files, keybinds,
security hardening, security audit, alias coverage, rust toolchain,
disk space, tool installation, path resilience, core protection.

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
All capability usage logged to `runtime/logs/capabilities.jsonl`.

### 🎣 Git Governance
Pre-commit hooks: rustfmt, clippy, secret scanning, merge conflict detection.
```bash
faelight-git commit    # Intent-aware commits
faelight-git risk      # Risk score before pushing
faelight-git sync      # Pull + push workflow
```

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
| **Bootstrap** | `faelight-bootstrap`, `faelight-daemon`, `faelight-cleanup`, `keyscan`, `teach`, `intent-guard`, `workspace-view`, `verify-bootstrap` |

[See full tool list →](TOOLS.md)

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
