---
id: 107
date: 2026-03-02
type: future
title: "faelight-search — Unified Rust Search"
status: complete
tags: [search, rust, files, intents, commits, events, rusty, glow]
version: TBD
priority: high
---

## Vision

One command that searches everything the forest knows.
Files, intents, commits, events, aliases, tools — all indexed,
all searchable, all in Rust.
```
fs "checkpoint"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 Intents (2)
  INT-098  Core v4 — checkpoint foundation
  INT-100  core pulse — checkpoint timeline

📁 Files (8)
  engine/src/domains/checkpoint/mod.rs
  runtime/checkpoints/2026-03-02-pre-v4.toml

🔀 Commits (5)
  8c71af5  feat(core): checkpoint system Phase 1
  b702c5a  docs: intent 099

⚡ Events (3)
  12:32:48  checkpoint  auto-intent-098-start
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Why This Is Special

The forest has unique data no search tool knows about:
- The event ledger
- The intent system
- The git history with risk scores
- The alias registry

faelight-search is the only tool that can search all of them together.

## Approach

- ripgrep-style file search as foundation (or use ripgrep as lib)
- Intent index: search titles, tags, content
- Git log search: commit messages, file changes
- Event search: query state.db by domain/content
- Alias search: find alias by description
- Unified ranking: most relevant results first

## Success Criteria

- [x] File search (ripgrep speed)
- [x] Intent search (title + content)
- [x] Commit search
- [x] Event search
- [x] Alias search
- [x] Unified ranked output
- [x] `fs` alias for instant access

---

*"The forest remembers everything. Now you can find it."* 🌲
