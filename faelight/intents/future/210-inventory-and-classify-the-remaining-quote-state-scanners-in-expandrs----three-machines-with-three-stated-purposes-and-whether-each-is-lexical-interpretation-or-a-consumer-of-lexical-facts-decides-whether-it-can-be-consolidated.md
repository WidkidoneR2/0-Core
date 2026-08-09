---
id: 210
date: 2026-08-08
type: arch
title: "inventory and classify the remaining quote-state scanners in expandrs -- three machines with three stated purposes, and whether each is lexical interpretation or a consumer of lexical facts decides whether it can be consolidated"
status: planned
tags: [architecture, rust, design]
---

## Vision
Know what each remaining quote-state scanner IS before deciding whether it can move. The answer may
be that some of them should not.

## The Problem
INT-209 removed strip_comments, a 55-line machine that tracked heredoc state and walked characters
with its own quote pair. It was one owner of several. expand.rs still holds at least three more, and
they are NOT obviously the same problem:

  ~240-258  a validity checker, with in_s/in_d AND a second in_s2/in_d2 pair for bracket depth
  ~346-356  needle scanning -- find a substring that is not inside quotes
  ~433-439  expansion gating, whose own comment records that double quotes still permit command
            substitution so only in_single gates it

Three stated purposes. Treating them as one refactor because they share two boolean names is how a
controlled architectural change becomes an uncontrolled rewrite.

## The Solution
Classify first, consolidate second -- and only what the classification says can move.

The question for each: is this LEXICAL INTERPRETATION, which the canonical scanner should own, or is
it a CONSUMER OF LEXICAL FACTS serving a narrower semantic operation? A consumer asking "is this
offset inside quotes?" is not a second lexer; it is a caller that would be better served by the
scanner exposing that fact. A machine that decides what a quote MEANS is a second lexer and must go.

⚠️ THE EXPANSION-GATING ONE IS THE INTERESTING CASE. Its rule -- double quotes allow substitution,
single quotes do not -- is semantics, not lexing. The scanner already records QuoteContext per
segment, so this may be a consumer that should read that fact rather than rediscover it. That would
make it a consumer migration rather than a deletion.

## Explicitly out of scope
Moving anything. This intent produces a classification anyone can act on. If it starts growing
implementation, it has failed -- the same fence INT-198 set, for the same reason.

## THE CLASSIFICATION (2026-08-09)

FIRST CORRECTION: there are SIX, not three. INT-209 counted by searching two variable spellings;
searching all of them -- in_s, in_d, in_s2, in_d2, in_single, in_double -- found six functions. The
number was wrong because the search was.

CONSUMERS. Each asks ONE question, "is this offset inside quotes?", and acts on the answer. None
decides what a quote MEANS.

  strip_quoted_regions      36 lines   remove quoted regions from a line
  rfind_unquoted            21 lines   the last occurrence of a needle not inside quotes
  expand_globs              64 lines   expand a glob only in an unquoted segment
  find_unmatched_globs      80 lines   report an unmatched glob only in an unquoted segment

TWO OF THEM ADMIT THE DUPLICATION IN THEIR OWN DOCS. find_unmatched_globs says it reuses "the same
quote-aware segmentation as expand_globs". And rfind_unquoted records the bug that created it: echo
with a quoted redirect arrow split at the QUOTED arrow, so the command was truncated, the target
became a fragment with a stray quote, and a file with that name appeared in the working directory
while the command printed nothing.

WHAT THEY NEED, AND THE SCANNER ALREADY HAS IT: QuoteContext, with Unquoted, Single and Double, is
recorded per segment on every Literal. Four consumers rediscovering it by walking characters is four
chances to disagree with the shell that runs the line -- and rfind_unquoted's own doc proves that is
not hypothetical, because the pipe scan in main.rs tracks only double quotes and would break a
single-quoted redirect.

CONSUMER MIGRATION IS NOT DELETION. Each needs the scanner to EXPOSE the fact -- something like
"which QuoteContext applies at this byte offset" -- and no such accessor exists today. Filing that
accessor is the actionable next step; moving four call sites is not.

NOT CONSUMERS. These decide structure, and each is a separate finding.

  expand_subshells          56 lines   tracks dollar-paren NESTING DEPTH, quote-aware inside the
                                       region. That is REGION DETECTION -- the same job the scanner
                                       does for WordSegment::CommandSub. A second recogniser of one
                                       construct, belonging with the substitution work rather than
                                       with quote consumers.

  is_complete_command      150 lines   A THIRD CONTINUATION CHECKER, AND IT IS LIVE. Called from
                                       main.rs:2155 in the heredoc collection loop and from
                                       expand.rs:320. It answers "is this input finished" -- the
                                       exact question INT-169 G1 gave the canonical scanner, and the
                                       exact question the validator was stripped of. So the shell
                                       has THREE answers to one question: the scanner, this, and a
                                       validator that now consumes the scanner.
                                       This is INT-169's problem rather than this intent's. It is
                                       quote-state shaped only incidentally; what it actually is, is
                                       the continuation rule living somewhere the scanner is not.

## THE COUNTABLE TARGET FOR INT-209's SOLE-OWNER GATE
Six functions outside spine/lexer.rs walk characters tracking quote state. Four are consumers needing
an accessor. One is a second region recogniser. One is a third continuation checker, and it is the
most serious of the six.

## Success Criteria
- [x] Each of the three machines is classified: lexical interpretation, or consumer of lexical facts
<!-- evidence: the classification above, 2026-08-09. SIX functions rather than three -- the
     original count searched two variable spellings. Four consumers, one region recogniser, one
     continuation checker. -->
- [x] For each, the evidence is its stated PURPOSE and its inputs, not its variable names
<!-- evidence: each is classified by what its doc and signature say it answers, not by the
     presence of in_single/in_double. rfind_unquoted returns an offset; expand_globs returns an
     expanded line; is_complete_command returns (bool, reason) -- that last signature is what
     makes it a continuation checker rather than a quote tracker. -->
- [x] Any that is a consumer names the fact it needs and whether the scanner already records it
      -- QuoteContext exists per segment today
<!-- evidence: all four consumers need the same fact -- which QuoteContext applies at a byte
     offset. The scanner records QuoteContext { Unquoted, Single, Double } per segment on every
     Literal (spine/ast.rs:59), so the fact EXISTS; the accessor does not. That accessor is the
     actionable next step and it is what makes this a migration rather than a deletion. -->
- [x] Any that is a second lexer gets a lane and a rough order, or an explicit deferral with reason
<!-- evidence: expand_subshells is region detection duplicating WordSegment::CommandSub -- lane:
     with the substitution work, after the accessor exists. is_complete_command is DEFERRED OUT OF
     THIS INTENT with a reason: it is a third continuation checker answering the question INT-169
     G1 gave the scanner, live at main.rs:2155, and it belongs to INT-169 rather than here. It is
     quote-shaped only incidentally. -->
- [x] A grep-able statement of how many quote-state machines remain outside the scanner, so
      INT-209's sole-owner gate has a countable target rather than a belief
<!-- evidence: SIX outside spine/lexer.rs, found with
     grep -rn 'in_s\b|in_d\b|in_s2|in_d2|in_single|in_double' -- the countable target INT-209's
     sole-owner gate needed. It was believed to be three. -->
- [x] Nothing moved. The close condition is a decision, not a diff
<!-- evidence: no source file changed under this intent. What it hands forward is a classification
     with a named next step (the offset accessor) and one finding routed to another intent. -->
- [x] Each gate carries evidence per INT-158
<!-- evidence: every gate above carries an HTML comment naming what was read and what it showed --
     function lengths, signatures, the docs that admit the duplication, and spine/ast.rs:59 for the
     fact the consumers need. No commit hashes, deliberately: this intent changed no source, which
     was its close condition. -->
