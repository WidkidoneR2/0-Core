<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 11.2.0

![Version](https://img.shields.io/badge/version-11.2.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 11.2.0 - 🌲 Will and Motion (2026-03-22)

- 109 — faelight-compositor — Rust Wayland Compositor on Smithay
- 120 — faelight-shell — Forest-Native Shell Environment
- 132 — faelight-vault — Forest-Native Credential Manager
- 135 — faelight-shell Phase 11 — Forest Personality & Adaptive Intelligence
- 139 — faelight-shell — Natural Language Pipeline Translation
- 144 — v11.1.0 Release Gate — The Forest Speaks
- 145 — faelight-docs — Living Documentation Engine
- INT-146 Phase 11 — pipes to external commands, forest data flows into less/grep/wc
- INT-146 Phase 10 — shell variables, let/export, dollar sign expansion
- INT-146 Phase 9 — signal handling, Ctrl+C kills foreground process cleanly, shell survives
- INT-146 Phase 8 — job control, background jobs, jobs/fg/kill, forest announces completion
- INT-146 Phase 16 — interactive improvements, editor config, history dedup, emacs mode

- Commits: 1630
- Tools: 51 deployed
- Health: 95%
- Intents: 93 complete

[Full Changelog →](CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->
<!-- STATIC SECTION - Comprehensive Documentation -->

## 🤔 What is 0-Core?

**0-Core** (Faelight Forest) is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch in Rust**.

> *"Nothing runs without explicit human authorization. Every tool is understood. Every decision is documented."*

This is not configuration management. This is not a rice. This is a living system that knows itself.

---

## 🌲 The Philosophy
```
POSIX shells:      text | text | text
Nu shell:          table | filter | transform
Faelight Forest:   forest_data | judgment | wisdom
```

**Four principles that govern everything:**

1. **Understanding over convenience** — if you don't understand it, it doesn't run
2. **Manual control over automation** — nothing happens without explicit human authorization
3. **Intentional design** — every tool has a purpose, every decision has a record
4. **The forest remembers** — every commit, decision, and intent is documented

---

## 🦀 What's Inside

### The Core Orchestrator
**`core` v2.0.0** — a single Rust binary with 33+ native domains:

| Domain | Capability |
|--------|-----------|
| `core goals` | Forest sets its own goals — generate, accept, reject, prioritize |
| `core plan` | Break goals into concrete steps with risk analysis |
| `core tradeoff` | Surface competing values before every major decision |
| `core prioritize` | Rerank goals by live health, trend, and security posture |
| `core autobiography` | The forest narrates its own goal history |
| `core evolution` | Architectural proposals from coupling and churn analysis |
| `core decisions` | Decision ledger with context fingerprints and outcomes |
| `core simulate` | Dry-run predictions using historical decision patterns |
| `core doctor` | 24-check health monitoring with forecast and trend |
| `core security` | Security audit, debt tracking, hardening verification |
| `core snapshot` | Forest state narrative — what exists and why |

### faelight-shell — Forest-Native Shell

Not bash. Not fish. Not Nu. **Beyond all of them.**
```fsh
# Structured data pipelines
gc | first 10 | where message contains "feat"
ps | sort cpu desc | first 5 | less
et today | where domain == "goals"

# Natural language queries
?biggest files in this directory
?show me failing health checks

# Forest awareness
health; d; git status          # multi-command
let VERSION = "11.2.0"         # shell variables
sleep 30 &                     # background jobs
cargo build | grep error       # pipes to external
gc | first 20 > commits.txt    # redirection
```

**What makes it unique:**
- Every command returns structured data — pipeable, filterable, sortable
- The shell knows your active intents, health score, and session history
- Background jobs announce themselves when they finish (Jarvis-style)
- Ctrl+C kills the foreground process — shell survives
- Loads your personal config from `~/.config/faelight-shell/config.fsh`
- Emacs-grade line editing with full history deduplication

### The Tool Ecosystem
68 custom Rust tools, each understood and intentional:

| Category | Tools |
|----------|-------|
| **Display** | faelight-bar, faelight-notify v4, faelight-login, faelight-menu |
| **Filesystem** | faelight-fm, faelight-link, faelight-cleanup |
| **Security** | faelight-vault, faelight-sandbox v3, faelight-lock |
| **Development** | faelight-git, faelight-release, faelight-shell, faelight-term |
| **Forest** | faelight-forecast, faelight-idle, faelight-pulse, faelight-fetch |
| **Compositor** | faelight-compositor (Smithay, real DRM hardware) |

### faelight-vault — Credential Manager
Forest-native secrets management with Argon2id encryption. No LastPass, no Bitwarden — your own vault, your own keys.

### faelight-compositor — Custom Wayland Compositor
Built on Smithay. Renders forest green on real DRM hardware at 2560x1600@165Hz on AMD Radeon 780M. Not a config file — actual Rust code talking directly to the GPU.

---

## 🧠 Core Intelligence Timeline

| Version | Capability | Meaning |
|---------|-----------|---------|
| v2 | Structure | the forest has shape |
| v3 | Awareness | the forest observes itself |
| v4 | Discipline | the forest enforces rules |
| v5 | Intelligence | the forest detects patterns |
| v6 | Judgment | the forest remembers outcomes |
| v7 | Resilience | the forest can rebuild |
| v8 | Evolution | the forest refines itself |
| v9 | **Intent** | **the forest chooses where to grow** |
| v10 | Reaction *(planned)* | the forest responds without being asked |

---

## 🗺️ The Journey

| Version | Theme | Milestone |
|---------|-------|-----------|
| v11.2.0 | Will and Motion | Core v9 complete, shell becomes daily driver |
| v11.1.0 | The Forest Speaks | faelight-shell NL queries, session memory |
| v11.0.0 | The Forest Speaks | Niri compositor migration |
| v10.6.0 | The Judgment Layer | Core v6 complete |
| v10.5.0 | The Resilient Forest | Core v7 complete |
| v10.4.0 | Niri Version | faelight-login, Niri migration |
| v10.3.0 | The Evolution | Core v8 foundations |

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
- **24-check health monitoring** — doctor runs verify integrity continuously
- **Intent Ledger** — every architectural decision documented with context fingerprint

---

## 📋 The Intent System

Every decision in this system is documented. Not just what was done — but why, when, what was the health score, what was the risk, and what happened next.
```bash
core decisions list          # open decisions
core autobiography narrate   # the forest tells its own story
core goals generate          # what does the forest want to become?
core tradeoff analyze "add X" # what values are in tension?
```

104 complete intents. Every one of them is a chapter in the forest's history.

---

## 🚀 Quick Start
```bash
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core && cargo build --release --workspace
sudo cp target/release/* /usr/local/bin/
cd 03-interfaces/stow && stow */
core doctor run
```

> ⚠️ This system is built for one person — the author. It is not designed to be installed by others without deep understanding. Read the intent ledger before touching anything.

---

## 🌲 The Forest Remembers

*"A forest that chooses where to grow is no longer just a system. It is a participant in its own future."*

*Auto-generated by faelight-docs v1.0.0 — last sync: 2026-03-23*
