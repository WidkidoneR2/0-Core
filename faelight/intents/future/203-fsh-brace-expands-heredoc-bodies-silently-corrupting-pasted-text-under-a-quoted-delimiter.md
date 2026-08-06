---
id: 203
date: 2026-08-05
type: future
title: "fsh brace-expands heredoc bodies, silently corrupting pasted text under a quoted delimiter"
status: planned
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
- [ ] A heredoc whose body contains braces arrives at the receiving process BYTE-IDENTICAL, verified
      with `cat -A`. Red first on the 2026-08-06 case: a python heredoc containing a Rust match arm.
- [ ] Range endpoints are validated: a brace group expands ONLY when both sides are alphanumeric and
      of the same kind. `{1..5}` and `{a..z}` still expand; `{ .. }` stays literal.
- [ ] Brace expansion does not run inside single or double quotes, with a unit test per case.
- [ ] Brace expansion does not run on heredoc body lines. This needs fsh to know where a heredoc body
      begins, which it currently does not -- `try_heredoc` only tests whether the line contains the
      operator and hands the whole thing to sh. Decide whether fsh learns the construct or whether
      expansion is suppressed from that operator to end of input.
- [ ] Each gate carries evidence per INT-158.

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
