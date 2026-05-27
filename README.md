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

A fully custom Arch Linux + Niri Wayland desktop built from first principles in ~96.4% Rust. Every tool is written or fully understood. No mystery packages. No magic. No convenience at the cost of comprehension.
POSIX shells:      text → text → text
Nu shell:          table → filter → transform
Faelight Forest:   forest_data → judgment → wisdom → anticipation → alignment

**Four principles that govern everything:**

1. **Understanding over convenience** -- if you don't understand it, it doesn't run
2. **Manual control over automation** -- nothing happens without explicit human authorization  
3. **Intentional design** -- every tool has a purpose, every decision has a record
4. **The forest remembers** -- every commit, decision, and intent is documented and learned from

---

## The Stack

### 🖥 faelight-compositor v2
Custom Wayland compositor built on [Smithay](https://github.com/Smithay/smithay). Auto-tiling window management, full Wayland protocol support, forest integration via state.db.

### 🐚 faelight-shell v2.1.0
The forest's own login shell. Speaks human first, UNIX as fallback.

```sh
? show health          # natural language → core doctor
deploy core            # intelligent deploy with cargo-audit
parallel { build ||| test }  # true parallel execution
friday where risk > medium   # Friday intelligence query
compare --git HEAD~3   # diff with context
```

### 🖥 faelight-term v3
GPU-accelerated terminal built on wgpu + cosmic-text. Full scrollback, Friday intelligence panel, 60fps rendering, bracketed paste, OSC 133 shell integration.

### 📁 faelight-fm v2
Forest-aware file manager built on libcosmic. Miller columns, git status per file, Friday context per directory, forest safety guard.

### 🧠 Friday Intelligence -- v45 (Forest Mind)
An intelligence layer that watches, learns, and speaks -- only when confident.

- 298 facts from 2721 commits and 248 intents
- 13 behavioral patterns with confidence scoring  
- 87% prediction accuracy
- Confidence-gated voice -- Friday speaks when it knows, stays quiet when it doesn't
- Persistent decision memory across sessions

### 🔧 core v3.0.0
A single Rust binary with 56+ native domains:

| Domain | Capability |
|--------|-----------|
| `core doctor` | 23-check health monitoring with forecast and early warning |
| `core friday` | Friday intelligence -- observe, suggest, recommend, challenge |
| `core intent` | Intent ledger -- dependency graph, velocity, health correlation |
| `core predict` | Session patterns, health trajectory, intent velocity |
| `core decisions` | Decision ledger with context fingerprints and outcomes |
| `core integrity` | 13-check integrity engine -- schema, ledger, dedup |
| `core strategy` | Planning across multiple time horizons |
| `core goals` | Forest sets its own goals -- generate, accept, reject, prioritize |

---

## 51 Tools

All written in Rust. All understood. All intentional.

| Category | Tools |
|----------|-------|
| **Compositor** | faelight-compositor v2 |
| **Display** | faelight-bar, faelight-notify, faelight-login, faelight-lock |
| **Shell** | faelight-shell v2.1.0, faelight-term v3, faelight-git, faelight-release |
| **Files** | faelight-fm v2, faelight-diff, faelight-link, faelight-clipboard |
| **Intelligence** | faelight-context, faelight-contextd, faelight-memory, faelight-digest |
| **Security** | faelight-vault, faelight-sandbox, faelight-lock v2 |
| **Updates** | faelight-update v4, faelight-maintain, faelight-pick |

---

## Security

Nothing runs without explicit human authorization.

- UFW firewall + fail2ban active  
- faelight-vault -- Argon2id encrypted credential manager  
- faelight-sandbox v3 -- policy engine, namespace isolation, seccomp  
- faelight-lock v2 -- native Rust Wayland lock via ext-session-lock-v1  
- Immutable core -- requires explicit unlock before any changes  
- 23-check health monitoring -- continuous integrity verification  
- cargo-audit + cargo-deny on every deploy -- no silent vulnerabilities  

---

## The Decision Record

274 complete intents. Every one documented -- not just what was built, but why, when, what the health score was, what risk was accepted, and what happened next. The forest does not forget.

---

## Standing on the Shoulders of Giants

Faelight Forest would not exist without the exceptional open source work of:

- **[Pop!_OS / System76](https://github.com/pop-os)** -- libcosmic, cosmic-text, cosmic-comp. The COSMIC stack is the visual layer that faelight-fm, faelight-term, and faelight-compositor are built on. Their architecture decisions shaped this forest's direction.
- **[Alacritty](https://github.com/alacritty/alacritty)** -- alacritty_terminal powers faelight-term v3's PTY layer, VTE parsing, and scrollback engine.
- **[Kitty](https://github.com/kovidgoyal/kitty)** -- Kitty's keyboard protocol and OSC sequences informed faelight-term's input handling and shell integration design.
- **[Rio Terminal](https://github.com/raphamorim/rio)** -- Rio's wgpu rendering architecture was studied during faelight-term v3's design phase. Their GPU pipeline approach influenced the rendering strategy.
- **[Smithay](https://github.com/Smithay/smithay)** -- The Wayland compositor toolkit that powers faelight-compositor v2.
- **[Niri](https://github.com/YaLTeR/niri)** -- The scrollable-tiling Wayland compositor that is the forest's daily driver.
- **[Helix](https://github.com/helix-editor/helix)** -- The modal editor at the heart of the forest's editing workflow.

---

## Installation

```bash
git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core && cargo build --release --workspace
sudo cp target/release/* /usr/local/bin/
cd 03-interfaces/stow && stow */
core doctor run
```

> ⚠️ **This system is built for one person.** It is not designed to be installed by others without deep understanding. Read the philosophy documents before touching anything. The forest has opinions.

---

## Philosophy

> *"A system that knows its values can detect when it betrays them.*
> *Alignment is not a constraint -- it is the compass that makes every decision navigable.*
> *A partner without principles is clever.*
> *A partner with principles is trustworthy."*

*Auto-generated sections maintained by faelight-docs v2.0.0*
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-05-26 21:47*
