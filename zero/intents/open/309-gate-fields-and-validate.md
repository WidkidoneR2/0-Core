---
id: 309
date: 2026-11-10
type: future
title: "Gate fields and validate"
status: planned
tags: [intent, ledger, gates]
---

## Vision
A gate keeps its checkbox. Under the box, the question, the ruling, and the
receipt are fields a reader can wrap. Old files stay as they are. validate can
name an old shape without failing the 344 v1 intents. No code is written
before the open gates below are ruled.

## The Problem
Re-read 2026-11-10 against 57ca26ea. The study's gate lines still match.

read_gates (gates.rs:143) is the one reader. A gate is a checkbox, a legacy
emoji, or a signed deferral. live_lines (gates.rs:106) drops the body of an
HTML comment that spans lines. evidence_follows (gates.rs:131) accepts the
receipt only when the next non-blank, non-indented line starts with `<!--` and
contains "evidence". A RULED line is not indented, so it ends that scan, and
RULED is not a token the reader knows. check_completion (gates.rs:228) refuses
an open box. It does not read the evidence flag. A ticked box with no receipt
completes. That is why a v1 file that completes today must still complete.

validate_issues (intent/mod.rs:446) checks frontmatter, status against the
directory, the id against the filename, and depends_on. It does not require
Vision, The Problem, The Solution, or Success Criteria. A file whose body is
still the template sentence is valid. The study counted, on 2026-11-10, 11
placeholder bodies, 20 copies of the INT-158 "delete this comment" boilerplate,
and 8 `[~]` gates. INT-294 R1 retired `[~]` for new intents. Those counts are
not re-measured in this filing. The reader that would name them does not exist
yet.

## The Solution
The shape of format 2 is ruled. What is not ruled is when a new file is born
in that shape, and whether cicomplete requires a receipt on a v2 gate that is
done. Each remaining gate is a question for Christian. A ruling is a RULED
line in this file, dated, in his words. Declining a gate, with the reason,
closes it. No code is written before G2 and G3 have those lines.

## Success Criteria
- [ ] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: evidence_follows is
      gates.rs:131, live_lines is 106, check_completion is 228, and
      validate_issues does not require a section. Proof: a dated line in this
      file saying the recon stands, or the correction.
- [ ] G1 The format. The shape is ruled. The code that reads it is not. Proof:
      the RULED line, and, when the reader exists, a v2 file whose evidence
      wraps and whose ruling is not hidden from the reader.
RULED 2026-11-10 (Christian): format 2 keeps the checkbox. Under each `- [ ] G<n>` box sit indented keys `ask:`, `ruling:`, and `evidence:`, which may wrap by indentation. A file opts in with `format: 2` in the frontmatter. There is no batch rewrite of the 344 v1 files.
- [ ] G2 The reader. Christian rules the split. One shape to accept, change,
      or reject: the gate reader keeps a wrapped receipt, and a RULED line no
      longer hides the evidence that follows it. A v1 file that completes
      today still completes. Turning the new check on for every old file is
      not this gate. Proof: the RULED line, and a v1 fixture that still
      completes.
- [ ] G3 validate. Christian rules what it prints. One shape to accept, change,
      or reject: validate lists old-shape issues -- placeholder bodies, the
      INT-158 boilerplate, `[~]` gates, and a missing section on a new file --
      and exits 0 for a v1 file. A format 2 file is judged by the v2 rules.
      Proof: the RULED line.
- [ ] G4 Whether create_in writes format 2 by default, and from when. Christian
      rules it. Until that line exists, create_in keeps writing v1. Proof: the
      RULED line.
- [ ] G5 Whether cicomplete requires evidence on a v2 gate that is done.
      Christian rules it. A decline, with the reason, closes the gate. Proof:
      the RULED line.
- [ ] G6 Red, then the reader and validate, only after G2, G3, G4, and G5 are
      ruled. Shown failing first, under its own plan Seal: a wrapped evidence
      block that the reader drops, a RULED line that hides a receipt, and a v1
      file that validate would fail if the old-shape list were a hard error.
      Then the same tests pass. The suite is cargo test -p core. Proof: the
      red output, then the green output, each under its plan Seal.
- [ ] G7 Doors, after any ship. cargo test -p core passes. cargo test -p
      novashell passes. nsh-test passes after cargo build -p novashell -p
      nsh-test. core intent validate reports every intent valid. d reports 0
      failed. ship, then reload, and the deployed artifact is what was watched.
      Proof: the door output.

## Out of scope
- A batch rewrite of the v1 intents.
- Replacing the checkbox. Format 2 keeps it.
- Failing d's Intent Ledger check on an old file.

## Open for Christian
- G2, how a wrapped receipt and a RULED line are read, without changing v1.
- G3, which old-shape issues validate lists.
- G4, whether create_in writes format 2 by default, and from when.
- G5, whether cicomplete requires evidence on a v2 gate that is done.
