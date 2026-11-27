---
id: 355
date: 2026-11-27
type: future
title: "fpatch review diff, absolute paths, and empty marker guards"
status: planned
tags: [fpatch, safety, dev]
depends_on: []
format: 3
---

## Vision
A whole-line fpatch review shows only the line that changes.
A relative path is refused before any write. An empty
patch_between start marker is refused before any write.
The bytes str.replace writes stay the bytes the anchor names.

## The Problem
Three guards in zero/scripts/dev/fpatch.py do not do what
the contract says. Measured 2026-11-27 against origin/main
1907242a. python3 -m unittest test_fpatch was 18 passed.
The probes below were run against that file.

1. _compute_patch paints one line past a whole-line anchor.
   first is the line of the match. last is first plus
   old.count("\n"), and the red range is inclusive
   (fpatch.py, the loop that builds the shown lines).
   new.split("\n") then emits a trailing empty green line
   whenever new ends in a newline. File keep\ndrop\nstay\n,
   dry-run and apply of drop\n to DROP\n, both printed:

       replacing lines 2..3
       >> 2: drop
       >> 3: stay
       ++ DROP
       ++

   Apply returned success and the file was keep\nDROP\nstay\n.
   stay was not deleted. The header sha256 was the real
   post-image (5118dc90fcb34a52). The painted lines disagree
   with those bytes, and the Seal covers both. The same
   painting showed for b\n to B\n in a\nb\nc\n (>> 3: c).
   No current test uses an anchor that ends in a newline
   and then checks the next line.

2. Plan.patch and patch_between accept a relative path.
   AGENTS.md says the path is absolute. The module docstring
   says the same. isabs does not occur in fpatch.py.
   Plan.patch stores str(path). From a temp directory,
   patch("rel.txt", "alpha", "BETA") returned success and
   the file was BETA.

3. patch_between accepts an empty start marker.
   patch refuses an empty anchor. patch_between does not.
   On a one-line file with no trailing newline,
   patch_between(path, "", None, ["PWNED"]) returned success
   and the file became PWNED. The same call exited 1 and
   left the bytes alone for hello\n, hello\nworld, and
   hello\nworld\n.

## The Solution
G1 adds four tests in zero/scripts/dev/test_fpatch.py and
does not edit fpatch.py. They fail on the unmodified module.
The existing 18 tests stay green.

G2 changes only zero/scripts/dev/fpatch.py so those four
tests pass, the 18 stay green, an anchor with no trailing
newline still paints only its own line, and the Seal equals
sha256 of the uncolored plan text, first 16 hex.

G3 is the laptop: python3 -m unittest -v test_fpatch,
doors, and d are green.

## Limits
- In scope: zero/scripts/dev/fpatch.py and
  zero/scripts/dev/test_fpatch.py.
- Out of scope: multi-file rollback, backups, and samefile
  (INT-356); exit-code cleanup for missing or unwritable
  paths; TOCTOU; a protected-path denylist; FPATCH_COLOR;
  nsh-test harness changes.
- How deep: three gates. G1 is tests only. G2 is the
  display arithmetic, an absolute-path refusal, and an
  empty-marker refusal. No new crate. No new dependency.
- Stop and ask: a fix seems to need a file outside In scope;
  the write bytes of str.replace would have to change;
  an existing test goes red for a reason other than the
  four new cases.
- Done means: G1, G2, and G3 ticked with evidence.
- Must not change: fpatch replace results for an anchor
  that does not end in a newline, the 18 existing tests'
  behaviour, and any file outside the two scripts above.

## Success Criteria
- [ ] G1 red first: four new tests in test_fpatch.py fail
      on unmodified fpatch.py. (a) whole-line diff of
      keep\ndrop\nstay\n, dry-run drop\n to DROP\n, does
      not mark >> 3: stay, has no blank ++ line, and the
      header says lines 2..2. (b) a relative path is
      refused with exit 1 and nothing written, for both
      patch and patch_between. (c) an empty patch_between
      start marker is refused with exit 1 and nothing
      written. The existing 18 tests stay green.
      ask: the four new tests fail on today's fpatch.py, and the existing 18 pass
      ruling:
      evidence:
- [ ] G2 fixes in fpatch.py only. All new tests and all
      18 existing tests pass: python3 -m unittest -v
      test_fpatch exits 0. An anchor without a trailing
      newline still paints only its own line. The Seal
      matches sha256 of the uncolored plan text, first
      16 hex.
      ask: the new tests and the existing 18 pass, and the Seal still matches the plan text
      ruling:
      evidence:
- [ ] G3 laptop: python3 -m unittest -v test_fpatch,
      doors, and d are green.
      ask: the laptop suite, doors, and d are green
      ruling:
      evidence:
