---
id: 280
date: 2026-10-07
type: fix
title: "Update Readiness gates Omarchy updates on the 0-core tree, which the Omarchy updater never touches"
status: in-progress
tags: [doctor, update-readiness, omarchy, fix]
depends_on: []
---

## Vision

d reports the health of the shell and the Zero tools. Update Readiness was a NixOS check: it
asked whether a rebuild from the 0-core tree was safe. On Omarchy the updater reads nothing
from that tree, and what is left of the check is a second copy of Reboot Needed. 280 retires
the check so each fact has one owner: the tree belongs to Git Repository, the kernel to
Reboot Needed.

## The Problem

The probe (zero/tools/zero-doctor/src/probes/hardening.rs:158-201) has two blockers, and neither
belongs to it:

1. Tree: `git -C <core root> diff --quiet HEAD` (hardening.rs:178-189). The Omarchy update
   never reads or writes ~/0-core. omarchy-update (1-64) names neither ~/0-core nor
   ~/.local/bin; omarchy-update-dev exits 0 when OMARCHY_PATH is /usr/share/omarchy (dev:7);
   the migrations name only wrapper files in ~/.local/bin; the three post-update hooks only
   send notifications. It is a leftover from NixOS, where the tree was the system
   configuration (222:1132-1134).
2. Kernel: running kernel not among the installed ones (hardening.rs:166-175). Reboot Needed
   measures the same fact with the same two readers (system.rs:39,43 and :171,:185;
   hardening.rs:163-164).

What that costs today:

- ONE FACT, TWO WARNINGS. A tracked change in 0-core warns under Git Repository and again under
  Update Readiness. A kernel mismatch would warn under Reboot Needed and again here.
- A WRONG INSTRUCTION. d says to hold off an Omarchy update until 0-core work is committed.
  Nothing in the update needs that.
- A VAGUE RECOVERY. "Resolve the above first" (checks.toml:264) names no next move.
- AN INCOMPLETE TREE CHECK. diff --quiet HEAD ignores untracked files, so it passed on
  2026-10-07 while Git Repository warned.
- A LOST SIGNAL. When a blocker exists, unreadable signals are dropped (hardening.rs:191-193).

## The Solution

- RETIRE THE CHECK. Remove the registry entry (zero/registry/doctor/checks.toml:257-264), the probe
  and its doc comment (hardening.rs:243-293), the Probe::UpdateReadiness variant (probe.rs:75-76),
  name map (probe.rs:118), list entry (probe.rs:153), the dispatch (probes/mod.rs:64) and the live
  test (hardening.rs:392 to the end of that test). Lines measured 2026-10-30.
- ONE OWNER PER FACT. Git Repository owns the tree and Reboot Needed owns the kernel. Neither
  changes.
- NOTHING ELSE MOVES. d goes from 29 checks to 28. The score formula and forecast are untouched.
- HISTORY STAYS READABLE. Stored runs that name the retired check still read.

## Decision

280 supersedes 222:1136-1139 (approved by christian 2026-09-20), which ruled that
update_readiness gates on "an out-of-date kernel or a dirty tree". Both halves fall. The dirty
tree is not read by the Omarchy update and Git Repository owns it. The kernel is measured by
Reboot Needed with the same readers. 222 is complete and is not edited.

## Non-goals

- No change to Git Repository or Reboot Needed, or to Omarchy's code or update command.
- No new check. A real precondition of the Omarchy update that d does not measure (a pacman
  lock, disk space) is filed with core intent find and ruled on by Christian, not added here.
- No rewrite of history: complete intents and CHANGELOG.md:190 keep their words.
- No change to the health score formula or the forecast (INT-279).

## Scope against neighbours

- INT-222 owns how checks report. 280 retires one check and amends one ruling by citation.
- INT-265 retired zero-update because Omarchy has its own updater. 280 removes the last place d
  still reasons about updates as if 0-core were the system.
- INT-279 owns the forecast. Retiring a check changes the count it reads, not the forecast code.

## Dependencies

depends_on is empty on purpose: nothing has to finish before this can start.

## Success Criteria

Each gate is watched failing before it is watched passing. Anchors stay ASCII-only. Every commit
made from a reviewed plan carries Seal: and its note in refs/notes/seals.

- [x] G0: recon recorded, each finding with file:line. (a) The probe, its blockers, unread
  branch and messages (hardening.rs:150-201). (b) The registry entry and what its severities
  permit (checks.toml:257-264, definition.rs:175-179). (c) Every consumer of update_readiness or
  its message (fsearch, including ship, nsh and scripts). (d) Omarchy's update path, read only:
  what it reads and writes, and whether any of it touches ~/0-core or ~/.local/bin.
  (e) That reboot_needed reads the same running_kernel and installed_kernels.
<!-- evidence 2026-10-30 (G0), all measured: (a) probe hardening.rs:250-293, doc comment 243-249. It reads running_kernel and installed_kernels, then runs git diff against HEAD in quiet mode. A kernel mismatch or a failed diff becomes a hold off blocker, an unreadable signal becomes unknown, otherwise pass with the message safe to update, kernel current, tree clean. (b) registry entry zero/registry/doctor/checks.toml:257-264 declares severities pass and warn. definition.rs:175-177 is_label is true only for pass alone, so this entry is judged, not a label; per the comment at definition.rs:185-186 unknown and blocked are always permitted; it can never report fail. (c) consumers: probe.rs:76,118,153, probes/mod.rs:64, hardening.rs:393-394 and one history comment at ship/src/main.rs:158; no script, alias or nsh reference outside the intent files. (d) read only scan of 832 files under /usr/share/omarchy: 21 omarchy-update scripts and 0 mentions of 0-core or of nsh, ship, core, d, fpatch, fsearch, ints, inta. 14 lines name local/bin: PATH setup (default/bash/env-bootstrap and envs, hypr envs.lua and helpers.lua, install/config/ssh-command-path.sh) and six migrations: one loop over local/bin that acts only on scripts holding a mise use line (1784909971, recorded as already run in the Omarchy state), removals of omp and herdr, a cursor-agent comment, an ssh PATH default and a security comment. (e) reboot_needed calls running_kernel and installed_kernels at system.rs:39 and 43, defined at system.rs:171 and 185; this probe calls the same two at hardening.rs:255-256. -->
- [x] G1: RED FIRST. At HEAD, d lists Update Readiness. With one tracked file in 0-core modified,
  d warns under Git Repository while Update Readiness still passes (F-0109: its git diff exits 0
  inside the doctor, 1 by hand). Saved as evidence before anything changes.
<!-- evidence 2026-10-30 (G1): deployed core, copied 2026-10-27 after the last commit of the probe. At HEAD d lists Update Readiness, 28 passed of 29 determinable, tree clean. One blank line appended to Cargo.lock and restored with git checkout (0 changed files after each run): d showed Git Repository uncommitted changes while Update Readiness stayed safe to update, 27 passed and 2 warnings (Rust Docs the other), also on a plain terminal. By hand git diff against HEAD in quiet mode exits 1 with stdout inherited, piped or devnull and stdin devnull. Under GIT_TRACE2_EVENT the call made by the doctor exited 0 while its git status porcelain saw the change. Filed as F-0109, cause not found. The premise of two warnings did not hold, so the gate was reworded to the measured red. -->
- [x] G2: REMOVED. The registry entry, probe, variant, name map, list entry, dispatch and live
  test are gone. fsearch finds no live reference (history excepted). cargo build -p zero-doctor
  and cargo test -p zero-doctor pass on the debug build.
<!-- evidence 2026-10-30 (G2), plan Seal 73284b793de5d1eb plus 2231113f78998a4d: registry entry, probe with its doc comment, the Probe::UpdateReadiness variant, name map line, list line, dispatch line and the live test are removed in zero/registry/doctor/checks.toml and zero-doctor probe.rs, probes/mod.rs and probes/hardening.rs. Two tests that pinned the old counts were updated: registry.rs all checks 29 to 28 and judging 27 to 26, definition.rs probes 29 to 28. fsearch update_readiness or UpdateReadiness in rs: only the history comment at ship/src/main.rs:158; in toml: no results. cargo build -p zero-doctor finished; cargo test -p zero-doctor 68 passed, 0 failed (one failure on the first run, the judging count, found and fixed). cargo doc -p zero-doctor --no-deps: 0 warnings. -->
- [x] G3: HISTORY. No stored run names update_readiness (health_patterns holds counts only), and the
  history readers still read the real state directory: core doctor trend and forecast exit 0 on the
  debug build. core doctor history fails for a reason this change did not cause (F-0110).
<!-- evidence 2026-10-30 (G3), real state directory ~/.local/state/zero/state.db opened read only. health_patterns (22081 rows) stores timestamp, health_pct, integrity_pct and the passed, warned, failed and unknown counts, no check names. A scan of every table for update_readiness or Update Readiness finds it only in typed command text: shell_history 138, term_commands 71, shell_history_audit 114, command_execution 36 typed and 36 executed, plus zero_events detail 2, shell_state value 1 and intent_commits message 1; none in health_patterns, horizon_snapshots or friday_patterns. Debug build target/debug/core after the removal: doctor trend exit 0 (22841 readings, current 92 percent), doctor forecast exit 0. doctor history exits 1 with no such column: score, on the deployed core as well, so it is not caused by this change; filed as F-0110 (horizon_snapshots has the columns id, horizon, snapshot, created_at). -->
- [x] G4: ONE OWNER, ON THE DEBUG BUILD. d shows 28 checks and no Update Readiness. With one tracked
  file modified, Git Repository warns once and nothing new warns. Restored, only the Rust Docs warning that was there before remains.
<!-- evidence 2026-10-30 (G4), target/debug/core built from 751693da. Clean and pushed: 27 of 28 checks, 27 passed, 1 warning (Rust Docs, one rustdoc warning in another crate, present before this intent), 0 failed, Update Readiness absent, Git Repository working tree clean and all commits pushed. Dirty half measured earlier the same day on the debug build with one tracked file modified: 26 of 28 checks, Git Repository warned once, the only other warning was the same Rust Docs. -->
- [x] G5: INT-222 IS AMENDED BY CITATION. The Decision section above stands, and git shows no
  commit under this intent touching the 222 file.
<!-- evidence 2026-10-30 (G5), git log of the 222 file in zero/intents/complete lists one commit, 7b79c725 (INT-252, the zero rename). No commit under INT-280 touches it. -->
- [x] G6: DEPLOYED. One ship at the end. With the deployed core, d shows 28 checks and no Update
  Readiness; with a tracked file modified, Git Repository warns once; restored, only the Rust Docs warning that was there before remains.
<!-- evidence 2026-10-30 (G6), core 4.4.6 shipped once for this intent and run from PATH. Before the push, commits unpushed: 26 of 28 checks, Update Readiness absent, Git Repository warned once. After gp: 27 of 28 checks, 27 passed, 1 warning (Rust Docs), 0 failed, Git Repository working tree clean and all commits pushed. The gate said in a new shell; core is its own binary, so the age of the shell does not change which core runs, and the wording is now with the deployed core. -->
- [x] G7: doors. nsh-test all passing against the deployed binaries, d 0 failed, cargo doc 0
  warnings for zero-doctor, commits carry Intent: INT-280.
<!-- evidence 2026-10-30 (G7), nsh-test 298 of 298 against the deployed binaries before the source commit, whose files are those same files. d 0 failed. cargo doc for zero-doctor 0 warnings. cargo test for zero-doctor 68 passed. Commits 33ef8a5d 089b10a3 e525e01a 60939fb2 751693da carry Intent: INT-280. -->
