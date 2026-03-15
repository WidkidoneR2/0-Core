---
id: 129
date: 2026-03-15
type: future
title: "Event Log Directory — File-Based JSONL Alongside SQLite"
status: planned
tags: [events, logging, jsonl, replay, resilience, structure, v10.9]
version: 10.9.0
priority: medium
depends_on: [122]
---

## Vision

The forest already logs everything to state.db.
But SQLite requires tooling to read.

A parallel file-based event log makes the forest's history:
- Human readable without any tools
- Portable — readable without the forest installed
- Replayable — reconstruct state at any point in time
- Archivable — compress old days, keep forever

## Structure
```
runtime/
├── state.db          # existing — queryable, fast
└── events/           # NEW — file-based, portable
    ├── 2026-03-14.jsonl
    ├── 2026-03-13.jsonl
    └── archive/
        └── 2026-02.jsonl.gz
```

## Format — JSONL (JSON Lines)

One event per line. Append-only. Human readable.
```jsonl
{"ts":1773366322,"domain":"git","action":"push","actor":"faelight-git","result":"ok"}
{"ts":1773366318,"domain":"git","action":"commit","actor":"faelight-git","result":"ok"}
{"ts":1773365566,"domain":"doctor","action":"run","actor":"core","result":"ok","detail":{"health":100}}
```

## Two Stores, One Source of Truth

Events are written to BOTH simultaneously:
- state.db — for fast queries, pipelines, Core v6 patterns
- runtime/events/YYYY-MM-DD.jsonl — for portability, replay, archive

They are always in sync. state.db is the query layer.
JSONL is the archive layer.

## New Commands
```bash
core events replay --date 2026-03-14    # replay a day's events
core events replay --from v10.7.0       # replay since a version
core events archive                      # compress old logs
core events export --format csv         # export for analysis
```

## Connection to Core v7

`core bootstrap plan` reads the event log to understand:
- what the system was doing before failure
- which tools were active
- what the last known good state was

The event log becomes the memory that enables reconstruction.

## Connection to Snapshot Narrative

`core snapshot narrative` reads both state.db AND the event log
to build the most complete picture of system state and history.

## Doctor Integration
```
╭─ 📋 Forest
│  ✅ Event Log    Today: 47 events across 6 domains
│  ✅ Event Log    Archive: 23 days, 4,847 total events
```

## Success Criteria

- [ ] runtime/events/ directory created
- [ ] Every event written to both state.db and JSONL
- [ ] Daily rotation — new file per day
- [ ] core events replay command
- [ ] core events archive command
- [ ] doctor monitors event log health
- [ ] core bootstrap plan reads event log

---
*"The forest that remembers everything can rebuild from anything."* 🌲
