<!-- DYNAMIC SECTION - Updated by bump-system-version -->
![Version](https://img.shields.io/badge/version-11.7.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)
> **A self-aware, path-resilient personal computing environment built from first principles.**
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
- Commits: 2001
- Tools: 54 deployed
- Health: 100%
- Intents: 157 complete
[Full Changelog →](00-meta/CHANGELOG.md)
---
<!-- END DYNAMIC SECTION -->
<!-- STATIC SECTION -->
A fully custom Arch Linux + Niri personal computing environment built from first principles in ~97.8% Rust. Every tool is written or fully understood. No mystery packages. No magic.
POSIX shells:      text | text | text
Nu shell:          table | filter | transform
Faelight Forest:   forest_data | judgment | wisdom | anticipation | alignment
**Four principles that govern everything:**
1. **Understanding over convenience** — if you don't understand it, it doesn't run
2. **Manual control over automation** — nothing happens without explicit human authorization
3. **Intentional design** — every tool has a purpose, every decision has a record
4. **The forest remembers** — every commit, decision, and intent is documented and learned from
---
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
| v13 | Autonomy | ✅ Complete | the forest chooses its own purpose |
| v14 | Partnership | ✅ Active | the forest thinks alongside you — Jarvis 105/100 |
| v15 | Alignment | ✅ Complete | the forest stays true to what matters |
| v16 | Self-Transformation | 🔜 Planned | the forest redesigns itself |
| v17 | Pattern Weight | 🔜 Planned | the forest knows what matters most |
**Jarvis Readiness: 105/100** — Partnership fully active. v14 engaged. v15 Alignment live.
The forest now declares values, checks behavioral drift, and grounds every decision in principle.
**Building toward Friday** — a forest-native intelligence that learns from every session,
grows from observation, and eventually thinks alongside you as a genuine partner.
---
**`core` v3.0.0** — a single Rust binary with 48+ native domains:
| Domain | Capability |
|--------|-----------|
| `core predict` | 9 commands — session patterns, health trajectory, intent velocity, coupling risk, accuracy |
| `core react` | 6 rules — health advisory, security aging, checkpoint staleness, intent overflow |
| `core strategy` | horizon/sequence/coherence/jarvis/trust — planning across multiple time horizons |
| `core goals` | forest sets its own goals — generate, accept, reject, prioritize |
| `core doctor` | 23-check health monitoring with forecast, trend, and early warning |
| `core integrity` | 13-check integrity engine — intent ledger, jarvis freshness, schema validation |
| `core partner` | 5-phase partnership — propose, discuss, disagree, consult, co-author roadmap |
| `core values` | declared values system — define, weight, scope principles |
| `core align` | alignment checking — behavioral drift detection against declared values |
| `core engines` | engine coordination — status, sync, signals, upgrade contracts |
| `core delegate` | delegation engine — trust contracts, simulation, accuracy tracking |
| `core decisions` | decision ledger with context fingerprints and outcomes |
| `core autonomy` | goal evaluation, delegation simulation, trust contracts |
Not bash. Not fish. Not Nu. The forest's own voice. **Login shell since 2026-04-03.**
```fsh
tools | where score > 80
gc | first 10
deploy core > /tmp/build.log 2>/dev/null
git status || echo "not a repo"
hs deploy
last
save session-data
recall session-data
core align check "starting new intent"
core engines status
core values list
```
**Current state:** Login shell on all terminals. 370 aliases, tab completion, frequency-scored history, timing intelligence, smarter DELETE confirmation.
Seven engines working as one:
core engines status
🌲 Engine Coordination Status
core              3.0.0    active    now
faelight-contextd 0.1.0    active    2 min ago
delegation        0.3.0    active    12 min ago
friday            0.0.0    dormant   —
pattern-weight    0.0.0    planned   —
alignment         0.0.0    planned   —
Every engine produces signals. Every signal flows through state.db.
No engine calls another directly. The schema is the contract.
When Friday wakes, it inherits everything every engine has learned.
🧬 System Profile
Host:        fealight
Kernel:      6.19.11-arch1-1
Shell:       faelight-shell
Last update: 1 day ago (FRESH)
Drift:       FRESH
📊 Update Summary
🔴 Critical:  2 (linux, systemd)
🟡 Important: 5 (git, neovim, rust)
🔵 Optional: 12 (AUR, npm, pip)
💡 Suggestions
• 7 orphan packages found — run: sudo pacman -Rns $(pacman -Qtdq)
Preview mode, maintenance mode, pre-flight warnings, drift score, risk categorization.
55 custom Rust tools, each understood and intentional:
| Category | Tools |
|----------|-------|
| **Display** | faelight-bar, faelight-notify v4, faelight-login, faelight-menu |
| **Shell** | faelight-shell v3, faelight-term, faelight-git, faelight-release |
| **Intelligence** | faelight-context, faelight-contextd, faelight-memory, faelight-digest, faelight-forecast |
| **Updates** | faelight-update v4.0.0 — risk levels, drift, pre-flight, suggestions |
| **Security** | faelight-vault, faelight-sandbox v3, faelight-lock |
| **Compositor** | faelight-compositor (Smithay, real DRM), faelight-niri-bridge |
| **Filesystem** | faelight-fm, faelight-link, faelight-clipboard |
---
| Version | Theme | Milestone |
|---------|-------|-----------|
| v11.7.0 | **The Intelligence Arc** | Core v15 Alignment, Engine Coordination, 15 intents, Jarvis 105/100 |
| v11.6.0 | **The Shell Lives** | fsh daily driver, pipes+heredoc, script debug mode, release pipeline |
| v11.5.0 | **The Shell Awakens** | fsh v3, Core v12 Strategy, shell intelligence layer, prediction feedback |
| v11.4.0 | **The Bloom** | Core v11 Prediction complete, chaos testing 5/5 PASS |
| v11.3.0 | The Forest Grows | Core v10 Reaction complete, fsh builtins sprint |
| v11.2.0 | Will and Motion | Core v9 complete, shell becomes daily driver |
| v11.1.0 | The Forest Speaks | 159 complete intents milestone |
| v11.0.0 | Niri Migration | Sway → Niri compositor, faelight-login |
---
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
Every decision is documented. Not just what — but why, when, what health score, what risk, what happened next.
```bash
core predict next
core react story
core strategy jarvis
core integrity run
core align drift
core engines status
core values list
```
157 complete intents. 13 planned. Every one a chapter in the forest's history.
---
```bash
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core && cargo build --release --workspace
sudo cp target/release/* /usr/local/bin/
cd 03-interfaces/stow && stow */
core doctor run
```
> ⚠️ This system is built for one person. It is not designed to be installed by others without deep understanding. Read the intent ledger before touching anything.
---
*"A system that knows its values can detect when it betrays them.
Alignment is not a constraint — it is the compass that makes every decision navigable.
A partner without principles is clever.
A partner with principles is trustworthy."*
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-04-08 19:26*
