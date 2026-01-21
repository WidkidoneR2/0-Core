---
id: 029
date: 2026-01-10
type: future
title: "teach-advanced - Interactive Learning"
status: cancelled
cancelled_date: 2025-01-21
tags: [rust, learning, documentation]
---

## The Problem
0-Core has grown complex. New features need discoverable documentation.

## The Solution
Interactive learning system that teaches the user about their own system.

## Implementation
`teach-advanced` tool:
- `teach-advanced tour` - Guided tour of 0-Core
- `teach-advanced quiz` - Test your knowledge
- `teach-advanced topic <name>` - Deep dive on topic
- `teach-advanced daily` - Daily tip

## Topics
- Intent system
- Profile management
- Health engine
- Git governance
- Security layers
- Rust tools
- Shell aliases

## Technical
- TUI with chapters and progress tracking
- Store progress in `~/.local/state/faelight/learning.json`
- Colorful, interactive terminal UI

## Success Criteria
- [ ] Guided tour works
- [ ] Progress is saved
- [ ] At least 10 topics covered
- [ ] Quiz system functional

---
_The forest teaches its ways._ 🌲
