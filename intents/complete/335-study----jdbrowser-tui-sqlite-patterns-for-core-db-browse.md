---
id: 335
title: "Study -- JDbrowser TUI SQLite patterns for core db browse"
status: complete
date: 2026-05-25
tags: [study, sqlite, tui, ratatui, state.db, browser]
---

## What Is JDbrowser

JDbrowser (https://github.com/Jkeyuk/JDbrowser) is a terminal TUI SQLite browser
written in Rust using ratatui. v1.4. It lets you browse a SQLite database from
the terminal with a text user interface.

## Why Study It

The forest's single source of truth is state.db. Right now, inspecting it
requires raw sqlite3 commands. This is a significant daily friction point.

A `core db browse` command would let you:
- Browse all tables in state.db interactively
- Query friday_knowledge, events, friday_decisions in a TUI
- See intent state, deploy history, prediction outcomes
- Debug Friday's reasoning by browsing the underlying data

This is not about adopting JDbrowser. It is about studying its TUI patterns
and building a forest-native version that understands state.db specifically.

## What To Study

1. How JDbrowser renders tables in ratatui
2. How it handles scrolling, filtering, column selection
3. How it manages async SQLite queries without blocking the TUI
4. What keybinds it uses and whether they conflict with forest conventions

## What We Build (After Study)

`core db browse` -- a ratatui TUI that:
- Opens state.db by default
- Shows all tables in a left panel
- Shows table contents with pagination in the right panel
- Supports filtering with `/` (vi-style)
- Supports exporting a query result
- Knows about forest tables specifically (shows intent status icons, decision counts, etc.)

## Gates

✅ JDbrowser source studied -- all patterns documented above 2026-05-26
✅ TableState 2D navigation, dynamic width calc, alternating rows -- all understood 2026-05-26
✅ JDbrowser opens Connection per query (simple, sufficient) -- forest can reuse persistent connection 2026-05-26
⏸ core db browse command scaffolded -- deferred: moved to INT-342 -- approved by: christian 2026-05-26
⏸ core db browse opens state.db, shows table list -- deferred: INT-342 -- approved by: christian 2026-05-26
⏸ core db browse shows table contents with pagination -- deferred: INT-342 -- approved by: christian 2026-05-26
⏸ Filter with / -- deferred: INT-342 -- approved by: christian 2026-05-26
⏸ Forest-specific: intent table shows status icons -- deferred: INT-342 -- approved by: christian 2026-05-26
⏸ Forest-specific: events table shows domain grouping -- deferred: INT-342 -- approved by: christian 2026-05-26
⏸ Demonstrated -- deferred: INT-342 -- approved by: christian 2026-05-26

## Study Findings (2026-05-26)

### Architecture (974 lines total, 4 source files)
- app.rs (133): data model + SQLite queries
- main.rs (102): event loop + terminal setup
- ui.rs (129): top-level rendering + input dispatch
- ui/talbe_view.rs (403): core table TUI widget

### Pattern 1: Event Loop
Simple and clean:
```rust
loop {
    terminal.draw(|f| ui.ui(f, app))?;
    if let Event::Key(event) = event::read()? {
        if event.kind == event::KeyEventKind::Release { continue; }
        ui.handle_input(&event, app)?;
        if event.code == KeyCode::Esc { break; }
    }
}
```
Uses stderr for terminal (stdout stays clean). Panic hook restores terminal safely.

### Pattern 2: Data Model
Elegant simplicity:
```rust
// (column_names, rows)
type TableData = (Vec<String>, Vec<Vec<String>>);
```
All SQLite types coerced to String. Works for browsing. Forest version should
preserve type information (Integer vs Text vs Null) for better display.

### Pattern 3: Dynamic Column Width Calculation
```rust
fn map_to_cell_calc_width(widths: &mut Vec<usize>, index: usize, text: &String) -> Cell {
    let value = Text::from(text.to_string());
    if let Some(w) = widths.get_mut(index) {
        *w = (*w).max(value.width())  // grow to fit content
    } else {
        widths.push(value.width());   // first time: add
    }
    Cell::from(value)
}
```
Auto-sizes columns to widest content. Adopt this directly.

### Pattern 4: TableState 2D Navigation
```rust
TableState::select_cell(Some((row, col)))
TableState::selected_cell() -> Option<(usize, usize)>
table_state.scroll_up_by(n)
table_state.scroll_down_by(n)
table_state.scroll_left_by(1)
table_state.scroll_right_by(1)
```
Native ratatui support for 2D cell navigation. Use this.

### Pattern 5: Layout
```rust
let [l, r] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(4)])
    .margin(2)
    .areas(frame.area());
```
Left 1/5 (table list) + Right 4/5 (table data). Good ratio.

### Pattern 6: Tab Navigation via strum
```rust
#[derive(Display, EnumIter)]
enum NavigationTab { Tables, Views }
// Tabs widget iterates enum variants automatically
Tabs::new(NavigationTab::iter().map(|x| Line::from(x.to_string())))
```
Clean enum-driven tab navigation. Adopt for forest version.

### Pattern 7: Schema View
Shows raw CREATE TABLE SQL for each table. Useful for understanding state.db schema.
Forest version should also show: row count, last modified, forest domain.

### Pattern 8: Preview Panel
Bottom section (3 lines) shows full cell content. Essential for long text fields
like friday_knowledge descriptions or event payloads.

### Pattern 9: Alternating Row Colors
```rust
if index % 2 != 0 { style = style.bg(Color::Black); }
```
Readability improvement. Adopt with forest colors.

### What JDbrowser LACKS (forest version must add)
1. No filter/search (critical for large state.db tables)
2. No row count display in table list
3. Generic colors (not forest palette)
4. No table categorization (forest tables have domains)
5. Keybind conflicts: q/e for panel switch (forest uses q for other things)
6. Opens new Connection per query (forest has persistent connection)
7. No SQL query input (only SELECT *)
8. No export/copy functionality beyond single cell yank

### Dependencies to adopt
- ratatui (already in forest) -- same version (0.29.0)  
- crossterm (already in forest)
- arboard with wayland-data-control -- for multi-cell copy
- strum with EnumIter -- for tab navigation (already in some tools)

### Forest-specific additions
- Open state.db directly: `core db browse` (no file picker)
- Table icons: 🌲 intent, 🔮 friday, 📋 events, 🚀 deploy, 🔒 security
- Filter with / (live filtering of visible rows)
- Row count in left panel: friday_patterns (14), events (2847), etc.
- Jump keys: 'i' → intents, 'f' → friday_patterns, 'e' → events
- Color DNA: Abyss Black-Blue background, Neon Azure borders
