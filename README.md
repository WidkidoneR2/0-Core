<!-- DYNAMIC SECTION - Updated by bump-system-version -->
# 🌲 Faelight Forest 14.0.0

![Version](https://img.shields.io/badge/version-14.0.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)

> **A self-aware, path-resilient personal computing environment built from first principles.**

## 🎊 Latest Release

### 14.0.0 - 🌲 The Forest Owns the Screen (2026-05-17)

- 138 — faelight-compositor v2 — EGL/OpenGL First Real Frame
- 235 — Friday Daemon v2 -- Anticipation Engine: Always Watching, Always Ready
- 239 — faelight-bar v2 -- Modernized, Intelligent, Friday-Aware
- 240 — archaeology-0-core Retirement -- Clean Removal of a Legacy Tool
- 243 — faelight-lock v2 -- Native Rust Wayland Lock
- 246 — Friday Architecture v2 -- The Voice That Thinks
- 247 — Intent Ledger v2 -- The Forest That Knows Itself
- 270 — faelight-login v2 -- slint native Rust greeter
- 271 — faelight-diff -- The Forest Sees What Changed
- 272 — core-protect v2 -- Single Source of Truth
- 273 — faelight-maintain -- The Forest Stays Current
- 274 — faelight-pick -- Fuzzy Selection Everywhere
- 282 — Docs Audit and Refresh -- Philosophy, Aliases, Workflows updated to 13.x reality
- 283 — faelight-docs generates COMMAND-GUIDE -- auto-generated from core domains, never stale
- 284 — faelight-term rendering bugs -- scrollback corruption, emoji width, mouse flash
- 285 — fsh shell friction -- heredoc hash stripping, for loops, knowledge add hang, command chains
- 286 — faelight-term v3 -- wgpu + cosmic-text rebuild
- 288 — helix editor evaluation -- daily driver decision
- 291 — fsh friction for systems work -- semicolon splitting, find paths, background Wayland processes
- 297 — cargo-deny setup -- dependency audit, license compliance, security advisories
- 298 — fsh shell audit v2 -- remaining command fixes, enhanced builtins, tilde expansion, heredoc, subshell pipes
- 299 — fsh Shell Integrity v1 -- grep, awk, command reliability, structural decomposition
- 300 — fsh Language Layer v1 — The Shell Speaks Human First
- 302 — tailspin evaluation -- log colorizer forest integration
- 303 — pastel evaluation -- color tool for forest color DNA workflow
- 304 — fsh test infrastructure v1 -- permanent regression suite, Friday-aware
- 305 — deploy pipeline v2 -- intelligent, dependency-aware, forest-native
- 310 — Forest Version Intelligence -- auto-versioning, smart releases, engine coherence

| Stat | Value |
|------|-------|
| Commits | 2719 |
| Tools | 50 deployed |
| Health | 100% |
| Intents | 248 complete |

[Full Changelog →](00-meta/CHANGELOG.md)

---

<!-- END DYNAMIC SECTION -->
<!-- STATIC SECTION -->
A fully custom Arch Linux + Niri personal computing environment built from first principles in ~96.4% Rust. Every tool is written or fully understood. No mystery packages. No magic.
POSIX shells:      text | text | text
Nu shell:          table | filter | transform
Faelight Forest:   forest_data | judgment | wisdom | anticipation | alignment
**Four principles that govern everything:**
1. **Understanding over convenience** -- if you don't understand it, it doesn't run
2. **Manual control over automation** -- nothing happens without explicit human authorization
3. **Intentional design** -- every tool has a purpose, every decision has a record
4. **The forest remembers** -- every commit, decision, and intent is documented and learned from
---
**`core` v3.0.0** -- a single Rust binary with 56+ native domains:
| Domain | Capability |
|--------|-----------|
| `core predict` | session patterns, health trajectory, intent velocity, coupling risk |
| `core react` | health advisory, security aging, checkpoint staleness, intent overflow |
| `core strategy` | planning across multiple time horizons |
| `core goals` | forest sets its own goals -- generate, accept, reject, prioritize |
| `core doctor` | 23-check health monitoring with forecast, trend, and early warning |
| `core integrity` | 13-check integrity engine -- intent ledger, schema validation |
| `core friday` | Friday intelligence layer -- observe, suggest, recommend, challenge |
| `core intent` | ledger v2 -- blocked, next, brief, graph, dependency enforcement |
| `core genealogy` | full intent family tree from INT-001 to present |
| `core decisions` | decision ledger with context fingerprints and outcomes |
---
**faelight-shell v2.1.0** -- the forest's own voice. Login shell since 2026-04-03.
```fsh
tools | where score > 80
deploy core > /tmp/build.log 2>/dev/null
? show health
parallel { cargo build -p core ||| cargo build -p faelight-shell }
core align check "starting new intent"
friday where risk > medium
compare --git HEAD~3
```
51 custom Rust tools, each understood and intentional:
| Category | Tools |
|----------|-------|
| **Display** | faelight-bar, faelight-notify v4, faelight-login, faelight-lock, faelight-menu |
| **Shell** | faelight-shell v2.1.0, faelight-term, faelight-git, faelight-release |
| **Intelligence** | faelight-context, faelight-contextd, faelight-memory, faelight-digest |
| **Updates** | faelight-update v4.0.0 -- risk levels, drift, pre-flight, suggestions |
| **Security** | faelight-vault, faelight-sandbox v3, faelight-lock v2 |
| **Filesystem** | faelight-fm, faelight-link, faelight-clipboard, faelight-diff |
---
Nothing runs without explicit human authorization.
Every change is intentional. Every tool is understood.
- **UFW** firewall + **fail2ban** active
- **faelight-vault** -- Argon2id encrypted credential manager
- **faelight-sandbox v3** -- policy engine, namespace isolation, seccomp
- **faelight-lock v2** -- native Rust Wayland lock via ext-session-lock-v1
- **Immutable core** -- requires explicit unlock before any changes
- **23-check health monitoring** -- continuous integrity verification
- **13-check integrity engine** -- ledger, schema, jarvis, duplicate detection
---
Every decision is documented. Not just what -- but why, when, what health score, what risk, what happened next.
248 complete intents. Every one a chapter in the forest's history.
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
Alignment is not a constraint -- it is the compass that makes every decision navigable.
A partner without principles is clever.
A partner with principles is trustworthy."*
*Auto-generated by faelight-docs v2.0.0 — last sync: 2026-05-17 01:23*
