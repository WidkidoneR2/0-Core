<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest v10.9.0

![Version](https://img.shields.io/badge/version-v10.9.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### v10.9.0 - 🌲 Roots and Branches (2026-03-16)

- 127 — Schema Layer — Registry and Policy Validation
- 128 — Domain Restructuring — Subdirectory Per Domain
- 129 — Event Log Directory — File-Based JSONL Alongside SQLite
- 130 — faelight-gen — Forest-Native Password & Secret Generator Suite
- 131 — faelight-teach upgrade — Interactive faelight-shell Tutorial
- teach v5.0.0 — faelight-shell tutorial, 5 lessons, interactive prompt (INT-131)
- faelight-gen v1.0.0 — 12 generator types, colored output, entropy display (INT-130)
- INT-129 complete — JSONL event log, lifecycle policy, core events status/archive
- Core v7 Phase 2 — bootstrap intelligence, plan/verify/diff commands (INT-122)
- INT-128 — doctor domain restructured into subdirectories, checks/cockpit/schema split (INT-128)

- Commits: 1469
- Tools: 51 deployed
- Health: 95%
- Intents: 82 complete

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
- 🦀 43 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat, no hidden automation)
- 💎 Complete ownership and control
- 🌲 A shell that knows it is a forest

### For Technical People
- **`core` v2.0.0** — single orchestrator binary with 20+ native Rust domains
- **Capability-gated dispatch** — every command checks permissions before executing
- **Core v7 — The Resilient Forest** — anomaly detection, bootstrap intelligence, security simulation, dependency graph, forest narrative, snapshot autobiography, deterministic rebuild
- **faelight-compositor** — custom Wayland compositor built on Smithay, renders forest green on real DRM hardware
- **faelight-shell** — forest-native structured shell with pipelines, streaming, living welcome
- **faelight-sandbox v3** — policy engine, namespace isolation, seccomp syscall filtering
- **24-check health monitoring** system
- **Intent Ledger** — 133 architectural decisions, 96 complete, fully documented

---

## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── engine/           # core binary — single orchestrator (Rust)
│   └── src/domains/  # 20+ native Rust domains
├── rust-tools/       # 43 custom Rust tools
├── 03-interfaces/    # Dotfiles (Niri, zsh, foot, yazi) via GNU Stow
├── 04-schema/        # JSON schemas — registry validation
├── scripts/          # Deployed binaries → PATH
├── intents/          # Architectural decision records (133 intents)
├── runtime/          # SQLite state, events/, snapshots/, checkpoints/
└── 01-registry/      # Zero-logic TOML declarations
```

### The `core` Orchestrator
```
core <domain> <command>

Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update, checkpoint, simulate, plugins,
         anomaly, bootstrap, deps, narrative, snapshot,
         events, decisions, audit, advise
```

Every domain call is capability-gated, logged, and lock-protected.

### Core Intelligence Timeline

| Version | Capability | Commands |
|---------|-----------|---------|
| v5 | Intelligence | pattern detection, audit scoring |
| v6 | Judgment | decision ledger, outcome tracking |
| v7 | Resilience | anomaly scan, bootstrap plan, deps graph, narrative, snapshot, rebuild |
| v8 | Evolution | architecture reflection, tool lifecycle *(planned)* |
| v9 | Intent | goal engine, task planning, tradeoff analysis *(planned)* |

### Layer Model

| Layer | Name | Purpose |
|---|---|---|
| 0 | Substrate | Kernel, Wayland, Niri — understood, not opaque |
| 1 | Core Engine | `core` binary — single surface |
| 2 | Registry | Zero-logic TOML declarations |
| 3 | Policy | Constraints, no execution |
| 4 | Runtime | All mutable state |
| 5 | Schema | JSON validation layer |

---

## ✨ Key Features

### 🏥 Self-Aware Health Monitoring (24 Checks)
```bash
doctor        # Run all 24 health checks
```

Checks: stow symlinks, services, broken symlinks, yazi plugins, binary deps,
git status, themes, scripts, intents, profiles, config files, keybinds,
security hardening, security audit, alias coverage, rust toolchain,
disk space, tool installation, path resilience, archaeology, core protection,
schema validation, sandbox health.

### 🌲 Core v7 — The Resilient Forest
```bash
core anomaly scan              # detect anomalies — changes without decisions
core bootstrap plan            # step-by-step rebuild guidance
core bootstrap verify          # verify system consistency
core deps graph                # visual dependency map of all tools
core deps risk                 # which dependencies carry the most risk?
core security simulate <pkg>   # what would happen if we patched this?
core narrative                 # the forest tells its own story
core narrative --intent 109    # story of a specific intent
core snapshot                  # forest autobiography — human voice
core snapshot --json           # machine-readable reconstruction seed
core snapshot --save           # save both to runtime/snapshots/
core doctor rebuild            # deterministic rebuild plan from first principles
```

### 🖥️ faelight-compositor — The Forest Renders Its Own Pixels

The last tool built from scratch. A Wayland compositor on Smithay that:
- Opens real DRM hardware (AMD Radeon 780M, eDP 2560×1600@165Hz)
- Renders forest green `#11140f` via raw GBM buffer
- Emits window events to state.db (window.open, window.focus)
- Runs nested in Niri (winit mode) or standalone from TTY2 (DRM mode)
```bash
fc              # launch nested (winit mode — for testing)
fc --drm        # launch on real hardware from TTY2
fc --probe      # probe DRM devices without starting compositor
```

### 🐚 faelight-shell — Forest-Native Structured Shell
```bash
faelight-shell    # or: shell
```

Not text streams. Structured data pipelines:
```
ps | where cpu > 20 | sort cpu desc | first 5
services | where status == running
files | sort size desc | first 10
ports | where port == 8080
ps | watch 3                          # live streaming pipeline
tt | where score < 70 | sort score    # audit tool health
gc | where author == christian         # git history as table
```

Living welcome on every open — reads health, commits, intents, and quotes live.

### 🧪 faelight-sandbox v3 — Security Boundary
```bash
faelight-sandbox run --policy strict -- ./unknown-script.sh
faelight-sandbox run --isolate net -- curl example.com
faelight-sandbox run --isolate full -- cargo build
faelight-sandbox run --isolate seccomp -- ./untrusted
faelight-sandbox run --profile -- cargo build
```

Isolation levels:
- `--isolate net` — network namespace
- `--isolate full` — network + PID + mount namespace
- `--isolate seccomp` — syscall filtering (blocks dangerous syscalls)
- `--profile` — memory + disk I/O measurement

### 📋 Intent Ledger

Every architectural decision is recorded:
```bash
core intent list          # all 133 intents
intent show 109           # story of the compositor
core narrative --intent 109
```

96 complete. 6 planned. Nothing is built without intent.

---

## 🦀 The Rust Ecosystem (43 Tools)

| Domain | Key Tools |
|--------|-----------|
| **Orchestrator** | `core` (20+ domains, 24 health checks) |
| **Compositor** | `faelight-compositor` (Smithay, DRM, GBM) |
| **Shell** | `faelight-shell` (structured pipelines, streaming, living welcome) |
| **Security** | `faelight-sandbox` v3 (policy, namespaces, seccomp), `faelight-gen` (12-type secret generator) |
| **UI** | `faelight-bar`, `faelight-menu`, `faelight-palette`, `faelight-wallpaper` |
| **Git** | `faelight-git` (risk scoring, event emission), `faelight-release` |
| **System** | `faelight-idle`, `faelight-notify`, `faelight-lock`, `faelight-login` |
| **Tools** | `faelight-fm`, `faelight-term`, `faelight-browser`, `faelight-fetch` |
| **Intelligence** | `faelight-forecast`, `faelight-pulse`, `faelight-clipboard` |
```bash
core audit scan          # score all 43 tools
core deps graph          # visual dependency map
core deps risk           # high-coupling dependency analysis
```

---

## 🗺️ The Journey

| Version | Theme | Milestone |
|---------|-------|-----------|
| v10.4.0 | Niri Version | Migrated from Sway, faelight-login born |
| v10.5.0 | The Forest Between Worlds | Core v5 complete |
| v10.6.0 | The Judgment Layer | Core v6 complete |
| v10.7.0 | The Forest Remembers | faelight-bar rewrite, sandbox v2 |
| v10.8.0 | The Forest Between Worlds | faelight-shell born, core audit |
| v10.9.0 | Roots and Branches | Core v7 complete, faelight-gen, faelight-compositor first render |

---

## 🔒 Security Philosophy
```
Nothing runs without explicit human authorization.
Every change is intentional.
Every tool is understood.
```

Security layers:
- UFW firewall + fail2ban
- faelight-sandbox with policy engine + namespace isolation + seccomp
- Immutable core (chattr +i) — cannot be modified without explicit unlock
- Security audit with debt tracking (core security debt/trend/simulate)
- 24-check health monitoring catches drift early

---

## 🚀 Quick Start (Rebuild from Scratch)
```bash
# 1. Install Arch Linux (vanilla)
pacman -S niri greetd rustup git stow

# 2. Clone the forest
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core

# 3. Build all tools
cd ~/0-core && cargo build --release --workspace
cp target/release/* scripts/

# 4. Deploy interfaces
cd ~/0-core/03-interfaces/stow && stow */

# 5. Validate
core doctor run  # should show 24/24 ✅

# 6. Understand
core narrative         # the forest tells its story
core doctor rebuild    # step-by-step rebuild plan
```

Or use the forest's own guidance:
```bash
core bootstrap plan    # what needs to be done to rebuild
core bootstrap verify  # verify current state
```

---

*"The forest does not fear the storm. It knows how to grow back."* 🌲
