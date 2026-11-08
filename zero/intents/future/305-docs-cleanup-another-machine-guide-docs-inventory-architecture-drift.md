---
id: 305
date: 2026-11-07
type: future
title: "Docs Cleanup: Another-machine guide, docs inventory, architecture drift"
status: planned
tags: [docs]
---

## Vision
The docs say what is true about Project 0 today. A reader on a new machine,
or a new agent, can find the right doc, trust it, and follow it without
asking Christian.

## The Problem
Three gaps were found in the read-only study of 2026-11-05/06:
1. There was no guide for working from a second machine.
2. There is no inventory of the docs, and docs/NOVASHELL.md and
   docs/public/NOVASHELL.md are copies (they differ only by a final newline).
3. The architecture docs have drifted from the code (for example, the parser
   RFC describes a logos hybrid that the current lexer does not use).

## The Solution
One finding at a time, docs only, each through a reviewed fpatch Plan:
1. docs/WORKING-FROM-ANOTHER-MACHINE.md, linked from README.md. Done:
   b18b4703 (Seal 2c09fe794f03f54f) and 4c20a06d (Seal 5ca383c7c6882881).
2. A docs inventory: every tracked doc outside zero/intents with its purpose
   and whether it is current, and a ruling on the NOVASHELL.md copy (keep one,
   or generate one from the other).
3. Architecture drift: list each claim in the architecture docs that the code
   no longer matches, with file and line, then fix the doc or mark it history.

## Rules
- Docs only. No code, no build or tool change, no new tooling.
- Nothing is applied without Christian's explicit approval. No sudo.
- Every edit goes through a reviewed fpatch Plan with a Seal he has seen.
- Recon is read-only and runs on Christian's tree, not a stand-in copy.
- Nothing is pushed without Christian's say.

## Out of scope
- Code fixes found while reading docs: each becomes its own finding.
- The rowan / parser evaluation (its own intent if Christian wants one).
- Unused smithay dependencies, CI, tests and the commands/mod.rs split.

## Success Criteria
- [x] G1 Another-machine guide written and linked from README.md, with the
      zero-gate and ship gaps fixed. Proof: commits b18b4703 and 4c20a06d.
- [ ] G2 Docs inventory written, with each doc's path, purpose and status
      (current, stale or history). Proof: the Plan Seal and commit.
- [ ] G3 Christian rules on docs/NOVASHELL.md vs docs/public/NOVASHELL.md,
      and the ruling is applied. Proof: the RULED line and the commit.
- [ ] G4 Architecture drift list: each stale claim with doc path, line and the
      code it contradicts. Proof: the recon output.
- [ ] G5 Each drift item fixed or marked history. Proof: the Plan Seal and commit.
- [ ] G6 Doors after any ship: nsh-test all passing, d 0 failed, core intent
      validate all valid, git status clean, then cicomplete 305.
      Proof: the door output and the close commit.
