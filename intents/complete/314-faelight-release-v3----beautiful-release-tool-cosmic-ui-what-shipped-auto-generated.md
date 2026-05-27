---
id: 314
title: "faelight-release v3 -- beautiful release tool, libcosmic UI, What Shipped auto-generated"
status: complete
date: 2026-05-17
tags: faelight-release, libcosmic, release, changelog, public, presentation
depends_on: []
blocks: []
---

## Why This Intent Exists

faelight-release currently:
- Dumps internal commit details publicly (fixed in v14, but the root cause remains)
- Generates a minimal stats-only dynamic section when publish runs
- Has no visual UI -- pure terminal text output
- Does not auto-generate "What Shipped" from intent completions

This intent makes faelight-release worthy of a public audience.
The tool that announces the forest to the world must be as beautiful as the forest itself.

---

## Vision

faelight-release v3 is two things:

1. A libcosmic GUI release dashboard -- browse releases, see health timelines, intent graphs
2. A clean public publisher -- generates beautiful release notes automatically

When `faelight-release publish 15.0.0 --theme "..."` runs:
- "What Shipped" is auto-generated from completed intents (titles only, no INT numbers)
- Fixes are pulled from fix-tagged commits (cleaned, no internal language)
- Stats are accurate and live
- Internal commits never appear
- The dynamic README section is rich and complete -- not minimal

---

## The "What Shipped" Rule

Every release must have a "What Shipped" section.
It is generated from the intent ledger -- completed intents since last tag.
Public title only. No gates. No INT numbers. No internal language.

Example output:
What shipped:

faelight-compositor v2 -- Custom Wayland compositor. Auto-tiling, zero protocol warnings.
faelight-fm v2 -- Forest-aware file manager. Miller columns, Friday context, safety guard.


This is the rule going forward. Every version. No exceptions.

---

## Gates

Phase 1 -- Clean publisher:
- [x] publish generates full "What Shipped" from intent ledger automatically 2026-05-26
- [x] Internal commits permanently excluded -- is_internal_title filter 2026-05-26
- [x] Dynamic README section writes What-Shipped + Forest-DNA + badges 2026-05-26
- [x] Stats always accurate -- tools 55, intents 271, health 100% 2026-05-26

Phase 2 -- libcosmic UI:
- [x] libcosmic UI -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Release history browse -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Intent graph -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Forest color palette -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Visual polish -- deferred to NixOS -- approved by: christian 2026-05-26

Phase 3 -- Presentation ready:
- [x] Release card -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Export -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Credits -- deferred to NixOS -- approved by: christian 2026-05-26

Final:
- [x] README worthy of Linus Torvalds -- What-Shipped clean, no internal noise 2026-05-26
- [x] No manual README fixes -- auto-generated from intent ledger 2026-05-26
- [x] Demonstrated: v14.1.0 published cleanly in one command 2026-05-26

---

"The tool that speaks for the forest must speak clearly.
No internal noise. No leaked ledger.
Only what the forest built -- and why it matters." 🌲
