---
id: 135
date: 2026-03-17
type: future
title: "faelight-shell Phase 11 — Forest Personality & Adaptive Intelligence"
status: planned
tags: [shell, personality, learning, adaptive, ai, v12]
version: 12.0.0
priority: medium
depends_on: [134]
---

## The Vision

Phase 10 gives the shell a personality.
Phase 11 gives it memory, growth, and understanding.

The shell learns how you work. It understands the project.
It knows what comes next — and it tells you, without being asked.

This is the bridge between shell-as-tool and shell-as-collaborator.

## The Three Pillars

### Pillar 1 — Session Memory
The shell remembers across sessions — not just commands,
but patterns, preferences, and momentum.
```
"Welcome back. Last session you were working on INT-125 seccomp.
 You left with 3 uncommitted changes in faelight-sandbox.
 Want to continue?"
```

Tracks:
- Which intents were active last session
- Which tools were most used
- Which pipelines were run most often
- Where you left off (uncommitted changes, open files)
- Time between sessions (adjusts urgency of reminders)

### Pillar 2 — Project Understanding
The shell understands the project deeply enough to give
contextually relevant suggestions.

Not generic help. Forest-specific guidance.
```
"You've been in security domain for 3 sessions.
 faelight-sandbox score is 90 but seccomp is incomplete.
 This might be worth finishing before moving on."
```
```
"forecast is trending -0.9. Last time this happened
 you ran core doctor run and found a schema issue.
 Might be worth checking."
```

Pattern sources:
- Event log — what domains are active
- Audit scores — what needs attention
- Intent ledger — what's in progress vs planned
- Git log — what areas are changing most
- Health history — what preceded past drops

### Pillar 3 — Adaptive Messages
Messages evolve based on what the forest knows.

**Early forest (< 500 commits):**
```
"The forest is young. Every commit shapes what it becomes."
```

**Growing forest (500-1000 commits):**
```
"The roots are deepening. The tools are finding their purpose."
```

**Mature forest (1000+ commits):**
```
"The forest remembers everything. 1502 decisions, each intentional."
```

**After a major milestone:**
```
"INT-109 complete — the last sibling came home.
 The forest renders its own pixels now."
```

**After a health drop:**
```
"Health at 91%. The forest has been through worse.
 What needs attention today?"
```

**Long gap between sessions:**
```
"3 days since last session. The forest waited patiently.
 Here's what changed while you were away..."
```

## The Learning Engine

Not machine learning. Pattern matching on structured data.
The forest already has everything it needs — events, decisions,
audit scores, health history, git log.

The shell connects these dots into natural language.
```
Session pattern detected:
  Last 5 sessions: 3 involved faelight-shell
  Current phase: 9 of 26
  Suggestion: "You're on a shell streak. Phase 10 is next."
```
```
Momentum pattern detected:
  4 intents completed this week
  Forecast: +3.4
  Message: "Strong week. The forest is growing well."
```

## Message Personality Modes

The shell adjusts its personality based on context:
```
FOCUSED   — working on a specific intent
  Minimal messages, direct suggestions

EXPLORATORY — browsing, no active intent
  More curious, asks questions, surfaces interesting data

RECOVERY  — health < 95%
  Calm, methodical, focuses on what needs fixing

MILESTONE — just completed something significant
  Celebratory but measured, points to what's next

IDLE      — long gap since last session
  Gentle reorientation, summarizes what changed
```

## Relationship to Core v9

Core v9 gives the forest goals and a planning engine.
faelight-shell Phase 11 gives those goals a voice.

When Core v9 generates a goal, faelight-shell can say:
```
"The forest identified a new goal:
 Reduce dependency risk — 6 high-coupling deps found.
 Want to review it? → core goals list"
```

The shell becomes the human interface to the forest's own intentions.

## Success Criteria
- ✅ Session memory — shell knows where you left off (2026-03-20)
- ✅ Project understanding — active intents shown on welcome (2026-03-20)
- ⬜ Adaptive messages — different based on commit count, health, milestones
- ⬜ Momentum detection — recognizes streaks and patterns
- ⬜ Personality modes — focused/exploratory/recovery/milestone/idle
- ⬜ Core v9 integration — surfaces forest goals in shell messages

## The Long-Term Vision

Phase 11 is not the end. It is the beginning of the shell
understanding the forest well enough to be a genuine collaborator.

Not AI bolted on. Forest intelligence expressed through language.
The data was always there. The shell just learned to read it.

Eventually:
```
"find biggest files"
→ files | sort size desc | first 10

"why is my computer slow"
→ ps | sort cpu desc | first 5 (checking...)
   memory at 87% — faelight-browser using 2.1GB
   suggestion: close unused browser tabs

"what should I work on today"
→ Based on forecast -0.9, open INT-125 seccomp,
   and 3 days since last sandbox session:
   Suggest: complete INT-125 seccomp (1 session estimate)
```

The forest that understands you
is the forest that grows with you.

## The Phrase

**"A shell with memory knows where you've been.
A shell with understanding knows where you're going.
A shell with personality makes the journey worth taking."**

---
*"Phase 11 is not automation. It is companionship."* 🌲
