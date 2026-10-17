---
id: 280
date: 2026-10-07
type: fix
title: "Update Readiness gates Omarchy updates on the 0-core tree, which the Omarchy updater never touches"
status: planned
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

- RETIRE THE CHECK. Remove the registry entry (checks.toml:257-264), the probe
  (hardening.rs:151-201), the Probe::UpdateReadiness variant, name map and list entry
  (probe.rs:76,116,150), the dispatch (probes/mod.rs:63) and the live test (hardening.rs:239-249).
- ONE OWNER PER FACT. Git Repository owns the tree and Reboot Needed owns the kernel. Neither
  changes.
- NOTHING ELSE MOVES. d goes from 28 checks to 27. The score formula and forecast are untouched.
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
- Filed, not fixed here: installed_kernels counts every directory in /usr/lib/modules
  (system.rs:185-197) while Omarchy counts only pacman-owned vmlinuz files
  (omarchy-update-restart:14-16).
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

- [ ] G0: recon recorded, each finding with file:line. (a) The probe, its blockers, unread
  branch and messages (hardening.rs:150-201). (b) The registry entry and what its severities
  permit (checks.toml:257-264, definition.rs:175-179). (c) Every consumer of update_readiness or
  its message (fsearch, including ship, nsh and scripts). (d) Omarchy's update path, read only:
  what it reads and writes, and whether any of it touches ~/0-core or ~/.local/bin.
  (e) That reboot_needed reads the same running_kernel and installed_kernels.
- [ ] G1: RED FIRST. At HEAD, d lists Update Readiness. With one tracked file in 0-core modified,
  d shows two warnings for that one change, Git Repository and Update Readiness. Saved as
  evidence before anything changes.
- [ ] G2: REMOVED. The registry entry, probe, variant, name map, list entry, dispatch and live
  test are gone. fsearch finds no live reference (history excepted). cargo build -p zero-doctor
  and cargo test -p zero-doctor pass on the debug build.
- [ ] G3: HISTORY. Run history that names update_readiness still reads after the change, proven on
  the real state directory.
- [ ] G4: ONE OWNER, ON THE DEBUG BUILD. d shows 27 checks and no Update Readiness. With one tracked
  file modified, Git Repository warns once and nothing else warns. Restored, d is clean.
- [ ] G5: INT-222 IS AMENDED BY CITATION. The Decision section above stands, and git shows no
  commit under this intent touching the 222 file.
- [ ] G6: DEPLOYED. One ship at the end. In a new shell, d shows 27 checks and no Update
  Readiness; with a tracked file modified, Git Repository warns once; restored, d is clean.
- [ ] G7: doors. nsh-test all passing against the deployed binaries, d 0 failed, cargo doc 0
  warnings for zero-doctor, commits carry Intent: INT-280.
