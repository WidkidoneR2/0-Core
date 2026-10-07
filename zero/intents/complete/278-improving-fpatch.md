---
id: 278
date: 2026-10-06
type: future
title: "Improving Fpatch"
status: complete
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

- [x] G0: recon recorded. The seven defects above, each with its fpatch.py line, read
  2026-10-06. One answer read before code: does INT-266 compute the seal anywhere in
  code (fsearch refs/notes/seals)? If yes, G9 matches it byte for byte. If no, fpatch
  is the one definition and this file says so.
<!-- evidence: demonstrated 2026-10-06: fpatch.py read in full (376 lines); the seven defects and their lines are in The Problem. The seal question answered 2026-10-07: trace.rs:184-191 seal_digest is the first 16 hex of sha256 over the note bytes, and check_seal (trace.rs:211) re-checks a Seal trailer against refs/notes/seals. So G9 must match it. -->
- [x] G1: RED FIRST. test_fpatch.py runs against the unchanged fpatch.py and fails one
  test per defect 1-7. That output is saved as evidence before fpatch.py changes.
<!-- evidence: demonstrated 2026-10-06: the INT-278 payload ran test_fpatch.py against the unchanged fpatch.py before writing: Ran 17 tests, FAILED (failures=8, errors=5). The four that passed are regression guards by design (mode bits, symlink, documented form, refusal). Landed in a7f87da1. -->
- [x] G2: exact read-back. patch() compares the file to the computed bytes. Red: a
  deletion whose write is sabotaged reports verified today. Green: refused, exit 1.
<!-- evidence: test_fpatch.py Defect4ExactReadBack.test_deletion_is_verified: red on the old fpatch (a deletion whose write was corrupted reported verified), green in a7f87da1 (exit 2, no OK line). -->
- [x] G3: patch_between verifies on disk, refuses a replacement identical to the span,
  and refuses a non-ASCII marker. Red: all three pass today.
<!-- evidence: test_fpatch.py Defect3BetweenGuards (verifies on disk, no-op refused, non-ASCII marker refused): all three red on the old fpatch, green in a7f87da1. -->
- [x] G4: atomic write that keeps mode bits. Red: a write interrupted mid-way leaves a
  truncated file today. Green: the original survives whole. An executable file stays
  executable after an edit -- the naive temp-file fix breaks that, so it is tested on
  purpose.
<!-- evidence: test_fpatch.py Defect5Atomic.test_failed_write_leaves_original_whole: red on the old fpatch (half-written file), green in a7f87da1 (original whole, no temp file left). test_mode_bits_kept and test_symlink_is_followed_not_replaced guard the regressions. a7f87da1. -->
- [x] G5: show writes nothing. Every target has the same sha256 and mtime before and
  after show, every site of a count>1 edit is printed, exit 0.
<!-- evidence: test_fpatch.py Defect1Dry.test_dry_writes_nothing (content unchanged, exit 0) and Defect6EverySite.test_count_two_shows_both_sites (line 10 shown): red on the old fpatch, green in a7f87da1. Content is asserted; mtime is not -- show never reaches a write call (_plan_show does not call _stage). Live: four show runs on 2026-10-06/07 wrote nothing before their apply. -->
- [x] G6: every check before the first write. A three-edit plan whose third edit
  refuses leaves every file unchanged. Red: three patch() calls leave two edits on
  disk today. The refusal leads with "No changes written to any file" and names the
  edit that refused (edit 3 of 3).
<!-- evidence: test_fpatch.py Defect2AllOrNothing.test_third_edit_refused_writes_nothing: both files unchanged, refusal names edit 3 of 3. a7f87da1. -->
- [x] G7: apply needs the reviewed seal. Wrong seal: refused, nothing written. Target
  changed after show: seal differs, refused. Right seal: every file applied and
  verified, one OK line per file.
<!-- evidence: test_apply_needs_the_seal and test_seal_moves_when_the_file_moves, a7f87da1. Live: AGENTS.md shown at Seal 7f5dd05e46b8c357 then applied and verified (14b667b4); nsh-test main.rs at 9dc3a3e8c475b7fc (2ae941a3); the two-file sealed-text fix at fb8c0ffade5c4ee8. -->
- [x] G8: edit to end of file. patch_between(path, start, None, new_lines) replaces from
  the start marker to EOF; an ambiguous start is still refused.
<!-- evidence: test_fpatch.py Defect7ToEndOfFile.test_between_to_eof: red on the old fpatch (TypeError on a None marker), green in a7f87da1. -->
- [x] G9: the seal is stable. Same plan gives the same 16 hex with FPATCH_COLOR=0 and
  FPATCH_COLOR=1; one byte changed in a target gives a different seal.
<!-- evidence: test_seal_ignores_colour and test_seal_moves_when_the_file_moves (a7f87da1). Matched to trace.rs: show printed less than it sealed (no summary lines), so no note could ever re-check. Fixed in the commit after 2ae941a3 (fpatch: show prints the exact text the Seal covers): SealMatchesTrace asserts the printed text equals p.text() and hashes to the printed Seal with the same definition as trace.rs:184-191. Ran 18 tests, OK. -->
- [x] G10: one engine, old form intact. The AGENTS.md section 2 Edit example runs
  unchanged against a stand-in with the same result as before, and fpatch.py has one
  write site.
<!-- evidence: OldFormStillWorks (documented call form, refusal writes nothing), a7f87da1. Every write goes through _plan_apply (_stage then os.replace); patch() and patch_between() are one-edit plans. The existing payload form ran unchanged for every edit on 2026-10-06/07. -->
- [x] G11: wired in, not configured. nsh-test runs test_fpatch.py. A deliberately broken
  test turns nsh-test red; restored, it turns green.
<!-- evidence: commit 2ae941a3. Red: with the pre-278 fpatch.py restored from a7f87da1~1, /tmp/red.txt:105 fpatch_tests_pass failed with the unittest failures. Green: restored, 227/227. Deployed nsh-test (2026-10-06 23:10:48) 227/227. -->
- [x] G12: the docs say what ships. The fpatch.py docstring shows the plan form.
  AGENTS.md is Christian's file: proposed text for section 2 Edit is handed to him, and
  the gate closes when he applies it or declines it.
<!-- evidence: fpatch.py docstring shows the Plan form (a7f87da1, p.text() added after 2ae941a3). AGENTS.md section 2 Edit applied by Christian: 14b667b4. -->
- [x] G13: doors. One ship at the end; nsh-test all passing against the deployed
  binary in a new shell; d 0 failed; commits carry Intent: INT-278, and Seal: where a
  reviewed plan produced them.
<!-- evidence: one ship 2026-10-06: nsh-test 2.0.1 deployed 23:10:48, after 2ae941a3 at 23:10:38; deployed suite 227/227 in a new shell; d 0 failed (26/28, the 2 warnings were uncommitted and unpushed work). Commits a7f87da1, 14b667b4, 2ae941a3 carry Intent: INT-278. Seal: declined: (1) the plan text was not kept under refs/notes/seals, so a trailer would read NoNote in core intent trace; (2) p.text() now makes keeping it possible, for the next intent that wants it. -->
