---
id: 275
date: 2026-10-05
type: future
title: "Fix core integrity"
status: planned
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

## Success Criteria

- [ ] G1 Rulings before code, written into this file: (a) how a retired row is recorded in pending_fixes (proposed: additive columns retired_at, retired_reason; existing rows untouched until a run retires them); (b) whether fix runs the suite itself or reads rows a run just reconciled; (c) what INT-308 should be (in-progress and moved, or its status corrected), since the tool must not guess it.
- [ ] G2 Baseline captured before code: `core integrity fix` verbatim (7, 8, 17, 19, 20) and the read-only pending_fixes dump, with created and applied times. The table above is that dump; G2 re-captures it after cistart so the before is on the record of this intent.
- [ ] G3 Ghosts retire. A pending row whose check is not in build_check_suite() is never listed by fix, refuses apply, and is retired with reason check removed. Red first in a cargo test against a temp database; then row 8 is absent from the deployed fix and present in the table as retired.
- [ ] G4 One row per identity. Two runs with the cargo version moved between them leave exactly one pending drift row for that tool, carrying the newer version; the older row is retired superseded. Red first: on today's persist_proposal the same test leaves two pending rows. On the real database, 17 and 19 end retired superseded and one core row is pending.
- [ ] G5 Resolved findings retire. A pending row whose check ran and no longer reports its subject is retired resolved. Fixture: make a drift true, run, correct the registry by hand, run; the row is retired, not left pending and not marked applied.
- [ ] G6 Apply refuses what is no longer true. apply on a retired or no-longer-found row prints one line naming why, exits non-zero, and the registry and the ledger are byte-identical before and after (cmp). Red first: on today's code, applying a stale drift row writes the old version into a temp registry.
- [ ] G7 Apply does what the proposal says. A move is carried out from the from and to stored on that row, and only that file moves. Red first: a fixture future/ intent with status planned whose body contains the text "status: complete" is moved by today's apply arm and is not moved after the change. Status is read by one function, shared by the check and the apply, and the check and the apply use one path owner.
- [ ] G8 Drift applies true. After apply of the one pending core drift row, registry/tools.toml says the cargo version of the moment, the next run proposes nothing for core, and no other line of the registry changed (diff of the file by line count and cmp of the rest).
- [ ] G9 INT-308 settled by the G1 ruling, not by the tool: either the deployed apply moves exactly that one file (git status shows that rename and nothing else), or its status is corrected and the check stops reporting it. Either way the row ends applied or resolved, with evidence.
- [ ] G10 The count tells the truth. The doctor's "N integrity proposal(s) pending" line (doctor/mod.rs:434) equals the number of rows fix lists, read on the deployed build. What the doctor's Integrity percentage measures is read before anything is claimed about it.
- [ ] G11 Doors. Engine cargo test passing; ship; the deployed core fix shows only proposals a fresh run finds (mtime of ~/.local/bin/core checked); nsh-test all passing; d 0 failed.
- [ ] G12 Sealed. The commit is made from a reviewed plan by the INT-266 method, carries Intent: INT-275 and Seal:, keeps its plan note, and `core intent seals -n 1` shows it sealed.

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
