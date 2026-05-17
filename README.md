<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 14.0.0

![Version](https://img.shields.io/badge/version-14.0.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-96.4%25-orange?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Arch_Linux_+_Niri-blue?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-purple?style=flat-square)

> **A self-aware personal computing environment built entirely from first principles in Rust.**
> Every tool written. Every decision documented. Nothing installed blindly.

---

## 🎊 v14.0.0 -- The Forest Owns the Screen

*Released 2026-05-17*

This release brings the forest's own compositor, file manager, and terminal into full production. The forest no longer depends on third-party window management -- it owns the screen from boot to shell.

**What shipped:**

- **faelight-compositor v2** -- A custom Smithay-based Wayland compositor. Auto-tiling, forest color borders, state.db window events. foot and faelight-term connect with zero protocol warnings.
- **faelight-fm v2** -- A libcosmic file manager. Miller columns, git status per file, Friday context per directory, forest safety guard on deletes.
- **faelight-term v3** -- GPU terminal rebuilt on wgpu + cosmic-text. Full scrollback, copy/paste, bracketed paste (heredoc works), 60fps rendering, Friday panel.
- **faelight-shell v2.1.0** -- The forest's own login shell. Natural language vocabulary, parallel execution, session save/load/replay, OSC 133 shell integration.
- **Friday intelligence v45** -- Forest Mind. 298 facts, 13 patterns, 87% prediction accuracy. Persistent decision memory, confidence-gated voice, system cartographer.
- **Deploy pipeline v2** -- cargo-audit, cargo-deny, rollback, parallel deploy, Friday deploy intelligence. Every deploy verified.
- **fsh test infrastructure** -- 81 tests, regression hard block, Friday-aware coverage reporting.
- **Forest Version Intelligence** -- Auto-versioning from git diff analysis. MAJOR/MINOR/PATCH classification. Intelligence version auto-computed from Friday state.

**Fixes:**
- Bracketed paste protocol in faelight-term -- heredoc now works
- Paste to browser fixed -- wl-clipboard-rs 0.9.3
- Shell pipeline fixes -- semicolons, heredoc, for loops, logical chains
- Zombie process reaping, SIGPIPE handling, cold-start test fluke eliminated

| Stat | Value |
|------|-------|
| Commits | 2726 |
| Tools | 51 deployed |
| Health | 100% |
| Intents complete | 248 |
| Friday facts | 298 |
| Test coverage | 81 tests |

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

248 complete intents. Every one documented -- not just what was built, but why, when, what the health score was, what risk was accepted, and what happened next. The forest does not forget.

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
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-05-17 02:00*
