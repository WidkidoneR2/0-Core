---
id: 114
date: 2026-03-07
type: future
title: "faelight-release — Intelligent Release & Generation Manager"
status: planned
tags: [release, versioning, generations, rollback, changelog, rust, architecture]
---

## Vision

**The forest already knows everything needed for a release. faelight-release just reads it.**

`bump-system-version` today is a ceremony — paste features, paste stats, paste
quotes, confirm five prompts. Every release requires manual gathering of
information the system already has: git log, intent ledger, health score,
tool versions, commit count.

`faelight-release` replaces that ceremony with intelligence. One command.
The forest publishes itself.

Beyond publishing, `faelight-release` introduces the generation model —
each release is an immutable snapshot. Rollback becomes switching a pointer.
No reinstalling. No rebuilding. Instant revert.

---

## The Generation Model

A generation is an immutable record of a released system state.
```
00-meta/releases/
  10.4.0/
    manifest.toml          # machine-readable release record
    installed-tools.toml   # tool versions at release time
    health-at-release.json # health snapshot
    intents-shipped.md     # intents closed in this release
  10.5.0/
    ...

runtime/generation         # single file — current active generation
```

**Immutable rule:** generations are never modified after creation.
A new release always creates a new generation directory.
Rollback switches `runtime/generation` — nothing else changes.

---

## Command Surface
```
faelight-release <version>       # publish new release
faelight-release history         # list all releases with summaries
faelight-release diff <version>  # what changed since a version
faelight-release rollback        # revert to previous generation
faelight-release rollback <ver>  # revert to specific version
faelight-release manifest        # show current generation manifest
faelight-release status          # current generation + health
```

---

## Intelligent Release Publishing

When you run `faelight-release 10.5.0`:

### 1. Pre-flight
- Run `d` — verify 95%+ health
- Verify git working tree clean
- Verify current version in `00-meta/VERSION`
- Check last git tag exists

### 2. Automatic Data Gathering
All of this is read, not typed:

**From git log (LAST_TAG..HEAD):**
- Group commits by conventional prefix: `feat/fix/perf/refactor/docs/chore`
- Count total commits since last release
- Extract breaking changes

**From `intents/complete/`:**
- Find intents with completion date after last release tag
- List as "Completed Intents" section

**From `01-registry/tools.toml`:**
- Diff tool versions against last release manifest
- Identify tools added, removed, version-bumped

**From `runtime/state.db`:**
- Current health score
- Total commits
- Security findings count
- Doctor run trend

### 3. Generation Creation
```
00-meta/releases/10.5.0/
  manifest.toml
  installed-tools.toml
  health-at-release.json
  intents-shipped.md
```

**manifest.toml example:**
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
```

### 4. Changelog Generation
Auto-generated `CHANGELOG.md` entry:
```
## v10.5.0 — The Intelligent Forest (2026-03-07)

### Completed Intents
- INT-100 — core pulse, live event stream
- INT-106 — faelight-forecast, predictive health intelligence

### Features
- feat(release): faelight-release v1.0.0
- feat(forecast): predictive health intelligence

### Fixes
- fix(aliases): resolve ff conflict
- fix(doctor): Niri-aware keybind check

### Stats
- Health: 95% | Commits: 1334 | Tools: 39 deployed
```

### 5. Git Operations
- Update `00-meta/VERSION`
- Update `README.md` dynamic section
- Commit all changes
- Create annotated git tag `v10.5.0`
- Push commits and tag

### 6. Generation Pointer
Write `runtime/generation`:
```
10.5.0
```

---

## Rollback
```
faelight-release rollback
```

Reads `runtime/generation`, finds previous generation in `00-meta/releases/`,
switches the pointer. Emits rollback event to event ledger.
```
faelight-release rollback 10.3.0
```

Walks back to specific version. Shows manifest diff before confirming.

**What rollback does:**
- Switches `runtime/generation` pointer
- Checks out the git tag for that version
- Runs `d` to verify health post-rollback
- Emits `release.rollback` event to ledger

**What rollback does NOT do:**
- Does not touch packages (that's `faelight-update`)
- Does not modify `intents/`
- Does not rewrite history

---

## History & Diff
```
faelight-release history
```
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📦 Release History
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  10.4.0  2026-03-03  Niri Version           95%  1308 commits  ← current
  10.3.0  2026-02-27  Core v3 Complete       95%  1282 commits
  10.2.0  2026-02-20  VTE Refactor           95%  1201 commits
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
```
faelight-release diff 10.3.0
```

Shows commits, intents, and tool changes between 10.3.0 and current.

---

## Replaces

- `bump-system-version` — retired after `faelight-release` ships
- Manual changelog editing
- Manual stats gathering
- Manual git tag creation

`bump-system-version` is kept as a read-only archive. Its logic is absorbed
and improved by `faelight-release`.

---

## What Stays The Same

- `faelight-update` — untouched, still handles package/tool updates
- `00-meta/VERSION` — still the source of truth for current version
- `CHANGELOG.md` — still exists, now auto-generated
- Git tags — still created, now automated

---

## Build Order

### Phase 1 — Generation Foundation
Create `00-meta/releases/` structure.
Backfill manifests for 10.3.0 and 10.4.0 from git history.
Write `runtime/generation` for current version.
No new tool yet — just the data structure.

### Phase 2 — Intelligent Changelog
Build the auto-changelog engine:
- `git log` parser grouped by conventional commit prefix
- Intent ledger diff (complete since last tag)
- Tool version diff against last manifest
Output: `CHANGELOG.md` entry + `intents-shipped.md`

### Phase 3 — faelight-release publish
The full release command:
`faelight-release <version>`
Pre-flight → gather → generate → git ops → generation pointer.
Retires `bump-system-version`.

### Phase 4 — History & Diff
`faelight-release history`
`faelight-release diff <version>`
Reads from `00-meta/releases/` manifests.

### Phase 5 — Rollback
`faelight-release rollback`
Generation switching with health verification.
Event ledger integration.

---

## Success Criteria

- [ ] Generation structure created and backfilled
- [ ] Auto-changelog from git log + intent ledger
- [ ] faelight-release publish — zero manual input
- [ ] history and diff commands working
- [ ] rollback switches generation pointer
- [ ] bump-system-version retired
- [ ] release events in event ledger

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
