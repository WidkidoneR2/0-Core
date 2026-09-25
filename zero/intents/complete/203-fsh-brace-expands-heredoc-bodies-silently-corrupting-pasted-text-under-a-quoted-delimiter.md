---
id: 203
date: 2026-08-05
type: future
title: "fsh brace-expands heredoc bodies, silently corrupting pasted text under a quoted delimiter"
status: complete
tags: [fsh, heredoc, delimiter, shell]
---

## The Problem
A heredoc carrying a Rust match pattern arrived in the file with the pattern gone. Written:

    Executed { .. } => continue

Landed:

    Executed   => continue

Three spaces where the braces had been. Three separate patch scripts reported success truthfully --
each counted seventeen replacements -- while grep and rustc disagreed, because the REPLACEMENT TEXT
was corrupted in transit rather than the write failing. It cost about an hour and produced two
rounds of blaming the scripts.

## THE MECHANISM, and it is not heredocs
`expand_braces` (main.rs:100) walks the line for an opening brace, finds the matching close, and
looks for `..` inside. For `{ .. }` the inner text is a space, two dots, a space. Neither side
parses as an integer, so it falls through to the CHARACTER-RANGE branch -- and that branch asks only
whether each side is exactly one character. A space is one character. So it builds the range from
space to space, which is one space, and replaces the whole construct with it.

    left = " "   right = " "   ls = 32   rs = 32   expanded = [" "]

fsh reads `{ .. }` as a character range from space to space.

TWO DEFECTS, NOT ONE.

  1. THE RANGE ENDPOINTS ARE NOT VALIDATED. Any single character is accepted, including a space, a
     dot, a quote. `{ .. }` is not a range and never was.
  2. BRACE EXPANSION IS NOT QUOTE-AWARE AND NOT HEREDOC-AWARE. It runs at main.rs:1316 over the
     whole input line, and bracketed paste delivers a whole heredoc as one line, so the body is
     expanded too. A quoted delimiter cannot protect it because the delimiter is never consulted --
     and `try_heredoc` (engine.rs:529) delegates the line to sh only AFTER this has happened. The
     same applies to `echo "{ .. }"`: there is no quote handling in the function at all.

## Why this ranks above the other open papercuts
It corrupts data silently and makes tooling report success. Every other finding on the list is
visible when it happens. This one produces a green result from a wrong file, which is the failure
class that undermines every other measurement taken with these tools.

BLAST RADIUS: any pasted heredoc carrying JSON, a Nix attrset, a Rust struct or match pattern, a
shell function body, an awk program, a C block. That is a large fraction of how this repo is
actually edited.

## Success Criteria
- [x] A heredoc whose body contains braces arrives at the receiving process BYTE-IDENTICAL, verified
      with `cat -A`. Red first on the 2026-08-06 case: a python heredoc containing a Rust match arm
<!-- evidence: the original case survives -- a heredoc carrying the Rust match arm comes through with
     cat -A showing the pattern intact and a single line-end marker. That specific case was
     closed by the ENDPOINT fix (gate 2), since a space is no longer a range endpoint. The
     GENERAL case needed gate 4: a body containing a VALID range was still eaten, measured
     expanding under a quoted delimiter, and is now covered by
     repl_203_a_heredoc_body_is_not_brace_expanded in the pty suite. -->.
- [x] Range endpoints are validated: a brace group expands ONLY when both sides are alphanumeric and
      of the same kind. `{1..5}` and `{a..z}` still expand; `{ .. }` stays literal.
<!-- evidence: both endpoints must be ASCII letters; the integer branch above claims numeric ranges
     first. Three unit tests including the verbatim Rust match arm from the incident, plus
     punctuation and mixed-kind cases. Live: letter and number ranges expand, the space-endpoint
     form stays literal. -->
- [x] Brace expansion does not run inside single or double quotes, with a unit test per case.
<!-- evidence: expand_braces segments with quote_runs and expands only unquoted runs -- the same
     shape and instrument expand_globs uses. Four unit tests written RED FIRST against the
     unfixed code (1 pass, 3 fail), plus repl_203_a_quoted_brace_range_is_not_expanded with the
     unquoted control in the same case, so a fix that merely disabled expansion cannot pass.
     The guardrail named commands::tokenize as the one scanner to reuse. quote_runs is a better
     answer and did not exist when this was written -- INT-209 built it three days later. -->
- [x] Brace expansion does not run on heredoc body lines. This needs fsh to know where a heredoc body
      begins, which it currently does not -- `try_heredoc` only tests whether the line contains the
      operator and hands the whole thing to sh. Decide whether fsh learns the construct or whether
      expansion is suppressed from that operator to end of input.
<!-- evidence: SUPPRESSED FROM THE OPERATOR, which is the second option, and the question the gate
     left open is answered by a third fact rather than by choosing blind: THE RECOGNISER ALREADY
     KNEW. find_heredoc_intro_inner walks the line with quote state and holds the operator position
     when it matches; it discarded it. It now returns that offset, the two existing public callers
     project it away so nothing they do changes, and a narrow accessor exposes it.
     THE OFFSET IS THE OPERATOR, NOT A BODY START. Nothing infers where a body begins or ends, which
     is what would drag brace expansion into parsing the construct -- the raw-text inference INT-196
     exists to remove. expand_braces expands only above the offset and copies the rest untouched.
     PROVEN: repl_203_a_heredoc_body_is_not_brace_expanded, a quoted-delimiter heredoc whose body
     contains a valid range, driven through the pty as ONE pasted line because that is how bracketed
     paste delivers the construct. Measured expanding before the fix.
     GHOST-CHECKED, and the split is the point: bypassing the offset lookup turns the heredoc case
     RED while the quoted-range case stays GREEN. Two independent mechanisms, each proven by its own
     case rather than one case covering both.
     ⚠️ DELIBERATE OVER-SUPPRESSION, recorded rather than silently solved. A second command AFTER the
     body ends, in the same pasted buffer, also stops expanding. Fixing that needs the delimiter line
     to find where the body terminates, which is a materially larger change and recreates exactly the
     inference this avoids. Recorded at the site as well as here. -->
- [x] Each gate carries evidence per INT-158.
<!-- evidence: every gate above carries a comment naming what was measured and how. Worth recording
     one thing beyond the gates, because it cost most of the session and will recur: THIS BUG ATE ITS
     OWN INVESTIGATION FOUR TIMES. The unit-test SOURCE was pasted through fsh and arrived with its
     ranges already expanded, so four assertions compared expanded text to itself and all passed
     while proving nothing. A test INPUT FILE written with printf from fsh contained the expanded
     form on disk, which cat -A showed. A bash CONTROL printed the wrong answer because the outer fsh
     expanded inside the single quotes before bash ran. And an anchor in this very intent was refused
     for the same reason, which is where fpatch said it outright: braces were consumed in
     transmission, build them from pieces.
     THE RULE: test input for the shell under test must be constructed OUTSIDE it. Python writes the
     file, subprocess runs the binary with an argument list, and braces are built with chr the way
     apostrophes already are. -->

## Scope guardrails
- The minimal fix is defect 1 and it is worth landing alone: validating endpoints removes the
  observed corruption and cannot break the numeric or alphabetic ranges that work today.
- Defect 2 is the INT-196 class -- execution-governing code inferring shell structure from raw text.
  Do not fold it into the endpoint fix; it needs its own evidence.
- Do NOT make expansion quote-aware by adding a second quote scanner. `commands::tokenize` is the
  shell's one quote-aware tokenizer (INT-171 gate 1) and a second one would be the two-owners bug
  INT-193 exists to prevent.

## Relationship
- Found while extracting `run_segment` for INT-201 gate 4, which is where the corrupted patches were.
- INT-196 owns the general rule this violates.
- INT-171 owns the single tokenizer any quote-awareness must reuse.
