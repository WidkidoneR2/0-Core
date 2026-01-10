---
id: 026
date: 2026-01-10
type: future
title: "v6.4.0 - Architectural Milestone"
status: planned
tags: [roadmap, architecture, v6.4.0]
---
## The Vision
Major architectural evolution: unified binary, typed configs, intelligent health.

## v6.4.0 Scope
| Intent | Component | Priority |
|--------|-----------|----------|
| 021 | faelight unified binary | P0 - Critical |
| 022 | faelight-config typed layer | P0 - Critical |
| 023 | Health engine upgrade | P1 - High |
| 024 | faelight-git governance | P1 - High |

## v6.5.0 Scope
| Intent | Component | Priority |
|--------|-----------|----------|
| 025 | core-diff policy mode | P2 - Medium |
| --- | UI polish (fonts, sizing) | P2 - Medium |
| --- | faelight-notify improvements | P2 - Medium |
| --- | faelight-launcher fuzzy search | P2 - Medium |

## Philosophy Preserved
- Manual control over automation
- Intent over convention
- Understanding over convenience
- Recovery over perfection

## Key Rule
| Failure is... | Use |
|---------------|-----|
| Annoying | Shell |
| Dangerous | Rust |

Shell keeps: aliases, REPL, exploration, one-liners, temporary hacks.

## Success Criteria
- [ ] Single `faelight` binary as entry point
- [ ] All config in typed TOML
- [ ] Health as dependency graph
- [ ] Git as governance layer
- [ ] Shell reduced to thin aliases

---
_The forest evolves._ 🌲
