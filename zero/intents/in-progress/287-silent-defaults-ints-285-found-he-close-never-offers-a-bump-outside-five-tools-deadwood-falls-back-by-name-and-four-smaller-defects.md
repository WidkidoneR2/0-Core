---
id: 287
date: 2026-10-12
type: future
title: "Silent defaults (INT-285) found: the close never offers a bump outside five tools, deadwood falls back by name, and four smaller defects"
status: in-progress
tags: [285, deadwood, cicomplete, ship, nsh]
---

## START HERE

Written 2026-10-13 at a break, after G2; it supersedes the START HERE written earlier the same day.
Open a new chat with `ints 287` and this section pasted. The tree is clean and this record is pushed
with every commit it names.

STATE: G0, G1 and G2 ticked. Since the last START HERE (50289ff4):
- 7925d0e8, 0bc0b95d, 3ba9648b: ruling 5 finished. clippy went 75 -> 0 across the workspace in four
  sealed batches (e701a229 was batch 1). Two orphaned doc comments in doctor/mod.rs were handled by
  Christian's clean-up ruling: the INT-094 lines removed, the INT-222 note moved onto from_engine.
- 36b6d7c8 G2 step 3, Fixes: F-0028. crates_touched decides from zero_core::registry; path_in_crate is
  the one rule (ruling 2, whole segments, CRATE_PARENTS_HISTORY for old git paths); TRACKED_CRATES is
  gone; a touched crate with no versioned row refuses the close and is named; crates are named by
  their registry row (core, nsh, nsh-test) and --bump accepts row, package or folder; bump-versions
  resolves through Workspace::find and lists every versioned row (19). Red plan 75758fa4e0f9569b, fix
  plan ca71b9068867f964, doc plan c9e69e5f05795868. 442 tests, clippy 0, nsh-test 238/238.
Deployed binaries are unchanged: nsh 5.3.2, nsh-test 2.0.3. One ship at the end of 287; the deployed
nsh still has the old bump-versions until then.

NEXT, in order:
1. G3, the close writes the registry. Red: a bump test whose registry row stays old. Fix:
   write_versions writes the row through integrity's FixAction::UpdateRegistryVersion writer
   (integrity/mod.rs, found at G0) in the same plan, refusing before any write if the row is missing.
   G0's hardcoded copies come with it: teach main.rs VERSION "4.0.0" (env!), zero-context clap
   version "1.0.0" (bare version), engines/mod.rs seed_registry ("core", "3.0.0") needs a ruling
   (INSERT OR IGNORE leaves the old row), teach build_tool_registry is F-0035, ruled in or out at G3.
   Read every site again first: G2 moved lines in intent/mod.rs.
2. G4 deadwood has one reader (nsh-test DEADWOOD_BIN, the repl::fsh_bin shape), G5 the nsh bytes
   ruling, G6 ship --dry-run wording, G7 repo_present's doc, G8 absolute NSH_BIN, G9 the F-0020
   ruling, G10-G12 doors, ship, commits, push, evidence.
3. cicomplete 287. The close now asks every versioned crate the commits touched: 13 measured before
   36b6d7c8, nsh joins with it. Names are registry rows; zero-core answers skip while its versioning
   is unsettled. Decide with --bump row=level or answer in the terminal.

TOOLS kept in ~/.cache/zero while 287 is open, deleted at its end: int287-doors-95ebceb056a0.py (build,
tests, clippy JSON count, rustfmt, nsh-test, status; it cannot run d, so commit payloads run core
doctor run themselves as a gate), int287-clippy.json, the seal-int287-*.txt plan texts, and the
int287-*.py generators.

LEARNED this session:
- A paste block never carries an angle-bracket placeholder: nsh reads <SEAL> as a redirect.
- git notes add -F runs stripspace and breaks a seal note; store the exact blob with git hash-object
  -w and attach it with git notes add -C, then read it back and hash it (trace hashes the raw blob).
- Generators keep themselves at ~/.cache/zero/<name>-<hash>.py at show, so an apply is one short
  line and no long payload is pasted twice. A commit payload can run the doors and core doctor run
  and refuse unless green.
- Plans that move non-ASCII text use between edits whose new lines are read from disk; markers stay
  ASCII. A model of every file's final bytes, checked against fpatch's own hashes, plus rustfmt on
  the modelled bytes, catches a wrong plan before show.
ALSO NOTED, to recon and file at the start of the next session (cause not measured): the first d
of the session printed "command not found: d" and the next d ran; recalling a long show line with
the up arrow and appending apply ran the show again.
docs/inventory.md says zero-core has 18 users; 16 measured after zero-zone left.

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
- [x] G2 one list of versioned crates. Ruling first: the source of truth
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
<!-- evidence: commit 36b6d7c8, 2026-10-13, Fixes: F-0028, seal ca71b9068867f964 intact (red plan
75758fa4e0f9569b, doc plan c9e69e5f05795868). Red first: a_touched_crate_outside_the_old_five_is_offered
failed on the tree with left [] right ["nsh-test"], cargo exit 101, before any fix. Green: 442 tests
passed and 0 failed, clippy 0, rustfmt clean, nsh-test 238/238, core doctor run 0 failed. A touched
crate with no versioned row refuses the close and is named, by the test
a_touched_crate_with_no_versioned_row_refuses_and_names_it. Real tree: the debug nsh's bump-versions
listed 19 versioned rows and refused frob with exit 1; INT-287's own close, modelled from 60 days of
its commits, asks 13 crates and refuses none. Earlier steps: 2819446c (step 0), 43e8705b (step 1),
9c93a774, d90c5ecc (step 2); the warnings cleanup of ruling 5, 75 to 0, in e701a229, 7925d0e8,
0bc0b95d, 3ba9648b. -->
- [ ] G3 the close writes the registry. Red: a bump test whose registry row
      stays old. Fix: write_versions updates the row in the same plan, and
      refuses before any write if the row is missing. Green, plus the drift
      check clean on a real close.
      Also any copy of a version hardcoded in source (277's fourth place): found at G0,
      and moved by the same plan or removed.
<!-- progress 2026-10-16. Box left open for one clause: the drift check clean on a real close,
run at cicomplete 287 once ship deploys the new core.
Ruled by Christian 2026-10-16: integrity's drift fix uses the one writer; seed_registry sets core
from the engine's Cargo.toml on every seed and the six engine-state seeds stay; F-0035 ruled in,
its doctor entry corrected (b); bump-versions fixed here; sync_lock filed (F-0041); the generated
tools README is regenerated at 287's end (F-0043 names its novashell row); teach's CHANGELOG
version filed (F-0044).
- 10e80699 zero_core::registry::set_version and write_version, the one row-version writer;
  integrity's UpdateRegistryVersion uses it. Seal 9aac4dbc9cd83ce8.
- 010abd46 Fixes F-0029: the close writes the row with the crate, and refuses before any write.
  Red 1d75f19cf3fbfb1d failed left 1.2.3 right 1.3.0; fix ee7a4e0c1a6e63d9.
- 805d14f6 Fixes F-0042: bump-versions writes the row too. Red ab0d4e7dda86c3ab failed left
  1.2.3 right 1.2.4; fix c647035b7e19189a.
- be8d5366 Fixes F-0035: no_crate_hardcodes_a_version names every x.y.z literal in non-test
  code (red: 8 sites; the seed red read 3.0.0 where 4.4.0 was due). teach, zero-context, the
  core seed and the two probes now read their versions. Red 04db8630c5d68161, fix
  459326d4f60b72bb. Doors: 453 passed, clippy 0, nsh-test 238/238, core doctor run 0 failed.
- Findings filed by 287 on 2026-10-16: F-0037 to F-0045. F-0045, a core test that reads HOME
  without HOME_LOCK, turned one push gate red. -->
- [x] G4 deadwood has one reader. The INT-285 pattern: red case, the reader
      refuses a missing target, green. nsh-test's count goes up by one.
<!-- evidence: commit 90d77e16, 2026-10-16, Fixes: F-0030, seal 4a42ced05fe208f1 (red plan
21d5df6b0a9a9409). Red: deadwood_bin_is_read_in_one_place failed on the tree, DEADWOOD_BIN read 1
times, expected once, in src/repl.rs: src/main.rs:152. Green: both one-reader cases pass, and
bin_tests shows both arms refusing a missing binary. The debug suite runs 239 cases, one more than
the deployed 238; the deployed count moves at the ship (G10). -->
- [x] G5 why the nsh bytes differ. Two release builds of one commit compared,
      and the deployed binary against a build of its own commit. Cause
      recorded. A fix only if it is a defect; ship nsh is Christian's call.
<!-- evidence: measured 2026-10-16 against the deployed nsh 5.3.2 (62ed755902981676), which ship
built from 57389885. Two cargo build --release -p novashell builds of 57389885 into different target
dirs were byte-identical (79dbc8f296ce811f): the build is reproducible and the target path is not
embedded. Both differ from the deployed binary, which is 2392 bytes larger. A cargo build --release
of the whole workspace, the command ship runs (ship main.rs:466), produced 62ed755902981676, byte
for byte the deployed binary. Cause: the bytes follow the build command, not the commit. A
workspace build unifies dependency features across crates and a -p build does not. Not a defect,
so no fix (ruled with G5's own text): a deployed binary is compared only with a workspace release
build of its commit. The 5.3.1 mismatch was not re-measured; ship keeps three earlier nsh backups. -->
- [x] G6 ship --dry-run says what it would do. Red, fix, green: "would ship"
      in the summary, and the real ship's wording unchanged.
<!-- evidence: commit e01656e5, 2026-10-16, Fixes: F-0032, seal 9413ad38d9065840 (red plan
5bf7bcce41c9eb02). Red: a_dry_run_summary_says_would_ship failed, left 1 shipped, right 1 would
ship. Green, with the real ship's wording asserted unchanged in the same test. -->
- [x] G7 repo_present's doc back on repo_present. cargo doc clean, nsh-test
      unchanged.
<!-- evidence: the doc move landed in commit 0153ec95 beside G8 (plan g7, seal ad77950a1b2ffc8a):
the pre-commit hook restaged nsh-test main.rs, so the separate G7 commit found nothing left. The
commit before this record fences an unrelated indented block at repl.rs:6 as text and carries
Fixes: F-0033; cargo doc -p nsh-test --no-deps then reports 0 warnings. nsh-test unchanged. -->
- [x] G8 NSH_BIN resolves to one absolute path. Red: a relative NSH_BIN shown failing the REPL
      cases, in a form G0 settles. Fix: fsh_bin, the one reader, makes the path absolute against
      nsh-test's working directory before any case runs, or refuses it; the help text at
      main.rs:100 and :5331 shows a form that works. Green.
<!-- evidence: commit 0153ec95, 2026-10-16, Fixes: F-0034, seal 9e53ccd84aba23ea (red plan
6375073ae86fb335). Red: a_relative_shell_path_is_made_absolute failed on the tree. Green, and the
REPL cases run with env NSH_BIN=target/debug/nsh, the relative form, passed 86/86 (measured
2026-10-16); before the fix all of them failed with No such file (INT-286). The help text at
main.rs:167 and :5500 shows env NSH_BIN=target/debug/nsh. -->
- [x] G9 ruling on F-0020: findings are history, so leave it with a note,
      or amend it.
<!-- ruled 2026-10-16 by Christian: findings are history, so F-0020 is left as written. Its text
names main.rs:2646, which was :2764 when it was fixed; its at: field is main.rs:325. This note is
the record of that. Also closed in 287 on 2026-10-16: F-0045 in ae77e56d. -->
- [ ] G10 doors: ship what changed, nsh-test all passing, d 0 failed, core
      intent validate clean.
- [x] G11 commits: one per fix, each with Intent and Fixes and its plan Seal
      and note; pushed; trace reads every seal intact.
<!-- evidence: 2026-10-16. Every commit naming INT-287 that carries a Seal: (22 on main, from
2819446c onward) was read with core intent trace and every seal is intact: the plan text kept
under refs/notes/seals hashes to it. All pushed to origin with refs/notes/seals. One per fix,
with one exception recorded at G7: G7's doc move landed in 0153ec95 beside G8 because the
pre-commit hook restaged main.rs, and 3199de06 carries Fixes: F-0033. Findings fixed by 287:
F-0028 36b6d7c8, F-0029 010abd46, F-0030 90d77e16, F-0032 e01656e5, F-0033 3199de06, F-0034
0153ec95, F-0035 be8d5366, F-0042 805d14f6, F-0045 ae77e56d. F-0031 closed without a fix by G5's
measurement. Open by ruling, for their own intents: F-0036 to F-0041, F-0043, F-0044. -->
- [ ] G12 each gate carries evidence per INT-158.

## Non-goals
- No change to how nsh behaves. No new crates. No sudo. Omarchy untouched.
- No retroactive bumps for tools closed before this intent.
