---
id: 310
date: 2026-11-10
type: future
title: "One reader for Friday and the intent screen"
status: planned
tags: [intent, ledger, friday, nsh]
---

## Vision
Friday and the nsh `it` screen ask the ledger the way core intent list does.
They do not keep a private list of folders. `it` gives the terminal back when
it fails. Findings stay with the findings screen. No code is written before
the gates below are ruled.

## The Problem
Re-read 2026-11-10 against 57ca26ea. The study's Friday and `it` sites are
still the private readers. Two line numbers moved.

phase2::plan (friday/phase2.rs:200) says it reads planned and in-progress
intents. The loop at line 212 reads the directories `future` and `active`.
`active` is not a ledger directory. in-progress is. Status is a
`contains("status: planned")` on the whole file.

planning.rs joins HOME to `0-core/intents/future` in two places. The stale
check is at 458-459. active_intents is at 1013-1014, and its comment says it
finds status in-progress inside intents/future. Neither path asks paths.rs,
and neither reads in-progress/.

seed_knowledge (friday/mod.rs:170) counts files whose text contains
"status: complete". The folder list at 176-185 is complete, decisions,
experiments, philosophy, future, cancelled, deferred, and incidents.
`deferred` is not a directory. planned and in-progress are not in the list.

load_all is the ledger's reader. None of these three call it. folders.rs is
the one list of directories. They do not ask it either.

`it` is an nsh REPL arm at novashell main.rs:3143. The study cited 3127. The
findings arm now sits above it, which is why the number moved. The screen is
intent_tui.rs. load_intents (460) reads only future, in-progress, and complete
(468). It does not read planned, cancelled, or the record directories.
run_intent_tui (53) enables raw mode and enters the alternate screen before
Terminal::new. If that call fails it disables raw mode (60-63) and returns
without leaving the alternate screen. A panic inside the loop does not run
the leave at 66-68. Findings are a separate screen, built by INT-307. This
intent does not fold them into `it`.

## The Solution
No private folder list is kept, and no second parser of an intent file is
added. Friday and `it` call the reader core intent list already uses. Each
gate is a question for Christian to rule. A ruling is a RULED line in this
file, dated, in his words. Declining a gate, with the reason, closes it. No
code is written before G1 and G2 have those lines.

## Success Criteria
- [ ] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: phase2.rs:212 names
      future and active, planning.rs:459 and 1014 join HOME to future,
      seed_knowledge's list is mod.rs:176, and it loads three directories at
      intent_tui.rs:468. Proof: a dated line in this file saying the recon
      stands, or the correction.
- [ ] G1 Friday. Christian rules the cut. One shape to accept, change, or
      reject: phase2::plan, the two planning.rs readers, and seed_knowledge
      ask load_all and folders. The private lists, including active, deferred,
      and the HOME joins, go. A quoted "status: planned" in a body is not a
      second status. Proof: the RULED line.
- [ ] G2 `it`. Christian rules the folders and the failure path. One shape to
      accept, change, or reject: load_intents uses the same reader as core
      intent list, including planned, cancelled, and the record directories.
      Findings stay out. The terminal is restored on failure, including when
      Terminal::new fails and when the loop panics. Proof: the RULED line.
- [ ] G3 Red, then the readers, only after G1 and G2 are ruled. Shown failing
      first, under its own plan Seal: Friday misses an in-progress intent, and
      `it` leaves the terminal raw when the screen fails to open. Then the
      same tests pass. The suite is cargo test -p core and cargo test -p
      novashell. Proof: the red output, then the green output, each under its
      plan Seal.
- [ ] G4 Doors, after any ship. cargo test -p core passes. cargo test -p
      novashell passes. nsh-test passes after cargo build -p novashell -p
      nsh-test. core intent validate reports every intent valid. d reports 0
      failed. ship, then reload, and the deployed artifact is what was watched.
      Proof: the door output.

## Out of scope
- A second parser of an intent file.
- Showing findings inside `it`. INT-307 owns that screen.
- Changing what Friday says beyond what the one reader returns.

## Open for Christian
- G1, which Friday sentences may change when the reader changes.
- G2, which record directories `it` shows, and how a failed open restores the
  terminal.
