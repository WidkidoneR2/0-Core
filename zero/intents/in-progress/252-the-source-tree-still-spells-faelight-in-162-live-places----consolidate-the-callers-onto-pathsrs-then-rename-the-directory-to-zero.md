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

## Success Criteria

- [ ] THE AUDIT IS REGENERATED AND CLASSIFIED before anything moves: every occurrence of
      `faelight/` in live code, manifests, scripts and docs, each marked live / historical /
      comment. The numbers above are from 2026-09-19 and will have drifted.
- [ ] ⭐ THE CALLERS ASK FIRST. Every live site resolves its path through `faelight_core::paths`
      rather than typing the prefix. Proven by the count: `faelight/` in live .rs falls from 144
      to what paths.rs itself contains, BEFORE the directory moves.
- [ ] `git mv faelight zero` lands in ONE commit together with the workspace members, RISK.toml
      and every path edit. Two commits is how half the tree moves and the other half compiles
      against a folder that is gone.
- [ ] `cargo check --workspace` is clean and `nsh-test` is green on a tree containing ZERO live
      `faelight/` path strings. Historical intents excluded, and the exclusion is stated.
- [ ] The nsh-test case at main.rs:881 asserts what it means instead of requiring the old word.
      **Proven by watching it fail first:** it must go red on the renamed tree before it is
      rewritten, or it was never testing what its comment claims.
- [ ] BOTH DOORS RUN AFTER THE MOVE: `nsh -c`, a PTY session, `core doctor`, and `history`.
      ⚠️ IF ANY OF THOSE LOOKS EMPTY, REVERT THE COMMIT. Do not fix forward on state.
- [ ] The state directories were NOT touched by this intent, and `~/.local/state/faelight` is
      exactly where it was. Demonstrated, not assumed.
- [ ] Historical intents still say `faelight/` and that is recorded as correct, not as debt.

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
