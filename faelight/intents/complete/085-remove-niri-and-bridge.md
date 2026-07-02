---
id: 085
date: 2026-06-23
type: future
status: complete
title: "Remove Niri + faelight-niri-bridge (retired compositor cleanup)"
tags: [decommission, niri, compositor, cleanup, friday, doctor]
version: TBD
---
## Why
Niri is the retired compositor -- MangoWM daily driver since 11.0.0; Miracle is the planned
second compositor (not Niri). Niri NOT installed (which niri empty) but referenced ~28 places
across engine + configs as "retired fallback". Dead weight muddying compositor logic, Friday's
knowledge, and the doctor.
## CRITICAL SAFETY (verified 2026-06-23)
- greetd does NOT reference niri -- login is on mango. NOT lockout-class.
- niri NOT on PATH, NOT built by framework16 flake (but pkgs.niri IS in hosts/vm:85).
## Full footprint (scanned 2026-06-23)
ENGINE (~28 refs): bootstrap/mod.rs:36, narrative/mod.rs:45, events/mod.rs:825,1928,
lock/mod.rs:12, doctor/checks.rs:445-458,1151-1156, doctor/mod.rs:181, doctor/entropy.rs:151,
friday/mod.rs:466,520,965-966 (DECIDE Friday's niri knowledge), knowledge/mod.rs:305-309,
fetch/mod.rs:58, integrity/mod.rs:1019,1050, faelight-release/learning.rs:118,
faelight-compositor/winit.rs:1+main.rs:8 (comments), faelight-login/main.rs:117,120
(niri-session option -- coordinate INT-005).
NIX: modules/desktop/niri.nix (whole module), hosts/vm:85 (pkgs.niri), hosts/framework16:50 (comment).
REGISTRY: aliases.toml:213-214, tools.toml:280.
CONFIGS: config/niri/, ~/.config/niri/.
PKGS: pkgs/faelight-logout/main.py:40 (NIRI_SOCKET branch).
LEAVE (history): intents/decisions/276, incidents/112+190.
## Approach
Total-scan-first (this IS the scan), leaf removal like INT-072. Friday/knowledge get a
deliberate decision. One build/deploy at end. Verify: d clean, no niri refs, mango intact.
## Open decision
Friday's niri facts: delete or repoint? faelight-login niri-session: coordinate INT-005.
## Sequencing
FIRST in compositor-swap chain: Niri -> Pinnacle (INT-086) -> Miracle (INT-087).
## The Rule
"What the forest no longer runs, it lets go -- cleanly." 🌲

## Gate Check
✅ DEMONSTRATED (2026-06-23) -- Niri fully removed from code; mango intact; NOT lockout-class.
Safety verified: greetd launches mango (--cmd mango), never referenced niri.
Removed across ~12 files / ~28 refs, grouped + cargo-checked at every step:
- Group A: dead prints/comments (bootstrap, narrative, doctor pacman, compositor comments)
- Registry: faelight-niri-bridge [[tool]] + [[alias]] (niri-bridge/nb)
- Group C: Friday's niri knowledge (knowledge niri_config_reload entry + 3 friday/mod.rs facts)
- Group B: doctor compositor detection (keybind if/else niri arm, process niri tuple, entropy list)
- Group D: events niri-bridge hints, fetch get_wm niri branch, release niri scope,
  faelight-update pkg list; the WHOLE AutostartRetiredToolCheck integrity feature
  (struct+impl+registration+fix arm -- it edited niri config.kdl, dead capability) + dead
  Category::Autostart variant
- Nix/config: modules/desktop/niri.nix (empty, git rm), hosts/vm pkgs.niri, config/niri/ dir,
  faelight-logout NIRI_SOCKET branch
- Docs: ~10 files updated Niri->MangoWM (headers, architecture, theory, boot flow, policies);
  forest-resilience recovery section -> honest 'NEEDS MANGO STEPS -- see INT-056' placeholders
  (did NOT guess mango recovery commands)
DEFERRED (deliberate): faelight-login niri-session option (main.rs:117,120) -> INT-005
(login-adjacent). LEFT: history/changelog/meta (correct), 2 doc-comments, migration-planning
language in design-system + PINNACLE-MIGRATION-PLAN.
VERIFIED LIVE (gen 223): which niri -> not found; d 0 failures, integrity 100%, MangoWM running,
mango keybinds clean; core registry retired 12->11. Builds clean (no dangling niri.nix import).
NOTE: Friday's runtime knowledge cache still surfaced a niri hint post-deploy -- source is
clean; state.db knowledge reconciliation on reload is a Friday-work item.
