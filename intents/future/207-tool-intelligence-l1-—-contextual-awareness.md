---
id: 207
date: 2026-04-08
type: planned
title: "Tool Intelligence L1 — Contextual Awareness"
status: in-progress
tags: [tools, intelligence, context, faelight-update, faelight-git, faelight-shell, doctor, friday, v1]
---

## The Problem

Right now our tools are functional but blind.
faelight-git commits without knowing which intent is active.
fsh executes without knowing system health.
doctor runs without showing alignment or engine status.
faelight-update checks for updates without knowing what you just built.

Each tool sees only its own domain.
None of them see the forest.

## Level 1: Contextual Awareness

The simplest and highest-impact upgrade.
Each tool reads system state before acting and shapes its output accordingly.

No learning yet. No coordination yet.
Just: know where you are before you speak.

## Tool Upgrades

### faelight-update v4.1.0
Already has system identity and drift score.
Level 1 additions:
- Show active intents at the top (what are you updating while building?)
- Show current alignment score before updating
- If a Critical-risk update is detected AND an intent is in-progress →
  warn: "Updating kernel during active development session — consider waiting"
- After update completes → log to engine_signals table
- Show engine coordination status: any engines need rebuild after update?

### faelight-git v3.1.0
Currently commits, pushes, syncs. That is all it knows.
Level 1 additions:
- Show active intent on every commit prompt: "Committing for: INT-188"
- Detect if commit message references no intent → suggest one based on changed files
- Show commit count for current intent vs total gates
- After every commit → emit signal to engine_signals (source: faelight-git, type: commit)
- If committing with uncommitted core changes → warn alignment check
- Weekly cadence summary on fg sync: "This week: 153 commits, 3 intents progressed"

### faelight-shell (fsh) v0.7.0
The shell is already intelligent. Level 1 makes it forest-aware.
Level 1 additions:
- On session start: show active intents and alignment score (not just health)
- After every failed command: check engine_signals for relevant insights
- After deploy: automatically suggest `d` if not run in last 5 minutes
- Session exit summary: "Today: 12 commands, 3 deploys, 2 commits, health stable"
- Detect when you are working on the same file 5+ times in a session →
  suggest: "You have modified commands.rs 7 times — consider a checkpoint"

### core doctor (d) v2.1.0
Currently shows health, integrity, forecast.
Level 1 additions:
- Show alignment score from v15 (core align drift summary inline)
- Show engine coordination status (core engines check inline)
- Show active pattern weight signals if any are Critical class
- If 5+ intents in-progress → flag focus concern in doctor output
- Doctor result → emit signal to engine_signals (source: doctor, type: health)
- Show Friday status: dormant | observing | active

## The Shared Pattern

Every Level 1 upgrade follows the same pattern:

1. READ state before acting
   - Active intents from intent ledger
   - Current alignment score from alignment_checks
   - Engine status from engine_registry
   - Recent signals from engine_signals

2. SHAPE output based on context
   - Surface what is relevant right now
   - Warn when context suggests caution
   - Confirm when everything looks right

3. WRITE result to shared state
   - Emit signal to engine_signals after completion
   - Log outcome to relevant state.db table
   - Update last_active in engine_registry

## Integration with Friday

Level 1 tools produce signals Friday will eventually consume.
Every tool that writes to engine_signals is teaching Friday.
Every warning that surfaces context is demonstrating what Friday will one day say.

Level 1 is the data foundation for Friday.
Build it right and Friday inherits intelligence from day one.

## Gate Check
⬜ faelight-update v4.1.0 — shows active intents and alignment before update
⬜ faelight-update — emits signal to engine_signals on completion
⬜ faelight-update — warns on Critical update during active development
⬜ faelight-git v3.1.0 — shows active intent on every commit
⬜ faelight-git — detects missing intent reference and suggests one
⬜ faelight-git — emits commit signal to engine_signals
⬜ faelight-git — weekly cadence summary on fg sync
⬜ fsh v0.7.0 — session start shows active intents and alignment
⬜ fsh — after failed command checks engine_signals for insights
⬜ fsh — session exit summary (commands, deploys, commits)
⬜ fsh — detects repeated file modifications and suggests checkpoint
⬜ core doctor v2.1.0 — shows alignment score inline
⬜ core doctor — shows engine coordination status inline
⬜ core doctor — emits health signal to engine_signals
⬜ core doctor — shows Friday status
⬜ All four tools writing to engine_signals consistently
⬜ deploy all four tools and d passes 100%

## The Phrase

"A tool that does not know where it is
cannot tell you where you are going.

Level 1 is not intelligence.
It is awareness.
The first step before wisdom
is simply: look around." 🌲
