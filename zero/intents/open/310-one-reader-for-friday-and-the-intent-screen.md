---
id: 310
date: 2026-11-10
type: future
title: "One reader for Friday and the intent screen"
status: complete
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
- [x] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: phase2.rs:212 names
      future and active, planning.rs:459 and 1014 join HOME to future,
      seed_knowledge's list is mod.rs:176, and it loads three directories at
      intent_tui.rs:468. Proof: a dated line in this file saying the recon
      stands, or the correction.
      ruling: RULED 2026-11-15 (Christian): re-read against 0bef81a7. Those folder lists are gone. phase2::open_intent_rows uses ledger::live. planning.rs and seed_knowledge use ledger::load. seed_knowledge still counts a record whose status is complete. it load_intents uses ledger::load and keeps planned and in-progress. Terminal::new failure disables raw mode and does not leave the alternate screen. Friday and it do not call gates::read_gates.
      evidence: the ruling line above.
- [x] G1 Friday. Christian rules the cut. One shape to accept, change, or
      reject: phase2::plan, the two planning.rs readers, and seed_knowledge
      ask load_all and folders. The private lists, including active, deferred,
      and the HOME joins, go. A quoted "status: planned" in a body is not a
      second status. Proof: the RULED line.
      ruling: RULED 2026-11-15 (Christian): keep Friday on ledger::with_status and ledger::live. Drop records from seed_knowledge's complete count. A body quote is not a status.
      evidence: the ruling line above.
- [x] G2 `it`. Christian rules the folders and the failure path. One shape to
      accept, change, or reject: load_intents uses the same reader as core
      intent list, including planned, cancelled, and the record directories.
      Findings stay out. The terminal is restored on failure, including when
      Terminal::new fails and when the loop panics. Proof: the RULED line.
      ruling: RULED 2026-11-15 (Christian): it shows planned and in-progress only. Records and findings stay out. A Drop guard restores raw mode and leaves the alternate screen when Terminal::new fails and when the loop panics.
      evidence: the ruling line above.
      ruling: RULED 2026-11-15 (Christian): Friday shows one display-only line per in-progress intent, INT-310 2/5 gates, read from gates::read_gates. No writes. cicomplete stays the judge.
- [x] G3 Red, then the readers, only after G1 and G2 are ruled. Shown failing
      first, under its own plan Seal: Friday misses an in-progress intent, and
      `it` leaves the terminal raw when the screen fails to open. Then the
      same tests pass. The suite is cargo test -p core and cargo test -p
      novashell. Proof: the red output, then the green output, each under its
      plan Seal.
      ruling: RULED 2026-11-15 (Christian): red first, then the shared reader.
      evidence: red commit 87186685, core 0 passed 3 failed, novashell 0 passed 1 failed. Green commit d14203fd: cargo test -p core 298 passed; cargo test -p novashell 263 passed. Rulings commit 2af3e72f.
- [x] G4 Doors, after any ship. cargo test -p core passes. cargo test -p
      novashell passes. nsh-test passes after cargo build -p novashell -p
      nsh-test. core intent validate reports every intent valid. d reports 0
      failed. ship, then reload, and the deployed artifact is what was watched.
      Proof: the door output.
      ruling: RULED 2026-11-15 (Christian): doors passed after ship core and ship nsh.
      evidence: laptop 2026-11-15. Rulings 2af3e72f, red 87186685, green d14203fd. Seal notes efa08eb3. ship core and ship nsh, then reload. Banner 304/304, shell 100%. doors 4 passed 0 failed: core pass 0.2s, shell pass 2.0s, nsh-test pass 133.2s, validate pass 0.0s.

## Out of scope
- A second parser of an intent file.
- Showing findings inside `it`. INT-307 owns that screen.
- Changing what Friday says beyond what the one reader returns.

## Open for Christian
Ruled 2026-11-15. G1 keeps the ledger. G2 is planned and in-progress, with a Drop guard. The gate line is display-only. G4 closed by the laptop doors.

## Versions
- core 4.9.0 -> 4.10.0 (minor)
- nsh 5.6.0 -> 5.7.0 (minor)
