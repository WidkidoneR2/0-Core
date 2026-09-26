---
id: 252
date: 2026-09-19
type: future
title: "the source tree still spells faelight in 162 live places -- consolidate the callers onto paths.rs, then rename the directory to zero"
status: in-progress
tags: [Zero, novashell, fixtures, caller, faelight, paths]
---

## Vision

The source tree is called `zero/`, every live path is asked for rather than typed, and the word
`faelight` survives only where it is history: completed intents, and crate names not yet rewritten.

## The Problem

⚠️ THE RENAME IS THE SMALL PART. THE CALLERS ARE THE WORK.

Measured 2026-09-19, occurrences of the path string `faelight/` across .rs .toml .py .md .sh:

```text
    172   intents (historical)    65 files   DO NOT TOUCH
    144   code (.rs)              25 files
     51   docs                    32 files
     11   scripts                  5 files
      7   manifests                3 files
    ---
    385   total, of which 213 are LIVE
```

The live code concentrates:

```text
     43   nsh-test/src/main.rs              fixtures BUILD the path
     21   novashell/src/commands/mod.rs
     13   faelight-deadwood/src/main.rs
     12   faelight-daemon/src/dbus.rs
      9   faelight-docs/src/main.rs
      9   faelight-docs/src/toolgen.rs
      6   engine/src/domains/intent/mod.rs
      6   faelight-core/src/paths.rs        <- ONLY SIX
```

⭐ `paths.rs` HAS SIX. That is INT-115 and INT-247 Layer 3a paying off: the crate that OWNS the
paths barely spells them. The other 138 are callers that still type what paths.rs already knows.

★ SO THE ORDER IS NOT "rename, then fix what broke". It is: make the callers ask, and the rename
becomes a six-line change to one file. Any other order means editing 138 strings by hand and
hoping, which is how a tree half-moves and the other half stops compiling.

## The Solution

```text
    1  CALLERS       every live site asks paths.rs instead of typing faelight/
                     This is INT-247's own work, finished rather than restarted.
    2  THE MOVE      git mv faelight zero, in ONE commit with the manifest edits
    3  FIXTURES      nsh-test's 43, which are a different kind: they CONSTRUCT a tree
```

### ⚠️ THREE THINGS THAT WILL BITE, NAMED BEFORE THEY DO

**The state directories stay out of this entirely.** `~/.local/state/faelight` is INT-247 Layer
3b, whose flip is 2026-09-24 and whose aliases are being watched daily by the Zero Alias doctor
check. Moving the source tree and the state tree in the same week means a failure cannot be
attributed to either. SOURCE ONE WEEK. STATE THE NEXT.

**Historical intents are not rewritten.** 172 occurrences live in completed intent files. They
record what was true when written. Rewriting them is busywork that destroys the record, and the
FORWARD-ONLY rule in INT-158 already says so.

**`nsh-test` asserts the old name exists.** At main.rs:881 there is a case whose own comment
explains it: *"novashell alone does not contain the string faelight, so grep would count zero"* --
it runs `ls ~/0-core/faelight/rust-tools | grep faelight | wc -l` and requires the answer to be
greater than zero. ⭐ THAT TEST PROTECTS THE PROTOTYPE LABEL. It must be rewritten to assert what
it actually cares about, not to require the old word.

### Naming, decided

`zero/`, NOT `0/`. Measured: a Cargo package named `0-*` is refused and an env var `0_FOO` is
refused, so the ecosystem has already ruled on the digit. A directory named `0/` is legal and a
bad idea -- `ls` shows something that looks like a typo, every relative path reads
`0/rust-tools/novashell`, and grep for `0/` returns noise.

ONE PARENT RENAME. Not 29 crate directories. `novashell` keeps its name; `faelight-core` keeps
its name until that crate is actually rewritten, because A RENAME IS NOT A REWRITE.

## 2026-09-25 -- THE MOVE LANDED, AND THE RULINGS

```text
    7b79c725   git mv faelight zero, 133 path strings in 28 files, ONE commit, pushed.
               ls ~/0-core shows zero; there is no faelight/ at the root
```

### THE ORDER CHANGED, RULED BY CHRISTIAN

The Solution above says callers first, then the move, because editing 138 strings BY HAND is how
a tree half-moves. The move was not done by hand. One payload took a census of every live path
string, refused unless it matched the reviewed fingerprint (12cb33529e87), did the git mv, and
rewrote every site through fpatch, checking each file against the planned text. Rehearsed on the
pushed tree first; the result was byte-identical to an independent rewrite, and the rollback
(git checkout, git mv back) was rehearsed too. The compiler and nsh-test were the gate.

NOT rewritten, on purpose: markdown, every intent, CHANGELOGs, the D-Bus names (INT-264) and the
dead /etc/faelight reads (INT-250).

### THREE TESTS PROTECTED THE OLD NAME -- RED ON THE RENAMED TREE

```text
    tilde_ls_root     ls ~/0-core                  required "faelight"
    tilde_pipe_grep   ls ~/0-core | grep faelight  required "faelight"
    pipe_ls_grep      ls ~/0-core | grep faelight  required "faelight"
```

199/202 on the moved tree, then 202/202 once they expect zero. Same commit.

### RULINGS, Christian 2026-09-25

```text
    the callers gate      rewritten to what was done, with evidence
    the audit gate        LEFT OPEN -- classified by file kind, not by comment
    the main.rs:881 gate  STAYS HERE. It checks ls zero/rust-tools | grep faelight, so it goes
                          red on the first crate rename -- that red is its proof
    completion            INT-252 is NOT completed until the entire flip is: no live faelight or
                          forest anywhere in the code. For completion this supersedes Not in scope
    the crates            faelight-insightd and faelight-context are KEPT, renamed zero-*
```

### Still open

```text
    AGENTS.md    names faelight/scripts/dev/fpatch.py; the file is zero/scripts/dev/fpatch.py
    markdown     the docs still say faelight/ -- the docs pass, last
    comments     a few say faelight/ as a word, not a path ("directories under faelight/")
```

## 2026-09-25, NIGHT -- CORRECTION: THE GREP-FAELIGHT CASE READS A FIXTURE

The rulings above say the main.rs:881 gate "goes red on the first crate rename". Read at
0d3be1b8, before the crate pass, that is not what the case does.

```text
    main.rs:212-216   the FIXTURE: mkdir zero/rust-tools/faelight-core and novashell/src
                      inside a temporary HOME
    main.rs:885-897   tilde_nested_pipe: ls ~/0-core/zero/rust-tools | grep faelight | wc -l
                      with HOME set to that fixture; passes while the count is > 0
```

It never reads the real tree. Renaming real crates cannot turn it red; six have been renamed and
it is green. It goes red when the fixture line naming faelight-core changes -- and the crate
script renames that line in faelight-core's own commit, the LAST of the pass. So the gate
closes at the end of the crate pass, not the start, and its red will come from the fixture, which
is the part of the case that encodes the old name.

### Done since the move

```text
    70d31161   AGENTS.md names zero/scripts/dev for fpatch -- the Still open line above
    33857379 .. a7a6376a   six crates renamed faelight-* -> zero-*; see INT-247
```

AGENTS.md:743 still describes the pre-move layout (the scripts "TODAY" and "INT-252 renames
that"); it goes with the docs pass.

## 2026-09-25, MORNING -- 14 OF 15 CRATES RENAMED; THE FIXTURE GATE WAITS FOR CORE

```text
    02e29c06 .. e09dcaf2   update, sandbox, release, docs, daemon, doctor, zone and git
                           renamed faelight-* -> zero-*, one crate per commit (INT-247)
```

The grep-faelight case (tilde_nested_pipe, main.rs:885-897) is still green, as the correction
above predicted: it reads the fixture at main.rs:212-216, not the real tree, and the fixture
names faelight-core. faelight-core is the one crate left, and its rename changes that fixture
line -- that commit is where this gate goes red and is rewritten to assert what it means.

Ruled by Christian 2026-09-25: the crate pass renames and rebrands only until INT-247 and this
intent are closed. What it found is filed as INT-265.

## 2026-09-26 -- THE FIXTURE GATE IS PROVEN; THE CRATE PASS IS DONE

```text
    c4634250   faelight-core -> zero-core, the fifteenth and last crate (INT-247)
```

The fixture at main.rs:215 now creates zero-core, and tilde_nested_pipe went red on the renamed
tree -- 201/202, expected >0 got 0, stored in state.db -- as the correction above predicted. In
the same commit it was rewritten to assert what it means: a tilde path through a nested pipe,
grep novashell, an exact count of 1, no brand name. 202/202. The gate "The nsh-test case at
main.rs:881" below is ticked with that evidence.

Open: the first gate, "THE AUDIT IS REGENERATED AND CLASSIFIED before anything moves", cannot be
ticked as written -- the move ran in 7b79c725 under the fingerprinted census 12cb33529e87. Christian
rules: rewrite it to name that census, or defer it. Asked 2026-09-26.

This intent still closes only when the entire flip is done -- no live faelight or forest in the
code (Christian, 2026-09-25). INT-247's START HERE holds the order.

## Success Criteria

- [ ] THE AUDIT IS REGENERATED AND CLASSIFIED before anything moves: every occurrence of
      `faelight/` in live code, manifests, scripts and docs, each marked live / historical /
      comment. The numbers above are from 2026-09-19 and will have drifted.
- [x] THE CALLERS -- RULED 2026-09-25: the order changed. Instead of converting the callers one
      crate at a time, the move ran as ONE scripted commit: a census of every live `faelight/`
      path string, reviewed and fingerprinted, rewritten through fpatch in the same commit as
      the git mv. Rehearsed on the pushed tree first; byte-identical to an independent rewrite.
      <!-- evidence: 7b79c725. Census 12cb33529e87: 133 sites in 28 files, then 'repo-path sites left in live files: 0'. Gate rewritten by Christian's ruling 2026-09-25. -->
- [x] `git mv faelight zero` lands in ONE commit together with the workspace members, RISK.toml
      and every path edit. Two commits is how half the tree moves and the other half compiles
      against a folder that is gone.
      <!-- evidence: 7b79c725: 652 paths, every file a rename; Cargo.toml members, zero/RISK.toml, .gitignore and all 133 path strings in the one commit. -->
- [x] `cargo check --workspace` is clean and `nsh-test` is green on a tree containing ZERO live
      `faelight/` path strings. Historical intents excluded, and the exclusion is stated.
      <!-- evidence: 2026-09-25, on the moved tree before the commit: faelight-core 13, novashell 221, core 2 passed; ship 21 shipped 0 failed; nsh-test 202/202; 0 repo-path sites left. Excluded and stated in the commit: markdown, every intent, CHANGELOGs. The D-Bus /org/faelight names and the dead /etc/faelight reads are not repo paths. -->
- [x] The nsh-test case at main.rs:881 asserts what it means instead of requiring the old word.
      **Proven by watching it fail first:** it must go red on the renamed tree before it is
      rewritten, or it was never testing what its comment claims.
      <!-- evidence: c4634250, 2026-09-26. The rename changed the fixture (main.rs:215 now creates zero-core); tilde_nested_pipe went red on the renamed tree, 201/202 expected >0 got 0, stored in state.db; rewritten to grep novashell with an exact count of 1 and no brand name: 202/202. Reviewed plans b218e73b9485 (rename) and e669c6b347ed (test). -->
- [x] BOTH DOORS RUN AFTER THE MOVE: `nsh -c`, a PTY session, `core doctor`, and `history`.
      ⚠️ IF ANY OF THOSE LOOKS EMPTY, REVERT THE COMMIT. Do not fix forward on state.
      <!-- evidence: 2026-09-25 after 7b79c725: nsh-test 202/202 covers nsh -c and the PTY; d 0 failed; history returned rows; nsh -c 'ls ~/0-core' listed zero; 27 shell_history rows written after the move commit, read through ~/.local/state/zero. -->
- [x] The state directories were NOT touched by this intent, and `~/.local/state/faelight` is
      exactly where it was. Demonstrated, not assumed.
      <!-- evidence: d after the move, before and after the commit: Zero Alias 'both names resolve to one directory -- real: state=zero, config=zero', unchanged since Layer 3b. The move touched tracked files under ~/0-core only. -->
- [x] Historical intents still say `faelight/` and that is recorded as correct, not as debt.
      <!-- evidence: 7b79c725 renames every intent at 100% similarity; the move excluded zero/intents/ by rule. Correct, not debt: INT-247 rule 1, history is never rewritten. -->

## Not in scope

`faelight-git` -> `zero-git`. ⚠️ THAT CRATE MAY NOT SURVIVE: Omarchy ships lazygit, and P2's
desktop reckoning has not ruled on whether the first-party tool is kept. A RENAME OF A CRATE YOU
ARE ABOUT TO DELETE IS CEREMONY. When P2 rules keep, the rename is a day's work -- crate
directory, `[package] name`, `[[bin]] name`, the `fg`-family aliases to `zg`, and the two places
that exec it by name (NovaShell and the core git domain).

Renaming any other crate. Rewriting `faelight-core`. Touching state. Any of INT-247's remaining
layers.

## Relationship

Continues INT-247. Depends on Layer 3b being FLIPPED AND SETTLED first -- not merely started.
