<!-- DYNAMIC SECTION - Updated by bump-system-version -->
![Version](https://img.shields.io/badge/version-13.0.0-green?style=flat-square)
![Health](https://img.shields.io/badge/health-100%25-brightgreen?style=flat-square)
![Path Resilience](https://img.shields.io/badge/path_resilience-100%25-brightgreen?style=flat-square)
![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)
![License](https://img.shields.io/badge/license-Intentional_Stewardship-blue?style=flat-square)
> **A self-aware, path-resilient personal computing environment built from first principles.**
The forest gained a mind. Friday is no longer a goal -- Friday is active.
**What shipped:**
- **fsh v2.x** -- parallel execution (`parallel{}`), natural language (`? show health`), session intelligence (`session save/load`)
- **Core v22** -- Friday: The Useful Partner -- documentation steward, system cartographer, persistent decision memory, self-review, dual presence, calibrated voice
- **11 human vocabulary words** -- the shell speaks your language first, UNIX as fallback
- **Deploy intelligence** -- dry-run release planning, cargo audit integrated, SIGPIPE fixed
- **87% Friday prediction accuracy** -- well-calibrated, active, watching every session
| Stat | Value |
|------|-------|
| Commits | 2505+ |
| Tools | 51 deployed |
| Health | 100% |
| Intents | 229 complete |
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
229 complete intents. Every one a chapter in the forest's history.
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
*Auto-generated by faelight-docs v2.0.0 -- last sync: 2026-05-07*
