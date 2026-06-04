---
id: 015
date: 2026-06-03
type: improvement
title: "faelight-fm v3.1: broot-style trees, branches, fuzzy navigation"
status: planned
tags: [faelight-fm, broot, trees, fuzzy, ratatui]
priority: high
---

## Why

faelight-fm v3 works but navigation is basic. broot's tree view with
fuzzy filtering is the target. The forest file manager should feel
like navigating a living forest, not a flat list.

## Approach

- Tree view with expandable directories
- Fuzzy search filters the tree in real time
- Branch indicators for git status
- Forest-aware context in preview panel
- / to enter fuzzy mode, Esc to clear

## Gate

faelight-fm shows tree view. Fuzzy filter works. Feels like broot.
