---
id: 311
date: 2026-11-10
type: future
title: "A doors command that does not ship"
status: in-progress
tags: [intent, ledger, nsh]
---

## Vision
One command runs the doors a person retypes at the end of an intent, and
prints what came back. It does not ship. It does not reload. d stays the
doctor's door. No code is written before the open gates below are ruled.

## The Problem
Re-read 2026-11-10 against 57ca26ea. There is no command that runs the doors.
Each intent's last gate names them again: cargo test -p core, cargo test -p
novashell, nsh-test after cargo build -p novashell -p nsh-test, core intent
validate, and d. The pairing matters. nsh-test's crate needs the registry
feature that a novashell build turns on, so `cargo test -p novashell -p
nsh-test` is one cargo invocation, not two.

ship copies release binaries into ~/.local/bin and has no rollback. A doors
command that called ship would deploy. reload starts the new nsh. d is core
doctor run. It is not a ledger count, and a copy of its result in another
command can disagree with it.

## The Solution
The command prints the results of the tests and of validate. What is not
chosen is its name, whether it is a core subcommand or a script, and whether
zero-doctor's tests are in the set. Each of those is a gate. A ruling is a
RULED line in this file, dated, in his words. Declining a gate, with the
reason, closes it. No code is written before G1, G2, and G3 have those lines.

## Success Criteria
- [x] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: no command runs this
      set, and ship is not a test. Proof: a dated line in this file saying the
      recon stands, or the correction.
<!-- evidence: 2026-11-10. Re-read against 25d085c7. The recon stands. There is no zero/scripts/doors. d is still core doctor run at novashell commands/mod.rs. novashell Cargo.toml turns on zero-core's registry feature and nsh-test's Cargo.toml does not, so the pairing still matters. ship is still the binary at zero/tools/ship. No command runs this set. -->
- [x] G1 The name. Christian rules the word a person types. Proof: the RULED
      line.
RULED 2026-11-10 (Christian): "I approve on your proposals." The name is doors.
- [x] G2 The home. Christian rules one place. The options left open: a core
      subcommand, or a script under zero/scripts. The ruling says what that
      home can start, and that it does not call ship and does not reload.
      Proof: the RULED line.
RULED 2026-11-10 (Christian): "I approve on your proposals." The home is a script at zero/scripts/doors, not a core subcommand. It starts cargo test -p core, cargo test -p novashell -p nsh-test, the nsh-test binary, and core intent validate. It must not call ship or reload.
- [x] G3 Whether zero-doctor is in the set. Christian rules yes or no. d stays
      a separate door either way. Proof: the RULED line.
RULED 2026-11-10 (Christian): "I approve on your proposals." zero-doctor is no. The 95 tests stay out. d stays core doctor run.
- [ ] G4 The run, only after G1, G2, and G3 are ruled. The command runs
      cargo test -p core, cargo test -p novashell -p nsh-test as one
      invocation, nsh-test, and core intent validate, and prints their
      results. It does not ship. It does not reload. Shown failing first,
      under its own plan Seal, if the ruled set is short a command or if the
      command reaches ship. Then the same test passes. Proof: the red output,
      then the green output, each under its plan Seal.
- [ ] G5 Doors, after any ship of this command. cargo test -p core passes.
      cargo test -p novashell passes. nsh-test passes after cargo build -p
      novashell -p nsh-test. core intent validate reports every intent valid.
      d reports 0 failed. ship, then reload, and the deployed artifact is what
      was watched. Proof: the door output.

## Out of scope
- Calling ship or reload from this command.
- Replacing d. The doctor keeps its own door.
- A second copy of d's health count inside this command's output.

