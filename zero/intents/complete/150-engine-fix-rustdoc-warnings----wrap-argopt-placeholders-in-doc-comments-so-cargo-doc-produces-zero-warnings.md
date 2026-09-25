---
id: 150
date: 2026-07-12
type: future
title: "engine: fix rustdoc warnings -- wrap <arg>/[opt] placeholders in doc comments so cargo doc produces zero warnings"
status: complete
tags: [engine, rustdoc, hygiene, cleanup]
---

## Vision
`cargo doc` on the engine (`core`) produces ZERO warnings. Doc comments that show CLI usage
(`core deploy check <tool>`) are formatted so rustdoc doesn't mis-parse their placeholders --
clean doc builds, so `dev doc core` (INT-134 Lane 3) opens tidy docs and future doc regressions
are visible instead of buried under 42 pre-existing ones.

## The Problem
Building the engine docs surfaces 42 rustdoc warnings (counted 2026-07-12 via `dev doc core`).
NOT environment-specific -- these are plain doc-comment formatting, identical on any OS/toolchain
(NOT Arch-era: no platform assumption, just angle-brackets/brackets rustdoc universally mis-parses).
They were latent because nothing built the docs until `dev doc` did. Two patterns:

1. **Unclosed HTML tag** (majority) -- `<tool>`, `<id>`, `<file>`, `<version>`, `<outcome>`,
   `<statement>`, `<question>`, `<type>`, `<payload>`, `<SEQ>`, `<date>`, `<term>`, `<action>`,
   `<goal1>`, `<engine>`, `<subject>`, `<proposed>`, `<actual>`, `<matched>`, etc. rustdoc reads
   `<word>` as an HTML tag and warns it's unclosed.
2. **Broken intra-doc link** -- `[domain]`, `[resolution]`, `[limit]`, `[tool]`. rustdoc reads
   `[word]` as a doc-link to an item named `word`, which doesn't exist.

## Scope (VERIFIED 2026-07-12)
~42 warnings across ~15 engine files. Known sites (from the cargo doc output):
cli/parser.rs, domains/{alignment, daemon, deploy, deps, delegate, engines, events, friday/planning,
friday/mod, journal, self_transform, weight_engine, db, genealogy, strategy}. All in `///` or `//!`
doc comments that are CLI-usage strings. Some lines carry multiple placeholders
(`core deploy record <tool> <version> <outcome> <duration_ms>` = 3 warnings on one line).
Some use em-dash (--) vs double-dash -- watch anchors.

## The Solution
Wrap each affected doc-comment usage string in backticks so its whole content becomes inline code,
which rustdoc does NOT parse for HTML tags or intra-doc links. One consistent transform kills both
warning classes at once, and it is CORRECT -- these ARE command examples, they SHOULD be
code-formatted:
  `/// core deploy check <tool>`  ->  ``/// `core deploy check <tool>` ``
Backticking the whole line (not each placeholder individually) handles multi-arg lines cleanly.
Where a line has prose after the usage (`-- pre-deploy gate`), backtick only the command portion,
leave the prose: ``/// `core deploy check <tool>` -- pre-deploy gate``.
Surgical, per-site (recon every location first) -- NOT a blind regex, to avoid mangling comments
that carry other content.

## Success Criteria
- [x] every rustdoc warning site identified from `cargo doc -p core` output <!-- 2026-07-12: 42 warnings across 31 lines in 14 files, enumerated from cargo doc output. -->
- [x] each affected doc-comment usage string backticked (command portion), prose preserved <!-- 2026-07-12: 31 sites wrapped in backticks. 23 double-dash applied first pass; 8 em-dash lines needed dash-free command-portion matches (paste layer normalizes -- to --); final 2 needed /// prefix to disambiguate from code string literals. -->
- [x] `cargo doc -p core` produces ZERO warnings <!-- 2026-07-12: cargo doc -p core --no-deps | grep -c warning: -> 0 (was 42). -->
- [x] docs render correctly (usage strings show as inline code) <!-- 2026-07-12: cargo doc generated index.html clean; usage strings now inline-code formatted. -->
- [x] engine rebuilt; no behavior change (doc-comment-only edits) <!-- 2026-07-12: cargo build -p core clean; doc-comment-only, zero runtime change. -->
- [x] no NEW warnings introduced <!-- 2026-07-12: total went 42 -> 0, no new warnings anywhere in the core doc build. -->

## Relationship
Surfaced by: INT-134 Lane 3 `dev doc` (the rustdoc-lookup command whose local-crate route ran
`cargo doc` and exposed these). `dev doc` going forward keeps engine docs honest -- this clears the
backlog so future warnings are visible.
Filter: clean doc builds deepen trustworthy tooling (a warning means something, not noise); 42 buried
warnings mean the next real one hides. In-filter, low-risk (doc-comment-only, no logic).

## Notes
- NOT Arch-era. Environment-independent formatting hygiene -- would warn identically on any OS.
  Latent only because docs were never built until `dev doc` existed. Filed to keep the "Arch-era"
  label sharp (it predicts environment-mismatch bugs; these aren't that).
- Doc-comment-only changes -- zero runtime behavior change. The risk is purely "did a backtick land
  in the wrong place," caught by re-running cargo doc.
- `dev doc core` is the verification tool: run it before (42 warnings) and after (0).
