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
- [x] The Diagnostic model is defined and OWNED by fsh -- no crate type in the signature
<!-- src/diagnostic.rs. Severity, message, labels, help, code. No crate appears in any signature.
     miette remains where INT-198 put it: a RENDERER at error.rs:223, its only use in the codebase.
     Commit 6d57243b. -->
- [x] One parse or lower error renders through it with its span, proving the spans that already
      exist can reach a user
<!-- PROVEN LIVE: `spine-exec echo a >` reports labels [{start:7, end:8}] -- the exact position of
     the `>`. That byte range has been recorded by the lexer since the spine was written and was
     consumed by nothing.
     ★ AND THE SITE THAT DESTROYED IT IS GONE: two sites read `format!("spine: {e:?}")`, flattening
     a SpineAttemptError whose four variants each carry the structured thing. spine::diagnose now
     projects them. Three of those variants carried #[allow(dead_code)] with a comment saying the
     data was kept for a consumer that did not exist yet -- all three allows removed, because it
     does now. Commit 3a614e24. -->
- [x] TWO renderers exist, and the second is not cosmetic: terminal plus one machine-readable form
<!-- Display for the terminal; to_json() for everything else, behind FSH_DIAGNOSTIC_JSON at
     engine.rs:224 -- a REAL consumer, not a test. A renderer only a test calls is decoration,
     which is the charge INT-222 makes against the doctor.
     Same failure, both ways: `x unterminated quote` + `  close the quote`, versus
     {"code":"fsh::spine::incomplete","help":"close the quote",...}. A tool matches the code without
     reading prose. serde_json was already a dependency, so no new crate and no ruling needed. -->
- [x] A test asserts a diagnostic by VALUE rather than by matching printed text
<!-- Four tests in diagnostic.rs. a_diagnostic_is_inspectable_not_just_readable checks fields;
     the_json_contract_is_stable PARSES the output back rather than string-matching it, so
     formatting can change and the contract cannot, and asserts absent help is null rather than an
     empty string -- the distinction a consumer would care about.
     a_string_becomes_a_message_and_nothing_more asserts the conversion FABRICATES NOTHING. -->
- [x] Today's terminal output is unchanged for the migrated site, or the difference is stated
<!-- Unchanged for the 313 mechanically converted sites: Display renders exactly the old string,
     asserted by display_renders_exactly_the_old_string, and 159/159 fsh-test green throughout.
     ⚠️ TWO DIFFERENCES, STATED RATHER THAN QUIET: (1) a diagnostic carrying help now prints a
     second dimmed line -- new output, and the point of the intent. (2) spine failures changed from
     Debug dumps to real messages: `unterminated quote` where it read
     `Incomplete(LexIncomplete { kind: UnterminatedQuote, .. })`. Both are improvements; neither is
     invisible. -->
- [x] The INT-199 convention is expressible in the model -- what happened, what changed, why, what
      to do next -- rather than reconstructed per site
<!-- message = what happened · labels = where · help = what to do next · code = a stable handle.
     Demonstrated end to end: a bare `&` reports "expected a command" with
     "shell structure here requires a command, and there is none" -- which is ParseError::NoCommand's
     own rustdoc, previously readable only by someone reading the source. The convention is now
     PRODUCED FROM THE MODEL rather than hand-written at each site. -->
- [x] Each gate carries evidence per INT-158
<!-- this block. -->
<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
