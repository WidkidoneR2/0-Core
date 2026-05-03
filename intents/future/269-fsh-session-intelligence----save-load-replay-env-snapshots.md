---
id: 269
date: 2026-05-03
type: feature
title: "fsh session intelligence -- save load replay env snapshots"
status: in-progress
tags: [shell, fsh, sessions, history, intelligence]
version: TBD
---
INT-245 Pillar 4. The shell that remembers across sessions.
    session save "building-term-v2"
    session load "building-term-v2"
    history-replay 10
    env-save debug-env
    env-load debug-env
- [ ] session save / load / list work with directory + history context
- [ ] history-replay <n> re-runs last n commands with confirmation
- [ ] env-save / env-load / env-diff work correctly
- [ ] Sessions persist across reboots
- [ ] Demonstrated: session restored from previous day's work
Ships as fsh v2.2.0.
