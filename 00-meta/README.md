<!-- DYNAMIC SECTION - Updated by bump-system-version -->
![Version](https://img.shields.io/badge/version-11.8.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Intents](https://img.shields.io/badge/intents-176_complete-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/built_in-Rust-orange?style=flat-square)
![Intelligence](https://img.shields.io/badge/intelligence-v18_Synthesis-brightgreen?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)
> **v11.8.0 -- The Self-Making Forest**
> The forest now participates in its own development.
19 intents shipped. The forest became self-aware about its own build process.
**Shell intelligence:** fsh v0.7.0 -- native execution layer, zero-/tmp workflow, syntax color, ht history intelligence
**Deploy intelligence:** Every deploy pre-checks health, records patterns, processes coordination signals automatically
**Git intelligence:** faelight-git v4.0.0 -- auto-intent detection, risk warnings, interactive rollback
**Dotfile intelligence:** faelight-link v3.0.0 -- every symlink traced to an intent, why/verify/audit commands
**Release intelligence:** faelight-release v2.0.0 -- synthesizes its own narrative, suggests themes from data
**Signal coordination:** Tools notice each other through engine_signals -- no direct calls, pure coordination
- Commits: 2127  ·  Tools: 50 deployed  ·  Health: 100%  ·  Intents: 176 complete
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
