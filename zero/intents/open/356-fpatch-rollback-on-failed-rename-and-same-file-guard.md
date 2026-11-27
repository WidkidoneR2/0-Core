---
id: 356
date: 2026-11-27
type: future
title: "fpatch rollback on a failed rename and a same-file guard"
status: in-progress
tags: [fpatch, safety, dev]
depends_on: []
format: 3
---

## Vision
A multi-file fpatch plan writes every edit or none. If a
rename fails partway, every file already written is put
back to its preimage, bytes and mode bits, and fpatch says
so. A plan that names one file twice through a symlink or a
hard link is refused before any write.

## The Problem
Two holes in zero/scripts/dev/fpatch.py break the every
edit or none contract. Measured 2026-11-27 against
origin/main 0ddb3118. python3 -m unittest test_fpatch was
22 passed. The probes below were run against that file.

1. A failed rename leaves the earlier files written.
   _plan_apply stages every changed file, then renames them
   into place one by one (fpatch.py, the landed loop near
   line 485). On an exception it removes the staged files
   not yet renamed and raises RuntimeError, which _guard
   turns into exit 2. Files already renamed stay written.
   No preimage is kept, so nothing restores them. Plan of
   a.txt alpha to ALPHA and b.txt beta to BETA, with the
   second os.replace call made to raise OSError, exited 2
   with:

       rename failed. Written: ['.../a.txt'].
       Not written: ['.../b.txt']. (injected)

   a.txt was ALPHA, b.txt was beta.

2. A symlink and its real path in one plan corrupt the file.
   _build keys files by the path string (near line 399), so
   real.txt and link.txt are read and computed apart.
   _plan_apply then maps both to the same realpath (line
   475) and renames twice. real.txt one\ntwo\n with one to
   ONE via real.txt and two to TWO via link.txt exited 2
   with WRITE NOT VERIFIED and the recovery line "Restore it
   with git checkout -- <file>". real.txt was one\nTWO\n.
   The first edit was lost.

3. A hard link pair in one plan splits the link.
   x.txt and y.txt share one inode, one\ntwo\n. Plan of
   one to ONE via x.txt and two to TWO via y.txt returned
   success. x.txt was ONE\ntwo\n, y.txt was one\nTWO\n, and
   they no longer share an inode. Neither name holds both
   edits.

## The Solution
G1 adds three tests in zero/scripts/dev/test_fpatch.py and
does not edit fpatch.py. They fail on the unmodified module.
The existing 22 tests stay green.

G2 changes only zero/scripts/dev/fpatch.py. Before the
first rename, _plan_apply keeps each target preimage, text
and mode bits, in memory. If any rename fails, every file
already renamed is restored from its preimage and read
back, staged temp files are removed, and fpatch exits 2
naming each file as restored or not restored. _build
refuses with exit 1, before any write and for show and
apply alike, a plan where two different path strings
resolve to the same file by realpath or by the same st_dev
and st_ino. Two edits on the same path string still chain.
The Seal equals sha256 of the uncolored plan text, first
16 hex.

G3 is the laptop: python3 -m unittest -v test_fpatch,
doors, and d are green.

## Limits
- In scope: zero/scripts/dev/fpatch.py and
  zero/scripts/dev/test_fpatch.py.
- Out of scope: the exit code for a missing or unwritable
  path (today 2, the contract says 1); restore after a
  read-back mismatch; a hard link split by a one-file
  patch; on-disk backup files; TOCTOU; a protected-path
  denylist; FPATCH_COLOR; nsh-test harness changes.
- How deep: three gates. G1 is tests only. G2 is an
  in-memory preimage with restore on a failed rename, and
  a same-file refusal in _build. No new crate. No new
  dependency.
- Stop and ask: a fix seems to need a file outside In
  scope; the bytes a successful plan writes would have to
  change; an existing test goes red for a reason other than
  the three new cases.
- Done means: G1, G2, and G3 ticked with evidence.
- Must not change: what a successful plan writes, the
  plan text and Seal of a plan with no same-file pair, the
  22 existing tests' behaviour, and any file outside the
  two scripts above.

## Success Criteria
- [x] G1 red first: three new tests in test_fpatch.py fail
      on unmodified fpatch.py. (a) a two-file plan, a.txt
      mode 0755 alpha to ALPHA and b.txt beta to BETA,
      with only the second os.replace call raising
      OSError, exits 2, leaves a.txt byte-identical to its
      preimage with mode 0755, leaves b.txt unchanged, and
      leaves no temp file in the directory. (b) a plan
      that edits real.txt and a symlink to it is refused
      with exit 1, real.txt is unchanged, and link.txt is
      still a symlink. (c) a plan that edits two hard
      links to one file is refused with exit 1, both names
      are unchanged, and they still share an inode. The
      existing 22 tests stay green.
      ask: the three new tests fail on today's fpatch.py, and the existing 22 pass
      ruling: confirmed on int-356 c750f4c6
      evidence: c750f4c6 -- 22 existing passed, 3 new tests failed as expected, none errored
- [x] G2 fixes in fpatch.py only. All new tests and all
      22 existing tests pass: python3 -m unittest -v
      test_fpatch exits 0. Two edits on the same path
      string still chain. The Seal matches sha256 of the
      uncolored plan text, first 16 hex.
      ask: the new tests and the existing 22 pass, and the Seal still matches the plan text
      ruling: confirmed on laptop bbaa823b
      evidence: 41f70cd2 and bbaa823b -- fpatch.py only, 25 of 25 pass, same-path chain kept, Seal unchanged for a plan with no same-file pair
- [x] G3 laptop: python3 -m unittest -v test_fpatch,
      doors, and d are green.
      ask: the laptop suite, doors, and d are green
      ruling: confirmed on laptop
      evidence: 25 of 25 test_fpatch in 0.006s, doors 4 of 4, d 96 pct with only cistart uncommitted
