---
id: 192
date: 2026-04-04
type: arch
title: "Deploy Pipeline v0.8.0 — registry_tools.py and Architecture Split"
status: in-progress
tags: [deploy, scripts, registry, architecture, python]
---
## Vision
Deploy script v0.7.0 is solid. v0.8.0 takes the next architectural step:
extract the embedded Python blocks into a proper registry_tools.py helper,
and split build/install/registry responsibilities cleanly.

## Background
From v0.7.0 feedback review — three deferred improvements:
- #2: registry_tools.py — single Python helper replacing 3 embedded blocks
- #3: Clean orchestration — deploy becomes a thin coordinator
- #9: Conceptual split — build / install / registry as distinct concerns

## Scope
### Phase 1 — registry_tools.py
Extract all three Python blocks from deploy into:
  ~/0-core/scripts/registry_tools.py
Commands:
  registry_tools.py list-deployable
  registry_tools.py update-version <pkg> <version>
  registry_tools.py list-missing

### Phase 2 — deploy v0.8.0
Replace embedded Python in deploy with:
  registry_tools list-deployable
  registry_tools update-version "$pkg" "$version"
  registry_tools list-missing
Deploy becomes pure bash orchestration.

### Phase 3 — Architecture validation
- deploy list still works
- deploy check still works  
- deploy all still works
- deploy <tool> still works
- All timing, dirty flag, cleanup preserved

## Gates
⬜ registry_tools.py created with all 3 commands
⬜ deploy update-version uses registry_tools.py
⬜ deploy check uses registry_tools.py
⬜ deploy all uses registry_tools.py
⬜ All existing deploy commands verified working
⬜ deploy version bumped to v0.8.0
⬜ Zero regressions from v0.7.0

---
*"The deploy script should read like a story, not a Python tutorial."* 🌲
