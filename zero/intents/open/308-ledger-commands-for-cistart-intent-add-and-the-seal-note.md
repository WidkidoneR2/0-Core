---
id: 308
date: 2026-11-10
type: future
title: "Ledger commands for cistart, intent add, and the seal note"
status: planned
tags: [intent, ledger, nsh, git]
---

## Vision
Three ledger commands do the thing a person already has to do by hand. cistart
moves a future intent into in-progress and says so if the move fails. core
intent add can be given the category, title, status, and tags without a
terminal, and the wizard stays when there is one. core intent note stores the
plan under refs/notes/seals and prints the seal. No code is written before the
gates below are ruled.

## The Problem
Re-read 2026-11-10 against 57ca26ea. The study cited start's replace at
mod.rs:1003 and the rename at 1014. Those lines moved. The behaviour did not.

start is zero/engine/src/domains/intent/mod.rs:909. It rewrites the whole file
(1007-1010): every "status: planned", "status: future", and "status: deferred"
becomes "status: in-progress". A body that quotes one of those strings changes
too. If the path contains /intents/future/, it renames the file into
in-progress/ (1013-1018). There is no create_dir for that directory. The rename
error is discarded: `let _ = fs::rename`. On a tree where in-progress is
missing, the status is already written and the file stays in future/.

add is mod.rs:2746. With no terminal it returns at 2764 and names core intent
new. new_intent (2976) takes a category, a template, and a title. Status and
tags come from TEMPLATES, not from the caller. create_in (2917) is the one
writer. A non-interactive add that set status or tags would still have to end
there.

There is no core intent note command. seal_digest (trace.rs:228) is the first
16 hex of sha256 of the plan bytes. The note is stored by hand, with
`git notes --ref=seals add --no-stripspace -F`, or with hash-object -w and
notes add -C. stripspace makes a true plan read Broken (F-0023, F-0063). The
command, if built, must not push.

## The Solution
No extra behaviour is chosen here. The three commands are the ones named in
the gates. Each gate is a question or a step for Christian to rule. A ruling
is a RULED line in this file, dated, in his words. Declining a gate, with the
reason, closes it. No code is written before G1, G2, and G3 have those lines.

## Success Criteria
- [ ] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: the whole-file status
      replace is mod.rs:1007, the discarded rename is 1018, add refuses at
      2764, new_intent does not take status or tags, create_in is the one
      writer, and there is no note command. Proof: a dated line in this file
      saying the recon stands, or the correction.
- [ ] G1 cistart. Christian rules the move. One shape to accept, change, or
      reject: create in-progress if it is missing, replace status only on the
      frontmatter line, and report a rename that did not happen instead of
      discarding it. Proof: the RULED line, and a test that watches a missing
      directory fail first, then pass.
- [ ] G2 core intent add without a terminal. Christian rules the flags. One
      shape to accept, change, or reject: --category, --title, --status, and
      --tags, calling create. The wizard stays when stdin is a terminal. A bad
      flag writes nothing. create_in stays the one writer. Proof: the RULED
      line.
- [ ] G3 core intent note. Christian rules the command. One shape to accept,
      change, or reject: `core intent note <plan> <commit>` stores the note
      with --no-stripspace -F, or with hash-object -w and notes add -C, prints
      seal_digest, refuses a missing commit or a missing plan, and does not
      push. Proof: the RULED line.
- [ ] G4 Red, then the commands, only after G1, G2, and G3 are ruled. Shown
      failing first, under its own plan Seal: a missing in-progress directory,
      a body that quotes "status: planned", add with the four flags and no
      terminal, and a note whose bytes hash to the printed seal. Then the same
      tests pass. The suite is cargo test -p core. Proof: the red output, then
      the green output, each under its plan Seal.
- [ ] G5 Doors, after any ship. cargo test -p core passes. cargo test -p
      novashell passes. nsh-test passes after cargo build -p novashell -p
      nsh-test. core intent validate reports every intent valid. d reports 0
      failed. ship, then reload, and the deployed artifact is what was watched.
      Proof: the door output.

## Out of scope
- Rewriting intents that were already moved by the old cistart.
- A second seal hash. seal_digest stays the one definition.
- A command that pushes refs/notes/seals.

## Open for Christian
- G1, how cistart reports a rename that did not happen.
- G2, the exact flags on core intent add.
- G3, which of the two note forms the command uses.
