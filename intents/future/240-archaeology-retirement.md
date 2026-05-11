---
id: 240
title: "archaeology-0-core Retirement -- Clean Removal of a Legacy Tool"
status: in-progress
date: 2026-04-18
tags: [archaeology, retirement, cleanup, legacy, doctor, aliases, registry]
---
archaeology-0-core was a valuable tool in its time.
It explored git history before core had that capability.
That capability now exists in core directly.
INT-193 said it was retired. It was not fully retired.
The source code remains in rust-tools.
The aliases remain in the registry.
The doctor still checks for it.
The docs still reference it.
This intent completes the retirement properly.
Confirm core archaeology/narrative/story covers all archaeology-0-core use cases:
- Timeline view: core narrative
- By-intent view: core intent show
- This-week view: core story
- Stats: core snapshot
rm -rf rust-tools/archaeology-0-core
Only after confirming no unique capability is lost.
Remove alias entries from 01-registry/aliases.toml
Mark as retired in tools.toml if entry exists
engine/src/domains/doctor/checks.rs has archaeology-specific checks
Remove or replace with a check that confirms core commands work instead
Remove from TOOLS.md, QUICK_REFERENCE.md, TOOL_REFERENCE.md
Update ARCHITECTURE-FUTURE.md
Remove archaeology-0-core references
⬜ All archaeology-0-core use cases verified covered by core commands
⬜ rust-tools/archaeology-0-core removed
⬜ Registry aliases removed
⬜ Doctor checks updated -- no more archaeology binary checks
⬜ TOOLS.md updated
⬜ QUICK_REFERENCE.md updated
⬜ TOOL_REFERENCE.md updated
⬜ ARCHITECTURE-FUTURE.md updated
⬜ COMMAND-GUIDE updated
⬜ No broken references remain in any file
⬜ d shows 100% health after removal
"A tool that served its purpose.
Retired with respect, not neglect." 🌲
