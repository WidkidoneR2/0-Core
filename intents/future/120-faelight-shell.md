---
id: 120
date: 2026-03-12
type: future
title: "faelight-shell — forest-native interactive environment"
status: planned
tags: [shell, repl, forest, language, rust, v11, architecture]
version: 11.0.0
priority: medium
---

## Vision

A forest-native interactive environment built in Rust.
Not a POSIX shell. Not Nu. Not bash.
A shell that speaks Faelight Forest natively.

## Phase 1 — Forest REPL (achievable in weeks)

An interactive query environment where forest concepts
are first-class commands — no `core` prefix needed.
```
faelight-shell
🌲 forest> events today
🌲 forest> decisions open
🌲 forest> health
🌲 forest> story
🌲 forest> advise "upgrade compositor"
🌲 forest> intents active
```

All commands query state.db, the intent ledger, and the
core engine directly. Structured output by default.

## Phase 2 — Forest Scripting Language

A simple declarative language for forest automation:
```
intent 109 {
  on complete → emit "compositor.ready"
  on fail → checkpoint "pre-109-recovery"
}
```

## Phase 3 — Full Shell (long-term)

A production shell that can replace zsh for forest-native
workflows while maintaining external command execution.

## Philosophy

This is not about replacing zsh for compatibility reasons.
It's about having a shell that understands what the forest IS.
Every command is forest-aware. Every output is structured.
Every action is logged to the ledger.

## Success Criteria

- [ ] Phase 1: REPL with 10+ forest-native commands
- [ ] Phase 1: Reads state.db natively
- [ ] Phase 1: Structured table output
- [ ] Phase 2: Basic scripting language
- [ ] Phase 3: Full shell (long-term)

---
*"A forest deserves a shell that knows it is a forest."* 🌲
