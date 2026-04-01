---
id: 160
date: 2026-03-26
type: future
title: "faelight-memory — Persistent Project Knowledge Layer"
status: in-progress
tags: [memory, knowledge, patterns, learning, v14, partner]
version: 14.0.0
priority: high
depends_on: [148, 151]
---

## The Purpose
The prediction engine knows WHEN you work.
faelight-memory knows WHAT you have learned.

Not event logs. Semantic knowledge that persists:
- "Christian prefers X pattern over Y"
- "This approach failed in INT-123 — reason documented"
- "This codebase convention: snake_case, domain-per-file"
- "When health drops after INT sprints — recovery takes 2 sessions"

## What It Stores
```
preference_patterns   — observed work preferences
failure_knowledge     — what has been tried and failed
convention_map        — codebase conventions per project
session_wisdom        — insights extracted from sessions
```

## Commands
```bash
memory show           # what does the forest know?
memory add <fact>     # explicitly teach the forest
memory forget <id>    # remove outdated knowledge
memory query <topic>  # what does the forest know about X?
memory confidence     # how confident is each memory?
```

## Integration
- Fed by prediction engine (v11)
- Fed by reaction engine (v10)
- Feeds v14 partner suggestions
- Feeds v12 strategy planning

## Gate Check
```
✅ memory tables in state.db — forest_memory table created (2026-03-31)
✅ memory show — categorized by preference/convention/wisdom/failure (2026-03-31)
✅ memory add — manual facts with category + confidence (2026-03-31)
✅ memory query — full-text search across all memories (2026-03-31)
✅ auto-extraction — extract from commit history, health runs, conventions (2026-03-31)
✅ confidence scoring — manual=80%, auto=70-95%, query shows distribution (2026-03-31)
✅ faelight-memory deployed, registered, --health verified (2026-03-31)
```

## The Phrase
**"The forest that remembers what it has learned
does not repeat its mistakes.
Memory is not storage.
It is wisdom made persistent."**
