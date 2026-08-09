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

## WHERE THIS REACHED ITS BOUNDARY (2026-08-08)

PARTIAL COMPLETION, AND THE DISTINCTION MATTERS: this did not fail. It reached the point where its
own invariant caught the remaining work, which is what that invariant is for.

DONE: comments are lexical state owned by the canonical scanner; strip_comments is gone; its caller
is audited; both doors agree structurally rather than by ordering; the removed machine is no longer a
competing comment/quote interpreter.

NOT DONE, and the gate stays unchecked: "the canonical scanner is the sole owner of lexical state."
expand.rs still contains additional quote-state scanners at roughly lines 240-258 (a validity checker
with in_s/in_d AND a second in_s2/in_d2 pair for bracket depth), 346-356 (needle scanning), and
433-439 (expansion gating, whose own comment notes that double quotes still permit command
substitution so only in_single gates it). Deleting strip_comments removed one owner of several, not
the last one.

⚠️ DO NOT CONSOLIDATE THOSE THREE BY INSPECTION. They look like one problem and have three stated
purposes -- needle scanning, expansion gating, bracket validity. Before anything moves, establish
whether each is genuinely lexical INTERPRETATION or a consumer of lexical facts serving a narrower
semantic operation. That distinction is exactly what this intent exists to force.

★ The remaining trackers are now KNOWN EVIDENCE rather than an unknown hole, which is itself a
result. Widening this intent because the invariant turned out to be still false is how a controlled
architectural change becomes an uncontrolled rewrite; the honest open gate is the safer artifact.

## Success Criteria
- [x] The canonical scanner recognises comments as a lexical state -- no pre-pass over the same text
<!-- evidence: 7cb10c46. The check sits in the OUTER loop, so word-start holds by construction --
     that loop has just skipped whitespace, and unquoted holds too because a # inside quotes is
     consumed by the inner walk and never reaches it. Four unit tests define the rule:
     `echo hi # tail` -> two words, `echo "# x"` -> two tokens, `echo foo#bar` -> one word,
     `# whole line` -> nothing. No pre-pass remains over the same text. -->
- [ ] Heredoc recognition and delimiter tracking live in the scanner
- [x] The arch-era INT-285 behaviour is preserved: lines inside a heredoc body are raw data and are
      never comment-stripped. Regression coverage names the cases rather than trusting the move
<!-- evidence: two cases written BEFORE the move so it had something that could fail --
     heredoc_body_keeps_hash_lines and heredoc_body_keeps_apostrophe, b52fd248. Both green after
     deletion. The rule holds for a structural reason worth recording: a heredoc is recognised
     BEFORE the scan begins, so a body never reaches the comment check at all. -->
- [x] `strip_comments`'s quote and heredoc state machine is REMOVED, not left beside the scanner's
<!-- evidence: 480d3617, 55 lines deleted. Zero callers remained; every surviving mention of the
     name is a comment. 152 unit tests and 143/143 after removal, with no warnings. -->
- [ ] `find_heredoc_delimiter` is audited: kept with one owner, or absorbed
- [x] Every `strip_comments` caller is audited -- callers may depend on the pre-pass shape
<!-- evidence: 480d3617. ONE caller, main.rs:2216, in repl_main. Measured before removing rather
     than assumed dead: a probe on that line fired for `echo hi # tail` and for a comment-only
     line. That measurement also corrected a claim: both doors agreed after the scanner learned
     comments, but NOT because the scanner governed both -- this pre-pass ran first on the REPL
     path and beat it there. Two owners, one rule, agreeing by ordering. Now structural. -->
- [ ] The stronger invariant holds and is stated: the canonical scanner is the SOLE owner of lexical
      state. INT-169 closed only the weaker one, that the validator is no longer an owner
- [ ] Each gate carries evidence per INT-158
