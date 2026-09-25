---
id: 265
date: 2026-09-25
type: future
title: "Zero Tools fixes ATTENTION REQUIRED"
status: planned
tags: [zero, nsh, novashell, tools]
---

## Vision

Every tool that came through the Project 0 rename works on Omarchy, tells the truth, and is
reached by the name it now has. The rename changed names and nothing else; this intent is where
the defects it walked past get fixed.

## Why this is its own intent

Ruled by Christian 2026-09-25: until INT-247 and INT-252 are complete and closed, the crate pass
renames and rebrands only. Everything else it finds is recorded here, with its file and line, and
fixed here. One concern per commit; red first wherever a test can hold the fix.

## The Problem -- found during the crate pass, 2026-09-25

### Proven defects

    ship          deploys a binary its registry marks deployable = false. faelight-zone came back
                  after three retirements (backups 09-23 19:26, 09-24 09:18, 09-24 21:39), and
                  zero-zone was shipped in 06e4d6e8
    observe.rs    four tests race on one process-wide environment variable under cargo's parallel
                  runner. targets_select_independently failed 144 of 200 runs of the observe
                  filter; the full novashell suite went red twice on 2026-09-25
    zero-sandbox  the audit query builds SQL with format!, so --tool and --limit are spliced into
                  the statement (src/main.rs near 1514)
    zero-update   maintenance mode runs `sudo journalctl --vacuum-time=2weeks` (src/main.rs:227):
                  automation plus sudo, the 2025-12-14 lockout class

### Dead, NixOS-era behaviour

    zero-update   step 3 advises nhclean (src/main.rs:235-241); --snapshot calls faelight-snapshot,
                  which does not exist (src/main.rs:40, 647-657)
    zero-release  the generation and rollback model: status answers "Current generation: unknown";
                  the description promises bootable, rollback-safe generations
    zero-git      src/commands/risk.rs:48 suggests `faelight snapshot`
    zero-daemon   daemon.rs:697 advises "run deploy" for paths containing faelight-;
                  engine events/mod.rs:732 names the unit faelight-daemon (INT-237: no user services)
    strategy      Factor 7 (strategy/mod.rs:1628) checks scripts that do not exist, so it always
                  scores 0 of 7; :1564 asks systemctl about faelight-insightd
    zero-vm       names from the NixOS build-vm runner, and no VM image exists -- does it drive
                  anything on Omarchy?

### Untrue or invented values -- INT-192's class

    zero-daemon   daemon.rs:886 Friday seed lists faelight-fm; daemon.rs:861 unwrap_or(100)
                  answers "All systems nominal" when health could not be read
    zero-docs     status shows "v1.0.0 -- Morphwood" and counts 21 tools where the index says 22;
                  src/main.rs:311 falls back to "The Living Forest"
    --version     zero-git prints 4.0.0 (package 4.4.2, typed into src/main.rs:17); zero-docs
                  prints its help screen
    Friday        friday/mod.rs:275 "all 22 checks" (27 now), :1046 "50+ custom Rust tools";
                  journal/mod.rs:309 "Health: 100%." as fixed text; the other unwrap_or(100) sites

### Hygiene

    paths         zero-update src/config.rs and zero-vm state_dir() build paths from HOME
    dead code     zero-sandbox emit_to_ledger, kept alive by #[allow(dead_code)]
    dead files    zero-git/Cargo.lock (the workspace uses the root lock);
                  teach/src/main.rs.v2.0.0; devbox census zero-zone.toml (retired binary)
    stale lists   novashell commands/mod.rs:14257 lists core-diff as a tool with tests;
                  AGENTS.md:743 describes the layout before the tree move
    census        zero-ade ignores its arguments, so `zero-ade --version` opens the ADE

### Aliases -- ~/.config/nsh/config.nsh (outside the repo) and zero/registry/aliases.toml

    ruled         z replaces the f prefix, which was named for faelight (Christian, 2026-09-25):
                    fg -> zg        fga fgc fgp fgs -> zga zgc zgp zgs          (:51, :167-170)
                    fu fudr fui fuup -> zu zudr zui zuup                        (:35, :177-179)
                    fr-history fr-preview fr-status -> zr-history zr-preview zr-status (:173-175)
                    f-daemon -> z-daemon (:164)    fdocs -> zdocs (:166)
                  fg also cannot work as it stands: fg is the job-control builtin, and reserved
                  names win over aliases (novashell src/main.rs:1565)
    dead          bar and bar-restart (:33, :78) -> the faelight-bar unit;
                  daemon-log and daemon-status (:136-137) -> the faelight-daemon unit
    broken        qc (:244) -- an unterminated quote
    missing       ship and zero-gate have none (d: Alias Coverage)

## Rulings needed before the work they gate

    zero-release  keep the generation and rollback model, rewrite it for Omarchy, or retire it
    zero-vm       INT-247's decision test: keep or retire
    zero-update   the journal step: drop it, or print the command for a human to run

## The Solution

    1  Wait for INT-247 and INT-252 to close. Nothing here starts while the crate pass is open.
    2  Each item that needs a ruling gets it before its work begins.
    3  One concern per commit, in this order: the proven defects, sudo, the dead behaviour, the
       untrue values, the paths, dead code and files, then the aliases in one sweep last,
       because they live outside the repository.
    4  Red first wherever a test can hold the fix; a class test where the defect is a class.
    5  The doors before every commit: the result on PATH, nsh-test all passing, d 0 failed, plus
       the door for the item.

## Success Criteria

- [ ] ship never copies a binary whose registry entry says deployable = false -- proven red
      first, and zero-zone is off PATH
- [ ] The observe.rs tests cannot race: 200 runs of `cargo test -p novashell --bin nsh observe`
      with 0 failures, and the fix covers every test that sets a process-wide variable
- [ ] The sandbox audit query binds --tool and --limit as parameters; a test with a quote in
      --tool proves it
- [ ] No Project 0 tool runs sudo on its own; zero-update's journal step follows its ruling
- [ ] Every dead NixOS-era path above is removed or rewritten, each by its ruling
- [ ] Every untrue or invented value above reads the real value or says it could not -- never a
      made-up one
- [ ] zero-update and zero-vm take their paths from paths.rs
- [ ] The dead code and dead files above are deleted, and the two stale lists are true
- [ ] Aliases: the f family is renamed to z as ruled, in config.nsh and aliases.toml, each new
      name checked first against every command on PATH and every builtin; zg reaches zero-git;
      the dead aliases are gone; qc is fixed; ship and zero-gate have aliases -- d's Alias
      Coverage is green and deadwood reports 0 dead aliases
- [ ] Each fix is its own commit, red first where a test can hold it; at the end nsh-test is all
      passing and d shows 0 failed

## Relationship

- Filed from INT-247's crate pass; starts after INT-247 and INT-252 are closed (Christian,
  2026-09-25)
- Not here: the schema (INT-263), the contracts (INT-264), the sayings and the remaining tree
  emoji (INT-247's word passes)
- The untrue values are INT-192's collapse: an unknown answered with a happy default
