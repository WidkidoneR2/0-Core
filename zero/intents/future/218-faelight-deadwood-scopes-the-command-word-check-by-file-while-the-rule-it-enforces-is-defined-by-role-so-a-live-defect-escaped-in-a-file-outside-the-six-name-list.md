---
id: 218
date: 2026-08-12
type: future
title: "faelight-deadwood scopes the command-word check by FILE while the rule it enforces is defined by ROLE, so a live defect escaped in a file outside the six-name list"
status: planned
tags: [deadwood, int-196, scope, architecture]
---

## Vision
A function declares its own role, and the checker scopes on that. Whether a derivation matters is a
property of what the code DOES, not of which file it happens to live in.

## The Problem
The check is scoped by a six-name allowlist and the rule it enforces is defined by role. Its own
documentation says so and calls the list a TEMPORARY COARSE FILTER, kept until per-function role
annotations exist. That gap is no longer theoretical.

MEASURED 2026-08-12: the explainer reported a quoted command word as not in the forest vocabulary
while the shell treated it as the bare word and challenged it. Same input, two answers, from the
tool a user consults to find out what the shell will do. The derivation was in semantic.rs, which
was outside the list, so the checker could not have flagged it. It surfaced only because the nine
author exemptions were being audited by hand and one of them pointed a layer down.

The list is also an over-approximation in the other direction. commands/mod.rs holds the dispatcher
AND every builtin body, so display-only builtins inside it are reported and then exempted one by
one. Eight of the current author exemptions live in that one file. Scoping by file forces both
errors at once: it misses code that governs execution elsewhere, and it flags code that governs
nothing.

## The Solution
Replace the file list with author-declared role at the site that knows its own role. The exemption
mechanism already proves the shape works -- a comment adjacent to the code, resolved textually,
where a missing declaration yields a visible finding rather than silence.

The direction is to invert it: instead of naming files that are in scope and exempting sites inside
them, let a site declare that it GOVERNS EXECUTION. Then the checker scopes on declarations and the
allowlist disappears.

NOT DONE IN THIS SESSION, deliberately. The immediate need was covered by adding semantic.rs on
demonstrated escape coverage: one file, one proven defect, auditable and minimal. Adding every
execution-adjacent file on suspicion would repeat the mistake the rule itself made when it was too
wide -- five findings and zero true positives.

## Evidence (measured 2026-08-12)
- The escape: semantic.rs:114 and :310 derived the verb with split_whitespace, outside IN_SCOPE.
  `why rm -rf /tmp` reported a destructive verb at 100 percent; the same line with the word quoted
  reported it as unknown. Fixed in a22d7977; the SCOPE hole is what this intent is about.
- The over-approximation: EIGHT of the current author exemptions live in commands/mod.rs alone,
  because that file holds the dispatcher and every builtin body.
- IN_SCOPE is six names, and its own comment calls it a temporary coarse filter.
- semantic.rs was added on demonstrated escape coverage. Exempt count went 9 to 8 -- the explainer
  no longer needs an exemption because its derivation is now canonical.

## Non-goals
- Removing the exemption mechanism. A site declaring an exception with a stated reason is healthy;
  the defect is that scope is decided by filename.
- Widening IN_SCOPE by suspicion. That repeats the mistake the rule made when it was too wide:
  five findings, zero true positives.
- Resolving provenance through types. syn carries no type information and this checker parses files
  independently, so a declaration by the author is the honest mechanism.

## Success Criteria
- [ ] G1 RED FIRST: a derivation in a file outside IN_SCOPE is demonstrated to escape TODAY, using
      a fixture rather than the live tree, so the escape is reproducible after the fix lands
- [ ] G2: the role declaration is DESIGNED before it is written -- what a site says about itself,
      where the comment may sit, and what a MISSING declaration means. A missing one must produce a
      finding rather than silence, which is the property the exemption window already has
- [ ] G3: every current author exemption is re-read against the new mechanism and either becomes a
      declaration or is removed. Eight sites, and this intent states which
- [ ] G4: IN_SCOPE is REMOVED, not merely extended. If the file list survives, the defect survives
- [ ] G5: the checker still catches the INT-172 index-slicing shape and the gen-432 whitespace
      shape. Both fixtures already exist in cmdword_check_tests and must stay green
- [ ] G6: the live tree is measured before and after. A finding count that changes is explained,
      not absorbed -- a checker whose numbers move for unexamined reasons is the thing this whole
      arc has been correcting
- [ ] G7: PRECISION IS STATED, not assumed. Every new finding is triaged as a true or false
      positive with the reason, because a checker at zero precision becomes a ritual
- [ ] G8: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
