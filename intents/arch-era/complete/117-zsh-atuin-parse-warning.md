---
id: 117
date: 2026-03-09
resolved_date: 2026-03-11
outcome: "Resolved. Fix confirmed in git log. Terminal opens in silence."
type: future
title: "Fix atuin zsh interactive parse warning on terminal open"
status: complete
tags: [zsh, atuin, shell, parse, bug, quality-of-life]
---
## Problem

Every time a new terminal opens, this warning appears:
```
/home/christian/.zshrc:185: parse error near `__atuin_tmux_popup_c...'
```

This is caused by atuin's generated `__atuin_tmux_popup_check()` function
which contains zsh syntax that conflicts with `promptsubst` being active
at parse time. The shell works correctly — all functions load, prompt
renders — but the warning is unprofessional and distracting.

## Root Cause (Confirmed)

- atuin embeds its init code directly in `.zshrc` via `atuin init zsh`
- The embedded code contains `__atuin_tmux_popup_check()` with complex
  parameter expansions that zsh's interactive parser misreads when
  `promptsubst` is active
- This is a known interaction between atuin's zsh init and promptsubst
- The warning was pre-existing before v10.6.0 session work

## Approaches to Investigate

### Option 1 — Upgrade atuin
Check if newer atuin version fixes the generated init code:
```bash
paru -S atuin
atuin init zsh  # regenerate and check if warning persists
```

### Option 2 — Use eval instead of embedded code
Replace the embedded atuin block in .zshrc with:
```bash
eval "$(atuin init zsh)"
```
This defers parsing to runtime when promptsubst context is correct.
Risk: may affect startup performance.

### Option 3 — Disable TMUX popup feature
Since Faelight Forest doesn't use tmux, disable the popup entirely:
```bash
export ATUIN_TMUX_POPUP=false
```
Then investigate if the popup check function can be stubbed out.

### Option 4 — File upstream atuin bug
Report to atuin maintainers that their zsh init generates code that
produces parse warnings with promptsubst active.

## Success Criteria
- [ ] No parse warnings on terminal open
- [ ] atuin history search still works (Ctrl+R)
- [ ] Shell startup time not degraded
- [ ] Fix is stable across atuin upgrades

## Notes
- Do NOT attempt to patch atuin's generated functions manually
- Test each approach in isolation before committing
- Verify with: `zsh -i -c 'echo clean' 2>&1`

---
*"The forest should open in silence."* 🌲
