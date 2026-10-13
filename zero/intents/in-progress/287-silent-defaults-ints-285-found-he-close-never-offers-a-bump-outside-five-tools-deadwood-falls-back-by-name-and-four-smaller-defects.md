---
id: 287
date: 2026-10-12
type: future
title: "Silent defaults (INT-285) found: the close never offers a bump outside five tools, deadwood falls back by name, and four smaller defects"
status: in-progress
tags: [285, deadwood, cicomplete, ship, nsh]
---

## START HERE

Written 2026-10-13 at a break; it supersedes the START HERE written earlier the same day. Open a
new chat with `ints 287` and this section pasted. The tree is clean, and this record is pushed with
the commits it lists.

STATE: G0 and G1 ticked. G2 in progress, steps 0 to 2 done, plus a warnings cleanup Christian added:
- 2819446c step 0: zero-zone removed (ruling 3). Its one user, the alias audit's Current Zone
  header (engine doctor/aliases.rs), is gone; the crate, its row and the engine dependency too.
- 43e8705b step 1: zero_core::registry behind the off-by-default feature "registry": parse,
  read_in, read, match_rows, crates_in, resolve_in, resolve, Workspace::find (row name, then
  package, then folder). zero-doctor gained its registry row. 7 tests; one runs against this
  checkout: every workspace crate has a versioned row and every versioned row has a crate.
- 9c93a774: paths.rs runtime_dir's resolution-order table moved into a text block (clippy).
- d90c5ecc step 2: touched_crates (engine intent/mod.rs:1108) reads the registry and stops the
  close when it cannot (ruling 4); its decision moved, unchanged, into crates_touched(paths, ws)
  just below it. The engine enables zero-core's registry feature.
- e701a229 warnings batch 1: clippy's own fixes, 42 sites in 23 files (75 warnings to 34).
Deployed binaries are unchanged: nsh 5.3.2, nsh-test 2.0.3. One ship at the end of 287.

RULED 2026-10-13, recorded under G2: (1) a registry row with no type counts as rust; (2) any file
under a crate's folder counts as touching it; (3) zero-zone is removed inside 287, no intent of
its own; (4) an unreadable registry stops the close; (5) every clippy warning in the workspace is
cleaned up inside 287.

NEXT, in order:
1. The 34 warnings left: unnecessary_sort_by 16, manual_checked_ops 9, items_after_test_module 2,
   and one each of collapsible_if (friday/doc_steward.rs:51), collapsible_match (predict/mod.rs:971;
   the guard would fall through to later arms, so read them first), let_unit_value
   (friday/attention.rs:270), empty_line_after_doc_comments (doctor/mod.rs:1594), type_complexity
   (zero-docs toolgen.rs:105), useless_format (snapshot/mod.rs:211, a non-ASCII line) and
   filter_next (new after batch 1, likely zero-doctor probes/internals.rs:40, where
   .filter().next_back() wants .rfind()). Batch 2: sort_by to sort_by_key with Reverse, the
   compiler checking each key is Copy. Batch 3: checked_div by hand, each site's types read first,
   because a signed total changes the meaning. Batch 4: the rest.
2. G2 step 3, red first. The red: in the engine's intent tests, crates_touched given a path under
   nsh-test and a workspace where nsh-test has a versioned row must offer nsh-test; rehearsed on
   step 2's code it fails left [] right ["nsh-test"]. Shown failing on the tree, then the fix:
   crates_touched decides from the workspace, every versioned crate whose folder a path lies under
   (ruling 2; paths::crate_of knows CRATE_PARENTS only, not CRATE_PARENTS_HISTORY, so settle
   whether the rule needs the history); a touched crate with no versioned row is an Err that names
   it; TouchedCrate.tool and VersionPlan.tool become String, named by the registry row (core, nsh,
   nsh-test), and --bump accepts a row, package or folder name through Workspace::find. The
   substring match path_in_crate uses for zero-git, friday-chat and db-browse (:1223) goes, and
   one_rule_places_a_path_in_a_crate flips for zero/engine/README.md. bump-versions (novashell
   commands/mod.rs: tool_cargo_path :16819, bump_versions_table :16923) reads the registry, and
   novashell's Cargo.toml:22 turns the registry feature on. The commit carries Fixes: F-0028.
   Then G2's tick with its record.
3. G3 to G12 as listed under them, then cicomplete. The cleanup touches about 13 crates, so the
   close will ask about each one (patch or skip).

AFTER G2, unchanged: G3 the close writes registry rows through integrity's UpdateRegistryVersion
writer, plus the hardcoded version copies; G4 deadwood's reader; G5 the nsh bytes ruling; G6
dry-run wording; G7 the repo_present doc comment; G8 absolute NSH_BIN; G9 F-0020; G10 to G12
doors, ship, commits, push, evidence.

TOOLS kept in ~/.cache/zero while 287 is open, deleted at its end: int287-doors-95ebceb056a0.py
measures the doors in python (build, workspace tests, clippy's JSON count, rustfmt on changed
files, nsh-test, status). It cannot run d, which is an nsh alias, so d runs by hand until the
script reads the alias. int287-clippy.json is the last clippy run, written by the doors.
int287-warn-b1v2-c59c0e359e7b.py is batch 1's generator: clippy's byte-range suggestions,
rustfmt, whole-line anchors that grow until unique. The seal-*.txt notes are the plans.

LEARNED: nsh drops 2>&1 on a pipeline's first command (F-0036), so no door pipes cargo through
the shell. Every payload is checked as the exact line sent, parsed and decoded, not only as the
file it came from: one copy carried a mistyped character and zlib refused it. The argv puts the
hash before the data, so a cut paste fails to decode instead of running half.

ALSO NOTED, no action yet: docs/inventory.md says zero-core has 18 users; measured 17 before
zero-zone left, 16 now.

After 287 the order is 289 -> 290 -> 277 -> 288. reedline stays deferred.

## Vision
Every tool that changes is offered a version at the close, every binary a suite
spawns is resolved by one owner and refused when missing, and what a tool prints
says what it did.

## The Problem (measured 2026-10-12 during INT-285)
1. cicomplete offers bumps only for TRACKED_CRATES (engine intent/mod.rs:1205:
   novashell, engine, zero-git, friday-chat, db-browse), and bump-versions knows
   the same five (novashell commands/mod.rs:16925). nsh-test changed in e1a19f83,
   and cicomplete 285 closed with no question and no word that it skipped one.
   The 2.0.2 patch was written by hand after the close.
2. write_versions bumps Cargo.toml and Cargo.lock but never the registry row.
   Every bump needs a follow-up "registry:" commit (1bec9193, 454f5ba8, e6a47c75),
   and nsh-test 2.0.1 was missed until core integrity proposal #22 caught it.
3. nsh-test main.rs:86 run_deadwood reads DEADWOOD_BIN with a bare
   "zero-deadwood" default, the shape F-0020 had: PATH decides, and a missing
   target is not refused.
4. The deployed nsh 5.3.1 and a fresh release build of the same source differ
   in bytes. The deployed one predates the unshipped e8679d19 (test-only code).
   Cause not established.
5. ship --dry-run ends "1 shipped" when nothing was shipped.
6. nsh-test main.rs:183-187, repo_present's doc comment, sits above
   fixture_dir's, so rustdoc gives it to fixture_dir.
7. F-0020's text names main.rs:2646, which was :2764 by the time it was fixed.
8. nsh-test accepts a relative NSH_BIN. repl.rs fsh_bin checks that the path exists from
   nsh-test's own working directory, but every REPL case starts nsh with current_dir /tmp, so
   NSH_BIN=target/debug/nsh fails all 109 REPL cases with "spawn nsh: No such file or directory"
   (measured 2026-10-12, INT-286). main.rs:100 and :5331 recommend exactly that relative form.

## Success Criteria
- [x] G0 census, read-only. Each item re-measured at its current file:line,
      with the shape of each fix and what each change reaches. Recorded here.
<!-- demonstrated 2026-10-13, two read-only reads (int287-g0, int287-g0b) at HEAD 8de4a061:
1. THREE copies of one five-tool list. TRACKED_CRATES, engine intent/mod.rs:1205-1211 (novashell,
   engine, zero-git, friday-chat, db-browse), read at :1124. bump-versions' tool_cargo_path,
   novashell commands/mod.rs:16819-16827: the same five, "core" and "engine" both meaning engine.
   bump_versions_table, :16925, a literal of the five again. Shape (G2): one owner both read, the
   source ruled at G2. Reaches: engine (cicomplete) and novashell (bump-versions).
2. write_versions, intent/mod.rs:1573-1636, writes Cargo.toml through zero_core::version::apply_bump
   and calls sync_lock (:1640-1676). Nothing in the close writes tools.toml. sync_lock only prints a
   warning when cargo update fails, and the close goes on. integrity already owns a registry-row
   writer: FixAction::UpdateRegistryVersion, applied at integrity/mod.rs:1624-1637. Shape (G3): the
   close uses that writer, refusing before any write if the row is missing. Reaches: engine.
3. run_deadwood, nsh-test main.rs:85-86: DEADWOOD_BIN, else a bare "zero-deadwood" found on PATH,
   never refused. Shape (G4): one reader in the shape of repl::fsh_bin. Reaches: nsh-test.
4. CHANGED SINCE FILING: the deployed nsh and target/release/nsh are byte-identical
   (62ed755902981676) since INT-286 shipped 5.3.2 from 57389885. The 5.3.1 mismatch this item names
   can only be re-measured from ship's backups in 0-core/bin; G5 rules whether to.
5. ship main.rs:572-574 prints "would ship" for each tool on --dry-run but pushes it onto the same
   list the summary at :619 counts as shipped. Shape (G6): a dry run's summary says would ship.
   Reaches: ship.
6. nsh-test main.rs:183-187, repo_present's doc, sits above fixture_dir's (:188-191, fn :192);
   repo_present, :272, has none. Shape (G7): move the five lines. Reaches: nsh-test.
7. F-0020 still names main.rs:2646 (its at: is :325). Ruling at G9.
8. fsh_bin, nsh-test repl.rs:43-81, accepts a relative NSH_BIN and returns it relative; the REPL
   spawn sets current_dir /tmp at repl.rs:522 (main.rs:3958 too); the -c door sets none. The help
   text is at main.rs:100 and :5485, not :5331. rustc 1.98.1, so std::path::absolute is there.
   Shape (G8): fsh_bin returns an absolute path. Reaches: nsh-test.
G3's hardcoded copies (Rust x.y.z literals; tests and comments left out):
- teach main.rs:12, const VERSION "4.0.0", printed at :1427; teach's Cargo.toml is 4.0.0 today.
  Shape: env!("CARGO_PKG_VERSION").
- zero-context main.rs:14, clap's version = "1.0.0"; its Cargo.toml is 1.0.0. Shape: clap's bare
  version, which reads Cargo.toml.
- engines/mod.rs:45-53, seed_registry: ("core", "3.0.0") while engine is 4.4.0, written INSERT OR
  IGNORE into state.db's engine_registry, so fixing the seed leaves a row already there unchanged.
  Ruling at G3.
- teach main.rs:249 on, build_tool_registry: a second hand-kept tool list carrying versions
  (zero-git "3.3.0", now 5.1.2; dot-doctor "2.0.0", a retired name). Larger than a version copy:
  filed as a finding at G1, not fixed here unless G3 rules it in.
-->
- [x] G1 findings. Each of 1-6 and 8 is filed as a finding before any fix, found_by
      this intent.
<!-- demonstrated 2026-10-13 through core intent find, --by 287 on every call (int287-g1, its
Seal 04037c99d7e6cedd over the exact argv), each exit 0: F-0028 item 1, F-0029 item 2, F-0030
item 3, F-0031 item 4 (at: unknown, it was never a line of code), F-0032 item 5, F-0033 item 6,
F-0034 item 8, and F-0035 the teach tool list G0 found under G3. Item 7 is F-0020 itself, ruled at
G9. core intent validate: all 326 intents valid. No fix was made before these were filed. -->
- [ ] G2 one list of versioned crates. Ruling first: the source of truth
      (registry deployable rows, workspace members, or one shared list
      cicomplete and bump-versions both read). Red: a test where a touched
      crate outside today's five is not offered, shown failing. Fix: both
      read the one owner. Green. A crate that is touched but undecided still
      refuses the close.
<!-- ruled 2026-10-13 by Christian, from the G2 reads (int287-g2r, int287-g2c, int287-g2d):
SOURCE: the registry, as Decision 149's owner. A crate is versioned when it has a registry row
with type = "rust" and retired = false whose crate field (or, without one, its name) is the
package name of a workspace crate. The close offers a bump for every such crate its commits
touched; a touched workspace crate with no such row refuses the close and names it. Option 3, a
shared list in code, is ruled out: it would be a fourth copy of the list.
OWNER: zero_core::registry, behind a new off-by-default feature "registry" built on serde and
toml, both existing workspace dependencies, nothing downloaded; engine and novashell enable it.
It parses tools.toml into typed rows, honours type, retired and deployable, and returns an
error, never an empty list, when the file cannot be read or parsed (Decision 149). A row finds
its folder through each crate's Cargo.toml package name (the engine's package is core, its
folder engine). A path belongs to a crate when it lies under the crate's folder name beneath any
parent in paths::CRATE_PARENTS_HISTORY, or inside a single crate such as zero/engine.
SCOPE: only the close (TRACKED_CRATES, path_in_crate, touched_crates) and bump-versions
(tool_cargo_path, bump_versions_table) move to the owner. The other readers Decision 149 lists
stay as they are: 149 gains its owner and two users here, and its migration stays its own work.
NAMES: a crate is named by its registry row (core, nsh, nsh-test); --bump and bump-versions also
accept its package or folder name, so engine and novashell keep working.
ZERO-DOCTOR: gets a registry row, type rust, deployable false, retired false, version 0.1.0: a
library the engine uses, the same shape as zero-core's row. A close that touches either one asks
about it; skip stays the answer for zero-core while its versioning is unsettled.
MEASURED: 20 workspace crates, 19 with a matching row whose version equals Cargo.toml, and
zero-doctor the one without.
CORRECTED 2026-10-13: that count matched rows by name alone. With type and retired honoured,
zero-zone had no versioned row either: its row was retired while the engine still built it.
RULED 2026-10-13 by Christian: (1) a row with no type field counts as rust, as integrity
rules (integrity mod.rs:1124-1129); (2) any file under a crate's folder counts as touching it;
(3) zero-zone is removed here as step 0, with no intent of its own. Its one user was the alias
audit's Current Zone header (doctor/aliases.rs:238), a remnant of the old file manager's
numbered folders; the header line, the engine's dependency, the row and the crate go.
RULED 2026-10-13 by Christian: (4) an unreadable registry stops the close (d90c5ecc, Decision
149); (5) every clippy warning in the workspace is cleaned up inside 287: 75 at the first
measure, 34 after batch 1 (e701a229). -->
- [ ] G3 the close writes the registry. Red: a bump test whose registry row
      stays old. Fix: write_versions updates the row in the same plan, and
      refuses before any write if the row is missing. Green, plus the drift
      check clean on a real close.
      Also any copy of a version hardcoded in source (277's fourth place): found at G0,
      and moved by the same plan or removed.
- [ ] G4 deadwood has one reader. The INT-285 pattern: red case, the reader
      refuses a missing target, green. nsh-test's count goes up by one.
- [ ] G5 why the nsh bytes differ. Two release builds of one commit compared,
      and the deployed binary against a build of its own commit. Cause
      recorded. A fix only if it is a defect; ship nsh is Christian's call.
- [ ] G6 ship --dry-run says what it would do. Red, fix, green: "would ship"
      in the summary, and the real ship's wording unchanged.
- [ ] G7 repo_present's doc back on repo_present. cargo doc clean, nsh-test
      unchanged.
- [ ] G8 NSH_BIN resolves to one absolute path. Red: a relative NSH_BIN shown failing the REPL
      cases, in a form G0 settles. Fix: fsh_bin, the one reader, makes the path absolute against
      nsh-test's working directory before any case runs, or refuses it; the help text at
      main.rs:100 and :5331 shows a form that works. Green.
- [ ] G9 ruling on F-0020: findings are history, so leave it with a note,
      or amend it.
- [ ] G10 doors: ship what changed, nsh-test all passing, d 0 failed, core
      intent validate clean.
- [ ] G11 commits: one per fix, each with Intent and Fixes and its plan Seal
      and note; pushed; trace reads every seal intact.
- [ ] G12 each gate carries evidence per INT-158.

## Non-goals
- No change to how nsh behaves. No new crates. No sudo. Omarchy untouched.
- No retroactive bumps for tools closed before this intent.
