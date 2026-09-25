---
id: 098
date: 2026-06-28
type: future
title: "Forest hygiene pass: registry reconciliation + Deadwood orphan cleanup"
status: complete
tags: [Nix, Deadwood, registry, cleanup]
---

## Why

037's README generator surfaced drift between registry/tools.toml and what is actually on
disk. faelight-deadwood independently flags 32 orphans. The registry, intent ledger, and a
few aliases/scripts have accumulated cruft that no longer matches reality. Clean it before
1.0.0 so the catalog is accurate, the Deadwood warning clears, and the intent ledger (the
forest's memory) is internally consistent.

## Scope -- the drift is four separate problems

1. **Registry orphans (2):** security-audit, faelight-notifyctl -- registered deployable,
   no binary on PATH. Mark retired or deployable=false.
2. **Unregistered tools (11):** real crates on disk not in the registry (db-browse,
   faelight-ade, faelight-context, faelight-core, faelight-deadwood, faelight-nix,
   faelight-wsd, friday-chat, fsh-test, gen-diff, + faelight-compositor). Add real entries.
3. **Dead alias + orphaned scripts (3):** palette alias (target gone), cache-status +
   cache-push scripts (referenced nowhere; note INT-068 created the cache commands).
4. **Ghost intent references (27):** INT-NNN references inside intent files pointing to
   nonexistent intent files. Per-reference judgment: repoint if the intent was renamed/
   renumbered (e.g. Arch-era INT-186/260/261/267), remove if cancelled/never-created.
   NEVER blind-delete -- each reference is part of the project's memory.

## Gates

- [x] Phase 1: 2 registry orphans retired (security-audit->core security, faelight-notifyctl->core notify; verified superseded, Deadwood clean)
- [x] Phase 2: 11 unregistered tools added to registry (real category + description each) <!-- INT-130 2026-07-10: verified LIVE -- all 11 present in faelight/registry/tools.toml (db-browse, faelight-ade, faelight-context, faelight-core, faelight-deadwood, faelight-nix, faelight-wsd, friday-chat, fsh-test, gen-diff, faelight-compositor -- checked each by name, 11/11 FOUND). Commit 085e1127. -->
- [x] Phase 3: dead palette alias removed (target faelight-palette decommissioned per INT-072). cache-status + cache-push VERIFIED LIVE (invoked by fsh `cache` builtin + needed by in-progress INT-043) -- KEPT, not removed; Deadwood false-positives them. Built faelight-deadwood --purge (interactive + bulk) for safe dead-weight ONLY (dead aliases, stale .bak, dead keybinds); scripts/ghosts/registry/modules unpurgeable by design (action:None). Guards: git-clean required, per-item default-skip, re-verify before act. Proven: surgical single-line removal + exclusion of unsafe categories.
- [x] Phase 4: RESOLVED BY REMOVAL -- removed the dangling-intent-reference CHECK from faelight-deadwood entirely. Intent cross-references are documentation (the intent ledger's domain), NOT dead code. Deadwood does only its real job now: dead aliases, stale .bak, dead keybinds, registry orphans, orphaned scripts, orphaned Nix modules. Dashboard parser (doctor/mod.rs) aligned to 7-field summary.
- [x] Deadwood re-run: all structural orphans cleared (registry 0, modules 0, ghost-intents check removed). 2 remaining items are cache-status/cache-push -- LIVE scripts (fsh `cache` builtin + INT-043), documented false-positives. Follow-up: teach Deadwood to recognize dynamically-invoked scripts.
- [x] Health 93% ADVISORY (2 warnings = transient generation-drift, clears on reboot), integrity 100%, tree clean + pushed. Deadwood dashboard line GREEN.

## Notes

- Prior art: INT-083 (registry-alias-hygiene, complete) -- check what it already did.
- Phases 1-3 are bankable quickly; Phase 4 is the long pole (27 judgment calls, may span
  sessions). Better to do fewer ghosts well than all 27 carelessly.
- faelight-deadwood reports only, never deletes -- every cut is a human decision.


### Phase 3 finding (2026-06-28): Deadwood false-positives live scripts
cache-status / cache-push are flagged "referenced nowhere" but are LIVE -- the fsh `cache`
builtin shells out to them (INT-068), and in-progress INT-043 depends on cache-push.
Deadwood's static scan can't see dynamic invocation. FOLLOW-UP: teach faelight-deadwood an
allowlist (or detect the fsh `cache` arm) so it stops flagging these. For now they are
correctly EXCLUDED from --purge (orphaned-scripts category is action:None, unpurgeable).

<!-- Gates reconciled per INT-130, 2026-07-10: GENUINE reconcile. 5/6 gates already ticked with commits (Phases 1/3/4, deadwood re-run, health). Phase 2 (register 11 tools) was the only open one -- VERIFIED LIVE: all 11 present in registry/tools.toml (11/11 by name), commit 085e1127. Deadwood check is GREEN in doctor (live). 7/23. -->
