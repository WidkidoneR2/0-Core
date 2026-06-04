---
id: 018
date: 2026-06-03
type: housekeeping
title: "Retire faelight-browser: brave is the forest browser"
status: planned
tags: [faelight-browser, retire, brave, nixos]
priority: low
---

## Why

faelight-browser was a wrapper/TUI for browser launching on Arch.
On NixOS, brave is declared in home.packages and works perfectly.
The wrapper adds no value and the binary doesn't exist on NixOS.

## Actions

- Remove faelight-browser from tool registry
- Update any remaining references
- Mod+Alt+b keybind already remapped to brave
- Archive the source in rust-tools/retired/

## Gate

No references to faelight-browser in active config.
brave launches cleanly from Mod+Alt+b.
