---
id: 275
date: 2026-10-05
type: future
title: "Fix core integrity"
status: complete
tags: [core, integrity]
depends_on: []
---

## The Problem

`core integrity fix` shows proposals that are not true, and `core integrity apply` would carry out some of them wrongly. Measured 2026-10-05, read-only, from state.db `pending_fixes` and engine/src/domains/integrity/mod.rs:

    id  check                    shown as                                          created      applying it would
    7   intent_status_directory  308 in-progress but lives in future/              2026-05-25   move nothing it names (see 3)
    8   autostart_retired_tool   keyscan still in niri autostart                   2026-05-26   a check that no longer exists
    17  registry_version_drift   core registry=4.1.3 cargo=4.1.4                   2026-10-03   write core 4.1.4 over 4.1.5
    19  registry_version_drift   core registry=4.1.5 cargo=4.2.0                   2026-10-05   write a stale 4.2.0
    20  registry_version_drift   core registry=4.1.5 cargo=4.3.0                   2026-10-05   the one true row

Four causes, each read in the code:

1. Proposals are stored conclusions, never re-derived. `fix` lists every row with `applied_at IS NULL` (mod.rs:594) without re-running any check, and without asking whether the check still exists. Row 8's check, autostart_retired_tool, is in no source file (fsearch, no results); this machine has no niri config, no niri and no keyscan. Decision 142: store facts, derive conclusions. This stores the conclusion.
2. The dedupe key is wrong. persist_proposal counts a row as already pending when check_name AND description match (mod.rs:293-300). The description carries the version, so every cargo bump adds a row and none is retired: 17, 19 and 20 are one drift. Each drift row carries its version as data (mod.rs:286-292, 1356-1370), so applying 17 or 19 writes an old version over a newer one.
3. The intent_status_directory apply does not do what its proposal says. The check reads the frontmatter status line exactly (mod.rs:700-710) and proposes a MoveFile with from and to. The apply arm (mod.rs:1331-1353) ignores the row it was given, scans future/ for any file whose text CONTAINS "status: complete", and moves each match to complete/. Applying 7 would not move INT-308; a planned intent that merely mentions the phrase in its body would be moved. Two owners of "what is this intent's status", and two path owners (ctx.fpath against zero_core::paths::intents_dir).
4. Only drift keeps its fix as data. Every other proposal stores its description as action_data (mod.rs:286-292), so apply has nothing to execute but a re-scan.

`core integrity` is a consumer of facts other tools own (the ledger, the registry, Cargo.toml). A proposal that outlives the fact that produced it is the same disease INT-274 found in %N: a display taken for the thing itself.

## The Shape

A proposal is shown, counted and applied only while a fresh run of its check still finds it.

- Identity is (check_name, subject): the tool for drift, the file for a move. One pending row per identity. A new finding for the same identity supersedes the old row; it does not add one.
- After every run, a pending row is retired when its check ran and no longer reports its subject (resolved), when a newer row has the same identity (superseded), or when its check is not in build_check_suite() (check removed). Retired is not applied: the row stays as history with its reason.
- `fix` lists what a fresh run finds, not what the table remembers.
- `apply <id>` re-runs that row's check first. A row that is no longer found refuses in one line, non-zero, and writes nothing. A row that is found is carried out from its stored data (from and to for a move, tool and version for drift), never by a re-scan.
- One function reads an intent's status, used by the check and the apply alike.

The schema question is G1, ruled before code: retired rows need a place to say so (proposed: two additive columns, retired_at and retired_reason, on pending_fixes). That is a change to a persistent format, which AGENTS section 0 says to stop and ask about.

## Rulings (G1, 2026-10-05)

Approved by Christian 2026-10-05, before any code.

(a) pending_fixes gains three nullable columns: subject, retired_at, retired_reason. ensure_tables adds each with ALTER TABLE, guarded by pragma table_info, so an existing table is extended and no row is rewritten. One backfill only: a drift row takes its subject from the tool in its action_data, never from its description. A proposal subject comes from its fix: from for a move, tool for drift, path for a file update. A move stores from and to as its action_data. Older binaries name their columns, so a checkout and ship back still runs; it would list retired rows again.

(b) run_pipeline ends with one reconcile over pending rows, first match wins: check not in build_check_suite() -> retired, check removed; no subject -> retired, pre-275 row; a newer pending row with the same (check_name, subject) -> retired, superseded by #N; its check ran in this pipeline and did not report that subject -> retired, resolved. Checks that did not run in a partial pipeline are left alone. fix, apply and heal run the same pipeline quick_scan runs, then read only rows still pending: applied_at and retired_at both NULL.

(c) INT-308 has no file. Completed 97f2191a (2026-05-25), moved to arch-era/ 445ba3a8 (2026-06-02), deleted with the archive c847e8b7 (2026-06-08). Nothing is moved or corrected. Row 7 retires as pre-275 row; G9 closes on that, with git status showing no intent file moved.

(d) heal reads only rows still pending after a run, carries each out through the same refusing apply, and counts a refusal as not healed instead of ending the loop.

The doctor is not changed. Its pending count is quick_scan proposed (doctor/mod.rs:351, 434), already a fresh run; it said 1 while fix listed 5 because fix reads the table (mod.rs:594). G10 is read on the deployed build.

## Success Criteria

- [x] G1 Rulings before code, written into this file: (a) how a retired row is recorded in pending_fixes (proposed: additive columns retired_at, retired_reason; existing rows untouched until a run retires them); (b) whether fix runs the suite itself or reads rows a run just reconciled; (c) what INT-308 should be (in-progress and moved, or its status corrected), since the tool must not guess it.
<!-- evidence: rulings (a) to (d) approved by Christian 2026-10-05, written under ## Rulings (G1, 2026-10-05) in this file. -->
- [x] G2 Baseline captured before code: `core integrity fix` verbatim (7, 8, 17, 19, 20) and the read-only pending_fixes dump, with created and applied times. The table above is that dump; G2 re-captures it after cistart so the before is on the record of this intent.
<!-- evidence: demonstrated 2026-10-05 after cistart 275 (checkpoint bb134bea). core integrity fix listed 7, 8, 17, 19, 20. pending_fixes read with mode=ro: 20 rows; pending 7 (created 1779715146), 8 (1779772448), 17 (1791070087), 19 (1791159791), 20 (1791221063); MoveFile rows store the description as action_data; drift rows store tool<TAB>version. d said 1 pending (quick_scan). -->
- [x] G3 Ghosts retire. A pending row whose check is not in build_check_suite() is never listed by fix, refuses apply, and is retired with reason check removed. Red first in a cargo test against a temp database; then row 8 is absent from the deployed fix and present in the table as retired.
<!-- evidence: demonstrated 2026-10-05. Red first: cargo test a_row_of_a_removed_check_is_not_pending FAILED on the old persist (/tmp/int275-red.txt) and passes after; applying_a_row_of_a_removed_check_refuses passes. Deployed core (mtime 2026-10-05 17:20:16): row 8 absent from core integrity fix, retired_reason check removed in pending_fixes; core integrity apply 8 printed Nothing applied: #8 was retired: check removed, exit 1. -->
- [x] G4 One row per identity. Two runs with the cargo version moved between them leave exactly one pending drift row for that tool, carrying the newer version; the older row is retired superseded. Red first: on today's persist_proposal the same test leaves two pending rows. On the real database, 17 and 19 end retired superseded and one core row is pending.
<!-- evidence: demonstrated 2026-10-05. Red first: one_pending_row_per_identity FAILED with two pending rows (alpha 2.0.0, alpha 2.1.0) and passes after. Real database on the deployed build: rows 17 and 19 retired superseded by #20, subject core backfilled from action_data; #20 was the one pending core row. -->
- [x] G5 Resolved findings retire. A pending row whose check ran and no longer reports its subject is retired resolved. Fixture: make a drift true, run, correct the registry by hand, run; the row is retired, not left pending and not marked applied.
<!-- evidence: demonstrated 2026-10-05. Red first: a_resolved_drift_is_not_pending FAILED (a drift corrected by hand is still pending) and passes after: the row is retired resolved and applied_at stays NULL. -->
- [x] G6 Apply refuses what is no longer true. apply on a retired or no-longer-found row prints one line naming why, exits non-zero, and the registry and the ledger are byte-identical before and after (cmp). Red first: on today's code, applying a stale drift row writes the old version into a temp registry.
<!-- evidence: demonstrated 2026-10-05. Red first: applying_a_stale_drift_refuses_and_writes_nothing FAILED (refused Some(false), the old apply carried the stale version out) and passes after. Deployed: apply 7, 8, 17 and 19 each printed one Nothing applied line naming the reason and exited 1; sha256 of registry/tools.toml (8477f60e7b7641f7) and of every file under zero/intents (9613197551d28bff) identical before and after. -->
- [x] G7 Apply does what the proposal says. A move is carried out from the from and to stored on that row, and only that file moves. Red first: a fixture future/ intent with status planned whose body contains the text "status: complete" is moved by today's apply arm and is not moved after the change. Status is read by one function, shared by the check and the apply, and the check and the apply use one path owner.
<!-- evidence: demonstrated 2026-10-05. Red on the deployed pre-change build: core integrity apply 900001 in a stand-in HOME moved a status: complete fixture whose body says status: complete, row naming another file; real ledger and state.db hashed unchanged. Green: a_move_moves_only_the_file_its_row_names passes (only the named file moves, the bystander stays). Status is read only by frontmatter_status, used by the check; the apply arm reads no status and carries out the from and to the check stored (ctx.fpath); apply no longer calls zero_core::paths::intents_dir. -->
- [x] G8 Drift applies true. After apply of the one pending core drift row, registry/tools.toml says the cargo version of the moment, the next run proposes nothing for core, and no other line of the registry changed (diff of the file by line count and cmp of the rest).
<!-- evidence: demonstrated 2026-10-05 on the deployed core: cargo version of core 4.3.0; core integrity apply 20 exit 0; registry/tools.toml line 8 version 4.1.5 -> 4.3.0, 500 lines before and after, every other line identical; the next core integrity fix printed No pending proposals. -->
- [x] G9 INT-308 settled by the G1 ruling, not by the tool: either the deployed apply moves exactly that one file (git status shows that rename and nothing else), or its status is corrected and the check stops reporting it. Either way the row ends applied or resolved, with evidence.
<!-- evidence: demonstrated 2026-10-05. Ruling (c): INT-308 has no file (completed 97f2191a, archived 445ba3a8, deleted c847e8b7). Row 7 retired pre-275 row on the deployed build; apply 7 refused, and the ledger hash was unchanged, so no intent file moved. -->
- [x] G10 The count tells the truth. The doctor's "N integrity proposal(s) pending" line (doctor/mod.rs:434) equals the number of rows fix lists, read on the deployed build. What the doctor's Integrity percentage measures is read before anything is claimed about it.
<!-- evidence: demonstrated 2026-10-05. The doctor count is quick_scan proposed (doctor/mod.rs:351, 434), already a fresh run. Deployed: d said 1 pending while fix listed only #20; after G8 fix lists none and d prints no pending line. Integrity percentage read before any claim: total_weight is 3x the found weight, so it reads 67 whenever anything is found and 100 otherwise (F-0012, filed, not fixed here); it went 67 -> 100 when #20 was applied. -->
- [x] G11 Doors. Engine cargo test passing; ship; the deployed core fix shows only proposals a fresh run finds (mtime of ~/.local/bin/core checked); nsh-test all passing; d 0 failed.
<!-- evidence: demonstrated 2026-10-05: engine cargo test 52/52 (/tmp/int275-engine.txt), integrity 10/10; ship shipped core 4.3.0, ~/.local/bin/core mtime 17:20:16 after HEAD bb134bea 12:38:54; the deployed fix shows only what a fresh run finds; nsh-test 215/215 (/tmp/int275-suite.txt); d 0 failed (2 warnings: the uncommitted changes). -->
- [x] G12 Sealed. The commit is made from a reviewed plan by the INT-266 method, carries Intent: INT-275 and Seal:, keeps its plan note, and `core intent seals -n 1` shows it sealed.
<!-- evidence: demonstrated 2026-10-05: commit 0e972772 made from the reviewed plan by the INT-266 method (Intent: INT-275, Finding: F-0011, Finding: F-0012, Seal: e445ae2c3b72933d); the plan text is kept as note blob 5d9ab178 under refs/notes/seals and its sha256 starts e445ae2c3b72933d; core intent seals -n 1 shows 0e972772 sealed; pushed, origin/main at 0e972772. -->

## START HERE (2026-10-05)

INT-275 is complete. The work is commit 0e972772 (sealed e445ae2c3b72933d, plan kept as a note), pushed. core integrity now shows, counts and applies a proposal only while a fresh run of its check still finds it; retired rows stay in pending_fixes with retired_at and retired_reason, and fix prints them as one history line.

Open findings filed by this intent, each its own intent when taken up: F-0011 (lock/mod.rs names swaylock via Niri), F-0012 (integrity_pct reads presence, not proportion), F-0013 (notify desktop lets the busctl reply into the doctor output). A new chat opens with: core intent trace INT-275.

Not touched, and possibly the same disease (a non-goal here): the evolution, friday_arch, self_transformation, partner and strategy proposal tables.

## Non-goals

- No new integrity checks, and none removed beyond retiring rows of checks already gone.
- No history deleted: retired rows stay, with their reason.
- The other proposal tables (evolution, friday_arch, self_transformation, partner, strategy) may have the same disease; each is its own intent if so. Not touched here.
- Auto-fix and alert paths are untouched, apart from sharing the status reader if G7 needs it.
- lock/mod.rs:12 still names swaylock "via Niri": NixOS-era, filed as a finding with core intent find, not fixed here.
- No change to Omarchy, and no edit to AGENTS.md.

## Dependencies

None. INT-274 is complete; its seal method is used at G12. depends_on stays empty.

## Order of work

cistart 275 -> G2 baseline -> G1 rulings -> recon (doctor count, run order, how fix and apply are dispatched) -> plan with fingerprint -> review -> apply -> red tests then green -> ship -> G3 to G10 on the deployed build -> doors -> sealed commit -> cicomplete 275.

## Versions
- engine 4.3.0 -> 4.3.1 (patch)
