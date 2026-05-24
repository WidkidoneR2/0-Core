---
id: 335
title: "Study -- JDbrowser TUI SQLite patterns for core db browse"
status: planned
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

⬜ JDbrowser source studied -- table rendering, scroll, filter patterns documented
⬜ Ratatui table widget patterns understood for state.db column types
⬜ Async SQLite query pattern identified (non-blocking TUI)
⬜ core db browse command scaffolded
⬜ core db browse opens state.db, shows table list
⬜ core db browse shows table contents with pagination
⬜ Filter with / (vi-style search within table)
⬜ Forest-specific: intent table shows status icons
⬜ Forest-specific: events table shows domain grouping
⬜ Demonstrated: full state.db inspection session using core db browse
