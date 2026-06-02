---
id: 123
date: 2026-03-12
type: future
title: "faelight-audit — Tool Intelligence Layer (core audit domain)"
status: complete
tags: [audit, tools, intelligence, core, registry, state-db, rust, v10.8]
version: 10.8.0
priority: high
---

## Vision

The forest notices when parts of itself are being neglected.

Not just "is this tool updated" — but "does this tool still
deserve to exist in its current form?"

52 tools. Each one should be understood, maintained, and active.
`core audit` makes the forest self-auditing.

## Architecture Decision

### Why `core audit` not standalone binary
Every major capability lives in `core` — advise, simulate, decide,
security. The pattern is established. `core audit` gets capability-gating,
event logging, and dispatcher integration for free. Grows with the forest.

### Data Architecture
```
tools.toml   → declarative truth — what tools SHOULD be
               + new field: expected_usage (high/medium/low/rare)
state.db     → runtime truth — what tools ARE doing
               + new table: audit_scores (computed, timestamped)
core advise  → reads audit_scores, surfaces stale tools
```

The separation matters:
- New tool → declare in tools.toml → audit picks it up automatically
- Tool goes stale → state.db shows it → tools.toml stays clean
- Tool archived → remove from tools.toml → state.db history remains

## The Scoring Model

Each tool scored 0-100 based on four factors:

| Factor | Weight | Source |
|--------|--------|--------|
| Usage frequency | 25% | state.db events (last 30 days) |
| Recency | 25% | git log — last commit touching tool |
| Documentation | 25% | README exists, Cargo.toml description |
| Version currency | 25% | version bumped in last 90 days |

### expected_usage context
Without context, rare tools always look stale.
`expected_usage` in tools.toml calibrates the scoring:
```toml
[[tool]]
name = "faelight-fetch"
expected_usage = "high"    # runs every terminal open

[[tool]]
name = "faelight-snapshot"
expected_usage = "rare"    # runs on demand only
```

A "rare" tool with 0 events in 30 days is healthy.
A "high" tool with 0 events in 30 days is a problem.

## Commands
```bash
core audit scan              # score all tools — full report
core audit show <tool>       # deep audit of one tool
core audit stale             # tools below health threshold
core audit coverage          # tools missing docs/description
```

## Output Vision
```
╭─ 🔍 Tool Intelligence Report ─────────────────╮
│  52 tools analyzed  │  3 need attention        │
╰────────────────────────────────────────────────╯
╭─ ⚠️  Needs Attention ──────────────────────────
│  faelight-browser   47d stale   low usage   68/100
│  faelight-term      no README   high usage  71/100
│  dotctl             0 events    stale       55/100
╰────────────────────────────────────────────────
╭─ 🟢 Healthy (49 tools) ────────────────────────
│  All other tools above threshold
╰────────────────────────────────────────────────
```

## core advise Integration

When tools score below threshold, surfaces in advisory:
```
→ 3 tools haven't been touched in 90+ days
  Consider running: core audit stale
```

## state.db Schema
```sql
CREATE TABLE audit_scores (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_name   TEXT NOT NULL,
    score       INTEGER NOT NULL,
    usage_score INTEGER,
    recency_score INTEGER,
    doc_score   INTEGER,
    version_score INTEGER,
    last_event_days INTEGER,
    last_commit_days INTEGER,
    timestamp   INTEGER NOT NULL
);
```

## Build Order
```
Phase 1 — core audit scan + show
  - Add audit domain to engine
  - Score all tools from registry
  - Read git log for recency
  - Read state.db for usage
  - Output cockpit-style report

Phase 2 — state.db integration
  - Write scores to audit_scores table
  - core audit stale reads from table
  - core advise reads stale tools

Phase 3 — registry enhancement
  - Add expected_usage to tools.toml
  - Calibrate scores by expected usage
  - doctor gains optional audit check
```

## Success Criteria

- [ ] `core audit scan` scores all 52 tools
- [ ] expected_usage field in tools.toml
- [ ] audit_scores table in state.db
- [ ] Stale tools surface in `core advise`
- [ ] `core audit stale` shows attention list
- [ ] doctor optionally runs audit check

## Philosophy Alignment

The forest notices. You decide.
`core audit` never archives, never modifies, never acts.
It observes and reports. The human chooses what to do.

---
*"A forest that knows its own health knows when a tree needs care."* 🌲
