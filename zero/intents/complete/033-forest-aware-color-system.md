---
id: 033
date: 2026-06-04
type: feature
title: "Forest-aware color system: semantic colors, context themes, git regions"
status: complete
tags: [colors, themes, semantic, fsh, ratatui, visual]
priority: medium
---

## Vision

Colors convey meaning, not just decoration.

Semantic colors:
- Intent ACTIVE → forest green
- Intent BLOCKED → red
- Intent RESEARCH → cyan
- Intent EXPERIMENT → purple
- Project stable → green
- Project sandbox → yellow

Context-based themes:
- Development: dark blue, Rust orange accents
- Research: purple, cyan
- Production: minimal, red warnings
- Recovery: amber, high contrast

Git-aware color regions:
- clean repo → green project indicator
- dirty repo → yellow
- ahead of remote → cyan
- behind remote → orange
- experimental branch → purple

## Approach

- Color tokens defined in faelight-core
- fsh reads active context, sets theme
- ratatui tools read same color tokens
- Consistent visual language across all tools

## Gate

- [x] Color tokens in faelight-core
- [x] fsh prompt reflects active context color
- [x] faelight-fm uses semantic colors for intent files
- [x] faelight-bar reflects forest health via color

## Completed
date: 2026-06-09

What was built:
- theme.rs: full neon candy palette + semantic color token constants
- prompt.rs: truecolor fc/fc_bold/fc_dim helpers, all prompt elements use semantic tokens
- faelight-fm: palette updated, intent files colored by status (green/purple/muted)
- faelight-bar: palette aligned to neon candy, health thresholds at 95/80, intent in purple
