---
id: 209
date: 2026-08-08
type: arch
title: "consolidate lexical-state ownership -- expand::strip_comments runs a second quote machine and its own heredoc tracking, so the canonical scanner is not yet the sole owner"
status: in-progress
tags: [architecture, rust, design]
---

## Vision
One owner of lexical state. INT-169 makes the canonical scanner report continuation and removes the
validator as a competing owner; this finishes the job by removing the last one.

## The Problem
`expand::strip_comments` (expand.rs 140-194) is not a string utility. It is a 54-line state machine
that tracks `in_heredoc` and `heredoc_delim`, calls `find_heredoc_delimiter` to recognise a heredoc
introduction, and walks characters with its own `in_single` / `in_double` pair to decide where a
comment begins. That is quote tracking and heredoc tracking -- the same knowledge `spine/lexer.rs`
owns -- implemented a second time in a different file.

⚠️ AND IT WORKS, which is why this is consolidation rather than repair. Its behaviour was learned from
a real failure: a heredoc body's lines are raw data and must never be stripped. That rule is correct
and must survive the move intact.

⚠️ PROVENANCE NOTE: the comment in expand.rs cites INT-285, and that number belongs to the ARCH-ERA
ledger, not this one -- the current ledger tops out in the low 200s. Cite it as arch-era INT-285 or
describe the behaviour directly. A bare number sends the next reader hunting an intent that does not
exist here, which is a live problem in this codebase rather than a hypothetical one.

## The Solution
Move the state, not the function. The scanner already walks characters with quote context; comment
recognition and heredoc tracking belong in that walk rather than in a pre-pass that runs over the
same text with its own rules.

⚠️ WHAT MAKES THIS SAFE TO DO SECOND RATHER THAN FIRST: INT-169 establishes the scanner as the place
continuation is REPORTED FROM. Until that exists there is nowhere for this state to move to.

## Explicitly out of scope
Native heredoc EXECUTION. The scanner knowing it is inside a heredoc body does not claim fsh can run
one; that capability is its own intent.

## Success Criteria
- [ ] The canonical scanner recognises comments as a lexical state -- no pre-pass over the same text
- [ ] Heredoc recognition and delimiter tracking live in the scanner
- [ ] The arch-era INT-285 behaviour is preserved: lines inside a heredoc body are raw data and are
      never comment-stripped. Regression coverage names the cases rather than trusting the move
- [ ] `strip_comments`'s quote and heredoc state machine is REMOVED, not left beside the scanner's
- [ ] `find_heredoc_delimiter` is audited: kept with one owner, or absorbed
- [ ] Every `strip_comments` caller is audited -- callers may depend on the pre-pass shape
- [ ] The stronger invariant holds and is stated: the canonical scanner is the SOLE owner of lexical
      state. INT-169 closed only the weaker one, that the validator is no longer an owner
- [ ] Each gate carries evidence per INT-158
