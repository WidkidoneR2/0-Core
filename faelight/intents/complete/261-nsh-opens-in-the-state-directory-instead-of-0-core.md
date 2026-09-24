---
id: 261
date: 2026-09-24
type: future
title: "nsh opens in the state directory instead of ~/0-core"
status: complete
tags: [shell, nsh, startup]
---

## Vision

A new terminal opens in `~/0-core`. Always -- not wherever the previous session happened to exit.

## The Problem

nsh made TWO startup moves. First `cwd::chdir(core_root)` (main.rs:2334), which is the wanted
default. Then, after the banner, a restore of the remembered `last_dir` from `session_state`
(main.rs:3594), which won over the first for any path that existed and was not under `engine/src`
or `rust-tools/`.

Found 2026-09-24: the INT-247 flip session ended in `~/.local/state/zero`, so the next terminal
opened there. The stored row read `/home/christian/.local/state/faelight` -- the old name, reached
through the compatibility link.

It was NOT Omarchy. `omarchy-cmd-terminal-cwd` printed `/home/christian`, and Ghostty launched nsh
from there; nsh moved itself.

### AND THE GUARDIAN COULD NOT SEE IT

`repl_206_forest_home_is_still_the_default` passed with the bad row in place. nsh-test gives every
case a FRESH database (`FAELIGHT_STATE_DB`, repl.rs:397, INT-204), so its shell never had a
`last_dir` to restore. A green case that could not go red on this bug.

## The Solution

Retire "resume where I left off" entirely -- the restore AND the save, so nothing writes a value
nothing reads. One startup rule remains: a fresh session starts in the core root, and
`NSH_KEEP_CWD` (INT-206) still lets a caller that chose a directory keep it.

The stale `last_dir` row in state.db was deliberately LEFT. After the fix it is the proof: the row
still names another directory and the terminal opens in `~/0-core` anyway, so the reader is gone
rather than the data being convenient.

The new case PLANTS a `last_dir` in its own case database, so it proves the class -- the startup
directory must not depend on remembered state -- without ever opening the real ledger.

## Success Criteria

- [x] A case that plants a `last_dir` goes RED on the unfixed binary, naming the planted directory
<!-- evidence: 2026-09-24, nsh-test against deployed nsh 3.9.0 BEFORE the fix: 200 / 201,
the only failure repl_start_directory_ignores_remembered_last_dir -- "restored the remembered
last_dir instead of starting in 0-core: /tmp/nsh-test-14235/planted-last-dir" -->
- [x] The restore and the save are both gone: nothing reads or writes `last_dir`
<!-- evidence: 7c446407. main.rs restore block (was 3589-3607) deleted; session.rs field, load
query, struct line and save deleted; cargo check -p novashell -p nsh-test clean -->
- [x] The same case goes GREEN on the deployed fixed binary, with nothing else changing colour
<!-- evidence: ship (nsh 3.9.0, nsh-test 2.0.0), then nsh-test 201 / 201 passed -->
- [x] Live: a new terminal opens in `~/0-core` while the stale row still names another directory
<!-- demonstrated: after ship, state.db session_state last_dir = /home/christian/.local/state/faelight;
SUPER+RETURN opened a new terminal in ~/0-core -->
- [x] `NSH_KEEP_CWD` still honoured -- the harness still runs every case from its chosen directory
<!-- evidence: nsh-test 201 / 201, whose conformance cases write files relative to /tmp; `d` after
the commit reported "working tree clean, all commits pushed", so nothing landed in the repository -->
- [x] No state was written or deleted to make this work
<!-- demonstrated: the only state.db access was read-only (sqlite mode=ro); the stale row is still
there by design -->

## Relationship

- INT-247 -- found during its Layer 3b flip, and listed in its OPEN section as "startup cwd ... its
  own intent, not this one"
- INT-206 -- `NSH_KEEP_CWD`, unchanged and still the harness's switch
- INT-204 -- the fresh database per case, which is both why the guardian was blind and why the
  new case can plant a row safely


