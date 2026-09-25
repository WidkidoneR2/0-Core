---
id: 179
date: 2026-07-19
type: future
title: "sd: evaluate sed-with-sane-syntax as an installed forest tool"
status: complete
tags: [sd, tools, cli]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [x] Decision made: ADOPT or reject, on Christian's own evaluation.
      <!-- DONE. Christian evaluated sd directly and decided ADOPT (sed-with-sane-syntax,
      genuinely helpful). Verdict banked in the DECISION block below. -->
- [x] sd installed as a system package (not already present -- verified first).
      <!-- DONE 2026-07-21, deployed gen 408. Verify-first confirmed sd was NOT installed
      (which sd -> not found pre-dep). Added pkgs.sd to nix/hosts/framework16/configuration.nix
      (line 198, alongside its siblings bat/eza/fd/ripgrep/zoxide). No alias -- sd is already the
      terse ergonomic name (Christian's call). -->
- [x] sd is on PATH and FUNCTIONAL on the deployed system (installed AND works, not just present).
      <!-- DONE 2026-07-21, deployed gen 408. which sd -> /run/current-system/sw/bin/sd.
      Proven working by hand: `echo "hello world" | sd hello goodbye` -> "goodbye world" (literal
      replace); `echo "foo123bar456" | sd '[0-9]+' 'N'` -> "fooNbarN" (regex replace). Installed
      and functional. -->
- [x] Each gate carries evidence per INT-158.
      <!-- DONE. Commit for the package add + this tick; deployed gen 408; by-hand functional proof
      above. -->

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->


## DECISION (2026-07-20): ADOPT -- Christian evaluated sd himself
- Christian has looked at sd directly and decided it earns a place in the forest: sed-with-sane-
  syntax, genuinely helpful. Verdict is ADOPT.
- Remaining work is the INSTALL, not the evaluation: add sd to the system packages, alias it if
  desired, confirm it deploys. Deferred to a fresh session (decision banked here so it is not lost).