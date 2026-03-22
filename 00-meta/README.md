<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 11.1.0

![Version](https://img.shields.io/badge/version-11.1.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-95%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 11.1.0 - 🌲 The Forest Speaks (2026-03-21)

- 126 — Core v8 — Evolution: The Forest Refines Itself
- INT-141 faelight-notify v4.0.0 — fontdue::layout renderer, clean text, D-Bus compliant, urgency levels
- Phase 25 — NL auto-diagnose, pipeline execution in diagnose, warning cleanup
- INT-143 Phase 1 — forest digest, morning summary on long gaps
- INT-139 custom TOML patterns — load from 01-registry/shell-patterns.toml
- INT-144 v11.1.0 release gate — The Forest Speaks, requirements documented

- Commits: 1586
- Tools: 49 deployed
- Health: 95%
- Intents: 86 complete

[Full Changelog →](CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->

<!-- STATIC SECTION - Comprehensive Documentation -->

## 🤔 What is 0-Core?

**0-Core** is a completely custom Linux environment built on vanilla Arch Linux, where every component is understood, controlled, and intentionally chosen. Not a dotfiles collection — a **personal operating system built from scratch in Rust**.

### For Everyday Users

Like **building a custom motorcycle** instead of buying one from a dealer. You know every bolt, every wire, every piece.

**You get:**
- 🎨 Custom everything (terminal, bar, launcher, login screen, notifications, compositor)
- 🦀 50 Rust tools you fully understand
- 🛡️ Security through comprehension (no mystery packages)
- ⚡ Lightning fast (no bloat, no hidden automation)
- 💎 Complete ownership and control
- 🌲 A shell that knows it is a forest — and speaks to you

### For Technical People

- **`core` v2.0.0** — single orchestrator binary with 27+ native Rust domains
- **Core v8 — Evolution** — architecture reflection, coupling detection, tool lifecycle, evolution proposals, future simulation, risk analysis
- **faelight-shell v0.6.0** — forest-native structured shell with SQL queries, joins, NL translation, time travel, event system, observability dashboard
- **faelight-notify v4.0.0** — D-Bus compliant notification daemon (org.freedesktop.Notifications), fontdue::layout renderer, urgency levels
- **faelight-term** — custom Rust terminal emulator, daily driver ready
- **faelight-sandbox v3** — policy engine, namespace isolation, seccomp syscall filtering
- **24-check health monitoring** system
- **Intent Ledger** — 142 architectural decisions, 97 complete, fully documented

---

## 🏗️ Architecture
```
0-core/
├── 00-meta/          # System identity (VERSION, CHANGELOG, PHILOSOPHY)
├── engine/           # core binary — single orchestrator (Rust)
│   └── src/domains/  # 27+ native Rust domains
├── rust-tools/       # 50 custom Rust tools
├── 03-interfaces/    # Dotfiles (Niri, zsh, faelight-term, yazi) via GNU Stow
├── 04-schema/        # JSON schemas — registry validation
├── scripts/          # Deployed binaries → PATH
├── intents/          # Architectural decision records (142 intents)
├── runtime/          # SQLite state, events/, snapshots/, checkpoints/
└── 01-registry/      # Zero-logic TOML declarations + NL patterns
```

### The `core` Orchestrator
```
core <domain> <command>
Domains: doctor, security, git, workspace, intent, profile,
         zone, link, fetch, lock, notify, launcher, sandbox,
         release, update, checkpoint, simulate, plugins,
         anomaly, bootstrap, deps, narrative, snapshot,
         events, decisions, audit, advise, evolution
```

Every domain call is capability-gated, logged, and lock-protected.

### Core Intelligence Timeline

| Version | Capability | Meaning |
|---------|-----------|---------|
| v5 | Intelligence | the forest detects patterns |
| v6 | Judgment | the forest remembers outcomes |
| v7 | Resilience | the forest can rebuild itself |
| v8 | Evolution | the forest refines itself |
| v9 | Intent | the forest chooses where to grow *(planned)* |
| v10 | Reaction | the forest responds without being asked *(planned)* |

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
d             # Run all 24 health checks (alias for core doctor run)
```

Checks: stow symlinks, services, broken symlinks, yazi plugins, binary deps,
git status, themes, scripts, intents, profiles, config files, keybinds,
security hardening, security audit, alias coverage, rust toolchain,
disk space, tool installation, path resilience, archaeology, core protection,
schema validation, sandbox health.

### 🌲 Core v8 — The Forest Refines Itself
```bash
core evolution map              # architecture coupling analysis
core evolution tools            # tool lifecycle — fresh/active/stable/dormant
core evolution suggest          # evidence-backed architecture suggestions
core evolution evolve-propose   # generate formal evolution proposals
core evolution evolve-list      # list proposals with status
core evolution evolve-accept <id>  # accept → creates intent record
core evolution future-sim "change"  # simulate an architectural change
core evolution future-risk "change" # risk score for a change
core evolution future-impact "change" # blast radius analysis
```

### 🐚 faelight-shell — Forest-Native Structured Shell
```bash
fs    # launch faelight-shell (alias)
```

Not text streams. Structured data pipelines with SQL syntax:
```
ps | where cpu > 20 | sort cpu desc | first 5
ps | join ports on pid                         # ad-hoc relational joins
select name, cpu from ps where cpu > 1         # SQL query language
gchurn | where ext == rs | first 10            # git hotspot detection
find | group ext | sort count desc             # file system index
gc | where message contains feat               # git history as table
?why is my computer slow                       # natural language diagnosis
dashboard                                      # observability dashboard
snapshot before && snapshot after && snap-diff # time travel
on health_drop 90 => notify "health low"       # event triggers
```

**Phase completions in v11.1.0:**
- Phase 14 — persistent file index (`find`, `find reindex`)
- Phase 15 — git data engine (`gc`, `gchurn`, `gbr` via faelight-git native bindings)
- Phase 16 — history analytics (`hstats`, `hpattern`)
- Phase 17 — event system (shell triggers — `on`, `on list`)
- Phase 18 — time travel (`snapshot`, `timeline`, `snap-diff`)
- Phase 21 — SQL query language
- Phase 22 — observability dashboard (`dashboard`, `dashboard system`, `dashboard forest`)
- Phase 25 — NL auto-diagnose (`?why is my computer slow`)

### 🔔 faelight-notify v4 — Proper Notifications

After 2 months of development — fixed. v4 rewrites everything:

- **D-Bus compliant** — `org.freedesktop.Notifications` spec
- **fontdue::layout renderer** — clean text, correct baseline, no jagged letters
- **Urgency levels** — green (normal), red (critical), muted (low)
- **Works with everything** — Brave, systemd, notify-send, any app
```bash
notify-send "Title" "Message"
notify-send -u critical "Alert" "Something needs attention"
notify-send -u low "Info" "Low priority message"
```

### 🖥️ faelight-term — Custom Rust Terminal

Daily driver ready:
```bash
ft    # launch faelight-term
```

- Ctrl+R atuin history search ✅
- Bracketed paste mode ✅
- BTM and TUI apps render correctly ✅
- Release build — performance matches foot ✅

### 🌅 Forest Digest — Morning Intelligence

On long gaps (4+ hours) or morning sessions, the shell greets you with:
```
🌲 Good morning.

→ Since last session:
  · 7 new commits
  · Health: 95% healthy
  · Working on: INT-120, INT-141
```

### 🧠 Natural Language Pipelines
```bash
?biggest files           → find | sort size desc | first 10
?memory hogs             → ps | sort memory desc | first 5
?why is my computer slow → auto-diagnose CPU + memory + disk
?active triggers         → on list
?forest dashboard        → dashboard
```

Custom patterns via TOML:
```toml
# ~/0-core/01-registry/shell-patterns.toml
[[pattern]]
phrases = ["my commits today"]
pipeline = "gc | first 10"
context = "git"
```

### 🧪 faelight-sandbox v3 — Security Boundary
```bash
faelight-sandbox run --policy strict -- ./unknown-script.sh
faelight-sandbox run --isolate net -- curl example.com
faelight-sandbox run --isolate full -- cargo build
faelight-sandbox run --isolate seccomp -- ./untrusted
faelight-sandbox run --profile -- cargo build
```

### 📋 Intent Ledger

Every architectural decision is recorded:
```bash
core intent list          # all 142 intents
intent show 141           # story of faelight-notify v4
```

97 complete. 10 planned. Nothing is built without intent.

---

## 🦀 The Rust Ecosystem (50 Tools)

| Domain | Key Tools |
|--------|-----------|
| **Orchestrator** | `core` (27+ domains, 24 health checks) |
| **Compositor** | `faelight-compositor` (Smithay, DRM, GBM) |
| **Shell** | `faelight-shell` (SQL, joins, NL, time travel, events, dashboard) |
| **Terminal** | `faelight-term` (custom Wayland terminal, daily driver) |
| **Notifications** | `faelight-notify` v4 (D-Bus, fontdue::layout, urgency levels) |
| **Security** | `faelight-sandbox` v3 (policy, namespaces, seccomp), `faelight-gen` |
| **UI** | `faelight-bar`, `faelight-menu`, `faelight-palette`, `faelight-wallpaper` |
| **Git** | `faelight-git` (risk scoring, event emission), `faelight-release` |
| **System** | `faelight-idle`, `faelight-lock`, `faelight-login` |
| **Tools** | `faelight-fm`, `faelight-browser`, `faelight-fetch`, `faelight-forecast` |
```bash
core audit scan          # score all 50 tools
core evolution suggest   # architecture suggestions from evidence
core decision patterns   # decision history analysis
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
| v10.9.0 | Roots and Branches | Core v7 complete, faelight-compositor first render |
| v11.0.0 | Where the Forest Becomes Whole | Shell speaks, sandbox v3, faelight-term |
| v11.1.0 | **The Forest Speaks** | Core v8 complete, faelight-shell phases 14-25, faelight-notify v4 fixed |

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
- Security audit with debt tracking (`core security debt/trend/simulate`)
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
core evolution map     # see the architecture
```

Or use the forest's own guidance:
```bash
core bootstrap plan    # what needs to be done to rebuild
core bootstrap verify  # verify current state
core evolution suggest # what should change next
```

---

*"The forest that speaks is the forest that connects."* 🌲

*"Not text streams. Not configuration. Structured wisdom."* 🌲
