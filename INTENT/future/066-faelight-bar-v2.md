---
intent_id: 066
title: "faelight-bar v2.0 - The Sentient Branch"
status: planned
priority: high
category: tool-improvement
target_version: 7.7.0
created: 2026-01-19
---

## Description
Transform faelight-bar from functional to legendary through architectural improvements and visual polish.

## Goals
1. **Model/View Separation** - Achieve 70-90% CPU reduction
2. **Animated Transitions** - Lock toggle, VPN glow, battery warnings
3. **Nerd Font Icons** - Replace text with glyphs
4. **Production Robustness** - Serde parsing, error handling, typed actions

## Success Criteria
- [x] PROGRESS.md tracking file created
- [ ] BarModel struct implemented
- [ ] Model update separated from draw()
- [ ] Glyph cache implemented
- [ ] CPU usage <5% average
- [ ] Lock/unlock icon animations working
- [ ] VPN glow animation working
- [ ] Battery pulse warning working
- [ ] Serde JSON parsing for workspaces
- [ ] Typed ClickAction enum
- [ ] README.md documentation
- [ ] v2.0.0 released and tagged

## Dependencies
- faelight-core Theme
- Hack Nerd Font
- serde_json crate

## Estimated Effort
9-12 sessions (1-2 hours each) over 2-3 weeks

## Notes
This is the flagship tool - taking time to do it right.
Multi-session epic, tracked in rust-tools/faelight-bar/PROGRESS.md

## Related Intents
- None (this is the capstone project)
