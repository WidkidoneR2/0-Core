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

## START HERE -- 2026-10-02 morning -- SUPERSEDED by START HERE -- 2026-10-02 evening, below

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

## START HERE -- 2026-10-02 evening -- SUPERSEDED by START HERE -- 2026-10-02 night, below

Written only if HEAD was 52a0cc57 and equal to origin/main, the tree was clean, every commit named
below exists, nsh-test/crate-paths-allowed.txt held 72 entries, paths.rs declared CRATE_PARENTS, and
the census script was cached. The morning START HERE above is superseded by this one.

### For the next chat -- read this first

Christian opens a new chat by pasting the output of `ints 267`. That chat continues from exactly
here, with no further instruction:

```text
    1  Read this whole intent: Vision, Recon 2026-10-01, Design 2026-10-01, Design addendum
       2026-10-02, this START HERE, and the Success Criteria
    2  Do NOT re-ask anything under RULINGS IN FORCE below. They are settled
    3  Work by METHOD below. It is binding, and it is how every step today was done
    4  Begin with NEXT, step 1: the read-only recon block, pasted verbatim. Say in two or three
       lines what it will show, then wait for the output
    5  Replies are short (two to three minutes of work), give ONE direction, never narrate a
       hypothesis that a lookup can settle, and never change Christian's AGENTS.md -- propose only
```

### Done -- pushed

```text
    3de890ff  INT-263 and INT-264 closed on evidence. Their work had landed under INT-247; every
              gate ticked with the INT-247 commit that did it, or declined with numbered reasons
              (rulings R1-R4, recorded in those two intents). INT-267's order gate is unblocked
    2776a7df  INT-267 Design addendum: rulings L1 L2 L3, the census at 3de890ff (144 lines in 37
              files, CENSUS FINGERPRINT dbb51c79ebf2), three new gates
    abe2adab  nsh-test crate_directory_has_one_owner -- THE GUARD. Fails on any live Rust string
              literal naming the crate directory outside zero_core::paths, unless listed in
              zero/rust-tools/nsh-test/crate-paths-allowed.txt (THE RATCHET, 72 entries). Seen RED
              with an empty list (72 named, 220 .rs files read), GREEN with the list, RED on one
              planted literal, GREEN after it was removed
    52a0cc57  zero-core THE OWNER (paths.rs, right after rust_tools_dir) and nsh-test
              crate_dirs_match_the_workspace. 7 unit tests pass. The case compares crate_dirs()
              with the Cargo.toml members both ways; seen RED on a planted member, GREEN after
```

State at 52a0cc57, measured: ship 16 shipped 0 failed; deployed nsh-test 215 of 215; d 0 failed;
Fingerprint PASS, schema 2, digest a56683c54812378b (tree.repo is the inode of ~/0-core plus the
origin url, so moves inside the repo do not change it).

### Rulings in force -- do not re-ask

```text
    L1  engine is core. The shell is NovaShell (nsh), with what exists only to prove or run it.
        Every other binary is a tool. A new crate goes in tools/ unless it only serves the shell
    L2  ONE owner: zero_core::paths says where every crate directory is; no other live code
        names it
    L3  three steps, each buildable: 0 ONE OWNER, 1 THE RENAME (zero/rust-tools -> zero/tools),
        2 THE BRANCH (novashell, nsh-test, devbox -> zero/shell). Census re-run and fingerprinted
        before 1 and before 2
    L3a step 0 lands as SEVERAL green commits, each shrinking the ratchet (proposed 2026-10-02,
        Christian proceeded with it). The guard proves each batch
    L3b the three DEAD sites (anomaly, integrity, zero-zone) are fixed by asking the owner in
        step 0. It is a declared behaviour change: read their output before and after
    L3c these stay in the ratchet on purpose: nsh-test fake-tree fixtures and tilde command
        strings (they test the shell with real paths; they change in step 1), and teach
        main.rs:956-962 (the whole tour is stale; its own fix)
    R5  every rehearsal runs rustfmt on the payload Rust before the plan is shown, so the plan
        is the text zero-gate commits. Staged stat must equal committed stat
```

### The owner -- zero_core::paths (paths.rs, after rust_tools_dir)

```text
    CRATE_PARENTS          ["zero/rust-tools"]   each child holding a Cargo.toml is a crate
    SINGLE_CRATES          ["zero/engine"]       a crate that is its own directory
    CRATE_PARENTS_HISTORY  ["rust-tools", "zero/rust-tools"]   oldest first, never shrinks
    crate_dirs_in(root) / crate_dirs()      io::Result<Vec<PathBuf>>; unreadable = Err, never empty
    crate_rel_dir_in(root, name) / crate_rel_dir(name)   Option<String>, "zero/rust-tools/<name>"
    crate_of(path)                          Option<String>: the crate a path string is inside
    crate_history_pathspecs(name)           git log pathspecs across every parent
    rust_tools_dir()                        now core_dir().join(CRATE_PARENTS[0]), same value
```

CRATE_PARENTS_HISTORY lists every parent a crate has ever lived in, the current ones included, so
git log follows a crate across moves. Step 1: CRATE_PARENTS becomes ["zero/tools"] and "zero/tools"
is appended to HISTORY. Step 2: "zero/shell" joins CRATE_PARENTS and is appended to HISTORY. The
Cargo.toml members change in the same commit; crate_dirs_match_the_workspace keeps the two in step.

### The ratchet -- how a batch moves

```text
    file    zero/rust-tools/nsh-test/crate-paths-allowed.txt, one line per occurrence:
            file<TAB>literal. Comment lines start with #
    rule    a found literal beyond its listed count FAILS (a new hardcoded path); a listed entry
            beyond what is found FAILS (remove it). Counted, not just present
    a batch rewrites its sites to ask the owner AND deletes exactly their lines in the same
            commit. Before the change the guard is green; after the rewrite but before the list
            edit it fails naming the removed literals as "allowed N but found M" -- that is the
            batch proving itself. Then the list edit makes it green
    done    the list has no entries: L2 holds
```

Today the list holds 72: anomaly 1, audit 4, evolution 1, integrity 1, intent 5, planning 1,
novashell commands/mod.rs 13, completion 1, prompt 1, nsh-test 22, teach 1, zero-context 2,
zero-deadwood 7, zero-docs toolgen 10, zero-update cargo_checker 1, zero-zone detect 1.

### Next -- batch 1: the three dead sites

```text
    anomaly    zero/engine/src/domains/anomaly/mod.rs:208-217  git log --since=30 days -- engine/
               rust-tools/  run from ctx.core_root (~/0-core). Both moved under zero/ on
               2026-09-25, so the scan reads nothing and reports nothing
    integrity  zero/engine/src/domains/integrity/mod.rs:872-905  core_root.join("rust-tools") and
               (:900) core_root.join("engine/Cargo.toml"). core_root is ~/0-core (context.rs:17-24,
               core_root_string()), so every registry-vs-Cargo version check hits !exists ->
               continue and passes having read nothing. Registry name "core" means the engine
    zero-zone  zero/rust-tools/zero-zone/src/detect.rs:11  home.join("0-core/rust-tools") never
               exists, so the Workspace zone never fires. detect_zone(path, home) takes home for
               tests: the fix must keep that, e.g. the owner's names joined onto home/0-core
```

Step 1 -- recon, read-only. Paste exactly:

```text
python3 -c 'import sys,glob,os
a=sys.argv[1:]
for g,s,e in zip(a[0::3],a[1::3],a[2::3]):
  ps=[p for p in sorted(glob.glob(g)) if os.path.isfile(p)]
  if not ps: print("== NO MATCH",g)
  for p in ps:
    print("==",p,s,e)
    for i,l in enumerate(open(p),1):
      if int(s)<=i<=int(e): print(str(i).rjust(5),l,end="")' /home/christian/0-core/zero/engine/src/domains/anomaly/mod.rs 190 240 /home/christian/0-core/zero/engine/src/domains/integrity/mod.rs 850 935 /home/christian/0-core/zero/rust-tools/zero-zone/src/detect.rs 1 60
fsearch anomaly --file parser.rs
fsearch integrity --file parser.rs
fsearch detect_zone --type rs --live
```

It shows each dead site whole, and how each is run (the core subcommands and zone's callers), so
step 2 can capture BEFORE output. Then:

```text
    2  capture BEFORE: run the anomaly scan, the integrity check and a zone detection inside a
       crate directory, each to a /tmp file (commands from step 1's recon)
    3  plan the batch: the three sites ask the owner (anomaly: pathspecs from SINGLE_CRATES,
       CRATE_PARENTS and CRATE_PARENTS_HISTORY; integrity: crate_rel_dir / the engine for "core";
       zone: the owner's parents under home/0-core), their 3 ratchet lines deleted. A unit or
       nsh-test check per site that fails on today's code first (red), then passes
    4  apply, build, cargo check --workspace, unit tests, suite (expect 215 of 215 with the list
       at 69), AFTER output captured and compared with BEFORE. Anything new that anomaly or
       integrity now report is a FINDING to read, not noise to silence
    5  ship, doors, commit (Method). Then batch 2: MATCH and SAFETY through crate_of
```

### Method -- binding, as practised today

```text
    RECON     read-only first. Search with fsearch (not grep or awk); read exact text with the
              numbered python reader above. Never claim how a command behaves without reading it
    PAYLOAD   a python script in ONE argv word: base64 of zlib. Installed under its own hash with
                python3 -c 'import base64,zlib,sys,hashlib,os; b=zlib.decompress(base64.b64decode(sys.argv[1])); h=hashlib.sha256(b).hexdigest()[:12]; p=os.path.expanduser("~/.cache/zero/<name>-"+h+".py"); os.makedirs(os.path.dirname(p),exist_ok=True); open(p,"wb").write(b); print("installed",p)' <b64>
              The reply names the expected file; any other name means a damaged copy: stop
    REHEARSE  in Claude's sandbox on a stand-in tree: refuse paths, wrong fingerprint, apply,
              second run refused. Rust compile-checked with rustc in a stub harness, formatted by
              rustfmt (R5). The copy in the reply decoded and compared byte for byte
    PLAN      `plan` prints every edit and a FINGERPRINT and writes nothing. Christian reviews.
              Claude then sends `apply <real fingerprint>`, never a placeholder. Every check runs
              before the first write; a REFUSE line means stop and paste
    EDIT      through fpatch: sys.path zero/scripts/dev, patch(abs_path, old, new). One concern
              per edit. Anchors byte-exact; em dash and double dash are not interchangeable
    RED FIRST a new check is seen failing (empty list, or a plant/unplant mode in the payload
              that adds one stray thing and then proves the file identical to git again)
    TEST      cargo build -p <crate> > /tmp/build.txt 2>&1 ; cargo check --workspace when zero-core
              changes ; unit tests ; the suite to a file, never a filter:
                env NSH_BIN=/home/christian/.local/bin/nsh /home/christian/0-core/target/debug/nsh-test > /tmp/suite.txt 2>&1
    SHIP      ship > /tmp/ship.txt 2>&1 ; tail -8 /tmp/ship.txt ; nsh-test > /tmp/suite.txt 2>&1 ;
              d (0 failed, Fingerprint digest a56683c54812378b) ; git status --short ;
              exec /home/christian/.local/bin/nsh as the LAST line of the block
    COMMIT    only after the doors are green, as its own block: git add exact paths ; git diff
              --cached --stat ; git commit -m subject -m what-was-false -m watched --trailer
              "Intent: INT-267" --trailer "Fingerprint: <plan fingerprint>" ; git push ; delete the
              finished cached script. If the committed stat differs from the staged stat, stop and
              read what zero-gate changed
    PASTE     no apostrophes except the single quotes around python -c, no heredocs, no bare
              --help, no $ in double quotes. NEVER sudo
    KEEP      ~/.cache/zero/int267-census-3de13dda82b5.py -- the move census, re-run before step 1
              and before step 2 (it writes ~/.cache/zero/int267-census.txt and prints a CENSUS
              FINGERPRINT). Delete it when INT-267 closes
```

### Found, not fixed -- recorded 2026-10-02, each for its own owner

```text
    zero-gate    gate_rustfmt (zero-gate main.rs:305-328) runs cargo fmt on staged files and
                 git-adds the result, so committed text can differ from reviewed text
                 (abe2adab: nsh-test lines 88-91, formatting only). Its decision: refuse with
                 --check, or keep rewriting. R5 covers this side meanwhile
    nsh labels   an external command's exit 2 is printed as "misuse of shell builtin" (seen on
                 ls of a missing file): a category guessed where a fact belongs (AGENTS.md s3)
    flea/        fsearch matched flea/src/paths.rs, which 0-core does not track. Unexplained
    Friday       for INT-039 (friday-daemon) or a new Friday intent Christian will file: the
                 missing-table-reads-empty collapse at friday reasoning.rs:201 and planning.rs:899
                 (R2), and zero-daemon -- no unit, not running, org.zero.Core unserved (R4)
    and above    scripts_dir(), completion.rs:420, experiment_list -- in the Design sections
```

## START HERE (2026-10-03, end of the long session; it replaces the START HERE below)

Paste this whole section into a new chat. Christian opens with `ints 267`; Claude reads this and
continues from NEXT, step 1. Every block Claude sends follows the METHOD at the end.

### Where the tree is

- HEAD: the commit that wrote this record (parent 7c765757). Tree clean, pushed.
- zero/shell/ holds novashell, nsh-test and devbox (THE BRANCH, 183d2f26). zero/tools/ holds every
  other crate. zero_core::paths names its parents: TOOLS_PARENT = "zero/tools",
  SHELL_PARENT = "zero/shell"; CRATE_PARENTS = [TOOLS_PARENT, SHELL_PARENT];
  CRATE_PARENTS_HISTORY = ["rust-tools", "zero/rust-tools", "zero/tools", "zero/shell"].
- Cargo.toml: members zero/tools/*, zero/shell/*, zero/engine; exclude = ["zero/shell/devbox"]
  (devbox holds cases, not a crate; cargo refuses a glob member without a Cargo.toml).
- Doors at 7c765757: nsh-test 215 of 215; novashell 231 tests; d 0 failed, Path Resilience 18/18,
  Fingerprint digest a56683c54812378b (unchanged across every move); Integrity 67 percent (expected
  until REGISTRY DRIFT RESOLVED).
- Census (for the final census at close): script ~/.cache/zero/int267-census-3de13dda82b5.py,
  writes ~/.cache/zero/int267-census.txt; last fingerprint 7b233342c899 at b92f6cf2 (before the
  branch). After the branch only history lines should name zero/tools/novashell or nsh-test.

### Commits this session (oldest first)

- 605a0222 step 2a: zero_core::paths::crate_dir(name) and tool_parent_entries(); 17 tools_dir()
  callers ask for one crate or every parent; catalog links from index_link (core -> ../engine/).
- 1671ca4e step 2b: nsh's source_root() is zero/ (@rust, ruling A); rm -rf guard from the owner
  (rm_guarded_dirs; it had guarded ~/0-core/engine, which did not exist); @scripts, fsearch --all
  and --scripts fixed; preexec lost its unused core_root.
- aa679574 step 2c: one owner of "is this runnable": zero_core::paths::on_path / on_path_in; the
  tools builtin's deployed column asks it (it checked the NixOS-era ~/0-core/scripts/<name>).
- cd23f537 step 2d: find_on_path / find_on_path_in; nine which:: calls ask the owner; the which
  crate left the engine and the doctor (Cargo.lock dropped which and winsafe).
- b92f6cf2 gate added: COMMAND COLOUR TELLS THE TRUTH.
- 2de63b5c step 2e: guard markers are whole parents (zero/tools/, zero/shell/) and a relative path
  is read from the current directory (ruling b); cicomplete reads CRATE_PARENTS_HISTORY.
- 183d2f26 step 2, THE BRANCH: git mv into zero/shell/ (85 files), the owner's named parents,
  fixtures and ratchet (21 entries), TREE.md with rule L1, ARCHITECTURE, RISK.toml, two AGENTS.md
  rules Christian ruled in (owner tested on the real tree; exit codes and cmp).
- 7c765757 command colour, commit A: commands::builtin_names::BUILTINS (160 groups, 211 names)
  generated from the dispatcher's 154 top-level arms, the REPL's exact catches (cheat, it, gt) and
  is_repl_state_command's job-control words (jobs, fg, bg, kill); a test reads those three sources
  and compares both ways; command_class() colours the prompt (dangerous magenta, builtin cyan,
  on_path or alias green, else red). Christian saw it work on the live shell.

### Rulings in force (new this session)

- A: @rust means zero/, the Rust source of every crate.
- b: guard markers are whole crate parents; relative paths are read from the current directory.
- (a): a word the dispatcher catches is cyan, because nsh's own code runs.
- Order: THE BRANCH, then COMMAND COLOUR TELLS THE TRUTH, then REGISTRY DRIFT RESOLVED, then close.
- Earlier rulings L1-L3c, R5, N1-N6 stand.

### NEXT (do these in order)

1. COMMAND COLOUR, commit B: the cheatsheet and completion read BUILTINS.
   - Rebuild the planner (the old one was pinned to HEAD 7c765757 and was deleted). Same edits:
     cheatsheet_tui.rs replaces its parse of commands/mod.rs on disk (which read only 700 lines of
     the match, so 44 builtins never reached it; the refresh says 105 builtins) with a walk over
     BUILTINS, keeping SKIP and the INSERT statement byte for byte; completion.rs iterates BUILTINS
     at both candidate sites (about lines 594 and 611) and const COMMANDS is removed.
   - FOUND at the end of the session by Christian's Tab tests, before anything was applied:
     (a) completion has more sources than the two COMMANDS sites -- Tab on fs listed fsearch with
     a description ("search repo files") and Tab on friday listed "friday dismiss" with one, so a
     description-carrying source (probably command_registry, which the cheatsheet fills) feeds it
     too. Read the whole completer in completion.rs, every candidate source, BEFORE rebuilding
     commit B's planner. (b) intent is probably run by nsh: Tab offers intent list, show, search,
     new and edit, and Christian has used intent add. It is no dispatcher arm, REPL catch, alias
     or program, so nsh routes it by a path commit A's list does not read (a prefix router or a
     core pass-through). FIRST: Christian types intent and reports its colour; then find the
     route. If nsh runs it, that route becomes the list's fourth source, read by
     the_list_is_the_dispatcher_and_the_repl as well, RED first (the colour test must fail on
     intent before the fix), in its own commit before commit B.
   - RULED (Christian, 2026-10-03), to apply after (a) and (b) are re-measured: a word nothing
     runs leaves first-word completion (the first report named hist, flow, fs and intent); "friday
     dismiss" is a subcommand of the friday builtin and belongs to a subcommand completer outside
     this gate. Any word the re-measure finds runnable stays. Record the ruling in commit B's
     message.
   - Proof is before and after (no new red test; this is wiring): BEFORE, Christian typed bump and
     Tab offered nothing (2026-10-03). AFTER ship and exec, Tab on bump offers bump-versions, and
     the cheatsheet refresh line reads more than 105 builtins.
   - Then tick COMMAND COLOUR TELLS THE TRUTH with evidence 7c765757 and commit B.
2. REGISTRY DRIFT RESOLVED: the four pending version proposals (db-browse 1.0.0 to 2.1.0,
   friday-chat 1.0.0 to 2.1.0, zero-git 4.4.1 to 5.1.0, and the fourth named in the gate) go
   through core integrity apply; d must read Integrity 100 percent; the only change in
   zero/registry/tools.toml is those version lines, in a commit of its own. Read the gate text and
   core integrity's CLI before running anything.
3. Close INT-267: rule on THE MOVE gate (step 1 landed in two commits, bbce22a1 and 6df319fd);
   run the final census; tick each gate with its evidence commit; cicomplete 267.

### Findings recorded, not fixed (outside this intent unless Christian rules otherwise)

- docs/ARCHITECTURE.md lists a root bin/ that docs/TREE.md does not name; it probably does not exist.
- make is nsh's own mkdir -p (commands/mod.rs, INT-270); typing make never reaches GNU make. It now
  shows cyan, which is the truth; whether the word should stay taken is Christian's call.
- export is caught nowhere in nsh and now reads red.
- nsh-test's case tilde_ls_rust_tools now lists zero/shell; its name kept for state.db continuity.
- The ratchet's move shows as add plus delete (over half its lines changed), so git log --follow
  does not follow it.
- ~/.config/nsh/config.nsh line 141 became alias diff = difft --exit-code (outside the repo), so
  diff exits 1 on differences.

### METHOD (every block)

- RECON first; read before claiming. One concern per commit.
- PLAN -> REVIEW -> APPLY: payloads base64 plus zlib, installed at ~/.cache/zero/<name>-<sha>.py,
  rehearsed on a stand-in shaped like the real file; planners read the base from git HEAD and
  refuse on a wrong HEAD, a dirty tree or an anchor found other than once.
- RED first for every new check; the RED stage must fail on Christian's machine before FIX.
- Before staging Rust: rustfmt --edition 2021 --check on the touched files (exit 0), so the staged
  stat equals the committed stat (R5). Compare files with cmp, never diff (it is difftastic).
- DOORS before every commit: workspace build with zero warnings; tests; nsh-test 215 of 215; ship;
  deployed suite; d 0 failed and Fingerprint unchanged; the door specific to the change; exec
  /home/christian/.local/bin/nsh last in its block.
- Move commits: name only paths that exist; show unstaged count 0 before committing; add and
  commit in separate blocks.
- Never touch sudo. Propose AGENTS.md changes; never edit it unasked.

## SUPERSEDED START HERE (2026-10-03, end of the long session; it replaces the START HERE below) (superseded 2026-10-03 by the START HERE above)

Paste this whole section into a new chat. Christian opens with `ints 267`; Claude reads this and
continues from NEXT, step 1. Every block Claude sends follows the METHOD at the end.

### Where the tree is

- HEAD: the commit that wrote this record (parent 7c765757). Tree clean, pushed.
- zero/shell/ holds novashell, nsh-test and devbox (THE BRANCH, 183d2f26). zero/tools/ holds every
  other crate. zero_core::paths names its parents: TOOLS_PARENT = "zero/tools",
  SHELL_PARENT = "zero/shell"; CRATE_PARENTS = [TOOLS_PARENT, SHELL_PARENT];
  CRATE_PARENTS_HISTORY = ["rust-tools", "zero/rust-tools", "zero/tools", "zero/shell"].
- Cargo.toml: members zero/tools/*, zero/shell/*, zero/engine; exclude = ["zero/shell/devbox"]
  (devbox holds cases, not a crate; cargo refuses a glob member without a Cargo.toml).
- Doors at 7c765757: nsh-test 215 of 215; novashell 231 tests; d 0 failed, Path Resilience 18/18,
  Fingerprint digest a56683c54812378b (unchanged across every move); Integrity 67 percent (expected
  until REGISTRY DRIFT RESOLVED).
- Census (for the final census at close): script ~/.cache/zero/int267-census-3de13dda82b5.py,
  writes ~/.cache/zero/int267-census.txt; last fingerprint 7b233342c899 at b92f6cf2 (before the
  branch). After the branch only history lines should name zero/tools/novashell or nsh-test.

### Commits this session (oldest first)

- 605a0222 step 2a: zero_core::paths::crate_dir(name) and tool_parent_entries(); 17 tools_dir()
  callers ask for one crate or every parent; catalog links from index_link (core -> ../engine/).
- 1671ca4e step 2b: nsh's source_root() is zero/ (@rust, ruling A); rm -rf guard from the owner
  (rm_guarded_dirs; it had guarded ~/0-core/engine, which did not exist); @scripts, fsearch --all
  and --scripts fixed; preexec lost its unused core_root.
- aa679574 step 2c: one owner of "is this runnable": zero_core::paths::on_path / on_path_in; the
  tools builtin's deployed column asks it (it checked the NixOS-era ~/0-core/scripts/<name>).
- cd23f537 step 2d: find_on_path / find_on_path_in; nine which:: calls ask the owner; the which
  crate left the engine and the doctor (Cargo.lock dropped which and winsafe).
- b92f6cf2 gate added: COMMAND COLOUR TELLS THE TRUTH.
- 2de63b5c step 2e: guard markers are whole parents (zero/tools/, zero/shell/) and a relative path
  is read from the current directory (ruling b); cicomplete reads CRATE_PARENTS_HISTORY.
- 183d2f26 step 2, THE BRANCH: git mv into zero/shell/ (85 files), the owner's named parents,
  fixtures and ratchet (21 entries), TREE.md with rule L1, ARCHITECTURE, RISK.toml, two AGENTS.md
  rules Christian ruled in (owner tested on the real tree; exit codes and cmp).
- 7c765757 command colour, commit A: commands::builtin_names::BUILTINS (160 groups, 211 names)
  generated from the dispatcher's 154 top-level arms, the REPL's exact catches (cheat, it, gt) and
  is_repl_state_command's job-control words (jobs, fg, bg, kill); a test reads those three sources
  and compares both ways; command_class() colours the prompt (dangerous magenta, builtin cyan,
  on_path or alias green, else red). Christian saw it work on the live shell.

### Rulings in force (new this session)

- A: @rust means zero/, the Rust source of every crate.
- b: guard markers are whole crate parents; relative paths are read from the current directory.
- (a): a word the dispatcher catches is cyan, because nsh's own code runs.
- Order: THE BRANCH, then COMMAND COLOUR TELLS THE TRUTH, then REGISTRY DRIFT RESOLVED, then close.
- Earlier rulings L1-L3c, R5, N1-N6 stand.

### NEXT (do these in order)

1. COMMAND COLOUR, commit B: the cheatsheet and completion read BUILTINS.
   - Rebuild the planner (the old one was pinned to HEAD 7c765757 and was deleted). Same edits:
     cheatsheet_tui.rs replaces its parse of commands/mod.rs on disk (which read only 700 lines of
     the match, so 44 builtins never reached it; the refresh says 105 builtins) with a walk over
     BUILTINS, keeping SKIP and the INSERT statement byte for byte; completion.rs iterates BUILTINS
     at both candidate sites (about lines 594 and 611) and const COMMANDS is removed.
   - RULING NEEDED FIRST: five COMMANDS words would leave completion because nothing runs them --
     hist, flow, fs, intent, and the two-word "friday dismiss". Claude recommends letting the
     first four go; "friday dismiss" is a subcommand of the friday builtin and belongs to a
     subcommand completer, not the first-word list.
   - Proof is before and after (no new red test; this is wiring): BEFORE, Christian typed bump and
     Tab offered nothing (2026-10-03). AFTER ship and exec, Tab on bump offers bump-versions, and
     the cheatsheet refresh line reads more than 105 builtins.
   - Then tick COMMAND COLOUR TELLS THE TRUTH with evidence 7c765757 and commit B.
2. REGISTRY DRIFT RESOLVED: the four pending version proposals (db-browse 1.0.0 to 2.1.0,
   friday-chat 1.0.0 to 2.1.0, zero-git 4.4.1 to 5.1.0, and the fourth named in the gate) go
   through core integrity apply; d must read Integrity 100 percent; the only change in
   zero/registry/tools.toml is those version lines, in a commit of its own. Read the gate text and
   core integrity's CLI before running anything.
3. Close INT-267: rule on THE MOVE gate (step 1 landed in two commits, bbce22a1 and 6df319fd);
   run the final census; tick each gate with its evidence commit; cicomplete 267.

### Findings recorded, not fixed (outside this intent unless Christian rules otherwise)

- docs/ARCHITECTURE.md lists a root bin/ that docs/TREE.md does not name; it probably does not exist.
- make is nsh's own mkdir -p (commands/mod.rs, INT-270); typing make never reaches GNU make. It now
  shows cyan, which is the truth; whether the word should stay taken is Christian's call.
- export is caught nowhere in nsh and now reads red.
- nsh-test's case tilde_ls_rust_tools now lists zero/shell; its name kept for state.db continuity.
- The ratchet's move shows as add plus delete (over half its lines changed), so git log --follow
  does not follow it.
- ~/.config/nsh/config.nsh line 141 became alias diff = difft --exit-code (outside the repo), so
  diff exits 1 on differences.

### METHOD (every block)

- RECON first; read before claiming. One concern per commit.
- PLAN -> REVIEW -> APPLY: payloads base64 plus zlib, installed at ~/.cache/zero/<name>-<sha>.py,
  rehearsed on a stand-in shaped like the real file; planners read the base from git HEAD and
  refuse on a wrong HEAD, a dirty tree or an anchor found other than once.
- RED first for every new check; the RED stage must fail on Christian's machine before FIX.
- Before staging Rust: rustfmt --edition 2021 --check on the touched files (exit 0), so the staged
  stat equals the committed stat (R5). Compare files with cmp, never diff (it is difftastic).
- DOORS before every commit: workspace build with zero warnings; tests; nsh-test 215 of 215; ship;
  deployed suite; d 0 failed and Fingerprint unchanged; the door specific to the change; exec
  /home/christian/.local/bin/nsh last in its block.
- Move commits: name only paths that exist; show unstaged count 0 before committing; add and
  commit in separate blocks.
- Never touch sudo. Propose AGENTS.md changes; never edit it unasked.

## SUPERSEDED START HERE -- 2026-10-02 night (superseded 2026-10-03 by the START HERE above)

Written at the end of the 2026-10-02 night session, at 6df319fd. Step 0 (ONE OWNER) and step 1 (THE
RENAME) are done and pushed. Step 2 (THE BRANCH) is next and gets its own chat. The evening START
HERE above is superseded by this one.

### For the next chat -- read this first

Open with `ints 267` and paste this section. With the Design addendum 2026-10-02 (rulings L1-L3c,
R5) and the Success Criteria below, it is everything needed: do not ask Christian to explain where
the work stands, and do not re-ask a ruling listed here.

### Done -- pushed, 2026-10-02 night

```
df771d1d  batch 1   anomaly and zero-zone ask the owner; both were dead sites      ratchet 72 -> 70
6aa11b70  batch 1b  integrity drift check: reads through the owner, PROPOSES       ratchet 70 -> 69
                    instead of auto-fixing, records every drift (dedup by
                    finding), stores tool<TAB>version, core integrity apply can
                    apply it. Its 3 tests seen RED then GREEN
1eae2a3f  gate      REGISTRY DRIFT RESOLVED added (Christian's ruling)
288fb6d4  batch 2   nsh guards: delete warns inside zero/ again (it was dead --   ratchet 69 -> 60
                    every guarded dir had moved), protected lists use
                    crate_parent_markers, cargo_checker counts engine changes
a8fbc189  batch 3   audit, deadwood, zero-docs, nsh-test: locating crates and     ratchet 60 -> 45
                    their git history ask the owner
07b642d2  batch 4   cicomplete bump suggestions and nsh version maps use          ratchet 45 -> 32
                    crate_rel_dir (same five tools)
68623b38  batch 5   displayed paths from the owner; cd completion offers the      ratchet 32 -> 22
                    real dirs. STEP 0 DONE -- 22 left, all L3c
b3783d96  step 1a   crate_directory_has_one_owner takes its words from
                    CRATE_PARENTS_HISTORY + CRATE_PARENTS, so it follows a move
bbce22a1  step 1    git mv zero/rust-tools zero/tools -- INCOMPLETE ALONE: only
                    the renames and six moved .rs files (zero-gate re-staged
                    them); the git add named zero/rust-tools, gone after the
                    move, so git refused it. Does not build by itself.
6df319fd  step 1    completed: the 20 files bbce22a1 left out                     ratchet 22 -> 20
```

State at 6df319fd: ship 18 shipped 0 failed; nsh-test 215/215 deployed; d 0 failed; Path
Resilience 18/18; Fingerprint PASS digest a56683c54812378b, unchanged across the move; Integrity
67% -- the four pending drift proposals (pending_fixes 9-12), expected until the drift gate.

### The owner now -- zero_core::paths (zero/tools/zero-core/src/paths.rs)

```
CRATE_PARENTS          ["zero/tools"]
SINGLE_CRATES          ["zero/engine"]
CRATE_PARENTS_HISTORY  ["rust-tools", "zero/rust-tools", "zero/tools"]  oldest first, never shrinks
tools_dir()            renamed from rust_tools_dir() in step 1 = core_dir().join(CRATE_PARENTS[0])
crate_dirs_in / crate_dirs, crate_rel_dir_in / crate_rel_dir, crate_of, crate_history_pathspecs
crate_parent_markers() last segment of each parent + "/" -- "tools/" now; the substring guards'
                       marker (rename, copy, move, write, prompt zone, zero-context, intent)
```

Step 2: "zero/shell" joins CRATE_PARENTS and is appended to CRATE_PARENTS_HISTORY; the workspace
members gain "zero/shell/*". crate_dirs_match_the_workspace keeps the two in step.

### The ratchet -- 20 entries, all L3c

zero/tools/nsh-test/crate-paths-allowed.txt: 19 nsh-test fixtures and tilde strings (they build a
fake tree and run `ls ~/0-core/zero/tools/novashell/...` through nsh) and teach's "zero/tools/".
The guard flags a live literal containing any CRATE_PARENTS_HISTORY or CRATE_PARENTS entry. In
step 2 the novashell fixtures become zero/shell/novashell/... and their entries change with them.

### Rulings in force -- do not re-ask

L1, L2, L3, L3a, L3b, L3c and R5 (Design addendum above), plus, 2026-10-02 night:
- N1 (Christian) No new intents: what INT-267 uncovers is fixed inside it, as a gate if needed.
- N2 (Christian) REGISTRY DRIFT RESOLVED is its own commit, after step 2, before close.
- N3 History stays history: absorbed-from and ported-from comments, the intent ledger and
  CHANGELOGs keep old paths. Docs that say where code IS follow the move -- including
  docs/history-inventory.md, whose paths every_backticked_repo_path_in_the_docs_exists guards.
- N4 crate_parent_markers is the last path segment: the guards err on protecting more.
- N5 (Christian) Claude words the AGENTS.md path lines; they go in the move commit, shown in the plan.
- N6 cd completion offers the crate dirs from the owner plus zero/intents (all three dead ones fixed).

### Next -- step 2: THE BRANCH, its own chat

1. RECON. Re-run the census: `python3 ~/.cache/zero/int267-census-3de13dda82b5.py` (last run
   89f59ab17730 at 68623b38, stale now). Paste its tail and `cat ~/.cache/zero/int267-census.txt`.
   Step 2's lines are the ones naming novashell, nsh-test or devbox.
2. HOME SWEEP for novashell, nsh-test and devbox outside the repo (Hyprland, .desktop, systemd
   user units, ~/.local/bin, rc files, ~/.config/nsh) -- step 1's sweep found none for rust-tools.
3. READ, do not ask: devbox is repo-root devbox/ (docs/TREE.md:23 "read by zero-sandbox verify";
   docs/inventory.md:15 runs `zero-sandbox verify devbox/census`). It is not a crate: find how
   zero-sandbox locates it before moving it to zero/shell/devbox.
4. DECIDE with Christian before building: crate_parent_markers would gain "shell/", a broad
   substring for the rename/copy/move/write guards (any path containing shell/ becomes protected).
   Options: accept (errs safe), or markers become the full parent path for "zero/shell".
5. PLAN with the step 1 planner pattern: one planner computes every rewrite on his machine from the
   files as they are, rustfmt-checks changed .rs, prints each changed line, binds the apply to a
   FINGERPRINT; apply does git mv (zero/tools/novashell, zero/tools/nsh-test -> zero/shell/;
   devbox -> zero/shell/devbox) then whole-file writes verified on disk. Owner: CRATE_PARENTS
   ["zero/tools", "zero/shell"], HISTORY gains "zero/shell". Cargo members gain "zero/shell/*".
   Watch: deadwood fixtures and fallback use CRATE_PARENTS[0].join("novashell/src") -- after step 2
   novashell lives under the SECOND parent; fixtures still agree with each other, but move them to
   the shell parent so they say what is true.
6. Regenerate docs/history-inventory.md with its own script after the move -- READ its CLI first.
7. DOORS: cargo build --workspace; cargo test -p zero-core crate; nsh-test all; ship 0 failed; d 0
   failed with Fingerprint digest a56683c54812378b; home sweep 0; census shows only history + L3c.
8. Then the drift gate (apply pending_fixes 9-12 with `core integrity apply <id>`, Integrity back
   to 100%, tools.toml changes only those four version lines, own commit). Then close: tick each
   gate with its evidence, cicomplete 267.

### Gates -- where each stands (tick only when the whole gate is demonstrated)

- THE ORDER: the move landed after 263/264 closed (3de890ff) -- read the full gate, then tick.
- THE MOVE ("ONE commit per branch"): step 1 landed in TWO commits (bbce22a1 + 6df319fd) because of
  the refused git add. Christian rules whether that meets the gate or is recorded as a deviation.
- THE DOORS, NOTHING OUTSIDE THE REPO BROKE, FINGERPRINT UNCHANGED, AGENTS.md: met for step 1
  (doors above; sweep 0; digest unchanged; AGENTS.md:62 in 6df319fd); open until step 2 meets them.
- HISTORY UNTOUCHED: met for step 1 (N3). STATE UNTOUCHED: Zero Alias probe not yet read -- read
  it before and after step 2.
- ONE OWNER: every live locating site asks the owner (step 0); the ratchet holds only L3c. Read
  the gate's own words for whether L3c entries count before ticking.
- SAFETY LISTS MOVE WITH THE TREE: proven by every_delete_guard_dir_exists (288fb6d4) and the
  markers; open until step 2's move commit keeps it green.
- SIMPLE, measured: after step 2.
- REGISTRY DRIFT RESOLVED: after step 2 (N2).

### Method -- binding (the evening section's method, plus what tonight taught)

- MOVE COMMITS: never name a pathspec the move removed. `git add` only paths that exist, then show
  `git diff --cached --name-status -M` counts AND `git status --short` must list nothing unstaged
  BEFORE committing -- add and commit in separate blocks for move commits. zero-gate's rustfmt
  re-stages staged .rs files at commit, and nothing else.
- Read a tool's flag parsing before calling it: `zero-docs readme-index --dry` wrote (the flag is
  --dry-run). Builtin doors (bump-versions, cd completion) run only after exec of the new nsh.
- fpatch refuses non-ASCII anchors and counts substrings: widen anchors with neighbour lines.
- Bulk move edits: whole-file writes bound to the plan's FINGERPRINT, each verified on disk.
- Replies stay within 2-3 minutes: one step per reply, rehearse only what that step needs.

### Found, not fixed -- recorded 2026-10-02 night

- anomaly: the classifier reads only the subject, so commits carrying an Intent trailer are flagged
  (11 Low); its registry/ pathspec is dead the same way (anomaly mod.rs near 260).
- integrity: the figure is 100 minus a third for any set of unresolved issues, whatever their
  weight. pending_fixes 7 (intent 308 placement) and 8 (keyscan in niri autostart, NixOS era) are
  stale proposals from before INT-267.
- deadwood: the command-word check walks nothing, silently, if novashell is absent.
- teach: the "Inside ~/0-core/" tour lists INTENT/, 01-configs/ and root scripts/, none of which
  exist, and "43 custom Rust tools".
- zero-docs readme-index ignores an unknown flag and writes instead of refusing.
- paths::scripts_dir() names ~/0-core/scripts; the scripts are at zero/scripts.
- CRATE_PARENTS_HISTORY lacks the era before 7b79c725 (a name the retired-name guard forbids).
- Friday R2/R4: reasoning.rs:201 and planning.rs:899 read a missing table as empty -- for INT-039.

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
- [ ] REGISTRY DRIFT RESOLVED (added 2026-10-02 by Christian; completed after the original gates):
      the four drift proposals the woken check recorded (pending_fixes 9-12: core 3.1.0 to 4.1.2,
      db-browse 1.0.0 to 2.1.0, friday-chat 1.0.0 to 2.1.0, zero-git 4.4.1 to 5.1.0) are applied
      through core integrity apply, d reads Integrity 100 percent, and the only change in
      zero/registry/tools.toml is those four version lines, in a commit of its own
- [x] COMMAND COLOUR TELLS THE TRUTH (added 2026-10-03 by Christian; done after THE BRANCH, before
      REGISTRY DRIFT RESOLVED): every builtin the dispatcher runs is coloured as native, a program
      on PATH or an alias as valid, and only what will not run as red. One list of builtins in
      novashell commands, checked against the dispatcher's arms by a test both ways; the
      highlighter, the cheatsheet and completion read it; external programs are answered by
      zero_core::paths::on_path, not by a hand list. Found 2026-10-03: tools runs and reads red,
      because NATIVE_COMMANDS and is_known_command's BUILTINS (completion.rs 799, 853) are kept
      by hand
<!-- evidence: 7c765757 commit A: commands::builtin_names::BUILTINS (211 names: the dispatcher arms, the REPL catches, the job-control words), kept equal to those sources by a test both ways; the highlighter asks it, then zero_core::paths::on_path, then shell_aliases. 4e52ee42 commit B: completion and the cheatsheet read it, COMMANDS removed; hist, flow, fs and intent left first-word completion by ruling 2026-10-03. Read at 4e52ee42: is_native_command, is_known_command, is_known_alias and command_class keep no word list; the only list left is DANGEROUS, 14 words, every one runs. Seen 2026-10-03: intent reads red; Tab on bump offers bump-versions; cheatsheet 147 builtins (was 105). Doors: build 0 warnings, novashell 231 passed, nsh-test 215 of 215, ship 0 failed, d 0 failed, Fingerprint a56683c54812378b. -->

## Relationship

- INT-247 -- origin (Christian's proposal, 2026-09-24) and order (ruling 2026-09-28). Its mental
  model is this intent's shape. The move waits on its code-side passes; its docs rewrite waits on
  the move
- INT-252 -- the move method: census, fingerprint, one commit, rehearsal, rollback
- INT-167 -- DevBox, which the proposal groups with the shell it tests
