<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 11.5.0

![Version](https://img.shields.io/badge/version-11.5.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 11.5.0 - 🌲 The Shell Awakens (2026-03-30)

- 120 — faelight-shell — Forest-Native Shell Environment
- 126 — Core v8 — Evolution: The Forest Refines Itself
- 136 — Faelight Forest — Visual Identity & Niri Cosmetics
- 140 — Core v10 — Reaction: The Forest Responds Without Being Asked
- 146 — faelight-shell v2 — The Shell Becomes the OS
- 148 — Core v11 — Prediction: The Forest Anticipates
- 149 — Tool Retirement Sprint — Clean What the Core Has Absorbed
- 151 — Core v12 — Strategy: The Forest Plans Across Horizons
- 153 — Intent Genealogy — The Forest Remembers How It Grew
- 155 — faelight-shell Prompt Themes — The Shell Has a Face
- 162 — Shell Architecture Hardening — The Foundation Must Be Solid
- 163 — Alias Audit — One Concept, One Command
- 164 — Core Deploy Pipeline — Versioned, Immutable, Rollback-Safe
- 165 — fsh Welcome Screen — Truth Only, No Stale Data
- 166 — state.db Backup and Recovery — Protect the Forest's Memory
- 171 — Pre-Command Decision Layer — The Shell That Understands Before It Executes
- 172 — Shell Config Stow — config.fsh Under Version Control
- 173 — Command Registry — The Shell Knows What It Can Do
- 174 — Structured Errors — The Shell Explains Its Failures
- fsh — core subcommand shortcuts, predict/react/stress/doctor/goals native, no prefix needed
- fsh — jarvis theme Phase 22, prediction inline in prompt, INT-136 + health state visible
- fsh Phase 23 — session persistence, directory restored on startup
- fsh Phase 25 — faelight-term launches fsh by default, falls back to zsh
- faelight-release — docs section added, newest first, live health, scope grouping

- Commits: 1779
- Tools: 50 deployed
- Health: 100%
- Intents: 115 complete

[Full Changelog →](CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION -->

## 🤔 What is 0-Core?

**0-Core** (Faelight Forest) is a completely custom Linux environment built on vanilla Arch Linux where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch in Rust**.

> *"Nothing runs without explicit human authorization. Every tool is understood. Every decision is documented."*

This is not configuration management. This is not a rice. This is a living system that knows itself — and now anticipates what comes next.

---

## 🌲 The Philosophy
```
POSIX shells:      text | text | text
Nu shell:          table | filter | transform
Faelight Forest:   forest_data | judgment | wisdom | anticipation
```

**Four principles that govern everything:**

1. **Understanding over convenience** — if you don't understand it, it doesn't run
2. **Manual control over automation** — nothing happens without explicit human authorization
3. **Intentional design** — every tool has a purpose, every decision has a record
4. **The forest remembers** — every commit, decision, and intent is documented and learned from

---

## 🧠 Core Intelligence Timeline

| Version | Capability | Status | Meaning |
|---------|-----------|--------|---------|
| v1–v5 | Foundation → Intelligence | ✅ Complete | structure, awareness, discipline, patterns |
| v6 | Judgment | ✅ Complete | the forest remembers outcomes |
| v7 | Resilience | ✅ Complete | the forest can rebuild itself |
| v8 | Evolution | ✅ Complete | the forest refines its architecture |
| v9 | Intent | ✅ Complete | the forest chooses where to grow |
| v10 | Reaction | ✅ Complete | the forest responds without being asked |
| v11 | Prediction | ✅ Complete | the forest anticipates before it happens |
| v12 | Strategy | 🔜 Planned | the forest plans across multiple horizons |
| v13 | Autonomy | 🔜 Planned | the forest chooses its own purpose |

**Jarvis Readiness: 65/100** — anticipatory partner territory. v12 targets 85. v13 is the destination.

---

## 🦀 What's Inside

### The Core Orchestrator

**`core` v2.0.0** — a single Rust binary with 36+ native domains:

| Domain | Capability |
|--------|-----------|
| `core predict` | 9 commands — session patterns, health trajectory, intent velocity, coupling risk, accuracy |
| `core react` | 6 rules — health advisory, security aging, checkpoint staleness, intent overflow |
| `core stress` | Verification suite — event storm, prediction load, reaction integrity, health chaos |
| `core goals` | Forest sets its own goals — generate, accept, reject, prioritize |
| `core doctor` | 23-check health monitoring with forecast, trend, and early warning |
| `core decisions` | Decision ledger with context fingerprints and outcomes |
| `core evolution` | Architectural proposals from coupling and churn analysis |
| `core security` | Security audit, debt tracking, hardening verification |
| `core simulate` | Dry-run predictions using historical decision patterns |
| `core checkpoint` | State snapshots with full forest context |

### faelight-shell — Forest-Native Shell

Not bash. Not fish. Not Nu. The forest's own voice.
```fsh
# Structured data pipelines
gc | first 10 | where message contains "feat"
ps | sort cpu desc | first 5

# Forest-native builtins
pwd          # forest builtin
which core   # shows: forest script + PATH
type d       # shows: alias → core doctor run
env          # structured environment table
theme minimal # instant prompt theme switch

# Prediction awareness
core predict next       # what intent ships next?
core predict sessions   # when do you typically build?

# Context-aware tab completion
core predict <TAB>      # shows all 9 predict subcommands
cistart 14<TAB>         # shows INT-140 through INT-149
```

**Current state:** 92% native command handling. 4 prompt themes. 8+ builtins.

**Themes:** `forest` (full context) · `minimal` (path only) · `classic` (user@host) · `jarvis` (prediction inline, coming)

### The Tool Ecosystem

44 custom Rust tools, each understood and intentional:

| Category | Tools |
|----------|-------|
| **Display** | faelight-bar, faelight-notify v4, faelight-login, faelight-menu |
| **Shell** | faelight-shell, faelight-term, faelight-git, faelight-release |
| **Forest** | faelight-digest, faelight-forecast, faelight-idle, faelight-pulse |
| **Security** | faelight-vault, faelight-sandbox v3, faelight-lock |
| **Compositor** | faelight-compositor (Smithay, real DRM), faelight-niri-bridge |
| **Filesystem** | faelight-fm, faelight-link, faelight-clipboard |

---

## 🗺️ The Journey

| Version | Theme | Milestone |
|---------|-------|-----------|
| v11.4.0 | **The Bloom** | Core v11 Prediction complete, fsh Phase 21, chaos testing |
| v11.3.0 | The Forest Grows | Core v10 Reaction complete, fsh builtins sprint |
| v11.2.0 | Will and Motion | Core v9 complete, shell becomes daily driver |
| v11.1.0 | The Forest Speaks | faelight-shell NL queries, session memory |
| v11.0.0 | Niri Migration | Sway → Niri compositor, faelight-login |
| v10.6.0 | The Judgment Layer | Core v6 complete |
| v10.4.0 | Niri Version | faelight-login, faelight-notify v3 |

---

## 🔒 Security Philosophy
```
Nothing runs without explicit human authorization.
Every change is intentional. Every tool is understood.
```

- **UFW** firewall + **fail2ban** active
- **faelight-vault** — Argon2id encrypted credential manager
- **faelight-sandbox v3** — policy engine, namespace isolation, seccomp
- **Immutable core** — `chattr +i` on core files, requires explicit unlock
- **23-check health monitoring** — continuous integrity verification
- **Chaos-tested** — deliberate failure injection verified 5/5 scenarios

---

## 📋 The Intent System

Every decision is documented. Not just what — but why, when, what health score, what risk, what happened next.
```bash
core predict next         # what does the forest anticipate shipping next?
core react story          # what has the forest been signaling today?
core decisions list       # open decisions awaiting resolution
core stress health-report # verify system survives deliberate failure
```

113 complete intents. 7 planned. Every one a chapter in the forest's history.

---

## 🚀 Quick Start
```bash
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core && cargo build --release --workspace
sudo cp target/release/* /usr/local/bin/
cd 03-interfaces/stow && stow */
core doctor run
```

> ⚠️ This system is built for one person. It is not designed to be installed by others without deep understanding. Read the intent ledger before touching anything.

---

## 🌲 The Forest Remembers

*"A forest that predicts the storm and plans the shelter before the first cloud appears — that is not intelligence. That is wisdom."*

*Auto-generated by faelight-docs v1.0.0 — last sync: 2026-03-26*
