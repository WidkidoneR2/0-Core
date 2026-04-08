<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 11.7.0

![Version](https://img.shields.io/badge/version-11.7.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 11.7.0 - 🌲 The Intelligence Arc (2026-04-08)

- 156 — Core v13 — Autonomy: The Forest Chooses Its Own Purpose
- 158 — The Partner Vision — The Forest Becomes a Genuine Collaborator
- 161 — Forest Build Order — The Path to Partnership
- 179 — faelight-shell v3 — The Daily Driver: The Shell Becomes Self-Aware
- 185 — faelight-contextd — The Nervous System: Background Awareness Daemon
- 188 — Core v15 — Alignment: The Forest Stays True to What Matters
- 191 — fsh Add-Ons and Bug Report — The Shell Grows
- 192 — Deploy Pipeline v0.8.0 — registry_tools.py and Architecture Split
- 193 — Tool Retirement Sprint — The Forest Prunes Itself
- 197 — Intelligence Layer v2 — The Forest Learns From Itself
- 198 — Intent Engine v2 — Smarter Intents, Deeper Connections
- 199 — Integrity Engine v2 — From Proposals to Auto-Healing
- 200 — Doctor v2 — No Stale Data, No False Positives
- 204 — faelight-update v4.0.0 — The Intelligent Update Manager
- 206 — Engine Coordination Layer — The Forest Thinks as One

- Commits: 1998
- Tools: 55 deployed
- Health: 100%
- Intents: 146 complete

[Full Changelog →](00-meta/CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION -->
## 🤔 What is 0-Core?

A fully custom Arch Linux + Niri personal computing environment built from first principles in ~97.8% Rust. Every tool is written or fully understood. No mystery packages. No magic.
POSIX shells:      text | text | text
Nu shell:          table | filter | transform
Faelight Forest:   forest_data | judgment | wisdom | anticipation

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
| v12 | Strategy | ✅ Complete | the forest plans across multiple horizons |
| v13 | Autonomy | 🔜 In Progress | the forest chooses its own purpose |
| v14 | Partnership | 🔜 Planned | the forest and human co-create |

**Jarvis Readiness: 90/100** — Strategic Advisor. v13 targets 95/100. Autonomous agent territory.

---

## 🦀 What's Inside

### The Core Orchestrator

**`core` v2.0.0** — a single Rust binary with 46+ native domains:

| Domain | Capability |
|--------|-----------|
| `core predict` | 9 commands — session patterns, health trajectory, intent velocity, coupling risk, accuracy |
| `core react` | 6 rules — health advisory, security aging, checkpoint staleness, intent overflow |
| `core strategy` | horizon/sequence/coherence/jarvis/trust — planning across multiple time horizons |
| `core goals` | forest sets its own goals — generate, accept, reject, prioritize |
| `core doctor` | 23-check health monitoring with forecast, trend, and early warning |
| `core integrity` | 13-check integrity engine — intent ledger, jarvis freshness, schema validation |
| `core decisions` | decision ledger with context fingerprints and outcomes |
| `core evolution` | architectural proposals from coupling and churn analysis |
| `core security` | security audit, debt tracking, hardening verification |
| `core checkpoint` | state snapshots with full forest context |
| `core autonomy` | goal evaluation, delegation simulation, trust contracts (v13 — in progress) |

### faelight-shell — Forest-Native Shell (Daily Driver)

Not bash. Not fish. Not Nu. The forest's own voice. **Login shell since 2026-04-03.**
```fsh
# Structured data pipelines
gc | first 10 | where message contains "feat"
ps | sort cpu desc | first 5

# External command pipes — fully working
grep -r "fn main" ~/0-core/engine/src/main.rs | head -5

# Heredoc support — python3, awk, sed all work
python3 << 'EOF'
print("the forest thinks in python too")
EOF

# Script debug mode
run deploy.fsh --trace     # every step with timing
run deploy.fsh --dry-run   # preview without executing

# Forest-native builtins
core predict next       # what intent ships next?
core strategy jarvis    # readiness score breakdown
core integrity run      # 13-check integrity scan

# Intelligence layer
core observe causality  # what caused recent patterns?
core memory decay       # entropy-aware knowledge management
```

**Current state:** Login shell on all terminals. Pipes, heredoc, 370 aliases, POSIX -c flag, session restore.
**Themes:** `forest` (full context) · `minimal` (path only) · `classic` (user@host) · `jarvis` (prediction inline)

### The Tool Ecosystem

55 custom Rust tools, each understood and intentional:

| Category | Tools |
|----------|-------|
| **Display** | faelight-bar, faelight-notify v4, faelight-login, faelight-menu |
| **Shell** | faelight-shell v3, faelight-term, faelight-git, faelight-release |
| **Intelligence** | faelight-context, faelight-memory, faelight-digest, faelight-forecast |
| **Security** | faelight-vault, faelight-sandbox v3, faelight-lock |
| **Compositor** | faelight-compositor (Smithay, real DRM), faelight-niri-bridge |
| **Filesystem** | faelight-fm, faelight-link, faelight-clipboard |

---

## 🗺️ The Journey

| Version | Theme | Milestone |
|---------|-------|-----------|
| v11.6.0 | **The Shell Lives** | fsh daily driver, pipes+heredoc, script debug mode, release pipeline |
| v11.5.0 | **The Shell Awakens** | fsh v3, Core v12 Strategy, shell intelligence layer, prediction feedback |
| v11.4.0 | **The Bloom** | Core v11 Prediction complete, chaos testing 5/5 PASS |
| v11.3.0 | The Forest Grows | Core v10 Reaction complete, fsh builtins sprint |
| v11.2.0 | Will and Motion | Core v9 complete, shell becomes daily driver |
| v11.1.0 | The Forest Speaks | 157 complete intents milestone |
| v11.0.0 | Niri Migration | Sway → Niri compositor, faelight-login |
| v10.4.0 | Niri Version | faelight-login, faelight-notify v3 |

---

## 🔒 Security Philosophy
Nothing runs without explicit human authorization.
Every change is intentional. Every tool is understood.

- **UFW** firewall + **fail2ban** active
- **faelight-vault** — Argon2id encrypted credential manager
- **faelight-sandbox v3** — policy engine, namespace isolation, seccomp
- **Immutable core** — `chattr +i` on core files, requires explicit unlock
- **23-check health monitoring** — continuous integrity verification
- **13-check integrity engine** — ledger, schema, jarvis, duplicate detection
- **Chaos-tested** — deliberate failure injection verified 5/5 scenarios

---

## 📋 The Intent System

Every decision is documented. Not just what — but why, when, what health score, what risk, what happened next.
```bash
core predict next         # what does the forest anticipate shipping next?
core react story          # what has the forest been signaling today?
core strategy jarvis      # Jarvis readiness score breakdown
core integrity run        # verify system state consistency
core decisions list       # open decisions awaiting resolution
```

142 complete intents. 10 planned. Every one a chapter in the forest's history.

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

*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-04-08 16:05*
