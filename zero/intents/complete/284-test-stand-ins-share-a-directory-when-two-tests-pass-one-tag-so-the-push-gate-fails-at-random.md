---
id: 284
date: 2026-10-08
type: future
title: "test stand-ins share a directory when two tests pass one tag, so the push gate fails at random"
status: complete
tags: [tests, integrity, flaky, push-gate, core, finding]
---

## START HERE

Complete 2026-10-08: G0 to G7 demonstrated, F-0026 fixed by e8679d19 and pushed. Nothing remains.

## Vision

A test's scratch tree belongs to that test alone. No test can build into, rewrite or delete
another test's tree, whatever tag it passes, so the push gate's verdict depends on the code and
never on thread timing.

## The Problem

Observed 2026-10-08 (F-0026). git push origin main passed the pre-push hook at 2bb11c7b. Forty
seconds later git push origin refs/notes/seals ran the same hook on the same code and failed:

    domains::integrity::tests::a_row_with_no_crate_is_reported_not_skipped

Run alone three times (cargo test -p core -- <that name>), it passed three times.

By reading zero/engine/src/domains/integrity/mod.rs:
- stand_in (:1822) names its root temp_dir/integrity-int267-{tag}-{pid} (:1824) and clears it
  first with remove_dir_all (:1825).
- Two tests pass the tag ghost: :1917 (INT-275) and :2115 (INT-283 G2).
- cargo runs a crate's tests as threads of one process, so both get one directory. The INT-283
  test appends a ghost row to that tree's registry (:2117-2119) and deletes the whole tree
  (:2121). The other test's clear-and-rebuild can rewrite tools.toml without the row, and the
  drift check then finds no ghost: alert -- the exact failure message.
- integrity-int283-real-{pid} (:2136) is named the same way.

The cause is the class, not the tag: any helper that names a directory from the process id and a
caller-chosen string can collide the same way, in this crate or another.

## The Solution

stand_in takes a per-call number from a static AtomicUsize, so every call gets its own directory
whatever tag it is given, and it still clears only that directory. G0 finds every other helper of
the same class and G3 rules on each one.

## Success Criteria

- [x] G0 census, read-only. Every test path in the workspace built from std::process::id() or a
      fixed name under temp_dir: file:line, the helper or test, its callers and their tags, and
      whether two callers can collide. Recorded here before any edit.
<!-- demonstrated 2026-10-08: census of temp_dir() in zero/ (code lines, outside target/), 32 sites.
IN-PROCESS (threads of one test binary): one site can collide -- integrity stand_in (mod.rs:1824),
whose 10 callers pass ghost twice (:1917, :2115). Every other helper takes distinct literal names
from every caller: zero-core fingerprint fake 10/10, paths scratch 3/3, paths tree 4/4, state_db
scratch 6/6, zero-deadwood fixture 5/5 and fixture_file 1/1, citation_fixture 2/2, zero-doctor
alias_scratch 7/7; trace.rs fixture also adds nanos.
CROSS-RUN ONLY (no process id, so two overlapping runs of one test binary share the path):
intent/findings.rs:231 core_finding_collision_test, findings.rs:339 core_finding_highest_test,
intent/mod.rs:3854 core_absent_ledger_test, zero-deadwood main.rs:1421 and :1438
deadwood_cmdword_{name}, :1550 deadwood_cite_{name}, :1605 deadwood_cite_noledger.
SAFE: every other site carries the process id with one caller or distinct names; guards.rs:78
is a planted path that is never created.
NOT A TEST: novashell main.rs:2104 joins the fixed name nsh-cwd.tmp in run_input (the yazi cwd
handoff) -- production code, outside this intent. -->
- [x] G1 red. Two new tests, both shown FAILING on the current code, output quoted:
      two_stand_ins_with_one_tag_never_share_a_directory (integrity/mod.rs), two stand_in calls
      with one tag must return different roots; and every_temp_dir_path_carries_the_process_id
      (zero-core paths.rs), a scan of zero/ naming every temp_dir() path without the process id,
      with two named exceptions that must still match a site (F-0027 nsh-cwd.tmp, ruled out of
      284; guards.rs int267-no-such-directory, planted absent). Ruled 2026-10-08: the seven
      cross-run sites are fixed inside 284, and the guard lives in zero-core (option a).
<!-- demonstrated 2026-10-08 on the code before the fix (red plan seal 8ab9ee40716f5eec):
two_stand_ins_with_one_tag_never_share_a_directory exit 101, left and right both
/tmp/integrity-int267-same-tag-32371; every_temp_dir_path_carries_the_process_id exit 101, naming
exactly the seven G0 cross-run sites (zero-deadwood main.rs:1421, :1438, :1550, :1605;
intent/findings.rs:231, :339; intent/mod.rs:3854) and no stale exception. -->
- [x] G2 fix. The counter in stand_in. G1 green, the ten existing stand_in callers unchanged,
      cargo test -p core integrity green.
<!-- demonstrated 2026-10-08 after the fix plan (seal 137fcb49e69f56b0): stand_in names its root
integrity-int267-{tag}-{pid}-{call}, the call number from a static AtomicUsize; the same-tag test
exit 0; the ten stand_in callers are unchanged; cargo test -p core 64 passed. -->
- [x] G3 the class. Every G0 site that can collide is fixed the same way, or recorded as safe
      with its single caller named. No site left unruled.
<!-- demonstrated 2026-10-08, same fix plan: the seven cross-run sites carry the process id; the
guard exit 0 with both exceptions still matching a site; zero-core 42 passed, zero-deadwood 9
passed; cargo fmt --all -- --check exit 0. -->
- [x] G4 doors. Five consecutive full cargo test -p core runs green, each result line quoted;
      nsh-test all passing; d 0 failed.
<!-- demonstrated 2026-10-08: five consecutive full cargo test -p core runs, each 64 passed and 0
failed; nsh-test 235 / 235 passed (nsh 5.3.1); d 0 failed (26 passed, 2 warnings: uncommitted
changes and unpushed commits, which G5 and G6 clear). No ship: every change is under
#[cfg(test)], so no deployed binary changes. -->
- [x] G5 commit. Intent: INT-284, Fixes: F-0026, and the plan Seal with its note under
      refs/notes/seals.
<!-- demonstrated 2026-10-08: e8679d19 carries Intent: INT-284, Fixes: F-0026, Finding: F-0027 and
Seal: 137fcb49e69f56b0; core intent trace e8679d19 reads fixes F-0026 and the seal intact. -->
- [x] G6 the push. The pre-push hook passes; main and refs/notes/seals reach GitHub, the note
      for 2bb11c7b included; trace reads the seal intact on 2bb11c7b and on the G5 commit.
<!-- demonstrated 2026-10-08: both pushes passed the pre-push hook (main 2bb11c7b..e8679d19,
refs/notes/seals f73684ba..13e7df73); the remote seals ref matches the local one at 13e7df73bfef;
trace reads seal 55175509f0a5097d intact on 2bb11c7b and 137fcb49e69f56b0 intact on e8679d19. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- demonstrated 2026-10-08: G0 to G6 each carry an evidence comment. -->

## Non-goals

- No change to RegistryVersionDriftCheck or any other check; only test scaffolding moves.
- No tempfile crate or any other new dependency; a counter is enough with what is on the box.
- Renaming the second ghost tag is not the fix: it repairs one collision and leaves the class.
- No retry or flaky allowance in the pre-push hook: a gate that passes on a second try is not
  a gate.
- No version bump: the change is test-only, under #[cfg(test)] (cicomplete 284 --skip-bumps).

## Versions
- engine skipped
