---
id: 013
date: 2026-06-03
type: feature
title: "Forest-Aware Launcher: forest start <context> -- environment as first class"
status: cancelled
tags: [launcher, environment, nix, friday, context-switching]
priority: low
---

## Vision

forest start coding    → activates dev shell, opens project, Friday briefs you
forest start research  → minimal env, browser, notes
forest start secure    → VPN enforced, hardened shell, Mullvad active

One command assembles the entire mental context, not just opens an app.
Friday knows which project, which intent, what comes next.

## Approach

- Named devShells in flake.nix per context
- niri workspace switching per context
- state.db stores last session per context
- Friday briefs on re-entry

## Gate

forest start coding activates rust devShell, switches workspace, Friday briefs.

## Gate Check
🚫 013 -- cancelled: faelight-launcher was retired long ago and its stale registry entry is the missing tool the doctor reports. A launcher intent for a launcher that does not exist. -- approved by: christian 2026-08-27
