---
id: 146
date: 2026-03-22
type: future
title: "faelight-shell v2 — The Shell Becomes the OS"
status: in-progress
tags: [shell, daily-driver, independence, zsh-replacement, v12, v13]
priority: high
---

## Vision

faelight-shell v1 proved the concept — structured pipelines,
NL queries, scripting, events, time travel, observability.

v2 is the migration — from a forest query tool to a daily driver
that replaces zsh for 10% of workflows, then 50%, then 100%.

The shell does not rush this. Each phase must be stable before the next.

## The Migration Path
```
Now (v11.x)    — forest queries, data pipelines, scripting
Phase 7        — external commands work, cd works, PATH works (10% driver)
Phase 8-12     — package helpers, completion, config (30% driver)
Phase 13-20    — session management, jobs, signals (60% driver)
Phase 21-28    — full interactive features (80% driver)
Phase 29-32    — zsh retirement, daily driver (100% driver)
```

## The 10% Rule

Before any phase ships:
- Run `d` in faelight-shell ✅
- Run `gc | first 5` in faelight-shell ✅
- Run `ps | sort cpu desc | first 5` ✅
- Run `histogram command` ✅

These already work. 10% is already achievable.
The goal is to make it comfortable enough that you WANT to stay.

## Phase 7 — External Commands (10% → 30% driver)

The single most important phase. Without this, fsh can't replace zsh.
```fsh
cd ~/0-core          # directory navigation
ls -la               # external command passthrough
git status           # any binary in PATH
cargo build          # build tools
nvim main.rs         # editors
```

Implementation:
- Unknown commands → check PATH → execute via execvp
- cd → update working directory in shell state
- Environment variables → read/write support
- Exit codes → propagate correctly
- stdin/stdout/stderr → passthrough correctly

Gate: `cd ~/0-core && cargo build --release -p faelight-shell` works in fsh

## Phase 8 — Job Control
```fsh
cargo build &        # background jobs
jobs                 # list running jobs
fg 1                 # bring to foreground
bg 1                 # send to background
kill %1              # kill job
```

Gate: faelight-notify running as background job from fsh

## Phase 9 — Signals & Process Groups

- Ctrl+C sends SIGINT to foreground process
- Ctrl+Z sends SIGTSTP
- Process groups properly managed

Gate: `btm` launches, Ctrl+C exits cleanly

## Phase 10 — Shell Variables & Environment
```fsh
let PATH = $PATH + ":/new/path"
export EDITOR = "nvim"
$HOME                # environment variable expansion
${var:-default}      # parameter expansion
```

Gate: `export EDITOR=nvim && nvim` works

## Phase 11 — Pipes to External Commands
```fsh
gc | first 5 | less          # pipe structured data to external
ps | sort cpu desc | head -5  # pipe to unix tools
cat file.txt | count          # unix → forest pipeline
```

Gate: `gc | first 10 | less` works

## Phase 12 — Package Helpers
```fsh
pkg install docker      # wraps pacman/yay
pkg remove docker
pkg search python
pkg list installed | where name contains python
pkg update
```

Not a new package manager — a forest-native interface to pacman.
Output is structured tables, not text.

Gate: `pkg list installed | count` works

## Phase 13 — Redirection
```fsh
gc | first 10 > commits.txt    # redirect to file
errors >> error.log             # append
cat < file.txt                  # stdin redirect
```

Gate: `gc | first 10 > /tmp/test.txt` works

## Phase 14 — Multi-Command Input
```fsh
gc | first 3; health; d        # semicolon separator
```

Gate: `gc | first 3; health` runs both commands

## Phase 15 — Configuration File
```fsh
# ~/.config/faelight-shell/config.fsh
alias ll = "ls -la"
alias gs = "git status"
set prompt_style = minimal
set history_limit = 10000
```

Gate: config.fsh loads on startup, aliases work

## Phase 16 — Interactive Improvements

- Up/down arrow key history navigation (full atuin integration)
- Ctrl+R reverse search
- Ctrl+L clear screen
- Home/End line navigation
- Alt+Backspace word deletion

Gate: Arrow keys work for history in fsh

## Phase 17 — Prompt v2

Context-aware prompt showing:
- Current directory (shortened)
- Git branch + status
- Last command exit code
- Active jobs count
- Forest health indicator

Gate: Prompt updates correctly on cd

## Phase 18 — Script Arguments
```fsh
# deploy.fsh
let tool = $1
let version = $2
run cargo build --release -p $tool
emit "tool.deployed" { name: $tool, version: $version }
```

Gate: `run deploy.fsh faelight-shell 0.7.0` works

## Phase 19 — fsh as Login Shell

- `/etc/shells` entry for faelight-shell
- `chsh -s /path/to/faelight-shell`
- Reads `/etc/profile` and `~/.profile`
- Compatible with greetd/faelight-login

Gate: Can be set as login shell without breaking system

## Phase 20 — zsh Retirement Plan

Document every zsh alias, function, and config that fsh must replace.
Migrate one by one. Keep zsh available as fallback.
Track % of daily commands handled by fsh.

Gate: 80%+ of daily commands work in fsh

## Phase 21-32 — Full Daily Driver

These phases complete the migration:
- Phase 21: Completion v2 ✅ DONE (2026-03-26) — dynamic intent IDs, core predict/react/stress subcommands
- Phase 22: Theme system ✅ DONE (2026-03-26) — forest/minimal/classic/jarvis, jarvis shows prediction inline
- Phase 23: Session persistence ✅ DONE (2026-03-27) — directory restored on startup
- Phase 24: Remote shell — ssh with forest context (lower priority)
- Phase 25: fsh as default for faelight-term ✅ NEXT
- Phase 26: Core v9 integration — goals surface in shell (partially done via predict next)
- Phase 27: Voice command foundation (INT-142, depends on voice intents)
- Phase 28: Predictive suggestions from history (HIGH VALUE — biggest daily driver win)
- Phase 29: zsh fully optional (natural progression)
- Phase 30: faelight-shell replaces zsh in autostart
- Phase 31: 93%+ Rust — only kernel interfaces remain
- Phase 32: The forest is its own operating environment

## Revised Phase Priority (2026-03-27)
```
Priority 1: Phase 25 — fsh in faelight-term (achievable now)
Priority 2: Phase 28 — predictive suggestions from history (biggest UX win)
Priority 3: Phase 26 — goals in shell (almost done)
Priority 4: Phase 29/30 — zsh retirement (as confidence grows)
Priority 5: Phase 19 — login shell (final gate, after 95%+ native)
Deferred:   Phase 24 — remote ssh (lower priority)
Depends:    Phase 27 — voice (blocked on INT-142)
```


## Phase 20 — zsh Retirement Audit (2026-03-25)

### What zsh has that fsh must replace

#### Functions — 8 total
| Function | Status in fsh | Action needed |
|----------|--------------|---------------|
| `sudo()` | ✅ works — PATH passthrough | none |
| `ya()` — yazi cd-on-quit | ✅ DONE (2026-03-26) — yazi + faelight-fm with cd-on-quit | Phase 20b |
| `git()` guardrail | ✅ DONE (2026-03-26) — git commit/push blocked when core locked | Phase 20b |
| `preexec()` intent-guard | ❌ missing | Phase 20b: triggers.rs |
| `update-check()` | ✅ works — external | none |
| `sync-0-core()` | ✅ works — external | none |
| `weekly-check()` | ✅ works — external | none |
| `dotctl()`, `dot-doctor()` | ✅ works — PATH | none |
| `command_not_found_handler` | ✅ covered — fsh error msg | none |
| `alias-help()` | ✅ replaced by help | none |

#### Aliases — 432 in zsh, 3 in fsh config
Critical aliases to port to config.fsh:

**Tier 1 — Daily use (port now)**
- `d` = `doctor` — health check
- `v` = `nvim` — editor
- `l` = `eza -lh --icons --group-directories-first`
- `b` = `bat --paging=never`
- `y` = `yazi`
- `g` = `git`
- `c` = `clear`
- `gc` = `git commit`
- `gp` = `git push`
- `gst` = `git status`

**Tier 2 — Forest tools (port now)**
- `fm` = `faelight-fm`
- `menu` = `faelight-menu`
- `bar` = `faelight-bar`
- `bump` = `faelight-release publish`
- `fu` = `faelight-update`
- `sec` = `security-audit`
- `lock` = `faelight-lock`
- `notify` = `faelight-notify`

**Tier 3 — Situational (port later)**
- All `sb-*` sandbox aliases
- All `sec-*` security aliases
- `weekly-check`, `update-check`, `update`

#### Shell features — zsh only
| Feature | Status | Phase |
|---------|--------|-------|
| `preexec` hook | ❌ not in fsh | 20b |
| git commit guardrail | ❌ not in fsh | 20b |
| yazi cd-on-quit | ❌ not in fsh | 20b |
| starship prompt | ✅ replaced by fsh prompt v2 | done |
| atuin history | ✅ fsh has own history | done |
| zsh completions | ⚠️ fsh has Phase 17b completion v1 | ongoing |

### Migration Confidence
| Category | fsh coverage |
|----------|-------------|
| External commands | 100% |
| Forest tool aliases | 20% (3/~50 ported) |
| Git workflow | 80% (guardrail missing) |
| File navigation | 70% (ya missing) |
| Pre-command hooks | 0% |
| **Overall daily driver** | **~45%** |

### Gate for Phase 19 (fsh as login shell)
- [ ] Tier 1 + Tier 2 aliases in config.fsh
- [ ] git guardrail in fsh
- [ ] preexec hook in triggers.rs
- [ ] ya() equivalent in fsh
- [ ] 80%+ daily commands handled by fsh

## Gate Check

- ✅ Phase 7  — external commands, cd, PATH DONE (2026-03-23)
- ✅ Phase 8  — job control DONE (2026-03-23)
- ✅ Phase 9  — signals DONE (2026-03-23)
- ✅ Phase 10 — shell variables DONE (2026-03-23)
- ✅ Phase 11 — pipes to external DONE (2026-03-23)
- ✅ Phase 13 — redirection DONE (2026-03-23)
- ✅ Phase 14 — multi-command input DONE (2026-03-23)
- ✅ Phase 15 — config file DONE (2026-03-23)
- ✅ Phase 16 — interactive improvements DONE (2026-03-23)

## Reordered Priority (2026-03-25 — strategy revision)
- ✅ Phase 17 — prompt v2 DONE (2026-03-25) — two-line, git branch+dirty(*), health, timing, alias recursion guard, live git_info() via --porcelain (8ms)
- ✅ Phase 17b — completion v1 DONE (2026-03-25) — path, PATH binaries, pipeline-aware, List mode grid
- ✅ Phase 12 — package helpers DONE (2026-03-25) — pkg list/search/install/remove/update, paru-backed, structured tables, pipeable
- ✅ Phase 18 — script arguments DONE (2026-03-25) — $1 $2 $# positional args, is_literal guard, correct arg splitting
- ✅ Phase 18b — flow mode DONE (2026-03-25) — flow/flow focus INT-NNN/flow clear, prompt live-updates, startup numeric guard
- ✅ Phase 20 — zsh retirement plan DONE (2026-03-25) — full audit, 28 aliases ported to config.fsh, migration confidence 45%→70%, gate for Phase 19 defined
- ⬜ Phase 19 — fsh as login shell (LAST — after 80% confidence)
- ✅ Phase 21 — context-aware completion DONE (2026-03-26)
✅ Phase 22 — theme system + jarvis DONE (2026-03-26)
✅ Phase 23 — session persistence DONE (2026-03-27)
⬜ Phase 24-32 — remaining daily driver phases

## Compatibility Contract (2026-03-25)

**faelight-shell is NOT a POSIX shell.**

It does not run bash or zsh scripts.
It runs `.fsh` scripts and forest pipelines.
For POSIX compatibility when needed, use the escape hatch:
```fsh
sh {
  awk '{print $1}' /etc/passwd | sort
}
```

This is a deliberate choice, not a limitation:
- You control every script in this system
- POSIX compatibility serves people running other people's scripts
- The forest is not trying to replace bash — it replaces the *need* for bash

**The forest model:**
```
text    → Unix shells
tables  → Nushell
buffers → Emacs
forest  → faelight-shell  (structured, observable, self-aware)
```

## The Philosophy


**The shell does not replace zsh overnight.
Each phase earns trust through stability.
You decide when to move each workflow to fsh.
The forest grows at your pace, not mine.**

## The Phrase

*"The shell that knows itself
needs no other shell to complete it.
It grows until it becomes
the ground you walk on."* 🌲
