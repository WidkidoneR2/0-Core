---
id: 159
date: 2026-03-26
type: future
title: "faelight-context — Deep Codebase Understanding Engine"
status: planned
tags: [context, codebase, understanding, v14, partner, ai]
version: 14.0.0
priority: high
depends_on: [151]
---

## The Purpose
The partner cannot propose intelligent next steps without
understanding what it is working with.

faelight-context builds a living map of any codebase:
- What exists (files, modules, functions, types)
- What patterns repeat (architectural conventions)
- What dependencies exist (coupling map)
- What decisions were made (links to intent ledger)
- What changed recently (churn and velocity)

## Commands
```bash
context scan              # scan current directory
context map               # show architectural map
context patterns          # recurring patterns detected
context decisions         # link code to intent decisions
context summary           # one-paragraph codebase summary
context diff <commit>     # what changed and why
```

## Integration
- Feeds core predict coupling (already exists)
- Feeds v14 partner suggestions
- Reads intent ledger for decision context
- Writes to state.db context_snapshots table

## Gate Check
```
⬜ context scan — index current codebase
⬜ context map — architectural visualization
⬜ context patterns — convention detection
⬜ context decisions — code ↔ intent links
⬜ context summary — natural language overview
⬜ integrated with core predict
```

## The Phrase
**"The partner that knows the codebase
can propose what the human would build next.
Context is the foundation of collaboration."**
