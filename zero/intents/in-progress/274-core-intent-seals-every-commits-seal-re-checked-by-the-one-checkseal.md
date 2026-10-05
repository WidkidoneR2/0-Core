---
id: 274
date: 2026-10-05
type: future
title: "core intent seals: every commit's seal, re-checked by the one check_seal"
status: in-progress
tags: [seal, fingerprint, core, intent]
depends_on: []
---

## The Problem

INT-266 gave commits a Seal: trailer (16 hex, sha256 of the reviewed plan text) and keeps that plan text as a git note under refs/notes/seals. `core intent trace` re-checks one seal at a time: one commit, one finding, one intent. Nothing answers the question across a stretch of history: which recent commits are sealed, which seal is broken, which claim a seal with no plan text kept, and which predate the Seal rule.

The first idea was an nsh builtin, `log`, reading a new store at refs/notes/nsh-seal. Recon on 2026-10-05 ruled it out:

- The store already exists (refs/notes/seals). A second one breaks the one-owner rule.
- The checker already exists: check_seal, zero/engine/src/domains/intent/trace.rs:205, with five outcomes (SealCheck: Intact, Broken, NoNote, Legacy, NoSeal).
- novashell links zero-core and zero-git, not the engine. An nsh builtin could never call check_seal: it would print undetermined forever, or grow a second digest.
- The seal covers the plan text, not the tree. tree / head / witness columns would claim a proof nothing computes.

So the reader goes where the checker is: a walk mode beside trace, in core.

## The Shape

    core intent seals [-n N] [--intent INT-x]

    date        commit    intent   seal              verdict
    2026-10-04  05dbc617  INT-266  -                 unsealed
    2026-10-04  272d7b68  INT-266  384222b57603b29a  sealed
                <commit>  INT-nnn  <seal>            broken
                          plan text kept in refs/notes/seals hashes to <got>

One row per commit, newest first, default 15. An indented second line carries the reason for broken, no plan and legacy, worded as seal_line already words it.

| SealCheck | verdict  | colour | exit |
|-----------|----------|--------|------|
| Intact    | sealed   | green  | 0    |
| Broken    | broken   | red    | 1    |
| NoNote    | no plan  | yellow | 2    |
| Legacy    | legacy   | dim    | 0    |
| NoSeal    | unsealed | dim    | 0    |

Refusals exit 3 (ruled 2026-10-05, the zero-sandbox precedent): git cannot be read, the intent is not in the ledger, or -n is not a count. The domain calls exit itself, as core fingerprint show does, so a refusal never reads as broken. A clap usage error (an unknown flag) exits 2, the same as no plan; the help says so. Each row ends with the commit subject.

Exit is the worst row in the window: 1 beats 2 beats 0. Legacy stays 0 (ruled at filing, open to veto at plan review): it is shown and named, but a walk deep enough to reach pre-Seal commits would otherwise always exit 2, and the code would stop telling scripts anything. no plan is 2 because it is a current claim missing its evidence. unsealed is 0: completion commits carry Intent: and no plan, legitimately.

## Success Criteria

- [x] G1 Red first. After cistart, before any code: `core intent seals` captured verbatim as an unknown subcommand, with its exit code.
<!-- evidence: demonstrated 2026-10-05, after cistart 274 at 2ee77f49: core intent seals run through a python3 subprocess -> stderr "error: unrecognized subcommand 'seals'" (tip: status, search, health, stats), exit 2. -->
- [x] G2 One checker. Every verdict comes from check_seal. The change adds no digest, no sha256 call, and no second reader of refs/notes/seals outside what trace.rs already owns. Proof: fsearch for seal_digest and sha256 after the change names only the sites that existed before it.
<!-- evidence: demonstrated 2026-10-05: fsearch seal_digest names trace.rs 179 (the definition), 209 (check_seal), the INT-266 seal_tests, and three new fixture lines in seals_tests that build fixture seals. fsearch Sha256 names only seal_digest and the unrelated doctor entropy module. No verdict path computes a hash. -->
- [x] G3 Five verdicts, each watched. A fixture repo in mktemp produces all five: a correct note (sealed), an altered note (broken, watched red first), a Seal: trailer with no note (no plan), a Fingerprint: trailer only (legacy), neither trailer (unsealed). The real refs/notes/seals is never written. How core is pointed at the fixture is decided at recon; if core_root cannot be pointed elsewhere, the fixture runs as a cargo test against the walk function given a root path.
<!-- evidence: demonstrated 2026-10-05: cargo test seals_tests, the_walk_gives_all_five_verdicts_newest_first passes with sealed, broken, no plan, legacy, unsealed newest first in a mktemp repo; the real refs/notes/seals was never written. Red first, the broken assertion: the altered note was mutated to the correct plan text and the test FAILED, then restored byte for byte and passed. Red first, the class: a_note_git_would_display_differently_still_hashes_exactly FAILED on the %N walk (raw 1 read as Broken) and passed after the blob read. -->
- [x] G4 The real tree. On ~/0-core the deployed core shows 272d7b68 sealed and 05dbc617 unsealed, and with -n deep enough to reach the 2026-09-28 trailer commits, at least one Fingerprint-trailer commit shows legacy.
<!-- evidence: demonstrated 2026-10-05 on the deployed core: intent seals -n 80 shows 272d7b68 sealed, 05dbc617 unsealed, 62f9e40b legacy, 0 broken (17 sealed, 23 legacy, 40 unsealed before this commit). -->
- [x] G5 Filter. --intent 266 and --intent INT-266 print the same rows: only commits whose Intent: trailer names INT-266 (a comma-joined trailer counts if any value matches). An intent with no commits in the window says so in one line and exits 0.
<!-- evidence: demonstrated 2026-10-05: the intent option given 266 and given INT-266 byte-identical over -n 40, 20 rows, every row INT-266, exit 0; given 213 over 3 commits it prints one no-commit line, exit 0. -->
- [x] G6 Exit codes measured, not declared: 1 and 2 from the fixture rows in a cargo test (no real seal is broken to prove 1); 0 and 3 read back from the deployed core through python, 3 by -n x and by --intent 99999.
<!-- evidence: demonstrated 2026-10-05: 1 and 2 from the_exit_is_the_worst_row on fixture rows; deployed core intent seals -n 8 exit 0, -n x exit 3, intent 99999 exit 3, read back through python. -->
- [x] G7 Cost does not grow with N. The number of git processes spawned is the same for -n 5 and -n 30. Measured with a counting git wrapper first on PATH that execs the real git; the wrapper lives in mktemp and is removed.
<!-- evidence: demonstrated 2026-10-05: a counting git wrapper first on PATH in mktemp, removed after: 4 git processes at -n 5 and 4 at -n 30 (log, notes list, ls-tree, one show). The first walk spawned 1 but hashed git log %N, which alters note bytes; see G3. -->
- [x] G8 Refusals. Outside a readable repo, or with -n 0 or -n x, it refuses in one line naming what it could not do, non-zero, as every INT-266 reader does.
<!-- evidence: demonstrated 2026-10-05: -n 0, -n x, intent 99999 and intent abc each print one could-not-answer line and exit 3; an unreadable repo refuses in seals_tests::an_unreadable_repo_refuses. -->
- [x] G9 trace is unchanged. `core intent trace` for one commit, one finding and one seal is captured before the change and compared with cmp after.
<!-- evidence: demonstrated 2026-10-05: core intent trace 272d7b68, F-0010, 384222b57603b29a and INT-266 byte-identical, deployed build before against the new build, run again after run() became a wrapper over run_bytes(). -->
- [x] G10 Doors. cargo test for the engine crate passing; ship; the deployed core prints the table (mtime of ~/.local/bin/core checked against the commit); nsh-test all passing; d 0 failed.
<!-- evidence: demonstrated 2026-10-05: engine cargo test 45 passed; ship shipped core 4.2.0, deployed mtime 0 s old; nsh-test 215/215; d 0 failed, the two warnings are the uncommitted tree. -->
- [ ] G11 Self-proof. The commit that lands this is made from a reviewed plan, carries Intent: INT-274 and Seal:, and keeps its plan note. `core intent seals -n 1` run after that commit shows it sealed.

## Non-goals

- No nsh builtin and no word added to nsh. core is already on PATH.
- No writer. Seals are written by the existing plan flow; this only reads.
- No tree, head or witness column. No TUI. No machine fingerprint: core fingerprint show and the doctor own that.
- No change to check_seal, seal_digest or the note format.
- No edit to AGENTS.md. If a line there should name the reader, it is proposed, not written.

## Dependencies

None. INT-266 (the Seal: trailer, refs/notes/seals, check_seal) is complete. depends_on stays empty: nothing makes this impossible to start.

## Order of work

cistart 274 -> recon (how trace reads trailers and notes, how core maps a result to an exit code, whether core_root can point at a fixture) -> plan with fingerprint -> review -> apply -> cargo test and debug core -> ship -> doors -> commit with seal -> G11 -> cicomplete 274.
