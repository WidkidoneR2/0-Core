---
id: 342
title: "core db browse -- Forest-Native state.db TUI Browser"
status: planned
date: 2026-05-26
tags: [core, db, tui, ratatui, sqlite, state.db, browse, friday]
depends_on: [335]
---

## Why This Intent Exists

state.db is the forest's single source of truth.
265 intents, 14 Friday patterns, 369 facts, 2800+ events, deploy history.
Currently: inspecting it requires raw sqlite3 commands.
This is daily friction. Every debugging session involves `sqlite3 ~/0-core/runtime/state.db`.

`core db browse` makes the forest's memory visible and navigable.

## Architecture (from INT-335 study of JDbrowser)

### Stack
- ratatui 0.29 (already in forest) -- TUI framework
- crossterm (already in forest) -- terminal backend
- rusqlite (already in forest) -- SQLite connection
- strum with EnumIter -- tab navigation
- arboard with wayland-data-control -- clipboard

### Layout
┌─ Tables ──────┬─ friday_patterns ─────────────────────────────────┐
│ 🌲 intents    │  id  trigger           action         confidence   │
│ 🔮 friday_*   │  1   frequent command  deploy core    0.99         │
│ 📋 events     │  2   deploy completes  fg commit      0.99         │
│ 🚀 deploys    │  3   frequent command  c              0.99         │
│ 🔒 security   │────────────────────────────────────────────────────│
│ 📊 history    │  Preview: deploy core                              │
└───────────────┴────────────────────────────────────────────────────┘
[?] help  [/] filter  [Tab] switch panel  [y] yank  [Esc] exit

### Data Model (from JDbrowser)
```rust
type TableData = (Vec<String>, Vec<Vec<String>>);  // (headers, rows)
```

### Key Patterns Adopted
1. Dynamic column width calculation (map_to_cell_calc_width pattern)
2. TableState 2D navigation (select_cell, scroll_up/down/left/right)
3. strum EnumIter for tab switching
4. Alternating row colors for readability
5. Preview panel (3 lines) for long cell content

## Forest-Specific Features (Beyond JDbrowser)

### Table Icons and Categorization
🌲 intent tables:    intents, intent_commits, intent_tags
🔮 friday tables:    friday_patterns, friday_knowledge, friday_decisions
friday_session_context, synthesis_snapshots
📋 event tables:     events, audit_scores
🚀 deploy tables:    deploy_patterns, deploy_history
🔒 security tables:  command_failures, integrity_log
📊 history tables:   shell_history, shell_snapshots, checkpoints
⚙️  system tables:    shell_state, shell_aliases

### Jump Keys (no table picker needed for frequent tables)
- `i` → jump to intents table
- `f` → jump to friday_patterns
- `e` → jump to events
- `k` → jump to friday_knowledge
- `h` → jump to shell_history

### Filter with `/`
- Type `/` to enter filter mode
- Live filtering of visible rows as you type
- Esc to clear filter
- Shows: "5/251 rows (filter: active)"

### Row Count in Left Panel
🌲 intents (265)
🔮 friday_patterns (14)
📋 events (2847)

### Forest Color Palette
- Background: #0a0f14 (Abyss Black-Blue)
- Border: #00bfff (Neon Azure)
- Text: #a9dfff (Soft Ice Blue)
- Highlight: #00ff88 (Sharp Forest Green)
- Warning: #ffd43b (Soft Amber)
- Header: #2affd5 (Aqua Mint), bold

### SQL Query Mode (JDbrowser lacks this)
- Press `:` to enter SQL query mode
- Run arbitrary SELECT queries
- Results displayed in table view
- Query history (up/down arrow)

### Export
- `y` yanks selected cell (from JDbrowser)
- `Y` yanks entire row as CSV
- `x` exports query result to /tmp/forest-export.csv

## Implementation Plan

### Phase 1: Core TUI (2 sessions)
- New file: engine/src/domains/db/mod.rs
- App struct with rusqlite Connection
- Left panel: table list with icons and row counts
- Right panel: table data with dynamic width
- Basic navigation: hjkl, u/d page scroll
- Gate: `core db browse` opens state.db, shows all tables

### Phase 2: Forest-Specific (1 session)
- Table categorization and icons
- Jump keys (i/f/e/k/h)
- Color DNA applied
- Row count in left panel
- Alternating row colors with forest palette
- Gate: intent table shows 🌲 icon, shows status column with color

### Phase 3: Filter + Query (1 session)
- `/` filter mode with live row filtering
- `:` SQL query mode
- Query history
- Gate: filter "active" in intents table shows only in-progress intents

### Phase 4: Export + Polish (1 session)
- Cell yank (y), row yank (Y), export (x)
- Schema view for each table (CREATE TABLE SQL)
- Preview panel for long text (events payload, knowledge text)
- Gate: yank event payload, export 10 rows to CSV

### Phase 5: Integration (1 session)
- Wire into `core db browse` command
- Wire into `core db browse <table>` to jump directly
- Wire into `db` alias
- Gate: `core db browse friday_patterns` opens directly to friday_patterns

## Gates
- [ ] Phase 1: core db browse opens state.db, shows all tables with row counts
- [ ] Phase 2: table icons, jump keys, forest color palette applied
- [ ] Phase 3: / filter live, : SQL query mode working
- [ ] Phase 4: y/Y yank, x export, schema view, preview panel
- [ ] Phase 5: core db browse <table> direct jump, db alias
- [ ] Final: full state.db inspection session -- no sqlite3 commands needed

## Note
JDbrowser is GPL-3.0. Forest version is a clean-room implementation
using the same underlying ratatui primitives. No JDbrowser code is copied.
The patterns studied are ratatui API patterns, not JDbrowser's implementation.

---
"The forest's memory should be readable.
Not just queryable -- readable.
Every decision, every pattern, every fact --
visible in the terminal, navigable with intention." 🌲
