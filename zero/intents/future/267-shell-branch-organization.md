---
id: 267
date: 2026-09-28
type: future
title: "Shell Branch organization"
status: planned
tags: [novashell, nsh, zero]
---

## Vision

One tree, one purpose per directory, and the shell in one place. NovaShell, the suite that
proves it, and DevBox -- the sandbox that runs it with the machine made invisible -- live together
as the shell branch, the way INT-247's mental model draws Project 0: the shell, the tools, the
memory.

Someone who opens ~/0-core can say what every top-level entry is for from its name and one line in
a map, and a test keeps the map true. Proposed by Christian 2026-09-24 (INT-247); filed 2026-09-28.

## The Problem

Measured 2026-09-28 at a464bb8f, tracked files only, read-only:

```text
    THE SHELL IS IN THREE PLACES
      zero/rust-tools/novashell/    59 files   the shell -- one crate among 22
      zero/rust-tools/nsh-test/      5 files   its suite -- the same
      devbox/                       23 files   at the REPO ROOT: outside zero/, and not a
                                               workspace member
    THE REPO ROOT HOLDS 19 ENTRIES
      zero/ 649 files, docs/ 30, devbox/ 23, assets/ 4, .githooks/ 4, .cargo/ 1, labs/ 1,
      AGENTS.md, README.md, ROADMAP.md, MIGRATION-RUNBOOK.md, LICENSE, Cargo.toml, Cargo.lock,
      deny.toml, census-core-coupling.py, .envrc, .gitignore, .gitleaks.toml
    zero/ HOLDS 9
      engine 88, intents 308, meta 31, policy 5, registry 8, rust-tools 197, schema 5,
      scripts 6, RISK.toml
    WORKSPACE MEMBERS
      zero/rust-tools/zero-docs, zero/rust-tools/*, zero/engine -- the first is already
      matched by the glob
```

Nothing says where a new file goes, so the layout grows by accretion: each file lands where its
author was standing. Single-file directories (labs/, .cargo/) and one-off scripts at the root
(census-core-coupling.py, INT-230's instrument, held on purpose) are what that looks like.

WARNING -- A MOVE IS NOT FREE. INT-247 recorded that a restructure breaks paths in AGENTS.md,
ship and the fixtures, and gave it its own intent and its own pace. This is that intent. INT-252
showed the cost can be paid safely: 652 paths and 133 path strings in one rehearsed commit.

## The Solution

Design first. Move once.

```text
    1  RECON     read-only. What each directory is for; what devbox/ holds and what reads it;
                 every live string naming a path the design would move, by kind; callers
                 outside the repo. Recorded here
    2  DESIGN    the target tree, ONE line of purpose per directory, ruled by Christian before
                 anything moves. Every path the design moves, it moves once
    3  THE MAP   the tree written as a map, and a test that goes red when tree and map
                 disagree. Landed BEFORE the move and seen red, so the move is checked by it
    4  THE MOVE  INT-252's method: a fingerprinted census, git mv plus every path rewrite in
                 ONE commit, rehearsed on a clean clone, rollback rehearsed. One branch per
                 commit
    5  DOORS     cargo test --workspace, ship, nsh-test, d, exec nsh, the home sweep
```

### WHEN -- ruled by Christian 2026-09-28

Recon and design touch nothing, so they can run any time. THE MOVE lands after INT-247's
code-side passes -- the sayings, the identifiers, INT-263 schema, INT-264 contracts -- and
BEFORE INT-247's docs rewrite, so the seven documents are written once, about the final tree.
This supersedes INT-247's line "designed after 247 and 252 close".

### Questions the design must answer

```text
    the branch      zero/shell/ holding novashell, nsh-test and devbox -- or another shape.
                    devbox/ is outside the workspace today; the recon says why before it moves
    rust-tools/     keeps its name, or becomes tools/ in the same move. Either way it moves
                    once, never twice
    the root        census-core-coupling.py, MIGRATION-RUNBOOK.md, labs/, .cargo/ -- each
                    moved, kept as history, or deleted, by the decision test
    documents       docs/, zero/meta/ and assets/ -- where Project 0's documents live.
                    docs/public/ is generated and never hand-edited (INT-247)
    the workspace   the duplicate zero-docs member
```

A STARTING POINT, PROPOSED AND NOT RULED -- recorded so the design has something to argue with:

```text
    ~/0-core/
      zero/
        shell/      novashell, nsh-test, devbox -- the shell and what proves it
        tools/      the other crates
        engine/     core's engine
        intents/    the ledger
        registry/ schema/ policy/ meta/ scripts/
      docs/  AGENTS.md  README.md  ROADMAP.md  LICENSE  and the Cargo and git files
```

### What a move touches -- the recon's checklist

```text
    Cargo       workspace members; every path = "../x" dependency. A crate moved out of
                rust-tools/ breaks its own relative paths and its dependents'
    ship        how it finds the crates it builds and deploys -- read it, do not assume
    paths.rs    source_dir() and anything that joins a subdirectory onto it
    nsh-test    fixtures that BUILD a fake tree (main.rs near 212-216 creates
                zero/rust-tools/...)
    scripts     fpatch at zero/scripts/dev/fpatch.py, the census scripts, .githooks
    registry    tools.toml and aliases.toml, where they hold paths
    AGENTS.md   every path, in the move commit -- agents follow it literally
    outside     Hyprland, .desktop files, systemd user units, ~/.local/bin, rc files,
                crontab, ~/.config/nsh/config.nsh
    NOT EDITED  completed intents, decisions, CHANGELOGs -- history keeps the old paths
```

### Not in scope

The directories outside the repo (INT-247 Layer 3, done). Renaming, rewriting or retiring crates
(INT-247's decision test). Documentation content, except AGENTS.md paths (INT-247's docs
rewrite). ~/0-core and WidkidoneR2/0-Core, which do not move.

## Success Criteria

- [ ] RECON recorded here: every top-level entry and zero/ subdirectory with its file count and
      purpose; what devbox/ holds and what reads it; every live path string naming a directory
      the design moves, by kind; callers outside the repo. Measured, not assumed
- [ ] THE DESIGN is ruled by Christian and recorded here before anything moves: the target tree,
      one line of purpose per directory, where NovaShell, nsh-test and DevBox live, and a ruling
      on each root one-off
- [ ] THE MAP exists and is guarded: a test fails when a directory exists that the map does not
      name, or the map names one that does not exist. Seen red on a planted directory first
- [ ] THE ORDER holds: the move lands after INT-247's sayings, identifiers, INT-263 and INT-264,
      and before its docs rewrite. Evidence: the commit hashes, in order
- [ ] THE MOVE: git mv and every live path rewrite in ONE commit per branch, against a
      fingerprinted census; rehearsed on a clean clone and byte-identical to the live tree;
      rollback rehearsed before it runs
- [ ] THE DOORS on the moved tree: cargo test --workspace all ok, ship 0 failed, nsh-test all
      passing, d 0 failed, exec nsh loads the aliases, and each moved crate builds by name
- [ ] NOTHING OUTSIDE THE REPO BROKE: the home sweep after the move finds no caller of an old
      path
- [ ] STATE UNTOUCHED: the Zero Alias probe reads the same before and after; nothing under
      ~/.local/state, ~/.config, ~/.cache or ~/.local/share changed
- [ ] HISTORY UNTOUCHED: completed intents and CHANGELOGs still name the old paths, and
      git log --follow finds a moved file's history
- [ ] AGENTS.md names the new paths, in the move commit
- [ ] SIMPLE, measured: the repo root and zero/ hold only entries the map names, each with its
      one line, and no single-file directory or one-off script remains without a ruling

## Relationship

- INT-247 -- origin (Christian's proposal, 2026-09-24) and order (ruling 2026-09-28). Its mental
  model is this intent's shape. The move waits on its code-side passes; its docs rewrite waits on
  the move
- INT-252 -- the move method: census, fingerprint, one commit, rehearsal, rollback
- INT-167 -- DevBox, which the proposal groups with the shell it tests
