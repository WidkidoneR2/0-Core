---
id: 098
date: 2026-06-28
type: future
title: "Forest hygiene pass: registry reconciliation + Deadwood orphan cleanup"
status: in-progress
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
- [ ] Phase 2: 11 unregistered tools added to registry (real category + description each)
- [ ] Phase 3: dead palette alias removed; cache-status + cache-push resolved
- [ ] Phase 4: 27 ghost intent references resolved (repoint or remove, per-reference)
- [ ] faelight-deadwood re-run: orphans cleared or remaining ones documented as intentional
- [ ] Health green, integrity 100%, tree clean

## Notes

- Prior art: INT-083 (registry-alias-hygiene, complete) -- check what it already did.
- Phases 1-3 are bankable quickly; Phase 4 is the long pole (27 judgment calls, may span
  sessions). Better to do fewer ghosts well than all 27 carelessly.
- faelight-deadwood reports only, never deletes -- every cut is a human decision.
