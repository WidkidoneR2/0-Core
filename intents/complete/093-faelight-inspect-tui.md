---
id: 093
date: 2026-06-26
type: future
status: complete
title: "faelight-inspect TUI: themed forest UX over the Nix option-resolution debugger"
tags: [nix, inspect, tui, ratatui, forest-ux, presentation]
priority: low
---
## Why
INT-088 built the Nix option-resolution debugger (`core nix inspect <opt> [--why]`) -- it
answers "why did this value win" with clean prose output. This is the carved-out Phase 3:
a themed TUI / richer forest-native presentation over that working engine. Pure UX -- the
data and logic already exist and are demonstrated; this makes them prettier/navigable.
## Scope (presentation only -- 088 is the engine)
- A ratatui TUI (or richer fsh builtin) over the existing inspect() output: option value,
  type, default, defined-by (winners), declared-by, the --why priority/merge breakdown.
- Navigable: search an option, jump to its definitions, maybe browse related options.
- Reuses the 088 engine entirely (core nix inspect is the data source -- do NOT duplicate
  the nixos-option/nix-eval logic; call or share it).
## Also fold in (from 088's carved notes)
- Obsolete-option-name flagging: `nixos-option -r` surfaces renames as traces -- detect and
  flag "this option was renamed to X" when inspecting a deprecated name.
## Priority
LOW. The CLI (088) is complete and useful as-is; this is polish. Not one of the three
priorities (0-Core, faelight-shell, Friday). Build when there's slack.
## The Rule
"The engine already knows why the value won. This just lets you see it beautifully." 🌲
