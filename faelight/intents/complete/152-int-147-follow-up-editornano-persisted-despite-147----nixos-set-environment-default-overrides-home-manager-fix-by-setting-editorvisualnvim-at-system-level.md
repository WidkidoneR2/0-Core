---
id: 152
date: 2026-07-12
type: future
title: "INT-147 follow-up: EDITOR=nano persisted despite 147 -- NixOS set-environment default overrides home-manager; fix by setting EDITOR/VISUAL=nvim at system level"
status: complete
tags: [editor, nvim, nixos, environment, int-147-followup]
---

## Vision
`EDITOR` is `nvim` everywhere -- current session, fresh login, git, `svi`, any tool that reads it.
The value that WINS the login chain is nvim, not just the file we believe is authoritative.

## The Problem
INT-147 was closed as "editor switched to nvim", but weeks later `$EDITOR` resolved to `nano` --
in the running session AND in a fresh login shell. So 147's fix did not actually hold.

## Root Cause (traced, not assumed)
The winning value comes from `/etc/set-environment` (NixOS system-level env file), which exported
`EDITOR="nano"`. This file is sourced in the login chain and WINS over home-manager's
hm-session-vars.sh (which correctly exported nvim). The nano value is a NixOS DEFAULT written into
set-environment -- it is NOT in any .nix file we wrote, and it survived `programs.nano.enable = false`
(that disables the nano package/module, but a default still set the EDITOR variable).

INT-147's model was "remove the system EDITOR, let home-manager be the sole source." That was wrong:
with nothing explicitly setting EDITOR at the system level, the NixOS default re-populated
set-environment with nano, which then overrode home-manager. 147's gate ("login-chain resolution
shows nvim") tested hm-session-vars (which IS nvim) -- NOT the full ordered chain where
set-environment overrides it. The gate tested the authoritative-in-theory file, not the
wins-in-practice value.

## The Fix
Set EDITOR at the SYSTEM level so set-environment itself exports nvim (nix/hosts/framework16/
configuration.nix):
  environment.variables.EDITOR = "nvim";
  environment.variables.VISUAL = "nvim";
Now BOTH layers (system set-environment + home-manager hm-session-vars) agree on nvim -- whichever
wins the ordering, the answer is nvim. Belt-and-suspenders, not redundancy: it removes the
"which layer wins" fragility entirely. Commit aac0c7ee.

## Success Criteria
- [x] root cause traced to set-environment (not assumed) <!-- 2026-07-13: bash --login -x trace showed `. /nix/store/...-set-environment` -> `export EDITOR=nano`. Definitive, from the trace not a hypothesis (2 earlier hypotheses -- env-save restore, stale session -- were disproven by testing first). -->
- [x] system-level EDITOR/VISUAL=nvim set in configuration.nix; deployed <!-- 2026-07-13: environment.variables.EDITOR/VISUAL=nvim added at configuration.nix:185-186. Deployed gen 363 (set-environment.drv rebuilt). -->
- [x] GATE: /etc/set-environment now exports nvim (was nano) <!-- 2026-07-13: grep EDITOR /etc/set-environment -> export EDITOR="nvim". -->
- [x] GATE: fresh FULL login chain resolves to nvim -- the test 147 should have run <!-- 2026-07-13: from a clean bash (bypassing fsh's frozen-env interception), env -i ... bash --login -c 'echo $EDITOR' -> [nvim]. This is the full-chain value, not just hm-session-vars. -->
- [x] no behavior regression; system healthy post-deploy <!-- 2026-07-13: dep clean, 0 failed checks, health 90% (drift-only, reboot clears). -->

## Relationship
Follow-up to: INT-147 (editor hx->nvim). 147 correctly fixed home-manager but its gate tested the
wrong layer, missing that set-environment overrides it. This intent fixes the actual winning layer
and uses the correct full-chain gate. 147 stays closed (its home-manager work was right); this
documents + fixes the incomplete part.

## Notes
- NOT a regression -- 147 was incomplete from the start; the nano was always winning, we just never
  ran the full-chain test to see it. The `d`/session showed nano all along; it surfaced now during
  INT-134 reconciliation when EDITOR was checked directly.
- The running mango session still shows nano (froze the old env at login) -- cosmetic, clears on next
  logout/login. The CONFIG is correct (proven by the clean-bash full-chain gate); next login is nvim.
- LESSON (banked): a "which config layer wins" question is answered by tracing the full chain's
  RESOLVED OUTPUT (`bash --login -x` / the actual $EDITOR a fresh login gets), NOT by checking the
  file you believe is authoritative. Same family as the stale-binary and em-dash lessons: verify the
  real resolved value, not the input you assume determines it. 147's gate checked the input file;
  the correct gate checks the output.
- Process lesson: theorized twice ahead of evidence (env-save restore; stale session) -- both wrong,
  both caught by testing before acting (the fresh-login test showed nano, killing the stale-session
  fix before we wasted a logout on it). Trace-first beats hypothesize-first.
