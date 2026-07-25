---
id: 195
date: 2026-07-25
type: arch
title: "every stage consumes the previous stage output, never the original string"
status: planned
tags: [architecture, rust, design]
---

## Vision
Every stage consumes the previous stage's output. No stage after the lexer
re-derives syntax from the original input string.

## The Problem
fsh's documented bug class is not "the parser was wrong". Twice the parser was
RIGHT and something downstream ignored it:
  - INT-143: four tokenizers with no pipeline between them, six bugs on 2026-07-16.
  - INT-172: detect_redirect scanned the raw line for '>' and dropped everything
    after `2>`. Silent every time, three sightings before it was caught.
  - INT-171 consolidated to one parsing entry point precisely because the entry
    points were never the problem. The bypasses were.
What those share is a stage re-inspecting the original string to rediscover
structure a previous stage had already computed correctly. This intent states
that shared cause once, so it can be checked instead of rediscovered.

## The Solution
State it as an invariant and enforce it mechanically. After the lexer, no stage
may call:
  - split_whitespace() or equivalent ad hoc splitting on shell input
  - a second tokenizer over already-tokenized input
  - find('>'), contains('|'), or any scan of the raw line for operators
  - a re-parse of the original string
Each stage takes the previous stage's typed output and produces its own.

CARVE-OUT, deliberate: text transforms that run BEFORE the lexer are not
violations. Alias expansion and history expansion are INPUT transformations by
design -- an alias is defined as text by the user, and bash expands aliases
during tokenization for the same reason. The invariant governs everything from
the lexer onward, not the text world upstream of it.

Enforceable by grep, which is the whole point. Every other principle in the
spine is a design judgement that has to be argued. This one is a search that
either returns hits or does not.

## Success Criteria
- [ ] The invariant is written down where code can be checked against it, with
      the banned-call list and the pre-lexer carve-out both named
- [ ] Every current violation is enumerated with file:line -- a census, not a fix
- [ ] Each enumerated violation is either removed, or recorded as a known
      exception with a stated reason
- [ ] A check exists that returns the violations, runnable on demand
- [ ] The check runs somewhere it will be seen (pre-commit or fsh-test), not
      only by hand
- [ ] RETRO-VALIDATION: the check is confirmed to catch INT-172's detect_redirect
      and INT-143's tokenizers, or it is explained why it does not. A check that
      would have missed the bugs it exists to prevent is the wrong check
