---
id: 278
date: 2026-10-06
type: future
title: "Improving Fpatch"
status: planned
tags: [fpatch, nsh, novashell, tooling, dx, safety]
depends_on: []
---

## Vision

Every edit fpatch makes can be shown in full before anything is written, applied only
as the plan that was reviewed, and proven on disk afterwards. A payload that stops
halfway writes nothing.

## The Problem

fpatch.py read in full 2026-10-06, 376 lines. Seven defects, each from the source:

1. THE DRY MODE DOES NOT EXIST. AGENTS.md section 2 Edit says "dry prints the plan and
   writes nothing". fpatch.py has no such mode: patch() and patch_between() print the
   region and write in the same call (lines 266-277, 340-350). The contract describes
   a tool that was never built.
2. A MULTI-EDIT PAYLOAD IS NOT ALL-OR-NOTHING. Each call writes at once, so a payload
   of three patches whose third refuses leaves the first two on disk. "Every check runs
   before the first write" lives in each hand-built payload, not in the tool.
3. patch_between NEVER VERIFIES THE WRITE. Line 277 writes, line 278 prints OK. Only
   patch() reads back, so the mode built for the most fragile edits has the weakest
   check. It also accepts a replacement identical to the span, and non-ASCII markers.
4. THE READ-BACK IN patch() IS TOO WEAK. Line 358 checks after.count(new) >= count. A
   deletion (new is empty) always passes, and a replacement already present elsewhere
   in the file passes even if the write changed nothing. The exact check is: the file
   equals the bytes that were computed.
5. THE WRITE IS NOT ATOMIC. Path.write_text truncates, then writes (lines 277, 350). A
   crash between the two leaves a truncated file, which is why _internal (line 167)
   can only say the file may or may not have changed.
6. count>1 SHOWS ONLY THE FIRST SITE. Line 338 says so. The other sites are replaced
   unseen.
7. NO EDIT TO END OF FILE. patch_between needs an end marker on the line after the
   span, so a span that runs to EOF cannot be expressed -- an intent body under kept
   frontmatter, the INT-145 edge INT-234 names. This body was written by a guarded
   whole-body script for exactly that reason.

Census: fsearch fpatch --all finds no code that imports fpatch. Every caller is a
payload. Internals can change; the documented call form must keep working.

## The Solution

One edit engine, two outcomes.

A Plan collects edits (patch, between) against one or more files. Nothing touches disk
while edits are added. Every existing check runs while collecting, against the content
as it will be after the earlier edits in the same plan, so two edits to one file chain
correctly.

plan.show() prints every edit, every site, the file it lands in, and the seal of the
plan text, then exits 0 having written nothing. This is the dry mode AGENTS.md already
names.

plan.apply(seal) recomputes the whole plan against disk, refuses unless the seal
matches, stages each new file beside its target (same directory, mode bits kept,
fsync), renames them into place, then reads each back and compares byte for byte.

plan.run(argv): no arguments means show; "apply SEAL" means apply. A payload cached at
~/.cache/zero/NAME-SHA.py then serves both steps with the same bytes.

patch() and patch_between() stay, signatures unchanged, as a one-edit plan applied at
once. One engine, no second write path.

The seal is the first 16 hex of sha256 over the uncoloured plan text, the form the
Seal: trailer in AGENTS.md uses. The plan text includes each target path and the
sha256 of its current content, so a file that changes between show and apply changes
the seal, and apply refuses.

## Non-goals

- No CLI. fpatch.py line 18: anchors on a command line are shell syntax again.
- No enforcement that edits use fpatch. That is INT-234, whose body says enforcement
  cannot live inside fpatch.
- No verbatim-text verb in nsh. Transit loss is INT-239, a different layer.
- No new dependencies. Standard library only.
- No rollback across files once renaming has begun. Renames start only after every
  file is staged; if one fails, the tool names which files landed (internal, exit 2).
  The gap is stated, not hidden.
- No change to what patch() prints on a refusal today, except where a gate names it.

## Scope against neighbours

- INT-234 owns WHO must use the primitive. INT-278 owns WHAT the primitive guarantees.
  234 gains from 278 and does not need it first.
- INT-239 owns transit. 278 assumes the payload arrived intact, and the exact read-back
  catches it when it did not.
- INT-233 (lines 91-95) places the transmission gap in another layer. 278 agrees.

## Dependencies

depends_on is empty on purpose: nothing has to finish before this can start.

## Success Criteria

Each gate is watched failing on the current fpatch.py before it is watched passing.
Tests live in zero/scripts/dev/test_fpatch.py, inside the zero/scripts entry that
docs/TREE.md already names, and run with python3 -m unittest, each in a mktemp tree.
No test touches a real file.

- [ ] G0: recon recorded. The seven defects above, each with its fpatch.py line, read
  2026-10-06. One answer read before code: does INT-266 compute the seal anywhere in
  code (fsearch refs/notes/seals)? If yes, G9 matches it byte for byte. If no, fpatch
  is the one definition and this file says so.
- [ ] G1: RED FIRST. test_fpatch.py runs against the unchanged fpatch.py and fails one
  test per defect 1-7. That output is saved as evidence before fpatch.py changes.
- [ ] G2: exact read-back. patch() compares the file to the computed bytes. Red: a
  deletion whose write is sabotaged reports verified today. Green: refused, exit 1.
- [ ] G3: patch_between verifies on disk, refuses a replacement identical to the span,
  and refuses a non-ASCII marker. Red: all three pass today.
- [ ] G4: atomic write that keeps mode bits. Red: a write interrupted mid-way leaves a
  truncated file today. Green: the original survives whole. An executable file stays
  executable after an edit -- the naive temp-file fix breaks that, so it is tested on
  purpose.
- [ ] G5: show writes nothing. Every target has the same sha256 and mtime before and
  after show, every site of a count>1 edit is printed, exit 0.
- [ ] G6: every check before the first write. A three-edit plan whose third edit
  refuses leaves every file unchanged. Red: three patch() calls leave two edits on
  disk today. The refusal leads with "No changes written to any file" and names the
  edit that refused (edit 3 of 3).
- [ ] G7: apply needs the reviewed seal. Wrong seal: refused, nothing written. Target
  changed after show: seal differs, refused. Right seal: every file applied and
  verified, one OK line per file.
- [ ] G8: edit to end of file. patch_between(path, start, None, new_lines) replaces from
  the start marker to EOF; an ambiguous start is still refused.
- [ ] G9: the seal is stable. Same plan gives the same 16 hex with FPATCH_COLOR=0 and
  FPATCH_COLOR=1; one byte changed in a target gives a different seal.
- [ ] G10: one engine, old form intact. The AGENTS.md section 2 Edit example runs
  unchanged against a stand-in with the same result as before, and fpatch.py has one
  write site.
- [ ] G11: wired in, not configured. nsh-test runs test_fpatch.py. A deliberately broken
  test turns nsh-test red; restored, it turns green.
- [ ] G12: the docs say what ships. The fpatch.py docstring shows the plan form.
  AGENTS.md is Christian's file: proposed text for section 2 Edit is handed to him, and
  the gate closes when he applies it or declines it.
- [ ] G13: doors. One ship at the end; nsh-test all passing against the deployed
  binary in a new shell; d 0 failed; commits carry Intent: INT-278, and Seal: where a
  reviewed plan produced them.
