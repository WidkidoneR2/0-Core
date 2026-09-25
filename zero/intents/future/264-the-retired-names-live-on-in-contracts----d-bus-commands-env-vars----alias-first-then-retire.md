---
id: 264
date: 2026-09-24
type: future
title: "the retired names live on in contracts -- D-Bus, commands, env vars -- alias first, then retire"
status: planned
tags: [contracts, dbus, naming, migration, zero]
---

## Vision

Every name another program calls Project 0 by -- a D-Bus name, a command, an environment
variable, a word the natural-language layer understands -- says Project 0 or zero. Each one
changed without a moment where a caller finds nothing.

## The Problem

A contract is a name something ELSE depends on. Renaming it in one commit breaks every caller
that is not in that commit, and the callers are not all in this repository.

Measured 2026-09-24 and 2026-09-25:

```text
    D-Bus        org.faelight.Forest.*, object paths /org/faelight/Forest/{Health, Intent,
                 Friday, Deploy} -- 10 literals in faelight-daemon dbus.rs;
                 GetForestContext and ForestContext on the daemon
    commands     forest-stats, cp-forest, mv-forest, forest-ade; the --forest flags
    env vars     FAELIGHT_STATE_DIR (nsh-test isolates per-case state with it), the FOREST_
                 prefix
    vocabulary   thirteen nl.rs words; the forest theme name
    old sites    /etc/faelight reads -- dead since NixOS, INT-250 owns them
```

The naming map (INT-247, ruled 2026-09-24) already decides the destinations:
org.faelight.Forest.* -> org.zero.*, FAELIGHT_* and FOREST_* -> ZERO_*.

## The Solution

The same method that moved state, config and cache: alias first, move the callers, remove the
old name last.

```text
    1  CENSUS     every contract, with EVERY caller: inside the repo, and outside it --
                  ~/.config, Hyprland binds, systemd units, scripts, ~/.local/bin
    2  BOTH       the new name lands BESIDE the old one. Both answer
    3  CALLERS    every caller moves to the new name, one commit per contract
    4  RETIRE     the old name goes, in its own commit, after the new one has carried a week of
                  ordinary use -- and the display-name guard learns it
```

## Success Criteria

- [ ] The census exists: each contract with every caller, inside the repo and outside it
- [ ] D-Bus: org.zero.* is served alongside the old name; every client moved; the old name
      removed. Proven by introspecting the bus before and after
- [ ] Env vars: ZERO_STATE_DIR is read first and FAELIGHT_STATE_DIR still honoured; nsh-test's
      isolation moved to the new name; then the old read removed, with a test proving it no
      longer applies
- [ ] Commands: the new names land first; for one week the old names say where the command
      went; then they are removed
- [ ] nl.rs vocabulary and the theme name renamed together with their tests
- [ ] No contract is removed in the same commit as its replacement lands
- [ ] Each retired contract name joins the display-name guard when its pass is finished

## Relationship

- Parent: INT-247 -- the rename. This is its CONTRACTS item, filed 2026-09-24
- Sibling: INT-263 -- schema. Stored data, not names other programs call
- INT-250 owns the dead /etc/faelight reads; this intent does not rename a path nothing writes
