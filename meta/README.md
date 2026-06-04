<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 14.1.0

![Version](https://img.shields.io/badge/version-14.1.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware personal computing environment built from first principles. Pure Rust. No Electron. No telemetry.**

## 🎊 14.1.0 -- Research and Resilience (2026-05-26)

### ✅ What Shipped

- Core v23 -- Friday Becomes Central
- Faelight Forest COSMIC Direction -- fsh as authority, COSMIC as visual layer
- Faelight-term v3 replaces v2 -- clean transition, clipboard, cursor, resize, path resilience
- Faelight-fm v2 -- COSMIC Files study, libcosmic, forest-first file manager
- Forest Event Bus v2 -- zbus D-Bus integration, system-level forest signals
- Faelight-bar v3 -- COSMIC panel study, ironbar, eww, i3status-rust, quickshell, libcosmic
- Faelight-term semantic intelligence -- shell integration, editor-aware protocols, structured command objects, tree-sitter
- Forest resilience -- keyboard-only mode and hardware failure recovery
- Power-profiles-daemon + Friday integration -- intelligent power management
- Faelight-compositor v2 -- client connections, XDG protocols, DRM backend
- Forest Tool Ecosystem -- cargo tools audit, unused removal, new Rust tools research
- Faelight-term v3 stabilization -- heredoc support, Ctrl+[ fix, nested compositor rendering
- Fsh v3 -- tab completion, structured output, PowerShell ideas, startup improvement
- Fsh v4 -- The Shell Grows Up
- Intent Ledger v3 -- clarity, gate enforcement, deferral control, in-progress separation
- Faelight-git v5 -- intelligence, integrity, drift prevention
- Fsh v4 -- borrow the best from Fish, Zsh, Nu -- autosuggestions, structured data, semantic verbs
- JDbrowser TUI SQLite patterns for core db browse
- Pinnacle compositor Smithay patterns for faelight-compositor v3
- Terax ADE patterns for faelight-term v4 and Friday Chat convergence
- Jarvis Purge -- remove all jarvis references, tables, and checks from the forest
- Friday Reasoning Engine -- Causal Chain Rule for Tool Retirement Regression Detection

### 🔧 Notable Changes

- Release v14.0.0 - 🌲 The Forest Owns the Screen

## 🌲 Forest DNA

| | |
|---|---|
| 🛠 **Tools** | 51 custom Rust tools |
| 📋 **Shipped** | 271 features complete |
| 🏥 **Health** | 100% |
| ⚡ **Stack** | Rust · Wayland · Smithay · ratatui · wgpu |
| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |

> Built by one developer. Every tool written or fully understood.

[Full Changelog →](00-meta/CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->
**0-Core** is a completely custom Linux environment built on vanilla Arch Linux. Every component is written from scratch in Rust, understood completely, and chosen deliberately.
Not a dotfiles collection. Not a rice. A **personal operating system with a philosophy**.
The philosophy: *manual control over automation, understanding over convenience, recovery over perfection.*
Every tool knows the forest it lives in. Every commit knows its intent. Every deploy warns before it acts. The system is building toward Friday -- a living intelligence that converses, learns, and grows with its builder.
---
0-Core
├── core (3.0.0)          -- single orchestrator, 50+ domains, the brain
├── faelight-shell (0.7.0) -- forest-native shell, SQL queries, native pipes
├── faelight-git (4.0.0)   -- intent-aware commits, risk warnings, rollback
├── faelight-link (3.0.0)  -- forest-aware dotfile intelligence
├── faelight-release (2.0.0) -- intelligent release synthesis
├── faelight-term          -- custom VTE terminal (in progress)
├── faelight-bar           -- custom Wayland bar (smithay + tiny-skia)
├── faelight-notify        -- D-Bus notifications, fontdue renderer
├── faelight-vault         -- forest-native credential manager (Argon2id)
└── 40+ more tools         -- all Rust, all intentional
**Compositor:** Niri (Wayland, scrollable tiling)
**Display:** AMD Radeon 780M -- 2560×1600 @ 165Hz
**Kernel:** Arch Linux -- rolling, understood
---
The forest is not just tools. It is a coordination system.
**State** -- everything flows through `state.db` (SQLite, WAL mode)
**Signals** -- tools communicate via `engine_signals` without direct calls
**Patterns** -- every commit, deploy, and session writes to pattern tables
**Prediction** -- `core predict next` surfaces the right intent at the right time
**Alignment** -- values declared, violated actions flagged, drift tracked
**Friday** -- dormant, watching, learning from every signal (coming next)
---
| Version | Theme | Milestone |
|---------|-------|-----------|
| v11.8.0 | The Self-Making Forest | Deploy/git/release intelligence, signal coordination |
| v11.7.0 | The Intelligence Arc | Core v17 pattern weights, Tool Intelligence L1-L3 |
| v11.6.0 | The Shell Lives | fsh native execution, zero-/tmp workflow |
| v11.5.0 | The Forest Thinks | Core v15 alignment, prediction engine |
| v11.4.0 | The Bloom | faelight-bar rebuild, Niri migration complete |
| v11.0.0 | Where the Forest Becomes Whole | Full Niri commitment |
| v10.x | The Intelligent Forest | Pattern learning, health monitoring |
---
Nothing runs without explicit human authorization.
Every change is intentional. Every tool is understood.
The core is immutable -- locked with chattr +i between sessions.
- **UFW + fail2ban** -- network hardening
- **faelight-vault** -- Argon2id credential manager
- **faelight-sandbox** -- policy engine, namespace isolation, seccomp
- **Immutable core** -- `lock-core` / `unlock-core` ritual
- **22-check health monitoring** -- `d` before and after everything
---
Every feature, fix, and architectural decision is an intent.
intents/
├── future/    -- 17 planned
├── complete/  -- 176 shipped
└── incidents/ -- 9 resolved
Intents are the unit of work. `cistart` before. `cicomplete` after. The forest remembers every intent that was ever started, what it delivered, and what came next.
---
**Friday** -- the living intelligence. Reads every signal. Learns every pattern. Speaks when it matters.
**Core v18** -- Synthesis Engine. The forest speaks with one voice.
**Prediction v2** -- dependency-aware ordering. The right intent at the right time.
**fsh v6** -- forest-aware completions, abbreviations, live intent timer.
---
*Built by one person. Understood completely. Growing deliberately.*
*"The forest remembers. The human decides."*
| Version | Name | Capability |
|---------|------|------------|
| v17 | Pattern Weights | Weighted signal engine |
| **v18** | **Synthesis Engine** | **One voice -- friday_brief live** |
| v19 | Friday Phase 1 | planned |
| v20 | Friday Phase 2 | planned |
Binary version (`core 3.0.0`) = infrastructure stability. Intelligence version = capability tier. `core version` shows both.
