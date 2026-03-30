---
id: 166
date: 2026-03-28
type: future
title: "state.db Backup and Recovery — Protect the Forest's Memory"
status: complete
tags: [database, backup, recovery, reliability, state, sqlite]
version: 11.5.0
priority: critical
---

## The Problem
state.db is the forest's memory.
It contains everything the forest has learned:
```
events           — 10,400+ events, 28 days of history
reaction_log     — every reaction ever fired
reaction_cooldowns — current rule states
forest_predictions — prediction history
forest_goals     — accepted goals
forest_plans     — generated plans
forest_tradeoffs — analyzed tradeoffs
session_patterns — learned work rhythms
session_state    — shell state, theme, focus intent
shell_history    — every command ever typed in fsh
shell_state      — persistent shell preferences
decisions        — architectural decision records
```

1718 commits of intelligence live here.
There is NO backup story.
SQLite can corrupt on power loss, disk failure, or bad write.
If state.db corrupts: all of this is gone.

This violates the forest's own recovery principle from Core v7.

## What Can Go Wrong
```
Power loss during write    → partial transaction, corruption
Disk full                  → incomplete write, corruption
Multiple writers           → WAL conflicts (fsh + core simultaneous)
Accidental deletion        → no recovery path
Schema migration bug       → data loss
```

## The Solution

### Phase 1 — Automatic Snapshot Before Writes
Before any significant write operation, snapshot state.db:
```bash
~/0-core/runtime/state.db
~/0-core/runtime/state.db.bak        # previous session
~/0-core/runtime/state.db.weekly     # last weekly snapshot
```

Core writes snapshot before:
- doctor run (most frequent writer)
- Any domain that batch-inserts events
- Version bumps

### Phase 2 — WAL Mode Verification
SQLite WAL (Write-Ahead Logging) prevents most corruption.
Verify WAL mode is enabled and document it:
```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA wal_autocheckpoint=1000;
```

Add to core startup: verify these pragmas are set.

### Phase 3 — Integrity Check on Startup
When core starts, run SQLite integrity check:
```rust
let result = ctx.runtime.db.query_row(
    "PRAGMA integrity_check", [], |r| r.get::<_, String>(0)
)?;
if result != "ok" {
    eprintln!("⚠️  state.db integrity check failed: {}", result);
    // attempt recovery from backup
}
```

### Phase 4 — Recovery Command
```bash
core db backup          # manual snapshot to timestamped file
core db restore <file>  # restore from snapshot
core db verify          # integrity check
core db status          # show db size, table counts, last backup
core db compact         # VACUUM to reclaim space
```

### Phase 5 — Scheduled Weekly Backup
Integrate with faelight-idle or systemd timer:
```
Every Sunday at midnight: copy state.db to
~/0-core/runtime/backups/state-YYYY-MM-DD.db
Keep last 4 weekly backups.
```

### Phase 6 — History Size Limits
Prevent unbounded growth:
```sql
-- shell_history: keep last 50,000 entries
DELETE FROM shell_history WHERE id NOT IN (
    SELECT id FROM shell_history ORDER BY id DESC LIMIT 50000
);

-- events: keep last 90 days
DELETE FROM events WHERE timestamp < unixepoch() - 7776000;
```

Run cleanup weekly alongside backup.

## Gate Check
```
✅ WAL mode enabled — PRAGMA journal_mode=WAL set on connection open (2026-03-30)
✅ Integrity check available via core db verify (2026-03-30)
✅ Manual backup via core db backup — automatic snapshot deferred to Phase 5 (2026-03-30)
✅ core db backup/restore/verify/status/compact commands live (2026-03-30)
⬜ Weekly backup via systemd timer — deferred (INT-169 Niri Autostart Audit)
⬜ shell_history cap — deferred to maintenance sprint
⬜ events table cap — deferred to maintenance sprint
✅ Recovery path exists: core db restore <file> tested (2026-03-30)
✅ Backup files excluded from git — runtime/backups/ in .gitignore (2026-03-30)
```

## The Phrase
**"The forest that remembers everything
must also remember to protect its memory.
A backup is not paranoia.
It is the forest taking care of itself."**

---
*"One corrupt database can erase
1718 commits of learned behavior.
That will never happen."* 🌲
