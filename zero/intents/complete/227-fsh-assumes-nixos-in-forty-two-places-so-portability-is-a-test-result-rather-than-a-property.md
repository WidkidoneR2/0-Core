---
id: 227
title: "fsh assumes NixOS in forty-two places, so portability is a test result rather than a property"
status: complete
type: infrastructure
priority: high
date: 2026-08-23
tags: [fsh, portability, platform, void, omarchy, architecture]
---

## Vision
fsh asks the platform what it is, in one place, and everything else reads the answer.

## The Problem
MEASURED 2026-08-23 across `faelight-shell/src/`, non-comment lines only:

    /home/christian        15      systemctl               5
    /run/current-system     7      journalctl              3
    /nix/store              5      nixos-rebuild           2
    nix-env                 1      pacman                  4  (see below)

Forty-two live sites. But they are THREE DIFFERENT PROBLEMS, and treating them as one
would produce a bad fix:

A -- WRONG ON EVERY SYSTEM, not only off NixOS.
    `mod.rs:11868` falls back to `"/home/christian"` when `$HOME` is unset.
    `mod.rs:14487` hardcodes `/home/christian/0-core/scripts/...`.
    These are defects for ANY user. Portability merely made them visible.

B -- REAL PLATFORM CAPABILITIES that should DEGRADE, not disappear.
    `systemctl`, `journalctl`, `nixos-rebuild`, `nix-store`, `/run/current-system`.
    A shell that shows journal logs on a systemd machine is not wrong. A shell that
    BREAKS without journald is.

C -- NOISE. `pacman` is a TYPO-CORRECTION list entry beside `vim`, `curl` and `ssh`
    (`mod.rs:9713`), not a package-manager call. Comments account for the rest.

## THE RULING (2026-08-23): DECLARE THE PLATFORM ONCE
Rejected: per-site detect-and-degrade. It spreads the same question across forty-two
places and answers it forty-two times, which is the shape this ledger keeps removing --
the doctor's hardcoded lists, `env`'s ten variables, three `sk` call sites that drifted.

CHOSEN: ONE module answers what the platform IS -- service manager, log source,
package manager, system-rebuild command, store paths -- and every caller reads it.
The design is Oligarchy's `custom.platform` applied to a shell.

⚠️ WHAT THIS IS NOT. It is not "support three distros". It is REMOVING ASSUMPTIONS.
The distinction matters because fsh may end up connected to Zero Core or independent,
and nobody knows which yet. A shell with no platform assumptions works BOTH ways -- so
this does not presume a direction, it makes both directions possible.

★ AND THE DESIGN RULE IT INSTALLS, which outlives the intent: every time the thought is
"I'll just make fsh assume X exists", ask -- would this work on Void, Omarchy, NixOS, or
any other distro? If no, that is a dependency worth questioning. Void is the sharp axis:
runit rather than systemd, optionally musl rather than glibc. Omarchy and NixOS both give
systemd and glibc, so neither would find these bugs.

## Success Criteria
- [x] G1 THE CENSUS IS AN ARTIFACT, NOT A PARAGRAPH: a committed file listing every
      site with its category (A wrong-everywhere / B platform-capability / C noise).
      Produced mechanically so it can be re-run and diffed
<!-- docs/platform-census.md + generate-platform-census.py. 56 sites, each with a stated reason.
     ⭐ AND IT GREW TWO CATEGORIES THE INTENT DID NOT HAVE, because reading forced them:
       D  already correct -- candidate lists that probe and fall through, PATH additions that are
          harmless when absent, and TEST FIXTURES where a literal path is right
       GUARD  the has_tool checks and their messages -- the fix, not a finding
     The check refuses to pass while any site is unread, and three rounds of that refusal found
     what a count would have hidden: a matcher believed machine-specific was already shape-based,
     and a "unread" site was a fixture whose #[test] sat four lines above the match. -->
- [x] G2 CATEGORY A IS FIXED FIRST AND SEPARATELY: no fallback to a literal home
      directory, no hardcoded absolute path into one user's checkout. These are defects
      independent of portability and should not wait for an abstraction
<!-- DONE 2026-08-23, commit be3fc0c1, before the module existed. mod.rs:11868's fallback to a
     literal home replaced with an early return; mod.rs:14487's hardcoded script path replaced with
     current_exe(). ★ THAT CHECK HAD NEVER PASSED -- the path did not exist on this machine, so it
     reported missing every run since it was written. -->
- [x] G3 THE PLATFORM MODULE EXISTS -- AND IT ANSWERS TWO QUESTIONS, NOT FIVE
<!-- ⚠️ THE GATE'S OWN LIST WAS THE WRONG SHAPE, and the first draft followed it: six questions
     with ONE caller between them. Six questions and one consumer is a dumping ground rather than
     an abstraction. Services, logs, rebuild and store queries are CAPABILITY DETECTION; build
     identity is IDENTITY. Different concerns, different lifetimes, and mixing them because both
     happen to be platform-dependent is how a module becomes a junk drawer.
     src/platform.rs answers exactly two:
       running_build_identity()  -- which build is running. On Nix the store path stays the
         identity, because the old code's comment recorded why current_exe() is unreliable there
         (the deployed binary is makeWrapper-wrapped); elsewhere the executable IS the artifact.
       has_tool(name)            -- is this executable on PATH. A GENERIC PRIMITIVE with specific
         callers: not has_systemctl(), which accumulates has_journalctl, has_nix_store, has_pacman
         until the module is the junk drawer again. Existence, not usability. Reads PATH directly
         rather than shelling to `command -v`, which would assume a shell inside a portability fix.
     ★ AND MOST OF THE REST NEEDED NO ABSTRACTION: three self-location sites already probe a
     candidate list and fall through, so on Void the Nix entries simply miss. -->
- [x] G4 A CATEGORY B CAPABILITY DEGRADES RATHER THAN BREAKS, demonstrated
<!-- ⚠️ AND WHAT THEY DID BEFORE IS THE SHARPEST ARGUMENT THIS INTENT HAS: THEY LIED.
     `services` used .ok() then unwrap_or_default(), so a missing systemctl produced an EMPTY
     TABLE -- a machine with services running told it had none.
     `logs -f` printed "streaming logs -- press Enter to stop", a separator, and then NOTHING
     FOREVER, because the spawn sat inside `if let Ok(child)`.
     A second, static journalctl spawn had the same defect and was found only by the lint.
     `nix_query_lines` returned vec![] on spawn failure, feeding FOUR callers that each read it as
     "no roots" / "no referrers" / "no references".
     `store_summarize_matches` printed "total closure: 0.0 B" as a MEASURED size.
     ★ Same defect class as a doctor check that cannot fail: the failure is indistinguishable from
     a legitimate empty result.
     Now: each names what is absent. The log guard had to move TWICE -- above the header, then
     above the stdin reader thread, which blocks waiting for the Enter that stops a stream that
     would never start. Commits 64d8215a, b86a4543. -->
- [x] G5 A TEST CONTROLS THE PLATFORM, so this is provable without a second machine
<!-- has_tool reads PATH, so stripping ONE directory simulates Void. Measured live:
     `services` with systemctl -> a real table; without -> "no systemctl on this system".
     `logs -f` without journalctl -> names the absence and RETURNS, where it used to block.
     `packages` without nix-store -> "cannot query the Nix store".
     `store why <path>` without nix-store -> "(unknown -- cannot query the store here)" for roots
     and referrers, where it would have claimed nothing pins the path. -->
- [x] G6 NO PLATFORM FAILURE BECOMES A SUCCESSFUL-LOOKING EMPTY RESULT. A mechanical
      check, red first.
      ⚠️ REWORDED 2026-08-23, AND THE ORIGINAL WAS UNACHIEVABLE. It asked that no site
      outside the platform module NAME a service manager or a store path -- but
      `generations` legitimately runs `nixos-rebuild`, and banning the name would make
      correct code fail the gate. Naming a tool is not assuming a capability. Same
      lesson as INT-169's logos gate: a gate must test the INVARIANT, not an accidental
      implementation detail, and a gate that cannot pass is the same defect as a check
      that cannot fail (INT-222).
      ★ THE INVARIANT THE WORK ACTUALLY FOUND, which is stronger and checkable: a
      capability failure must never be converted into a result that reads as an answer.
      Every defect this intent fixed was one of four patterns --
        `.ok()`                 swallowing a spawn or query failure
        `unwrap_or_default()`   turning failure into an empty collection
        `if let Ok(..)`         turning failure into an apparently empty operation
        a zero or default value standing in for an unavailable capability
      MEASURED EXAMPLES: an empty services table on a machine with services running; a
      log stream that printed a header and nothing forever; a store query returning
      "no GC roots" without asking the store; `total closure: 0.0 B` as a measured size.
      ⭐ AND THE RULE THAT FALLS OUT: `Result` and `?` fail honestly by construction.
      The three patterns above are where lies come from.
<!-- ⚠️⚠️ THE FIRST ATTEMPT AT THIS GATE PROVED IT COULD NOT FAIL, WHICH IS WHY IT MOVED.
     A scanner read source text for swallowed errors near a platform spawn. Disabling a live guard
     with `if false &&` left the guard TEXT in place, so the scanner still saw the call and passed.
     A checker that reads text cannot establish a runtime property, and the PLATFORM-CHECKED marker
     that grew alongside it is a DECLARATION rather than evidence -- a comment can claim anything.
     ★ SO THE GATE IS THE RUNTIME TEST, and the scanner is demoted to a LINT that surfaces new
     candidates. fsh-test cases, 162/162:
       services_without_systemctl_reports_unavailable
       logs_without_journalctl_reports_unavailable
       packages_without_nix_store_reports_unavailable
     Each strips ONE tool from PATH, runs the real command, and asserts SEMANTICALLY -- not that
     the platform was consulted, but that a person is told the capability is missing rather than
     shown an empty result. Commit 09c65d0c. -->
- [x] G7 each gate carries evidence per INT-158
<!-- this block. -->

## Non-goals
- Supporting distros nobody will run. The matrix is NixOS, Void, Omarchy -- and the
  point is the absence of assumptions, not the presence of three code paths.
- Removing NixOS capabilities. `nixos-rebuild` and `nix-store` are genuinely useful on
  the machine that has them; they simply must not be assumed.
- The separate fsh repository. That is structural and belongs in its own intent, though
  it expresses the same idea: a shell that lives inside 0-Core cannot be installed
  without it.
- Anything outside `faelight-shell/src/`. The engine and the other tools have the same
  question and a different answer; mixing them would make this too large to finish.
