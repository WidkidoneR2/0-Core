---
id: 314
title: "faelight-release v3 -- beautiful release tool, libcosmic UI, What Shipped auto-generated"
status: in-progress
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
- [ ] publish generates full "What Shipped" from intent ledger automatically
- [ ] Internal commits permanently excluded (not just commented out)
- [ ] Dynamic README section always writes full rich format
- [ ] Stats always accurate (read live from state.db + /etc/faelight/)

Phase 2 -- libcosmic UI:
- [ ] faelight-release opens a libcosmic window
- [ ] Browse release history with health timeline
- [ ] Intent completion graph per release
- [ ] Forest color palette throughout
- [ ] As visually polished as faelight-fm v2

Phase 3 -- Presentation ready:
- [ ] `faelight-release show 14.0.0` renders beautiful release card
- [ ] Export release card as image (for social/presentation)
- [ ] Credits section auto-maintained from a credits.toml file

Final:
- [ ] faelight-release publish produces a README worthy of Linus Torvalds
- [ ] No manual README fixes ever needed after publish
- [ ] The release tool is itself a demonstration of what the forest can build

---

"The tool that speaks for the forest must speak clearly.
No internal noise. No leaked ledger.
Only what the forest built -- and why it matters." 🌲
