---
id: 151
date: 2026-07-12
type: future
title: "doctor: Rust Docs health check -- cargo doc must produce zero warnings, surfaced on every d (prevents INT-150-class doc-warning accumulation)"
status: planned
tags: [doctor, rustdoc, health-check, engine, prevention]
---

## Vision
Rustdoc warnings are caught on every `d`, not latent until someone happens to build the docs. The
doctor reports a "Rust Docs" check: green when `cargo doc` is clean, a visible Warn the moment a doc
comment regresses. The forest watches its own documentation the way it watches services, binaries,
and symlinks.

## The Problem
INT-150 cleared 42 rustdoc warnings that had accumulated INVISIBLY -- nothing ever ran `cargo doc`
until INT-134's `dev doc` command existed, so the warnings sat latent for an unknown span. Without a
guard they will silently return: every new `/// core foo <arg>` doc comment reintroduces the same
`<arg>`-as-HTML-tag warning, and no one sees it until the next manual doc build. The system that
catches services-down and binaries-missing should also catch docs-broken. Right now it doesn't.

## The Solution
Add a doctor check `check_rust_docs()` (faelight/engine/src/domains/doctor/checks.rs, alongside
check_services / check_binaries) that runs `cargo doc -p core --no-deps` and counts warnings:
- 0 warnings -> Status::Pass, "Rust docs clean" (or "N crates documented, 0 warnings")
- >0 warnings -> Status::Warn, "N rustdoc warnings" + fix hint ("dev doc core to see them")
Wire it into the doctor run (mod.rs check registry) and the cockpit label list (cockpit.rs), same as
the other checks. Follows the exact pattern of existing checks: spawn a Command, parse output, return
a CheckResult.

## OPEN DESIGN QUESTIONS (decide at cistart -- do NOT assume)
1. **Latency / when it runs.** `cargo doc` is ~0.1s when the doc cache is warm, but a COLD cache
   (after any engine code change) triggers a full doc rebuild -- potentially seconds. Options:
   (a) run on every `d` and accept occasional slowness; (b) run only in a deeper mode (`d --full`);
   (c) cache the last result like the health-status cache and refresh on a schedule/after deploy.
   MUST decide before building -- a `d` that randomly takes 5s would be a regression.
2. **Which crates.** `-p core` only (the engine, where all 42 were), or the whole workspace
   (fsh, all tools)? Start with `core`; widening is a later call.
3. **Warn vs Fail.** Doc warnings are cosmetic, not functional -> Warn (not Fail) so they don't drop
   health into DEGRADED or block a deploy. Confirm this is the desired severity.
4. **Interaction with INT-148 (Status::Unknown).** If the toolchain/cargo is unavailable (unlikely
   on this box, but e.g. a broken rust env), the check can't run -> should render UNKNOWN, not a
   false Pass/Warn. If 148 lands first, use Status::Unknown here; if not, degrade gracefully.

## Success Criteria
- [ ] design questions above RESOLVED and recorded (when-it-runs, which-crates, severity, unknown-handling)
- [ ] `check_rust_docs()` added to checks.rs, following the existing check pattern (spawn, parse, CheckResult)
- [ ] wired into the doctor run + cockpit label list so it renders in `d`
- [ ] GREEN demonstrated: with docs clean (post-150), `d` shows "Rust Docs" Pass / 0 warnings
- [ ] WARN demonstrated: temporarily break one doc comment (e.g. add `/// foo <bar>`), rebuild, run
      the check -> shows the warning count; then REVERT the break (leave docs clean)
- [ ] latency acceptable: `d` runtime with the check added stays within the chosen budget -- measured,
      not assumed (per design Q1's resolution)
- [ ] engine rebuilt + deployed; `d` clean and the new check green on the live system

## Relationship
Prevention half of: INT-150 (which cleared the 42-warning backlog). 150 fixed the instances; 151
builds the guard so they can't silently accumulate again. Same pattern as INT-146 -> INT-148 (fix the
symptom, then build the system that prevents the class).
Consumes: INT-134's `dev doc core` as the human-facing "show me the warnings" tool the fix-hint points to.
Coordinates with: INT-148 (Status::Unknown) for the can't-run case -- see design Q4.
Filter: a health system that watches its own docs deepens trustworthy self-observation; a blind spot
that let 42 warnings hide is exactly the kind of gap the forest should close. In-filter.

## Notes
- Chosen over the alternatives (pre-deploy gate; `#![deny(rustdoc::...)]` compile-error) because the
  doctor check INFORMS without INTERRUPTING -- surfaces regressions on `d` without blocking work or
  failing builds mid-prototype. A compile-deny could be added LATER if "impossible to regress" is
  wanted, but starts too aggressive.
- The 42 warnings were NOT Arch-era (environment-independent formatting) -- this check guards a
  formatting-hygiene class, distinct from the environment-mismatch bugs the "Arch-era" label predicts.
- Verification tool for the WARN gate: `dev doc core` (INT-134) shows the actual warnings a human
  would read.
