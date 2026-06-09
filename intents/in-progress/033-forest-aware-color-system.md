---
id: 033
date: 2026-06-04
type: feature
title: "Forest-aware color system: semantic colors, context themes, git regions"
status: in-progress
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

- [ ] Color tokens in faelight-core
- [ ] fsh prompt reflects active context color
- [ ] faelight-fm uses semantic colors for intent files
- [ ] faelight-bar reflects forest health via color
