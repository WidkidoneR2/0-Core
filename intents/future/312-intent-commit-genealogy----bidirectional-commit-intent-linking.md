---
id: 312
title: "Intent-Commit Genealogy -- bidirectional commit-intent linking, research trail, fallback intelligence"
status: planned
date: 2026-05-16
type: intelligence
tags: [git, intents, genealogy, research, history, traceability, friday, state.db]
depends_on: [247]
---
## The Problem

A commit hash like 7b55031 carries no context about:
  - Which intent was active when it was made
  - What phase of that intent it represents
  - Which gates had just been met
  - What the forest health was at that moment
  - What Friday knew at that moment
  - Whether it was part of a fix, a build, or a revert

An intent like INT-310 carries no direct link to:
  - Every commit made while it was active
  - Which commits fixed which gates
  - Which commits introduced problems
  - The exact sequence of decisions made

This means:
  - Going back to understand a change requires manual archaeology
  - Reverting an intent's work requires guessing which commits to revert
  - Research into past decisions loses context
  - Debugging a regression requires manually correlating timestamps

"We have memory but no memory of what the memory means."

---
## The Vision

Every commit in the forest is a node in a graph.
Every intent is a cluster of commits.
Every gate completion is a milestone on that cluster.
Every health check is a snapshot on that timeline.

Given any commit hash:
  core genealogy show 7b55031
  → INT-310 (forest version intelligence) — active at commit time
  → Phase: planning (intent created in this commit)
  → Gates met: none yet (intent just created)
  → Health: 100% at commit time
  → Friday: 287 facts, 13 patterns
  → Session: 20260516-202827-32607
  → Previous commit: 3fb4bfcd (INT-305 complete)
  → Next commit: (next session)

Given any intent:
  core genealogy intent 305
  → 10 commits while active
  → First commit: faf07206 (Phase 1 start)
  → Last commit: 3fb4bfcd (all gates met)
  → Duration: 1 session (4.5 hours estimated)
  → Gates met in order: [cargo-audit, rollback, smoke tests, deps, parallel, assets, friday]
  → Health: 100% throughout
  → Commits by phase:
      Phase 1: faf07206, 34c939d5, 0dc23c19
      Phase 2: faf07207 (rollback)
      Phase 3: (smoke tests — INT-304 gate)
      Phase 4: 075eece8 (dependency ordering)
      Phase 5: 55195d62 (parallel deploy)
      Phase 6: e0336cb3 (asset bundling)
      Phase 7: 50d5ae45 (Friday intelligence)
      Final:   3fb4bfcd (all gates met)

---
## Architecture

### New state.db table: intent_commits

CREATE TABLE intent_commits (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    commit_hash     TEXT NOT NULL,
    intent_id       INTEGER,
    intent_status   TEXT,        -- planned/in-progress/complete at commit time
    phase_hint      TEXT,        -- "Phase 1", "Phase 2", etc. parsed from message
    gate_hint       TEXT,        -- gate description parsed from commit message
    health_at       INTEGER,     -- health % at commit time
    friday_facts    INTEGER,     -- friday fact count at commit time
    friday_patterns INTEGER,     -- friday pattern count at commit time
    session_id      TEXT,        -- session active at commit time
    committed_at    INTEGER,     -- unix timestamp
    author          TEXT,
    message         TEXT
);

### Population Strategy

1. REAL-TIME: fg done hook writes to intent_commits on every commit
2. BACKFILL: parse git log to reconstruct history from commit messages
   - Extract INT-NNN from commit messages
   - Match timestamps to session records
   - Match health snapshots from checkpoints

### Core Commands

core genealogy show <commit>
  - Full context for any commit hash
  - Intent, phase, gates, health, Friday state

core genealogy intent <INT-NNN>
  - All commits while intent was active
  - Chronological with phase markers
  - Gate completion milestones highlighted

core genealogy diff <commit1> <commit2>
  - What changed between two commits
  - Which intents were active
  - Health delta
  - Friday knowledge delta

core genealogy blame <file>
  - Which intent caused each section of a file to exist
  - Forest-aware git blame

core genealogy search <term>
  - Find commits by intent, phase, gate description
  - Full text search across commit messages + intent context

core genealogy rollback-plan <INT-NNN>
  - Generate exact commit list to revert an intent
  - Topologically sorted for safe revert order
  - Shows which commits are safe to revert vs risky

### Friday Integration

Friday reads intent_commits to:
  - Understand which intents cause health drops
  - Correlate commit patterns with success rates
  - Surface "this type of change historically causes regressions"
  - Suggest: "INT-287 work at this depth usually needs 3 commits to stabilize"

Friday can answer:
  "What was I doing when commit 7b55031 was made?"
  "Which intent has the most commits per gate?"
  "What was the health trend during INT-305?"
  "Which commits have been reverted most often?"

### Research Trail

Every time you investigate a past decision:
  core genealogy show <hash>
  → Instantly reconstructs the full context
  → No manual archaeology through git log
  → Health, intent, phase, Friday state all present

Every time you need to revert an intent's work:
  core genealogy rollback-plan INT-305
  → Exact commits to revert, in safe order
  → No guessing which commits belong to the intent

### Fallback Intelligence

If a future session asks "why did we do X?":
  core genealogy why <file or command or gate>
  → Traces back through intent_commits
  → Shows the intent that motivated the change
  → Shows the gate that required it
  → Shows the Friday knowledge that informed it

---
## Gates

Phase 1 -- Schema and real-time capture:
- [ ] intent_commits table created in state.db
- [ ] fg done hook writes to intent_commits on every commit
- [ ] active intent ID, health, Friday stats captured per commit
- [ ] phase and gate hints parsed from commit messages

Phase 2 -- Backfill:
- [ ] git log parser extracts INT-NNN from all 2682 commits
- [ ] timestamps matched to session records
- [ ] health snapshots matched from checkpoint files
- [ ] intent_commits populated for all historical commits

Phase 3 -- Core commands:
- [ ] core genealogy show <commit> — full commit context
- [ ] core genealogy intent <INT-NNN> — all commits for intent
- [ ] core genealogy search <term> — full text search
- [ ] core genealogy rollback-plan <INT-NNN> — safe revert list

Phase 4 -- Friday integration:
- [ ] Friday reads intent_commits for pattern analysis
- [ ] Friday correlates commits with health events
- [ ] Friday can answer "why did we make this change?"
- [ ] Friday suggests revert risk based on commit history

Phase 5 -- Research trail:
- [ ] core genealogy diff <c1> <c2> shows full context delta
- [ ] core genealogy blame <file> shows intent-aware blame
- [ ] core genealogy why <term> traces motivation for any change
- [ ] every past decision recoverable from state.db

Final:
- [ ] no commit is context-free
- [ ] every intent is a fully traceable cluster of commits
- [ ] reverting any intent is a safe, documented operation
- [ ] Friday understands the forest's entire decision history

---
"A forest that cannot remember why it grew
cannot know where to grow next.
Every ring tells a story.
The genealogy makes the rings readable." 🌲
