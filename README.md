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

<!-- STATIC SECTION -->

## What is Faelight Forest?

A self-aware personal computing environment, built from first principles on **NixOS 26.05**.
Every piece a modern desktop needs -- a shell, an intelligence layer, and 36 custom Rust
tools -- written or fully understood. No mystery packages. No magic. No convenience at the
cost of comprehension.

**~97% Rust** (109k lines across 250 files), with a thin Nix layer for declarative system
management and small amounts of Lua and shell where they serve best. The forest is not Rust
for its own sake -- it is Rust because understanding every line is the point.
POSIX shells:      text -> text -> text
Nu shell:          table -> filter -> transform
Faelight Forest:   forest_data -> judgment -> wisdom -> anticipation -> alignment

## Origin

Faelight Forest began in a failure. A catastrophic update broke a working system, and the
rebuild that followed asked a harder question than "how do I fix this?" -- it asked "why
don't I understand my own machine?" The answer became a principle: build it from parts you
understand, or don't run it at all.

That rebuild started on Arch Linux. In June 2026, after another Arch failure, the forest
migrated to **NixOS 26.05** -- a deliberate move toward declarative, reproducible,
rollback-safe computing. Every system change is now a bootable generation. Nothing is lost,
nothing is mysterious.

## Philosophy

Four principles govern everything:

1. **Understanding over convenience** -- if you don't understand it, it doesn't run.
2. **Manual control over automation** -- nothing happens without explicit authorization.
3. **Intentional design** -- every tool has a purpose; every decision has a record.
4. **The forest remembers** -- every commit, decision, and intent is documented and learned from.

This is stewardship, not consumption: the forest is tended intentionally, every part known.

## The thesis

A computing environment can be coherent, self-documenting, and self-aware -- grown one
intent at a time, with understanding rather than assembly at its core. Faelight Forest is
that proof, in daily use: a shell that speaks human, an engine that reasons about its own
health, and an intelligence layer that learns. Not text streams. Not configuration.
Structured wisdom.

## Architecture

The forest rests on three pillars, plus an ecosystem of tools:

- **fsh (faelight-shell)** -- the forest's own shell. Speaks human first, UNIX as fallback.
- **core** -- a single Rust engine of native domains: health, intent ledger, integrity,
  prediction, decisions, strategy.
- **Friday** -- an intelligence layer that watches, learns, and speaks only when confident.
  Persistent memory across sessions; confidence-gated voice.

```sh
? show health                  # natural language -> health dashboard
deploy core                    # intelligent deploy with audit
build ||| test                 # true parallel execution
friday where risk > medium     # Friday intelligence query
```

Around these sit 36 custom Rust tools -- compositor helpers, a GPU terminal, a file
manager, git governance, a release manager, a credential vault, a sandbox, and more.

**See the full, always-current tool catalog:** [rust-tools/](faelight/rust-tools/)

## Going deeper

This README is the front door. The depth lives here:

- [Theory of Operation](docs/THEORY_OF_OPERATION.md) -- how the forest thinks
- [Architecture](docs/ARCHITECTURE.md) -- how the pieces fit
- [Philosophy](docs/PHILOSOPHY.md) -- why it is built this way
- [Shell Philosophy](docs/FSH-PHILOSOPHY.md) -- the case for a human-first shell
- [Release Process](docs/RELEASE.md) -- how the forest publishes itself
- [Tool Catalog](faelight/rust-tools/) -- every active tool, generated from source
- [Changelog](faelight/meta/CHANGELOG.md) -- the full history, Arch era through NixOS

## Security

Nothing runs without explicit authorization.

- UFW firewall + fail2ban active
- faelight-vault -- encrypted credential manager
- faelight-sandbox -- policy engine with namespace isolation
- Immutable core on NixOS -- system changes are declarative and reviewable
- Health + integrity monitoring -- continuous verification
- cargo-audit on every deploy -- findings surfaced, triaged, and documented, never silent

## The decision record

Every intent is documented -- not just what was built, but why, when, what the health score
was, what risk was accepted, and what happened next. The forest does not forget.

## License

MIT -- see [LICENSE](LICENSE). Use it, learn from it, build on it.

---

*Every tool written or fully understood. Nothing runs blindly.*
🌲
