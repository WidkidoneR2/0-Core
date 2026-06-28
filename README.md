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

<!-- STATIC SECTION -->

## What is Faelight Forest?

A self-aware personal computing environment, built from first principles on **NixOS 26.05**.
One developer, working with an AI partner, building the pieces a team normally builds:
a shell, an intelligence layer, and ~38 custom Rust tools -- each one written or fully
understood. No mystery packages. No magic. No convenience at the cost of comprehension.

**~97% Rust** (113k lines), with a thin Nix layer for declarative system management and
small amounts of Lua and shell where they serve best. The forest is not Rust for its own
sake -- it is Rust because understanding every line is the point.

```
POSIX shells:      text -> text -> text
Nu shell:          table -> filter -> transform
Faelight Forest:   forest_data -> judgment -> wisdom -> anticipation -> alignment
```

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

## The thesis

One person, partnered with AI, can build what teams build -- in months, not years -- if
the work is done with understanding rather than assembly. Faelight Forest is the proof:
a coherent, self-documenting, self-aware computing environment, grown one intent at a time.
The pure-Rust-OS question is being answered, in daily use. The work now is refinement, stability, and 1.0.0.

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

Around these sit ~38 custom Rust tools -- compositor helpers, a GPU terminal, a file
manager, git governance, a release manager, a credential vault, a sandbox, and more.

**See the full, always-current tool catalog:** [rust-tools/](rust-tools/)

## Going deeper

This README is the front door. The depth lives here:

- [Theory of Operation](docs/THEORY_OF_OPERATION.md) -- how the forest thinks
- [Architecture](docs/ARCHITECTURE.md) -- how the pieces fit
- [Philosophy](docs/PHILOSOPHY.md) -- why it is built this way
- [Shell Philosophy](docs/FSH-PHILOSOPHY.md) -- the case for a human-first shell
- [Release Process](docs/RELEASE.md) -- how the forest publishes itself
- [Tool Catalog](rust-tools/) -- every active tool, generated from source
- [Changelog](meta/CHANGELOG.md) -- the full history, Arch era through NixOS

## Security

Nothing runs without explicit authorization.

- UFW firewall + fail2ban active
- faelight-vault -- encrypted credential manager
- faelight-sandbox -- policy engine with namespace isolation
- Immutable core on NixOS -- system changes are declarative and reviewable
- Health + integrity monitoring -- continuous verification
- cargo-audit on deploy -- no silent vulnerabilities

## The decision record

Every intent is documented -- not just what was built, but why, when, what the health score
was, what risk was accepted, and what happened next. The forest does not forget.

---

*Built by one developer, in partnership with AI. Every tool written or fully understood.*
🌲
