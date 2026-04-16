---
id: 232
title: "faelight-term v12 -- Fluent, Intelligent, Friday-Aware"
status: planned
date: 2026-04-16
tags: [terminal, faelight-term, friday, rendering, conversation, fluency, v12]
---
faelight-term v11 achieved parity with foot.
v12 surpasses foot.
Not by being more complex -- by being more intelligent.
A terminal that renders beautifully, handles like foot,
shows Friday in the corner, and holds a conversation.
- Rendering quality matching or exceeding foot (ligatures, powerline, true color)
- Friday conversation pane -- persistent, toggleable, always available
- Fluent UX -- no lag, no corruption, no resize artifacts
- Forest status always visible without being intrusive
- Handle exactly like foot for muscle memory compatibility
Fix resize corruption completely -- no artifacts on any resize
Font ligature support: -> => != <= >= (coding ligatures)
Powerline glyph support for prompt segments
Bold/italic rendering correct for bat/syntax highlighting
True color 24-bit verified across all color schemes
Performance profiling -- must match foot at 165Hz
Ctrl+Shift+F toggles Friday pane (right side, 30% width)
Friday pane shows:
- Last 10 exchanges with Friday
- Current synthesis brief
- Active knowledge lessons for current domain
- Contradiction warnings if any
Friday pane is stateful -- persists across commands
Not a separate process -- reads from state.db directly
Command output syntax awareness:
- Rust errors highlighted with knowledge engine lookup
- HTTP status codes colored
- File paths made clickable
- JSON/TOML/YAML pretty-printed automatically
When a build fails: Friday pane shows relevant lesson automatically
When health drops: status strip turns amber
Split panes (Ctrl+Shift+H/V)
Session memory -- restores working directory and intent context
Long command notification (>30s → faelight-notify)
14 days as primary terminal without regression
⬜ Resize corruption eliminated -- no artifacts under any condition
⬜ Font ligatures working for common coding sequences
⬜ Powerline glyphs rendering correctly
⬜ Performance matches foot at 165Hz refresh
⬜ Friday conversation pane live -- Ctrl+Shift+F toggle
⬜ Friday pane shows synthesis brief and active lessons
⬜ Build error triggers automatic knowledge lookup in pane
⬜ Command output syntax awareness (Rust errors, paths, JSON)
⬜ Split panes working -- H/V split, navigation
⬜ Session memory -- directory and intent context restored
⬜ Long command notification working
⬜ 14 days as primary terminal without regressions
⬜ Handles identically to foot for all common operations
"Every other terminal shows you text.
faelight-term v12 shows you context, knowledge, and Friday.
The terminal that thinks -- and now speaks." 🌲
