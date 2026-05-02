---
id: 263
date: 2026-05-01
type: feature
title: \"db -- native state.db query builtin for fsh\"
status: planned
tags: [feature, rust, faelight]
version: TBD
---

## Vision
fsh currently has no native way to query state.db. Every time the forest
needs data from its own database -- events, shell history, friday_knowledge,
predictions, patterns -- it must spawn a `sqlite3` subprocess or rely on
`core` commands that are slow and output-heavy.
This intent adds a `db` builtin to fsh that speaks directly to state.db
through rusqlite (already a dependency). Two modes:
**Raw SQL mode:**
db "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit'"
**Forest vocabulary mode (no SQL required):**
db events --domain git --action commit --today
db history --limit 10
db friday --facts --limit 5
db predictions --pending
The forest should be able to ask itself questions in plain language.
`db` is the voice of that self-inquiry.
Three signals from today's session:
1. **INT-258 health TUI** needed commits-today and active-intents counts.
   We had to write custom rusqlite queries inside health_tui.rs because
   there was no general-purpose tool. Every future TUI will face the same
   problem.
2. **The vocabulary thesis (INT-261)** demands it. If the forest speaks
   human first, it should be able to say `db events --domain git --today`
   instead of spawning sqlite3 with raw SQL.
3. **Friday context (INT-234)** reads from state.db constantly. Today
   that means `core friday status` -- a heavy process just to read a few
   rows. A lightweight `db` builtin would let Friday surface data in
   milliseconds from any fsh context.
**Mode 1 -- Raw SQL passthrough:**
db "SELECT COUNT(*) FROM shell_history WHERE exit_code=0"
Runs the SQL directly against state.db. Returns tabular output with
column headers. Useful for power users and TUI internals.
**Mode 2 -- Forest vocabulary:**
db events                          -- show recent events (last 20)
db events --domain git             -- filter by domain
db events --domain git --today     -- filter by domain + today only
db events --action commit          -- filter by action
db history                         -- recent shell history
db history --limit 50              -- last 50 commands
db history --failed                -- only failed commands
db friday                          -- friday_knowledge summary
db friday --facts                  -- list knowledge facts
db predictions                     -- pending predictions
db patterns                        -- shell patterns
- Table format by default (aligned columns, header row)
- `--json` flag for machine-readable output (pipes into `query`)
- `--count` flag for just the count
- Colors: header in forest green, data in default, warnings in amber
`rust-tools/faelight-shell/src/commands/mod.rs` -- new match arm `"db" =>`
alongside `delete`, `find`, `fsearch`. Uses `db: &ForestDb` already
passed to every builtin -- no new connection needed.
The builtin knows the state.db schema:
- `events` (id, domain, action, payload, timestamp)
- `shell_history` (id, command, exit_code, cwd, duration_ms, timestamp)
- `friday_knowledge` (id, domain, fact, confidence, created_at)
- `forest_predictions` (id, pattern, prediction, confidence, created_at)
- `session_patterns` (id, pattern, weight, last_seen)
Forest vocabulary flags map to WHERE clauses on these tables.
Raw SQL mode bypasses this entirely.
- Read-only by default. No INSERT/UPDATE/DELETE unless `--write` flag.
- `--write` requires explicit confirmation prompt.
- Protects against dropping tables or schema changes.
- rusqlite (already in fsh Cargo.toml)
- ForestDb connection already passed to every builtin
- state.db at `~/0-core/runtime/state.db`
- [ ] `db "SELECT COUNT(*) FROM events"` returns correct count
- [ ] `db events` shows last 20 events in table format
- [ ] `db events --domain git` filters correctly
- [ ] `db events --domain git --today` filters to today
- [ ] `db history` shows last 20 shell commands
- [ ] `db history --failed` shows only exit_code != 0 commands
- [ ] `db friday` shows friday_knowledge summary
- [ ] `db predictions` shows pending predictions
- [ ] `--count` flag returns just the number
- [ ] `--json` flag outputs JSON for piping
- [ ] Read-only by default, `--write` required for mutations
- [ ] Output table aligned with header row
- [ ] `db` added to known commands list (command-not-found suggester)
- [ ] `rm` and UNIX commands continue to work unchanged
- [ ] No regression in existing builtins
- Raw SQL passthrough mode
- Forest vocabulary for: events, history, friday, predictions, patterns
- Table output with `--count` and `--json` flags
- Read-only safety
- Schema migrations (state.db schema is managed by core engine)
- Cross-database queries
- Write operations beyond `--write` flag
- Graphical output (charts, graphs -- separate intent if needed)
- `db watch` (live-updating stream of events) -- future intent
- `db export` (dump tables to CSV/JSON files) -- future intent
- Integration with `find` and `fsearch` pipelines -- after basic db works
**Mitigation:** Parse the SQL before execution. Reject any statement
containing DROP, DELETE, UPDATE, INSERT unless `--write` is passed.
Even with `--write`, prompt for confirmation.
**Mitigation:** Forest vocabulary maps to named tables and columns.
If a query fails (table doesn't exist, column renamed), return a clear
error explaining which table/column was expected and what to do.
**Mitigation:** Default LIMIT 20 on all vocabulary queries. `--limit N`
for larger results. Raw SQL has no limit (user's responsibility).
■ Not started
---
*"The forest should be able to ask itself questions.
`db events --domain git --today` is a human sentence.
`SELECT * FROM events WHERE domain='git' AND timestamp >= strftime...`
is a 1970s answer to a 2026 question.
The forest speaks human first." 🌲*
