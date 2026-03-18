---
id: 134
date: 2026-03-17
type: future
title: "faelight-shell Phase 10 — Shell Personality & Living Welcome"
status: in-progress
tags: [shell, personality, welcome, creative, v11]
version: 11.0.0
priority: medium
depends_on: [120]
---

## The Vision

faelight-shell should feel alive every time you open it.
Not a tool you launch. A forest you enter.

Every session should feel different — because the forest IS different.
1502 commits in, the forest knows things. It should speak them.

## Phase 10 — Shell Personality

### The Living Welcome
Every time faelight-shell opens, it reads live forest state
and generates a contextually aware greeting:
```
  🌲 The forest stirs...

  v10.9.0 — Roots and Branches
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  95 intents complete  ·  1502 commits
  100% health          ·  forecast stable

  Last session: INT-109 complete — first render
  Since last session: 3 commits, 2 events

  3 things worth knowing:
  → faelight-hooks score: 65 — needs attention
  → sandbox seccomp: incomplete (INT-125)
  → forecast trending up +3.4

  Forest quote:
  "Nothing runs without explicit human authorization."

  Type help · q to exit
```

Every element is live:
- Version and theme from VERSION file
- Commits and intents from registry and git
- Health from cache
- "Last session" from most recent event timestamp
- "Since last session" from event delta
- "Things worth knowing" from audit scores, open intents, forecast
- Forest quote rotates from a curated list

### Ctrl+C Handling
Proper SIGINT — interrupt current command, return to prompt.
Never exit on Ctrl+C. Exit only on q, exit, or Ctrl+D.

### Forest Quotes
A curated collection of forest philosophy quotes shown on startup.
Never the same quote twice in a row. Weighted toward current themes.
```rust
// Quote selection algorithm:
// 1. Read last quote from state.db
// 2. Select from pool excluding last shown
// 3. Weight toward current forest theme
//    (health < 95 → resilience quotes)
//    (new tools → growth quotes)
//    (recent intents → purpose quotes)
```

Initial quote pool (expandable):
```
"Nothing runs without explicit human authorization."
"The forest remembers. The human decides."
"Every tool is understood. Nothing is installed blindly."
"Freedom without structure is not empowerment — it is entropy."
"A forest that knows itself can survive anything."
"The roots hold. The branches grow."
"Every commit is intentional. Every tool has a purpose."
"Understanding over convenience. Always."
"The forest does not fear the storm. It knows how to grow back."
"A wise forest studies its own rings."
```

### Graceful Exit
```
q or exit:

  🌲 The forest rests.
  Session: 14 commands · 3 pipelines · 2m 34s
  See you next time.
```

### Dynamic "Today's Focus"
Generated from:
- Most recent incomplete intent
- Lowest audit score tool
- Forecast trend direction
- Recent event patterns

## Success Criteria
- ⬜ Ctrl+C interrupts command, never exits shell
- ⬜ Living welcome reads live forest state on every open
- ⬜ Forest quote rotates, stored in state.db to avoid repeats
- ⬜ "Things worth knowing" generated from live data (3 items max)
- ⬜ Session stats shown on exit
- ⬜ "Today's focus" dynamically generated
- ⬜ Graceful exit message

## The Phrase

**"A shell that knows the forest
does not need to be configured.
It simply understands."**
