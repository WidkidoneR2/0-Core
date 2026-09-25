---
id: 009
date: 2026-06-03
type: study
title: "Study: Yazelix -- multiplexer + shell + FM convergence patterns"
status: complete
tags: [study, yazelix, zellij, yazi, helix, convergence]
priority: low
---

## Why

Yazelix puts yazi as sidebar inside Helix inside Zellij. This is the same
convergence idea behind faelight-ade. Study before building.

## What to Learn

- How Yazelix handles layout management
- Zellij plugin API patterns
- Yazi event integration
- What faelight-ade could learn from this approach

## Gate

Written analysis in labs/graduated/ documenting key patterns.
Decision: adopt Yazelix approach or build forest-native equivalent.

## Study Findings (2026-06-03)

### What Yazelix Is
Zellij orchestrates everything. Yazi is the sidebar. Helix is the editor.
Three tools, one config, smart pane orchestration. Available as a Nix flake.
Latest version is v17 (luccahuguet/yazelix).

### Key Patterns to Learn
- Pane orchestrator: opens files in existing editor pane if found, new pane if not
- Alt+Shift+J/K popup system for lazygit, config UI, btm process viewer
- Tab auto-renamed to git repo/directory name
- Alt+y reveal: jump current file back into Yazi sidebar
- All Zellij/Helix keybinding conflicts remapped cleanly
- Available via flake: one-line home-manager install

### Forest Relevance
This is what faelight-ade (INT-346 arch-era) was trying to become.
The convergence of file manager + editor + multiplexer as one workspace.

### Decision
Try Yazelix via nix shell before building faelight-ade v2.
If it fits the forest workflow, adopt it. If not, the patterns inform faelight-ade.

### To Try
Add to home.nix:
  programs.yazelix.enable = true; (via flake overlay)
Or test with: nix shell github:luccahuguet/yazelix

### Verdict
Study complete. Recommend trying before building.
INT-010 (environment switching) pairs naturally with Yazelix workspaces.
