---
id: 123
date: 2026-07-06
type: future
title: "Release changelog polish: cap What Shipped + strip INT-numbers from Notable Changes"
status: complete
tags: [faelight-release, changelog, public]
---

## Why
The release README front page dumped all ~86 intents into "What Shipped", leaked internal
INT-numbers into public Notable Changes (INT-030/033/040 were visible), included a
"Built by one developer" builder-reveal, and had wrong Forest DNA stats. Polish it so the
public front page is scannable, INT-free, and builder-hidden -- fixed in the generator so
future releases inherit it.

## Gates (reconstructed from commit 2193e026 per INT-130 -- charter was an empty stub)
- [x] "What Shipped" capped at 15 + full-changelog link (was dumping all ~86 intents) <!-- STAMP-123-DONE / INT-130 2026-07-10: VERIFIED IN SOURCE -- readme.rs:164-172 ('What Shipped -- public titles only'). Commit 2193e026. -->
- [x] INT-numbers stripped from public Notable Changes titles <!-- INT-130 2026-07-10: VERIFIED IN SOURCE -- clean_intent_title() at readme.rs:112, applied at :168 + :203. INT-030/033/040 no longer leak. -->
- [x] "Built by one developer" builder-reveal removed <!-- INT-130 2026-07-10: per commit 2193e026 'remove Built by one developer builder-reveal'. -->
- [x] Forest DNA stats corrected (36 tools / 109k lines; was 46/99) <!-- INT-130 2026-07-10: per commit 2193e026. -->
- [x] Fixed in the readme.rs generator (future releases inherit) + live Morphwood README regenerated <!-- INT-130 2026-07-10: commit 2193e026 'Fixed in readme.rs generator AND regenerated the live Morphwood README.' faelight-release bumped 1.0.1->1.0.2 (commit c9c11b16). -->

## Note (INT-130 2026-07-10)
This charter was filed as an EMPTY TEMPLATE STUB (no Why/gates/body). The changelog-polish
WORK was genuinely done -- verified in source (readme.rs clean_intent_title + What-Shipped
cap) and in commit 2193e026. Gates above are RECONSTRUCTED from the actual work. Scope was
narrow: the release README FRONT PAGE only. A separate, broader effort -- per-tool READMEs
with changelog sections auto-updated on cicomplete (minor/patch/major), across all ~32 tools
via faelight-docs; only ~5/32 done -- is NOT this intent. If pursued, file it as its own
FEASIBILITY intent (recon whether auto-changelog-on-cicomplete is practical; decide by
demonstration, do not assume).

---
