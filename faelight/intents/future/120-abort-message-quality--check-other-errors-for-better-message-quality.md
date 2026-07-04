---
id: 120
date: 2026-07-04
type: future
title: "Abort-message quality + check other errors for better message quality"
status: planned
tags: [errors, abort, messaging]
---

## Why
The count==N-or-abort edit pattern (assert every anchor matches before an atomic write;
abort if not) is EXCELLENT and stays -- it prevents partial/corrupt edits by failing
before the write. But when it aborts, the message is often terse ("ABORT: 0" / "ABORT
sanity"), so as a USER it's not always clear WHY. This intent improves abort + error
MESSAGE QUALITY so failures are legible: what was expected, what was found, which anchor
failed, what to do next. Keep the behavior; improve the explanation.

## Principle
An abort is a FEATURE (fail-safe before write). A good abort message turns a fail-safe
into a DIAGNOSTIC: the user should understand the cause without re-reading the code.

## Targets
1. The edit/patch abort messages (fsh-patch, rspatch, patch-multi, the count-assert
   pattern used in tooling scripts): on abort, report
     - which anchor/string failed,
     - expected count vs found count,
     - a hint (e.g. "anchor may contain a unicode char -- see em-dash/box-drawing note",
       or "string appears N times, expected 1 -- add more context to disambiguate").
2. Sweep other tool error paths for terse/opaque messages -- especially where a tool
   exits non-zero with little context. Prioritize the ones hit during daily driving.
3. Consistency: a shared error-message STYLE (what failed / why / what to try) so errors
   across tools read the same way.

## Approach
- Start with the abort/edit messages (highest daily-friction, clearest win).
- Then sweep tool error returns for opaque cases; improve incrementally, build-gated.
- Consider a small helper/pattern for "expected vs found + hint" so it's reusable, not
  re-hand-written per site.

## Gates
- [ ] Edit/patch aborts report failed anchor + expected-vs-found count + a hint
- [ ] A consistent error-message style defined (what/why/what-to-try)
- [ ] Opaque error paths found during daily use improved (list them as found)
- [ ] Full workspace builds clean, zero warnings after changes

## Relationship
- Quality/usability improvement; touches fsh (patch builtins) + any tool with terse
  errors. NOT a 1.0.0 blocker. Good incremental daily-driving work.
