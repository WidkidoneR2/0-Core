<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 12.0.0

![Version](https://img.shields.io/badge/version-12.0.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 12.0.0 - 🌲 The Forest Speaks Human (2026-05-02)

- 180 — Sway Removal — Full Niri Commitment
- 203 — Friday: The Living Intelligence
- 219 — Core v20 — Friday Phase 2: Deep Pattern Synthesis and Predictive Strategy
- 231 — Friday Vocabulary Expansion -- Name the Patterns the Forest Repeats
- 232 — faelight-term v2 -- The Terminal That Thinks in 2136
- 233 — fsh v8 -- Structured Shell: Data, Syntax, Intelligence
- 234 — Core v21 -- Friday Planning Layer: From Reaction to Anticipation
- 238 — forest-stats -- The Forest Visualizes Its Own Growth
- 241 — Integrity Engine Audit -- No Phantom Fixes, No Silent Failures
- 242 — faelight-login v2 -- The Forest Greets You First
- 248 — friday_knowledge deduplication -- clean signal from noise
- 249 — fsh heredoc delimiter leak warning
- 250 — Release tool intelligence layer -- faelight-release and faelight-docs translate, not just account
- 252 — fsh connection unification
- 253 — gt -- Git Workflow as Ratatui TUI
- 254 — it -- Intent Ledger as Ratatui TUI
- 257 — Strip Tracked Versioned Binaries from Git History via filter-repo
- 258 — Ctrl+D Health Display as Ratatui TUI
- 259 — Command and Keybind Registry -- Universal Source of Truth
- 260 — Cheatsheet TUI -- Live Reference for Commands and Keybinds
- 261 — fsh Vocabulary -- A Human-Readable Language for the Forest
- 262 — \"faelight-term Dead Code Audit and Renderer Decision\
- 263 — \"db -- native state.db query builtin for fsh\
- 264 — \"faelight-synthesis -- Semantic Translation Layer for Release Intelligence\
- 266 — \"fsh vocabulary -- copy move list read write as forest-native commands\
- INT-231 complete — Friday vocabulary: 6 abstractions named, language born from real patterns
- INT-233 gp fixed, clippy scoped to staged packages, echo redirect verified
- INT-233 rspatch \n newline fix — world\nextra now becomes two lines
- INT-233 command not found suggests alternative, fsh-patch helper added
- INT-233 startup 4ms verified, em dash handled, echo gate marked

- Commits: 2421
- Tools: 50 deployed
- Health: 100%
- Intents: 212 complete

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
