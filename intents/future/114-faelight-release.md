---
id: 114
date: 2026-03-07
type: future
title: "faelight-release — Intelligent Release & Generation Manager"
status: planned
tags: [release, versioning, generations, rollback, changelog, rust, tui, ratatui, architecture]
---

## Vision

**The forest already knows everything needed for a release. faelight-release just reads it.**

`bump-system-version` today is a ceremony — paste features, paste stats,
paste quotes, confirm five prompts. Every release requires manually gathering
information the system already has: git log, intent ledger, health score,
tool versions, commit count.

`faelight-release` replaces that ceremony with intelligence and presence.
Like `faelight-pulse` shows the forest breathing in real time, `faelight-release`
shows the release being built — a living TUI where you see everything the
system gathered, edit the theme inline, and confirm when ready.

One command. The forest publishes itself.

Beyond publishing, `faelight-release` introduces the generation model —
each release is an immutable snapshot. Rollback becomes switching a pointer.
No reinstalling. No rebuilding. Instant revert.

**The learning principle:** each release manifest records what was written.
Over time, `faelight-release` learns your patterns — how you name themes,
which intents matter most, what commit patterns define a release. Version 1.0.0
is smart. Version 3.0.0 knows the forest.

---

## The TUI Experience

Running `faelight-release 10.5.0` opens a ratatui TUI:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌲 faelight-release v1.0.0 — building 10.5.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 Intents Shipped (4)           🔀 Commits since 10.4.0 (42)
  ✅ INT-100  core pulse            feat     12
  ✅ INT-101  faelight-login        fix       8
  ✅ INT-102  faelight-clipboard    chore     6
  ✅ INT-106  faelight-forecast     docs      4
                                    perf      2

🛠️  Tools                         🏥 Pre-flight
  new:     faelight-forecast        Health:   95% ✅
  new:     faelight-pulse           Git:      clean ✅
  new:     faelight-niri-bridge     Tag:      v10.4.0 ✅
  bumped:  faelight-notify 3.0.0    Commits:  1334

📝 Release Theme
  [ The Intelligent Forest                              ]
    ↑ editable — type your theme, press Enter to confirm

─────────────────────────────────────────────────────────────
  Enter  confirm & publish    e  edit theme    q  abort
```

You see everything. Edit the theme inline. Press Enter — it executes.
Commits, tags, pushes, writes manifest. All visible as it happens.

---

## Smart Changelog

The changelog engine understands context, not just commits.

**Intent-first grouping:**
Commits that reference an intent ID are grouped under that intent's story.
`feat(niri): add faelight-niri-bridge` and `fix(niri): correct output eDP-2`
both belong to INT-099's narrative — they appear together, not scattered.

**Conventional commit sections:**
Commits without intent references are grouped by prefix:
`feat / fix / perf / refactor / docs / chore`

**Signal over noise:**
Chore commits about dependencies are condensed to a single line.
Breaking architectural changes are surfaced prominently.
Internal housekeeping is summarized, not listed exhaustively.

**Generated output:**
```
## v10.5.0 — The Intelligent Forest (2026-03-07)

### Completed Intents
- INT-100 — core pulse — live event stream TUI
- INT-101 — faelight-login — Rust greeter, pixel perfect
- INT-102 — faelight-clipboard — native wlr-data-control, zero C
- INT-106 — faelight-forecast — predictive health intelligence

### Features
- faelight-niri-bridge — compositor events in event ledger (INT-099)
- WM abstraction — ControlSway→ControlWM, Niri-aware across all domains

### Fixes
- doctor: Niri-aware keybind check detects niri/sway automatically
- aliases: resolve ff conflict, remove duplicate forecast block

### Stats
Health: 95%  ·  Commits: 1334  ·  Tools: 39 deployed  ·  Intents: 73 complete
```

**Learning over time:**
Each release manifest stores the theme, intent count, commit distribution.
Future releases use this history to suggest themes and flag unusual patterns.
"Last 3 releases averaged 8 intents — this release has 4, confirm?"

---

## Smart README

The README has two sections — dynamic (lines 1-37, owned by the release tool)
and static (line 38+, human-maintained).

`faelight-release` writes the dynamic section from the manifest:
- version badge from new version
- health badge from `runtime/state.db`
- latest release section from auto-generated changelog
- path resilience badge from doctor output

The static section is never touched unless explicitly requested.
No hardcoded strings. No manual badge updates. The README reflects
reality because it reads from the same source of truth as everything else.

---

## The Generation Model

Each release creates an immutable generation record:
```
00-meta/releases/
  10.4.0/
    manifest.toml          # machine-readable release record
    installed-tools.toml   # tool versions at release time
    health-at-release.json # health snapshot from state.db
    intents-shipped.md     # intents closed in this release
  10.5.0/
    ...

runtime/generation         # single file — active generation pointer
```

**manifest.toml:**
```toml
version = "10.5.0"
date = "2026-03-07"
theme = "The Intelligent Forest"
git_sha = "abc1234"
health_at_release = 95
commits_since_last = 42

[tools_added]
faelight-release = "1.0.0"
faelight-forecast = "1.0.0"

[tools_version_bumped]
faelight-notify = { from = "2.0.0", to = "3.0.0" }

[intents_shipped]
ids = [100, 101, 102, 106]
titles = [
  "core pulse",
  "faelight-login",
  "faelight-clipboard",
  "faelight-forecast",
]
```

**Immutable rule:** generations are never modified after creation.
Rollback switches `runtime/generation` — nothing else changes.

---

## Command Surface
```
faelight-release 10.5.0          # publish new release (TUI)
faelight-release history         # list all releases with summaries
faelight-release diff 10.4.0     # what changed since a version
faelight-release rollback        # revert to previous generation
faelight-release rollback 10.3.0 # revert to specific version
faelight-release manifest        # show current generation manifest
faelight-release status          # current generation + health
```

---

## Rollback
```
faelight-release rollback
```

Reads `runtime/generation`, finds previous generation manifest,
shows a diff of what will change, confirms, switches pointer.
```
faelight-release rollback 10.3.0
```

Shows manifest diff between current and 10.3.0 before confirming.
Checks out git tag. Runs `d` to verify health post-rollback.
Emits `release.rollback` event to event ledger.

**What rollback does:**
- Switches `runtime/generation` pointer
- Checks out the git tag for that version
- Runs `d` to verify health post-rollback
- Emits `release.rollback` event to ledger

**What rollback does NOT do:**
- Does not touch packages (that is `faelight-update`)
- Does not modify `intents/`
- Does not rewrite history

---

## Replaces

`bump-system-version` is retired after `faelight-release` ships.
Its logic is absorbed, understood, and improved.
The binary is kept as a read-only archive — the forest remembers.

---

## Build Order

### Phase 1 — Generation Foundation
Create `00-meta/releases/` structure.
Backfill manifests for 10.3.0 and 10.4.0 from git history.
Write `runtime/generation` for current version.
No new tool yet — just the data structure in place.

### Phase 2 — Smart Changelog Engine
Build the auto-changelog core in Rust:
- `git log` parser with conventional commit grouping
- Intent ledger diff — find completed intents since last tag
- Tool version diff against last manifest
- Context grouping — commits under their intent stories
Output: structured release data ready for rendering.

### Phase 3 — faelight-release TUI
The full ratatui TUI experience:
- Live display of gathered data
- Inline theme editing
- Pre-flight checks visible
- Confirm → execute: commits, tag, push, manifest write
Retires `bump-system-version`.

### Phase 4 — Smart README Writer
Write the dynamic README section from manifest.
Badge updates, latest release section, stats line.
Never touches static section.

### Phase 5 — History, Diff & Rollback
`faelight-release history` — reads all manifests
`faelight-release diff <version>` — cross-manifest comparison
`faelight-release rollback` — generation switching with health check
Event ledger integration for release and rollback events.

### Phase 6 — Learning Layer
Store release patterns in manifest history.
Theme suggestions based on previous release naming.
Anomaly detection — flag unusual commit distributions.
Confidence grows with each release.

---

## Success Criteria

- [ ] Generation structure created and backfilled to 10.3.0
- [ ] Smart changelog from git log + intent ledger
- [ ] TUI shows all gathered data, inline theme editing
- [ ] faelight-release publish — zero manual input
- [ ] README dynamic section auto-written from manifest
- [ ] history and diff commands working
- [ ] rollback switches generation pointer with health check
- [ ] bump-system-version retired
- [ ] release and rollback events in event ledger

---

## Stats Context
```
System:   v10.4.0
Commits:  1334
Tools:    39 deployed
Health:   95%
```

---

## The Phrase

**"The forest already knows everything needed for a release.
faelight-release just reads it."**

*"A release is not an event you perform.
It is a moment the forest recognizes."* 🌲
