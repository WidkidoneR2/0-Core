---
id: 098
date: 2026-02-27
updated: 2026-02-27
type: future
title: "Core v4 — The Reliable System"
status: planned
tags: [v11, architecture, recovery, intent-discipline, security, reliability]
version: 11.0.0
absorbs: [052]
---

## Vision

**v2 gave structure. v3 gave awareness. v4 gives discipline — the forest stays on course.**

v3 made the system self-aware. It knows its history, can simulate its future,
and broadcasts events in real time. But awareness without discipline is just
noise. v4 closes the loop:

- When something goes wrong, there is always a way back (Recovery)
- When work is in progress, the system holds you to it (Intent Discipline)
- When vulnerabilities exist, the system tracks debt and celebrates reduction (Security)

The philosophy stays the same: **manual control over automation**.
v4 makes you more disciplined, not less in control.

---

## The Three Pillars

### Pillar 1 — Recovery Engine
**"There is always a way back."**

Before any risky operation — update, release, major refactor — the system
captures a checkpoint. Named snapshots of full system state: package versions,
tool versions, config hashes, git HEAD, health score. Recovery is one command.
```
core checkpoint create <name>    # snapshot current state
core checkpoint list             # all named checkpoints
core checkpoint diff <name>      # what changed since checkpoint
core checkpoint restore <name>   # restore to named state
core restore last-good           # restore to last healthy checkpoint
```

Two tiers:
- **Automatic** — lightweight state manifest written before every risky op.
  Captures versions + hashes. Milliseconds. Always on.
- **Named** — btrfs reflink of ~/0-core via faelight-sandbox infrastructure.
  Explicit, manual, exact. Used before major version bumps.

The causality engine (v3) already knows what changed and when.
Recovery closes the loop — not just "what changed" but "undo it."

Implementation:
- `runtime/checkpoints/` — TOML manifests with version + hash state
- `core restore` reads manifest, applies inverse operations per domain
- Integration with bump-system-version — auto-checkpoint before every release
- Integration with faelight-update — checkpoint before system updates

---

### Pillar 2 — Intent Discipline
**"One thing at a time. Finish what you started."**

Absorbs INT-052 (deferred since v8.0.0 — the time is now).

The core problem: good ideas derail in-progress work. An interesting thought
becomes a new intent before the current one is done. The system watches this
and makes drift visible — not a hard block, but conscious friction.

#### Focus System
```
core intent focus <id>      # declare active intent — sets runtime focus
core intent status          # what's focused, days active, completion %
core intent drift           # what you've been doing vs what you declared
core intent unfocus         # explicitly release focus
```

Focus is stored in `runtime/focus.toml`. faelight-git's commit hook reads it.
If commits touch areas outside the focused intent's declared scope:
```
⚠️  This commit touches engine/src/domains/security/ 
    Active focus: INT-098 (core v4 planning — engine/domains/*)
    Proceed? [y/n]
```

#### Workflow States
```
planned → in-progress → testing → complete → archived
```
```
core intent start <id>      # planned → in-progress (sets focus automatically)
core intent test <id>       # in-progress → testing
core intent complete <id>   # testing → complete (moves file, clears focus)
core intent block <id>      # check dependencies before starting
```

#### Dependency Tracking
Intent frontmatter gains:
```yaml
dependencies:
  - 097   # must complete first
blocks:
  - 099   # cannot start until this done
relates:
  - 093   # related context
```

`core intent start 099` checks: "INT-097 is a dependency and is not complete.
Proceed anyway? [y/n]"

#### Templates
```
core intent new arch         # architecture template
core intent new feature      # feature template  
core intent new fix          # bug fix template
core intent new tool         # new tool template
```

Auto-generates proper frontmatter, vision section, success criteria, phase
structure. No more blank-file intents.

#### Analytics
```
core intent stats            # completion rate, velocity, avg time by type
core intent burndown         # open intents over time
core intent velocity         # intents completed per month
```

Uses the event ledger (v3) — every `intent start/complete` writes an event.
Analytics are computed from the ledger, not a separate database.

---

### Pillar 3 — Security Debt Tracking
**"Zero is the goal. Every fix is a win."**

The current security scan reports findings. v4 makes security a living metric
with history, debt tracking, and active hardening tools.

#### Security Debt Score
Every finding has an age. High severity findings older than 30 days with no
upstream patch available accumulate debt. The score is visible everywhere:
```
core security debt           # current debt score with breakdown
core security trend          # debt over time — is it improving?
core security history        # all past scans with delta
```

When upstream ships a patch and you update:
```
✅ AVG-2898 resolved — libxml2 patched
   Security debt reduced: 47 → 31 (-16)
   3 high severity findings remaining
```

The system celebrates improvement. Not "you have 17 findings" but
"you've resolved 3 findings this month — debt down 18%."

#### Active Hardening
Beyond scanning — the system knows your security posture and can improve it:
```
core security harden         # apply your own hardening policies
core security audit-config   # review SSH, sudoers, file permissions
core security check-exposure # what services are exposed on the network
```

Hardening is declarative — policies defined in `00-meta/security-policy.toml`.
Apply once, the system knows your intended posture and detects drift from it.

#### Security Events in the Bus
Every scan writes to the event ledger. `cew` shows security events live.
`cw` shows security findings in the daily summary. Security becomes part of
the system's continuous self-awareness, not a separate manual command.

---

## Continuity — v2 and v3 Maturation

v4 does not freeze v2 and v3. Alongside the three new pillars:

- **Event bus** — extend `cew` with domain filtering, structured output
- **Causality engine** — deeper payload analysis, cross-domain correlation
- **Plugin system** — plugins declare their checkpoint/restore capabilities
- **Forecasting** — health forecasting feeds into checkpoint decisions
  ("health is trending down — consider a checkpoint before proceeding")
- **Simulation** — `core simulate restore <checkpoint>` before committing

The system matures continuously. v4 adds discipline; everything else grows.

---

## Build Order

### Phase 1 — Checkpoint Foundation
**Lightweight auto-checkpoints. Zero risk.**

Write state manifests to `runtime/checkpoints/` before risky ops.
Wire into bump-system-version and faelight-update.
```
core checkpoint list
core checkpoint diff <name>
```
No restore yet — just building the capture infrastructure.

### Phase 2 — Intent Focus + Workflow States
**Absorbs INT-052. The long-deferred discipline layer.**

Focus system, workflow state transitions, commit hook integration.
```
core intent focus/unfocus/status/drift
core intent start/test/complete
```
Templates and analytics follow once states are wired.

### Phase 3 — Recovery Engine
**Restore from checkpoint. Completes the loop.**

`core checkpoint restore` — applies inverse operations from manifest.
`core restore last-good` — finds last 95%+ health checkpoint and restores.
Integration with faelight-sandbox for named btrfs snapshots.

### Phase 4 — Security Debt + Hardening
**Security as a living metric.**

Debt scoring, history tracking, trend analysis.
`core security debt/trend/history`
Declarative hardening policies.

### Phase 5 — Intent Analytics + Dependencies
**Full INT-052 completion.**

Dependency graph, `core intent stats/burndown/velocity`.
Integrates with event ledger for historical analysis.

---

## Session Rules (inherited from v3, strengthened)
```
1. One phase per session.
2. Every session ends at 95%+ health with a clean commit.
3. No phase starts until previous is tested, committed, and stable — each phase gets its own session to breathe.
4. v4 work requires INT-098 focused — practice what we preach.
5. Each phase must have a working demo before the next begins.
```

---

## Gate Check
```
✅ v10.3.0 released
✅ Core v3 all 6 phases complete
✅ Intent 093 closed
✅ INT-052 absorbed — ready to ship in Phase 2
⬜ Phase 1 unblocked — can start next session
```

---

## Stats Context (at time of writing)
```
System:    v10.3.0
Health:    95%
Commits:   1282
Intents:   65 complete, 1 planned (this one)
Security:  17 findings, 0 critical, all upstream pending
Event bus: live, 20+ doctor readings in ledger
```

---

## The Phrase

**"v2 gave structure. v3 gave awareness. v4 gives discipline — the forest stays on course."**

*"The best architectural system isn't the one that never breaks.
It's the one that always knows how to recover, and never loses sight of the work."*
