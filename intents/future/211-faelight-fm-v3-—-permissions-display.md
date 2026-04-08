---
id: 211
date: 2026-04-08
type: planned
title: "faelight-fm v3 — Permissions Display and Forest Integration"
status: planned
tags: [faelight-fm, file-manager, permissions, v3, forest-integration]
---
DEC-003 (2026): "migrate faelight-fm to v3 with permissions display"
Decision has been pending since early in the forest's development.
faelight-fm exists and is deployed but has not been upgraded since v2.
- Permissions display (rwxr-xr-x style + symbolic)
- Forest-aware context (shows active intent, health inline)
- state.db integration — file operations logged as forest events
- Integration with faelight-contextd signals
- Better sorting: by size, date, name, type
- Preview pane improvements
⬜ Permissions display working (full rwx + octal)
⬜ Forest context header (health, active intent)
⬜ File operations logged to forest_events in state.db
⬜ Sorting by size/date/name/type
⬜ deploy faelight-fm and d passes 100%
⬜ DEC-003 resolved as success
