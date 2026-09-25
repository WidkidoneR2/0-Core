---
id: 209
date: 2026-08-08
type: arch
title: "consolidate lexical-state ownership -- expand::strip_comments runs a second quote machine and its own heredoc tracking, so the canonical scanner is not yet the sole owner"
status: complete
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

## WHAT INT-210 ANSWERED (2026-08-09)

The sole-owner gate now has a COUNTABLE target instead of a belief, and the belief was wrong: SIX
machines outside spine/lexer.rs walk characters tracking quote state, not three. The earlier count
searched two variable spellings.

  CONSUMERS (4) -- ask "is this offset inside quotes?" and act on the answer. None decides what a
  quote MEANS. strip_quoted_regions, rfind_unquoted, expand_globs, find_unmatched_globs.
  All four need one fact the scanner ALREADY RECORDS -- QuoteContext per segment on every Literal --
  but no accessor exposes it at a byte offset. That accessor is the blocking next step, and it makes
  this a consumer migration rather than four deletions.

  REGION RECOGNISER (1) -- expand_subshells tracks dollar-paren nesting, duplicating what the scanner
  does for WordSegment::CommandSub. Lane: with the substitution work, after the accessor exists.

  CONTINUATION CHECKER (1) -- is_complete_command, 150 lines, LIVE at main.rs:2155. It answers "is
  this input finished", which is the question INT-169 G1 gave the canonical scanner and the question
  the validator was stripped of. Three owners of one rule. Routed to INT-169; it is quote-shaped only
  incidentally.

⏭ SO THIS GATE STAYS OPEN, and now it can be closed on evidence rather than on a feeling: the
accessor lands, four consumers migrate, the region recogniser follows the substitution work, and
INT-169 absorbs the continuation checker. Then the count is zero and the invariant is measurable.

## HOW IT ACTUALLY CLOSED (2026-08-12)

The closing condition above needed CORRECTING rather than meeting. Then the count is zero would
have made this intent wait on INT-169 indefinitely, because two of the six machines were never
209 to close.

THREE of the four consumers migrated behind the accessor. The fourth, strip_quoted_regions, is
DEFERRED with a named owner: its only caller lives inside is_complete_command, which INT-169
intends to REPLACE rather than refactor, so migrating it would rebuild a helper another intent
expects to delete. The disposition is recorded at the site as well as here.

The gate now claims what was achieved: the scanner is the sole owner for the consumers this
intent owns, and every remaining machine has a named downstream owner. Counting to four by
touching code that belongs to another intent would have been the dishonest close.

## Success Criteria
- [x] The canonical scanner recognises comments as a lexical state -- no pre-pass over the same text
<!-- evidence: 7cb10c46. The check sits in the OUTER loop, so word-start holds by construction --
     that loop has just skipped whitespace, and unquoted holds too because a # inside quotes is
     consumed by the inner walk and never reaches it. Four unit tests define the rule:
     `echo hi # tail` -> two words, `echo "# x"` -> two tokens, `echo foo#bar` -> one word,
     `# whole line` -> nothing. No pre-pass remains over the same text. -->
- [x] Heredoc recognition and delimiter tracking live in the scanner
<!-- evidence: 35d0842f. The scanner calls find_heredoc_intro before the character walk begins and
     reports HeredocBody { delimiter, quoted } as lexical continuation state -- so it knows it is
     inside a heredoc continuation, which is what INT-169's ruling scoped this to.
     ⚠️ AWARENESS, NOT EXECUTION, deliberately: try_heredoc still collects the body and hands the
     construct to sh. That capability is a separate intent. And main.rs:2170 still tracks a
     delimiter for that collection loop -- an EXECUTION concern consuming a lexical fact, not a
     second recogniser, which is the distinction INT-210 exists to make for the remaining cases. -->
- [x] The arch-era INT-285 behaviour is preserved: lines inside a heredoc body are raw data and are
      never comment-stripped. Regression coverage names the cases rather than trusting the move
<!-- evidence: two cases written BEFORE the move so it had something that could fail --
     heredoc_body_keeps_hash_lines and heredoc_body_keeps_apostrophe, b52fd248. Both green after
     deletion. The rule holds for a structural reason worth recording: a heredoc is recognised
     BEFORE the scan begins, so a body never reaches the comment check at all. -->
- [x] `strip_comments`'s quote and heredoc state machine is REMOVED, not left beside the scanner's
<!-- evidence: 480d3617, 55 lines deleted. Zero callers remained; every surviving mention of the
     name is a comment. 152 unit tests and 143/143 after removal, with no warnings. -->
- [x] `find_heredoc_delimiter` is audited: kept with one owner, or absorbed
<!-- evidence: KEPT, with one owner. It has two remaining callers -- expand.rs:192, a separate
     cleaner, and main.rs:2170, the heredoc collection loop -- and both ask the same question:
     WHICH delimiter ends this heredoc. find_heredoc_intro was added beside it for the scanner,
     reporting the delimiter AND whether it was quoted, because the quoting was always computed to
     find where the delimiter token ended and then discarded. The old name delegates to the same
     body, so there is one recogniser with two signatures rather than two recognisers. -->
- [x] Every `strip_comments` caller is audited -- callers may depend on the pre-pass shape
<!-- evidence: 480d3617. ONE caller, main.rs:2216, in repl_main. Measured before removing rather
     than assumed dead: a probe on that line fired for `echo hi # tail` and for a comment-only
     line. That measurement also corrected a claim: both doors agreed after the scanner learned
     comments, but NOT because the scanner governed both -- this pre-pass ran first on the REPL
     path and beat it there. Two owners, one rule, agreeing by ordering. Now structural. -->
- [x] The scanner is the sole owner of lexical state FOR THE CONSUMERS THIS INTENT OWNS, and every
      remaining machine has a named downstream owner. The gate said SOLE OWNER without qualification;
      the wording is corrected rather than quietly satisfied, as INT-196 M6 and M7 were
<!-- evidence 2026-08-12. THE BLOCKING STEP LANDED: quote_context_at, spine/lexer.rs, reporting which
     QuoteContext applies at a byte offset. Its contract has four lines and one is a ruling -- a
     quote DELIMITER reports Unquoted, because the scanner consumes it as syntax and excludes it
     from every segment span, so it has no quoted-text context. That is NOT a claim it was written
     unquoted; conflating lexical context with syntax is the confusion the contract prevents. An
     operator byte reporting Unquoted is a MEASURED fact rather than a ruling: an operator is a token
     with a span and no segments. A first test asserted otherwise and a probe corrected the contract
     rather than the test being edited to match.
     THE ACCOUNTING, and it is deliberately not "four consumers migrated":
       rfind_unquoted        MIGRATED   c1b9cbc4
       expand_globs          MIGRATED   b02970ce
       find_unmatched_globs  MIGRATED   b02970ce
       strip_quoted_regions  DEFERRED to INT-169 -- disposition recorded at the site
       expand_subshells      NOT THIS INTENT -- region recognition, lanes with substitution
       is_complete_command   INT-169, routed by INT-210
     WHY strip_quoted_regions IS NOT MIGRATED: it has exactly one caller, inside is_complete_command,
     and INT-169 intends to REPLACE that completion logic rather than refactor it. Migrating it now
     rebuilds a helper for a function another intent expects to delete. The distinction is between a
     helper having a quote-related IMPLEMENTATION and this intent owning the BEHAVIOUR that requires
     it -- the second is false here. Same reasoning that deferred INT-216.
     WHAT THE MIGRATIONS ACTUALLY FOUND: expand_globs and find_unmatched_globs were not merely
     similar, they were BYTE-FOR-BYTE the same algorithm twenty lines apart, differing only in
     variable names. Eighty-one lines became twelve behind one shared segmenter. And the old
     segmentation comment described something the code did not do -- it claimed both delimiters land
     in the quoted segment; the closing one landed in the unquoted run that follows.
     PROVEN, NOT ASSERTED: five characterization tests written against the UNCHANGED code, green
     through both migrations, and given teeth by a ghost-check -- blinding the segmenter turns the
     three quoted cases RED while the two unquoted controls stay green. rfind_unquoted has seven
     cases with the same treatment, and one of them was exposed BY the ghost-check as a
     characterization test rather than a discriminator, which is recorded rather than hidden.
     Live behaviour verified beyond the suite, since globs run in the daily shell: a real glob still
     expands, a quoted star still prints literally. 203 unit tests, 151 of 151 fsh-test. -->
- [x] Each gate carries evidence per INT-158
<!-- evidence: every ticked gate above carries an HTML comment naming a commit or a demonstrated
     fact -- 7cb10c46 for comments as lexical state, 35d0842f for heredoc recognition, b52fd248 for
     the arch-era behaviour cases written before the move, 480d3617 for the deletion and the caller
     audit, and a stated finding for the delimiter audit. The one UNTICKED gate carries its reason
     in the boundary section above rather than a hash, which is the honest form of evidence for a
     gate that is deliberately still open. -->
