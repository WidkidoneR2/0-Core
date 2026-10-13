---
id: 287
date: 2026-10-12
type: future
title: "Silent defaults (INT-285) found: the close never offers a bump outside five tools, deadwood falls back by name, and four smaller defects"
status: in-progress
tags: [285, deadwood, cicomplete, ship, nsh]
---

## START HERE

Written 2026-10-13 at a break. Open a new chat with `ints 287` and this section pasted.

STATE: G0 and G1 ticked with their records. G2's rulings are recorded under G2 (the "ruled
2026-10-13" comment); G2 is not ticked and none of its code is written. Deployed: nsh 5.3.2,
nsh-test 2.0.3. After 287 the order is 289 -> 290 -> 277 -> 288; 289 and 290 were expanded
2026-10-13. reedline stays deferred.

TWO RULINGS OPEN (asked 2026-10-13, not yet answered):
1. A registry row with no type field counts as rust, as integrity already rules (integrity
   mod.rs:1122-1157), so the close and integrity agree on which rows are versioned. Claude
   recommends yes. G2's record says type = "rust"; a yes refines that to "type absent or rust".
2. Any file under a crate's folder counts as touching it. Claude recommends yes. It flips the
   test one_rule_places_a_path_in_a_crate, which pins zero/engine/README.md as not touching
   core. The alternative keeps today's rule: only src/ and Cargo.toml count.
Step 1 needs ruling 1 only; ruling 2 is first used in step 2.

READ STILL TO PASTE: zero-core's Cargo.toml and src/lib.rs in full, the zero-core and zero-doctor
rows of zero/registry/tools.toml, and that file's tail. Claude sends the line again.

G2 PLAN, three steps, each shown green before the next:
1. The owner in zero-core: feature registry = ["dep:serde", "dep:toml"], both workspace deps
   already in Cargo.lock. src/registry.rs modelled on zero-doctor's serde reader
   (probes/tools.rs:6-46): typed rows (name, version, type, crate, deployable, retired);
   parse, read_in and read; an error, never an empty list, for a missing or invalid file; a
   resolver from rows to workspace crates through paths::crate_dirs_in (which includes
   zero/engine) and each Cargo.toml's package name; unit tests. zero-doctor's row (rust,
   deployable false, retired false, 0.1.0). No behaviour change anywhere else.
2. Engine extraction with behaviour preserved: cargo test -p core (64 tests) and the suite stay
   green. Then red: a commit that touches nsh-test is not offered a bump.
3. Fix: the close (touched_crates :1108, TRACKED_CRATES :1205, path_in_crate :1217 in
   engine/src/domains/intent/mod.rs) and bump-versions (tool_cargo_path :16819,
   bump_versions_table :16923 in novashell's commands/mod.rs) read the owner. TouchedCrate.tool
   goes from &'static str to String (decide_versions, write_versions, evidence_lines). Red
   goes green; a touched crate with no versioned row refusing the close gets its own test.

AFTER G2: G3 the close writes registry rows through integrity's UpdateRegistryVersion writer,
plus the hardcoded version copies; G4 deadwood's reader; G5 the nsh bytes ruling; G6 dry-run
wording; G7 the repo_present doc comment; G8 absolute NSH_BIN; G9 F-0020; G10-G12 doors, ship,
commits, push, evidence. Then cicomplete.

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
zero-doctor the one without. -->
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
