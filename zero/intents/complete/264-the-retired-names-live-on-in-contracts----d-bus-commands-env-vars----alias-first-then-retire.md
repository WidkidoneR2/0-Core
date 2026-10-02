---
id: 264
date: 2026-09-24
type: future
title: "the retired names live on in contracts -- D-Bus, commands, env vars -- alias first, then retire"
status: complete
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

- [x] The census exists: each contract with every caller, inside the repo and outside it
<!-- evidence: the census was INT-247 pass 5's, with the outside sweep recorded in 719cb38d and c68f1c28: rc files, nsh config, Hyprland, systemd user units, ~/.local/bin and desktop entries, no caller outside the repo. Re-checked 2026-10-02: fsearch --live for org.faelight, FAELIGHT_STATE_DIR, forest-stats, cp-forest, FOREST_ and forest in nl.rs: no results. -->
- [x] D-Bus: org.zero.* is served alongside the old name; every client moved; the old name
      removed. Proven by introspecting the bus before and after
<!-- evidence: c68f1c28 (INT-247 pass 5B): org.faelight.Forest is org.zero.Core, both ends in one commit (zero-daemon dbus.rs:241-245). busctl --user list 2026-10-02, rc 0: no faelight name. Served-alongside and the before/after introspection are DECLINED, R3 and R4: 1) no unit and no client start zero-daemon, and it was not running at 5B either, so there was no caller to keep answering; 2) whether zero-daemon runs and serves org.zero.Core is Friday's question and goes to the Friday intent. -->
- [x] Env vars: ZERO_STATE_DIR is read first and FAELIGHT_STATE_DIR still honoured; nsh-test's
      isolation moved to the new name; then the old read removed, with a test proving it no
      longer applies
<!-- evidence: 719cb38d (INT-247 pass 5A): ZERO_STATE_DB, ZERO_STATE_DIR and ZERO_ADE; every reader and writer moved together, nsh-test isolation included. The compatibility read is DECLINED, R3: the sweep found no caller outside the repo. That the old name no longer applies anywhere is held by nsh-test no_live_retired_name_in_any_tracked_file (12772479), 213/213 on 2026-10-02. -->
- [x] Commands: the new names land first; for one week the old names say where the command
      went; then they are removed
<!-- evidence: 7c2753b3 (INT-247 pass 5C) renamed the typed commands; eceb401d removed forest-ade. The week of redirects is DECLINED, R3: no caller outside the repo (719cb38d, c68f1c28 sweep). -->
- [x] nl.rs vocabulary and the theme name renamed together with their tests
<!-- evidence: 7c2753b3: the typed commands, theme and phrases stop saying forest. fsearch forest --file nl.rs: no results, 2026-10-02; nsh-test 213/213. -->
- [x] No contract is removed in the same commit as its replacement lands
<!-- evidence: DECLINED, R3, ruled by Christian 2026-10-02. Not followed as written: 719cb38d and c68f1c28 each moved both ends of their contracts in one commit, deliberately. 1) every caller was in the repo and moved in that same commit; 2) the outside sweep found none; 3) names approved 2026-09-29 (c68f1c28). The gate exists so no caller finds nothing; with no caller outside, none could. -->
- [x] Each retired contract name joins the display-name guard when its pass is finished
<!-- evidence: 12772479: no_live_retired_name_in_any_tracked_file matches both retired words in any tracked file, any case, so every retired contract name (org.faelight, FAELIGHT_*, FOREST_, forest-*) is held. RED then GREEN, 247:3546. -->

## Close 2026-10-02

The work of this intent landed under INT-247: env vars in 719cb38d (pass 5A), the daemon contracts
in c68f1c28 (5B), the typed commands, theme and phrases in 7c2753b3 (5C), forest-ade in eceb401d.

```text
    RULINGS -- Christian, 2026-10-02
    R3  the alias-first method is declined for these contracts: every caller was in the repo and
        moved in the same commit, and the 2026-09-29 sweep outside the repo found none
        (719cb38d, c68f1c28). Names approved 2026-09-29
    R4  zero-daemon has no unit and no client and is not running. Whether it runs and serves
        org.zero.Core goes to the Friday intent (INT-039, friday-daemon, is the candidate home)
```

## Relationship

- Parent: INT-247 -- the rename. This is its CONTRACTS item, filed 2026-09-24
- Sibling: INT-263 -- schema. Stored data, not names other programs call
- INT-250 owns the dead /etc/faelight reads; this intent does not rename a path nothing writes
