---
id: 267
date: 2026-09-28
type: future
title: "Shell Branch organization"
status: in-progress
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

## Recon 2026-10-01 (read-only, at a519b7d7, 692 tracked files)

```text
    THE ROOT -- 16 entries
      zero/ 634, devbox/ 20, docs/ 19, .githooks/ 4, assets/ 3, labs/ 1, .cargo/ 1, and
      AGENTS.md, README.md, LICENSE, Cargo.toml, Cargo.lock, deny.toml, .envrc, .gitignore,
      .gitleaks.toml
      gone since a464bb8f, by INT-247: ROADMAP.md (82982b6e), MIGRATION-RUNBOOK.md and
      census-core-coupling.py (948040e5, 92f9093b)
    zero/ -- 9 entries
      intents 311, rust-tools 184, engine 88, meta 27, registry 8, scripts 6, schema 5,
      policy 4, RISK.toml
    THE LEDGER -- 9 directories: cancelled complete decisions experiments future in-progress
      incidents philosophy planned; and README.md, migration-log.md
    devbox/ -- DATA, NOT CODE: cases/ 3 and census/ 17, all TOML. Read by zero-sandbox verify
      <dir> (zero-sandbox main.rs:82 Verify, :1691 run_verify), run by hand: 2026-09-30 on
      devbox/census, 14 passed, 0 failed, 3 undetermined (docs/inventory.md:15). Outside the
      workspace because it holds no crate. Three things share the name: the devbox sandbox
      policy and its nsh builtin (novashell commands/mod.rs:5295-5497), this directory, and
      devshell (zero/scripts, INT-257)
    WHAT A MOVE BREAKS, by kind
      path owner   zero-core paths.rs:392 source_dir().join("rust-tools") -- one owner
      workspace    Cargo.toml members zero/rust-tools/* plus a duplicate zero/rust-tools/zero-docs
      cargo deps   18 crates path = "../zero-core" (novashell also "../zero-git"); engine
                   "../rust-tools/zero-core", "../rust-tools/zero-doctor", "../rust-tools/zero-zone"
      nsh-test     fake-tree fixtures 212-227 and 579-598; command strings 698-970; scanners
                   1459 (base.join("rust-tools")), 1999 (rust-tools/README.md), 4565
      untouched    ship (target/release only, main.rs:372); registry (devbox policy paths are
                   /tmp); .githooks (target/ only)
    OUTSIDE THE REPO -- 74 text files read: Hyprland, systemd user units, ~/.local/bin scripts,
      the nsh config, autostart, desktop entries, rc files. 1 hit: config.nsh:218
      alias loch = tokei ~/0-core/rust-tools -- already broken today, that path does not
      exist. crontab is not installed
    ONE-OFFS -- labs/ holds only RISK.toml; .cargo/ holds audit.toml (cargo-audit, read there by
      Cargo convention); assets/ two branding images and a font
    A NON-ASCII NAME -- zero/intents/incidents/190-... carries an em dash (U+2014). History; it
      keeps its name, and the map test must walk past it
    THE ORDER GATE -- INT-263 and INT-264 are planned, so THE MOVE waits (ruling 2026-09-28).
      Recon, design and the map do not
```

## Design 2026-10-01 -- ruled by Christian

THE TARGET TREE -- lands after INT-263 and INT-264 (ruling 2026-09-28). One line of purpose for
every directory the move creates or touches; the unchanged directories get theirs in THE MAP,
read from their own files rather than written from memory.

```text
    ~/0-core/
      zero/
        shell/              the shell and what proves it
          novashell/        NovaShell, the shell
          nsh-test/         its suite
          devbox/           DevBox cases and census -- what zero-sandbox verify reads
        tools/              the other crates (rust-tools/, renamed in the same move)
        engine/             core's engine
        intents/            the ledger
        meta/ registry/ schema/ policy/ scripts/ RISK.toml      unchanged by the move
      docs/                 Project 0's documents; docs/TREE.md is the map
      labs/                 experiments, listed by nsh's experiment command
      assets/ .githooks/ .cargo/                                unchanged by the move
      AGENTS.md README.md LICENSE Cargo.toml Cargo.lock deny.toml .envrc .gitignore .gitleaks.toml
```

RULINGS -- Christian, 2026-10-01

```text
    1  the branch    zero/shell/ holds novashell, nsh-test and devbox/                    yes
    2  tools/        rust-tools/ becomes tools/ in the same move, so it moves once       yes
    3  labs/         kept for now: nsh's experiment command reads it                     keep
                     (novashell commands/mod.rs:12039-12095)
    4  .cargo/       kept: cargo-audit reads audit.toml there                            yes
    5  workspace     drop the duplicate member zero/rust-tools/zero-docs (Cargo.toml:4)  yes
                     now, as its own change: zero/rust-tools/* already matches it
    6  loch          delete the broken alias, config.nsh:218 (outside the repo)          delete
    7  documents     docs/, zero/meta/ and assets/ stay; their content is INT-247's      yes
                     docs rewrite
    8  the map       docs/TREE.md describes the tree AS IT IS, so it is green today. A   yes
                     test fails on any root or zero/ entry it does not name, and is seen
                     red on a planted directory first. The other direction -- a name in
                     the map that does not exist -- is already guarded by
                     every_backticked_repo_path_in_the_docs_exists. The move commit
                     changes tree and map together
    AGENTS.md        the move commit rewrites line 62, the one AGENTS.md path the move
                     changes (zero/rust-tools/zero-core/src/fingerprint.rs). With the map,
                     one "where a file goes" sentence -- wording Christian's
```

### Found, not fixed -- each for its own intent

```text
    experiment_list   reads labs/ and, when read_dir fails, prints "No active experiments":
                      an unreadable directory answering as an empty one (the INT-250 class)
```

## Design addendum 2026-10-02 -- ruled by Christian

```text
    THE CENSUS AT 3de890ff (re-measured; the Recon above is at a519b7d7)
      693 tracked, 342 history exempt. 144 live lines in 37 files name a path the design moves.
      CENSUS FINGERPRINT dbb51c79ebf2 (int267-census-3de13dda82b5.py, list written to
      ~/.cache/zero/int267-census.txt). Split by why each line changes:
        A  60  names novashell, nsh-test or devbox/      ruling 1, the branch
        C  28  scans every tool, generic                 ruling 1 too, or it goes blind
        B  56  names rust-tools alone                    ruling 2, the rename
    CORRECTION TO THE RECON -- "path owner: paths.rs:392, one owner" was wrong. rust_tools_dir()
      (paths.rs:391) is the owner and 19 sites ask it; 28 generic sites hardcode the path
      instead, and integrity/mod.rs:875 joins it by hand. The 19 follow a rename by themselves
      but scan one directory, so the branch would hide novashell and nsh-test from them.
      Every crate with a C site already depends on zero-core; zero-gate and zero-gen stay off
      it by INT-256 and hold no C site
    THE FINGERPRINT -- tree.repo is ~/0-core's inode plus the origin url (fingerprint.rs:294-318);
      tree.dirs is the five home directories (fingerprint.rs:35-41). A move inside the repo
      changes neither. Before: d reads digest a56683c54812378b

    RULINGS -- Christian, 2026-10-02
    L1  engine is core. The shell is NovaShell (nsh), with what exists only to prove or run it.
        Every other binary is a tool. A new crate goes in tools/ unless it exists only to serve
        the shell. This rule goes into docs/TREE.md with the move
    L2  one owner: zero_core::paths answers where every crate directory is (crate_dirs()), and
        no other live code names it
    L3  three commits, each one concern, each buildable; the census re-run and fingerprinted
        before each:
          0  ONE OWNER   the guard, red first; crate_dirs() checked against the Cargo.toml
                         members; the C sites ask the owner. Behaviour unchanged
          1  THE RENAME  zero/rust-tools/ -> zero/tools/: the owner, the Cargo paths, the B
                         lines, doc paths, zero/RISK.toml, AGENTS.md:62 (shown to Christian
                         before it is written)
          2  THE BRANCH  novashell, nsh-test and devbox/ -> zero/shell/: the A lines; the
                         owner learns shell/ in one place
        Doc PATHS change in the move commit, because every_backticked_repo_path_in_the_docs_exists
        goes red otherwise. Doc prose waits for INT-247's docs rewrite
```

### Found, not fixed -- 2026-10-02

```text
    scripts_dir()     paths.rs:395 joins scripts onto core_dir(), which is ~/0-core (paths.rs:24);
                      the scripts live at zero/scripts/. An owner naming a missing directory,
                      the INT-240 class. Not step 0: one concern per change
    completion.rs:420 offers cd ~/0-core/rust-tools, a path gone since the tree moved under zero/
```

## START HERE -- 2026-10-02

Written only if HEAD was cb676191, clean and pushed; the commits below exist; docs/TREE.md is
tracked; and config.nsh no longer holds the loch alias. A new chat opens with ints 267 and this
section.

### Done -- pushed

```text
    0e9a92a2  the tree map: docs/TREE.md (25 entries), nsh-test the_tree_map_names_every_entry
              (red on a planted zz-planted/, then green), the AGENTS.md where-a-file-goes rule.
              RECON, THE DESIGN and THE MAP ticked with evidence
    cb676191  workspace: the duplicate zero-docs member dropped (ruling 5)
    (home)    ~/.config/nsh/config.nsh: the broken loch alias deleted (ruling 6); 244 aliases load
```

### Blocked -- and on what

```text
    THE MOVE waits for INT-263 and INT-264, both planned, per the ruling of 2026-09-28. Every gate
    after THE MAP depends on the move. Check them first: ints 263, ints 264
```

### Next -- when 263 and 264 are complete

```text
    1  re-measure: the Recon numbers above are at a519b7d7; run the census again at the new HEAD
    2  the move census: every live string naming zero/rust-tools/, novashell/, nsh-test/ or
       devbox/ -- the kinds under Recon, WHAT A MOVE BREAKS -- fingerprinted, INT-252's method
    3  rehearse git mv plus every rewrite on a clean clone, byte-identical to the live result;
       rehearse the rollback before it runs
    4  one commit per branch: the tree, the Cargo paths, paths.rs:392, the nsh-test fixtures,
       docs/TREE.md and AGENTS.md line 62 together. Doors: cargo test --workspace, ship,
       nsh-test, d, exec nsh, the home sweep
```

### Found, not fixed -- recorded, each for its own intent

```text
    experiment_list   a labs/ read error answers as "No active experiments" (under Design)
    fsearch           ignores > file (INT-270's list)
    globs             an absolute-path glob reached cat with its asterisks intact (INT-270's list)
```

## Success Criteria

- [x] RECON recorded here: every top-level entry and zero/ subdirectory with its file count and
      purpose; what devbox/ holds and what reads it; every live path string naming a directory
      the design moves, by kind; callers outside the repo. Measured, not assumed
<!-- evidence: measured 2026-10-01 at a519b7d7. git ls-files census by entry; fsearch devbox; numbered reads of ship, zero-core paths.rs, zero-sandbox and nsh-test; every crate Cargo.toml; the registry, scripts, .githooks, .cargo and labs; a home sweep of 74 text files; crontab absent. Recorded in Recon 2026-10-01 above. -->
- [x] THE DESIGN is ruled by Christian and recorded here before anything moves: the target tree,
      one line of purpose per directory, where NovaShell, nsh-test and DevBox live, and a ruling
      on each root one-off
<!-- evidence: ruled by Christian 2026-10-01 in this session -- yes to items 1 to 8, the loch alias deleted, labs/ kept for now after its reader was read. Recorded in Design 2026-10-01 above. -->
- [x] THE MAP exists and is guarded: a test fails when a directory exists that the map does not
      name, or the map names one that does not exist. Seen red on a planted directory first
<!-- evidence: 2026-10-01, the debug nsh-test with docs/TREE.md (25 entries): 213 of 213. With zz-planted/note planted: 212 of 213, the_tree_map_names_every_entry naming zz-planted/ and its fix. Removed: 213 of 213. The other direction, a name in the map that does not exist, is every_backticked_repo_path_in_the_docs_exists. -->
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
- [ ] ONE OWNER (L2, commit 0): every live Rust site that locates a crate directory asks
      zero_core::paths. A guard fails on any string literal outside paths.rs that names the crate
      directory (its exemptions a written list), and a test fails when crate_dirs() and the
      Cargo.toml members disagree. Both seen RED first; behaviour unchanged, its own commit
- [ ] SAFETY LISTS MOVE WITH THE TREE: the novashell protected lists (commands/mod.rs 2599, 3551,
      3631, 3866, 3983) and zero/RISK.toml name directories that exist after each move commit,
      proven by a test that goes red on a protected entry naming a missing directory
- [ ] FINGERPRINT UNCHANGED: d's Fingerprint check reads digest a56683c54812378b before and after
      each move commit

## Relationship

- INT-247 -- origin (Christian's proposal, 2026-09-24) and order (ruling 2026-09-28). Its mental
  model is this intent's shape. The move waits on its code-side passes; its docs rewrite waits on
  the move
- INT-252 -- the move method: census, fingerprint, one commit, rehearsal, rollback
- INT-167 -- DevBox, which the proposal groups with the shell it tests
