---
id: 208
date: 2026-08-08
type: arch
title: "fsh diagnostics have no internal model -- every failure is an eprintln with a coloured marker, so nothing can render as JSON, an editor diagnostic, or a test snapshot"
status: in-progress
tags: [architecture, rust, design]
---

## Vision
A failure in fsh should be a VALUE before it is a string. Severity, message, a primary span, labels,
help and a code -- so the same failure can be rendered richly in a terminal, as JSON for a tool, as an
editor-facing diagnostic, or as a snapshot in a test. Today it can only be one of those, because it
is already a string by the time anyone sees it.

## The Problem
Every failure in fsh is an eprintln with a coloured marker. The pattern is uniform and it works for
the terminal: a bright red cross, a message, sometimes a code formatted into the text. But the
information is destroyed at the moment of printing. A caller cannot ask what went wrong, only read
what was said.

⚠️ AND THE SHELL HAS ALREADY PAID FOR THIS ONCE. INT-169 recorded that a real exit status was
formatted INTO an error message and then discarded, so the shell printed "exited 2" while $? reported
1. The fix was to carry the code as data on the variant rather than parse it back out of the string.
That is this intent's thesis applied to one field; this applies it to the whole diagnostic.

⚠️ THE SPANS ALREADY EXIST AND ARE BEING THROWN AWAY. The spine's AST carries Spanned<T> everywhere,
ParseError and LowerError both carry a Span, and a nested command substitution preserves the absolute
span of its inner source specifically so diagnostics can later be rendered against the original line.
Nothing consumes any of it. The parser knows exactly where the problem is and the user is told in
prose.

## The Solution
An internal Diagnostic model owned by fsh: severity, message, primary span, labels, help, code.
Rendering is a separate concern with more than one implementation.

⚠️ THIS INTENT IS NOT ABOUT MIETTE, and writing it that way would be the mistake INT-198 exists to
prevent. INT-198 ruled miette ADD NOW as a RENDERER of the internal model, not as the model. If the
model is defined in terms of the crate, the crate becomes the architecture and the JSON, editor and
snapshot renderers never happen.

⭐ ITS NEIGHBOUR IS INT-199, whose thesis is that every failure answers what happened, what changed,
why, and what to do next. INT-199 established the CONVENTION and proved it on fpatch, where a refusal
now names the reason, what was compared, the likely cause and the recovery. This intent is the
MECHANISM that makes the same shape available everywhere without hand-writing it each time.

## Explicitly out of scope
Rewriting every existing error site. The model and one renderer that matches today's output are the
deliverable; migration is per-site and follows the evidence, starting where spans already exist.

## Success Criteria
- [ ] The Diagnostic model is defined and OWNED by fsh -- no crate type in the signature
- [ ] One parse or lower error renders through it with its span, proving the spans that already
      exist can reach a user
- [ ] TWO renderers exist, and the second is not cosmetic: terminal plus one machine-readable form
- [ ] A test asserts a diagnostic by VALUE rather than by matching printed text
- [ ] Today's terminal output is unchanged for the migrated site, or the difference is stated
- [ ] The INT-199 convention is expressible in the model -- what happened, what changed, why, what
      to do next -- rather than reconstructed per site
- [ ] Each gate carries evidence per INT-158
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
