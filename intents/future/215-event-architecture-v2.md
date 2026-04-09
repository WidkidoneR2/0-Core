---
id: 215
date: 2026-04-09
type: planned
title: "Event Architecture v2 — Append-Only Log and Signal Ontology"
status: planned
tags: [architecture, events, signals, append-only, ontology, friday, audit]
---
Current event system: mutable rows, free-form payloads, no schema enforcement.
Two problems this creates:
1. Ordering ambiguity
   engine_signals has no guaranteed ordering beyond created_at timestamp.
   Two engines emitting simultaneously have ambiguous ordering.
   Friday cannot reconstruct causality — only correlation.
2. Semantic drift
   payload is a free-form string.
   "health:100" and "{"health":100}" are both valid payloads for health signals.
   Over time: divergent interpretations of the same signal class.
   Result: "false wisdom" — patterns that look real but aren't.
Replace mutable event rows with append-only architecture:
- Events are NEVER updated or deleted
- New state is a new row, not an update to existing row
- Sequence number (monotonic, gapless) on every event
- Every event has: sequence, timestamp, source, kind, payload, intent_id (optional)
This enables:
- Deterministic replay: reconstruct any past state from sequence 0
- Causality chains: event A caused event B (via intent_id linking)
- Audit integrity: nothing in the past can be changed
Schema:
  CREATE TABLE forest_events_v2 (
      seq         INTEGER PRIMARY KEY AUTOINCREMENT,  -- monotonic, never gaps
      timestamp   INTEGER NOT NULL,
      source      TEXT NOT NULL,    -- which tool/engine emitted this
      kind        TEXT NOT NULL,    -- signal class (typed)
      payload     TEXT NOT NULL,    -- JSON, validated against schema
      intent_id   TEXT,             -- which intent was active (if any)
      session_id  TEXT,             -- which session (for replay isolation)
      schema_ver  INTEGER NOT NULL DEFAULT 1
  );
Each signal class has a defined schema. Validated at emission.
Rejected if malformed. No silent failures.
Signal classes:
  health          — {"health": u32, "checks_passed": u32, "checks_failed": u32}
  git_commit      — {"hash": str, "message": str, "files_changed": u32}
  intent_start    — {"id": str, "title": str}
  intent_complete — {"id": str, "title": str, "duration_secs": u64}
  prediction      — {"suggestion": str, "confidence": f64, "source": str}
  alignment       — {"score": f64, "drift": f64, "values_checked": u32}
  deploy          — {"tool": str, "version": str, "success": bool}
  watchdog_alert  — {"health": u32, "previous": u32, "threshold": u32}
Every new signal class requires:
  1. Schema definition committed to intents/
  2. Validation function in signal_registry
  3. Documented in core-commands.md
When Friday asks "why did this happen?":
Current: timestamp correlation only
v2: explicit causality linking
  forest_events_v2.caused_by INTEGER REFERENCES forest_events_v2(seq)
Example chain:
  seq=1001: git_commit (hash=abc123)
  seq=1002: deploy (tool=core, caused_by=1001)
  seq=1003: health (health=100, caused_by=1002)
Friday can now answer: "Core was deployed because of commit abc123,
and health went to 100% as a result."
  core events replay --from-seq 1000 --to-seq 2000
Reconstructs system state at any point in time.
Used for: debugging, Friday training, audit.
Existing events table preserved as events_legacy.
New forest_events_v2 runs in parallel.
Tools emit to both during transition.
After 30 days: events_legacy archived, v2 becomes canonical.
⬜ forest_events_v2 table created with seq + causality fields
⬜ Signal ontology defined — 8 core signal classes with JSON schemas
⬜ Validation function in signal_registry — rejects malformed signals
⬜ core events emit <kind> <payload> — validated emission
⬜ core events replay — reconstruct state from sequence range
⬜ causality chain linking (caused_by field) on key events
⬜ Migration path from events_legacy to v2
⬜ doctor emits to v2 with intent_id context
⬜ faelight-git emits to v2 with causality links
⬜ Friday can query causality chains (for "why" answers)
- Append-only: no UPDATE or DELETE on forest_events_v2
- Monotonic sequence: no gaps, no reuse
- Schema validation: every payload validated before INSERT
- Causality: seq + caused_by enables full audit trail
- Replay: deterministic from seq=0
"A system that can only tell you what happened
cannot tell you why it happened.
Causality is not a feature.
It is the difference between memory and understanding.
The event log is not a record of the past.
It is the proof of reasoning." 🌲
