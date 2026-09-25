<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 1.0.0

![Version](https://img.shields.io/badge/version-1.0.0-green?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-96.5%25-dea584?style=flat-square)
![Lines](https://img.shields.io/badge/lines-113k-blue?style=flat-square)
![NixOS](https://img.shields.io/badge/NixOS-26.05_Yarara-7ebae4?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

> **A self-aware personal computing environment built from first principles. Pure Rust. No Electron. No telemetry.**

## 🎊 1.0.0 -- Morphwood (2026-07-06)

### ✅ What Shipped

- NixOS Era Begins -- Faelight Forest on NixOS
- Fenix: Rust 1.93+ toolchain via flake overlay
- NixOS scripts layer: lock-core, unlock-core, deploy, core-protect
- Faelight-fm v3: broot-inspired, ratatui, forest-native navigation
- Faelight-login + faelight-menu: proper NixOS login flow with greetd
- Pinnacle WM: compositor migration path, i3-style ownership model
- Friday-dev shell: nix develop environment for Friday/forest development
- Forest GitHub organization: repo structure, README, public face cleanup
- Study: Yazelix -- multiplexer + shell + FM convergence patterns
- UFW→nftables doctor fix: update hardcoded security check for NixOS
- Intent ledger NixOS improvements: intent shorthand, display, workflow
- Faelight-fm v3.1: Nix-aware, git-first, semantic navigation
- Tool audit: Nix/Rust boundary -- what should be Nix vs Rust
- Faelight-git NixOS audit: review paths, assumptions, improvements
- Retire faelight-browser: brave is the forest browser
- Faelight-notify v5: NixOS-native, noti research, layer-shell ready
- Faelight-shell v4: NixOS-native, nix develop aware, forest-first
- Pinnacle VM study: prove compositor in nixos-lab before touching real system
- Retire NixOS-obsolete tools: faelight-bootstrap, verify-bootstrap, core-protect, dotctl
- Replace faelight-wallpaper and faelight-idle with NixOS services
- Forest R&D Environment -- VM-based sandbox, experiment graduation pipeline, hypothesis-test-gate-graduate
- Core-protect retirement: remove 19-file dependency chain, NixOS-native replacement
- Forest dev tooling: nix-tree, nvd, nh, bacon, cargo-nextest
- Fsh semantic domains: project/intent/experiment as first-class shell objects
- Faelight-release v2: NixOS-native release manager
- Faelight-fm v4: full Nix explorer, plugin system, semantic engine
- Forest-aware color system: semantic colors, context themes, git regions
- Forest release v2: generation + commit + intent triad tracking
- Forest safety net: pre/post health gate, VM-first workflow, rebuild guard
- Config/ cleanup: remove Arch-era configs, retire core-diff and faelight-diff
- Rust-tools documentation: README and CHANGELOG for all 38 tools
- Pinnacle compositor config: Lua config, layer-shell, lock screen
- Fsh-completions: tab completion for domain objects and NixOS vocabulary
- Generation-diff: rich visual diff between NixOS generations
- Nix-dev-shells: per-project devShells that auto-activate on cd
- Faelight-lock v2: NixOS-native lock screen for Pinnacle and MangoWM
- Doctor v2: NixOS-aware health checks
- NixOS structure: user modules, compositor modules, flake cleanup
- MangoWM: daily driver configuration, keybinds, and autostart
- Faelight-bar v2: i3-style wlr-layer-shell bar for MangoWM and Pinnacle
- Fsh crashes (closes terminal) on df
- Making config.fsh the declarative source of Truth
- Canonical 0-Core repository structure on NixOS
- Fsh prompt: nix-context awareness -- current flake + dirty flake state
- Faelight-FM vs Superfile vs Broot
- \"faelight-logout: candy-neon Wayland power menu\
- Faelight-notify managed systemd user service
- Fsh cache commands: cache status + cache push
- Faelight-FM: full listing, arrow-key nav, and Superfile-style layout polish
- Fix intent-add numbering: derive next id across all intent dirs
- Friday: restore Nix-era parity (commit-to-intent recording, then learning)
- Decommission faelight-palette (unused since Niri 11.0.0)
- Generation count control: prune policy and boot-menu cap
- Faelight-Update v-next: update manager + generation browser
- Nix store explorer: GC roots, reverse-deps, and what keeps paths alive
- Nix package search TUI: search to declarative config-add
- Smooth VM workflow
- Faelight-vm launch hardening: atomic lock, stale-state janitor, vm debug
- Vm gui single-window: drop leftover egl-headless GL surface
- Fsh reload thinks Nix: hot-swap the rebuilt binary
- Faelight-git v-next: Nix + GitHub-native rewrite, shed Arch-era lock model
- Registry alias-hygiene: fix collapsed [[alias]] blocks in aliases.toml
- Faelight-launcher: GTK app launcher with faelight-logout-grade polish
- Remove Niri + faelight-niri-bridge (retired compositor cleanup)
- Nix Inspector: why did this value win? (option-resolution debugger)
- Fsh: clearer errors when && chains hit a builtin
- Adopt nixvim as a Nix-learning vehicle (Helix stays primary daily driver)
- Evaluate Stylix: declarative system-wide theming (vs the hand-crafted forest visual language)
- Cheatsheet v2: sync command_registry to reality + live verification (hybrid)
- Faelight-inspect TUI: themed forest UX over the Nix option-resolution debugger
- Faelight-deadwood: forest-native dead-code & orphan detector
- Fsh: kill hijacked to pattern-match -- kill <PID> does not signal that PID
- Fsh reload: identify the new build (stop blind re-exec)
- Fsh needs a clean Nix/Shell operator path
- Forest hygiene pass: registry reconciliation + Deadwood orphan cleanup
- Fsh: handle multi-line command blocks (per-line execution + abbreviation expansion)
- Fsh: variable assignment and $VAR expansion (VAR=$(...) name-case bug)
- Fsh: fresh-db schema ordering (shell_history cwd column, ALTER-before-CREATE)
- Improving Fsh Prompts
- Shell SnapShots Schema Intent
- Paths.rs consolidation follow-ups: rename rules_dir, fix hardcoded font, route hardcoded paths through the module
- Decommission Arch-era stow/link subsystem
- Profile .profile-mechanism
- Bump-versions lightweight per-tool versioning
- Final Arch sweep: retire safe-update, de-Arch fsh pkg command, purge pacman/AUR remnants for true NixOS-native 1.0.0

### 🔧 Notable Changes

- INT-030: fix intents() to read all three dirs, replace vm_list() with qcow2 scanner
- INT-033: neon candy truecolor prompt -- semantic color tokens in theme.rs, truecolor in prompt.rs
- INT-033: faelight-bar neon candy colors -- match semantic palette, health thresholds, intent purple
- INT-033: faelight-fm neon candy palette -- semantic intent file colors by status
- INT-040: domain verb subcommands + vm/rebuild dynamic completions

## 🌲 Forest DNA

| | |
|---|---|
| 🛠 **Tools** | 46 custom Rust tools |
| 📋 **Shipped** | 99 features complete |
| 🏥 **Health** | 100% |
| ⚡ **Stack** | Rust · Wayland · Smithay · ratatui · wgpu |
| 🌍 **Philosophy** | Understanding over convenience · No mystery packages |

> Built by one developer. Every tool written or fully understood.

[Full Changelog →](meta/CHANGELOG.md)

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
