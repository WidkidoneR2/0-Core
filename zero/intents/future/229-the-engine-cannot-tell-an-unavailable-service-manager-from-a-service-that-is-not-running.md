---
id: 229
title: "the engine cannot tell an unavailable service manager from a service that is not running"
status: planned
type: fix
priority: medium
date: 2026-08-23
tags: [engine, portability, capability, void, int-227, int-222]
---

## Vision
A probe that could not run says so. It does not answer.

## ✅ RECON FIRST, and it narrowed the intent twice
Measured 2026-08-23 across `faelight/engine/src/`, non-comment lines.

    systemctl 14 · flake 18 · /run/current-system 4 · journalctl 3
    nixos-rebuild 2 · nix-env 1 · nix-store 1 · /nix/store 1        = 44 sites

⭐ AND `/home/christian` IS ZERO. The engine has none of the hardcoded-home defects the shell had --
better hygiene than fsh, and worth saying so.

**The fourteen `systemctl` sites are not fourteen problems:**
- THREE ARE NOISE -- Friday's distro-hint tables (`friday/mod.rs:530`, `:583`, `:1029`) are strings
  MAPPING the word `systemctl` to "arch"/"nixos". Not calls.
- TWO ARE MESSAGE TEXT -- `delegate/mod.rs:198` and `events/mod.rs:733` print a command for a person
  to run. Not calls.
- FIVE BELONG TO INT-222 -- `doctor/checks.rs` ×5. A health check reporting a false Pass on a
  non-systemd machine is that intent's own thesis, and splitting the doctor across two intents is
  how a census ends up owned by nobody.
- **FOUR ARE THIS INTENT.**

⚠️ AND THERE IS NO SEAM. Unlike fsh, where four store queries went through one `nix_query_lines`
helper, every engine site spawns `Command::new("systemctl")` directly. There is nothing to fix once.

## The Problem -- and it is NOT "systemctl"
Reading the four found all three failure shapes, one each, and ONE OF THEM IS ALREADY CORRECT:

    notify/mod.rs:36     .unwrap_or_else(|_| "unknown".to_string())     ✅ HONEST
    strategy/mod.rs:1575 .unwrap_or(false)                              ⚠️ LIES INTO A SCORE
    doctor/entropy.rs:130 if let Ok(output)                             ⚠️ SILENTLY EMPTY

★ `notify` IS THE MODEL, NOT A DEFECT. It reports "unknown" and prints it. That is exactly the
degrade INT-227 spent a session installing in the shell, already here, already right.

⚠️⚠️ `strategy/mod.rs:1571` IS THE SHARP ONE. It probes whether `faelight-insightd` is active and
`unwrap_or(false)` turns "could not ask" into "not running" -- then subtracts FIVE POINTS from a
strategy score. On a machine without systemd the score is silently lower and nothing says why. That
is a NUMBER COMPUTED FROM A QUESTION NEVER ASKED, the same defect as the store summary printing
`total closure: 0.0 B` as a measured size.

⚠️ `doctor/entropy.rs:130` enumerates services inside `if let Ok(..)`. Enumeration failure produces
the same output as a machine with no services.

## THE RULING (2026-08-23): SERVICE STATE IS TRI-STATE
    active  ·  inactive  ·  unknown

⭐ AND THE RULE THAT OUTLIVES THIS INTENT, stated so a future build cannot satisfy the letter and
miss the point: **a missing service manager, an unavailable service state, or a failed probe must
never silently become a valid negative answer or a fabricated score.**

★ THE REPLACEMENT'S RETURN TYPE IS PART OF THE RULING, because otherwise the build swaps
`Command::new("systemctl")` for a helper and preserves the semantic bug exactly. A caller computing
a score MUST NOT collapse `unknown` into `inactive`. For enumeration: **failure is not zero
services.**

⚠️ NOT A MECHANICAL REPLACEMENT INTENT. The seam is not "systemctl calls" -- it is distinguishing
ABSENCE from a legitimate negative result. That distinction applies beyond systemd and beyond NixOS,
which is why the invariant is written without naming either.

## Success Criteria
- [ ] G1 SERVICE STATE IS TRI-STATE IN THE TYPE, not by convention: a probe returns
      active/inactive/unknown (or an equivalent typed result), and `unknown` is representable
      without a sentinel
- [ ] G2 THE SCORE CANNOT BE FABRICATED. `strategy/mod.rs` no longer converts an unanswerable probe
      into a five-point deduction. What it does instead is stated -- omit the factor, or report the
      score as incomplete -- but it does not quietly subtract
- [ ] G3 ENUMERATION FAILURE IS NOT ZERO SERVICES. `doctor/entropy.rs` distinguishes "could not
      enumerate" from "enumerated, found none"
- [ ] G4 `notify` IS LEFT ALONE, and the reason is recorded rather than assumed: its `"unknown"`
      degrade is already the correct behaviour and rewriting it would be churn
- [ ] G5 RUNTIME PROOF, per INT-227 G6: a test strips the tool from PATH and asserts the score is
      not silently reduced and the enumeration is not silently empty. ⚠️ NOT a source-text check --
      INT-227's scanner passed with a live guard disabled, because a checker reading text cannot
      establish a runtime property
- [ ] G6 THE BOUNDARY HELD: no change to `doctor/checks.rs` (INT-222), the Friday hint tables, or
      the user-facing instruction strings
- [ ] G7 each gate carries evidence per INT-158

## Non-goals
- The five `doctor/checks.rs` spawns. INT-222 owns the doctor, and a census split across two intents
  is a census nobody owns.
- Redesigning Friday's distro-hint tables or the printed instructions. They name a tool; they do not
  assume one.
- ⚠️ BECOMING A NIX INTENT. The eighteen `flake` references are RECONNAISSANCE, not scope: they
  cluster in `nix/mod.rs` and `app/context.rs` but leak into `doctor/` and `friday/`. A deploy path
  that only works on NixOS is not a portability bug if the platform says so. Whether that leak
  matters is a separate question, asked separately.
- Removing systemd support. The machine that has it should use it.
