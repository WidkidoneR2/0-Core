---
id: 180
date: 2026-03-30
type: arch
title: "Sway Removal — Full Niri Commitment"
status: planned
tags: [architecture, niri, sway, wayland, faelight-login, cleanup]
version: 11.6.0
---

## Vision
Sway is fully retired. Niri is the only compositor.
The forest no longer carries dead weight from a compositor it does not use.
faelight-login boots directly into Niri with zero Sway references anywhere.

## Why Now
- Niri has been the daily driver for months
- Sway references still exist in configs, scripts, and autostart
- faelight-login was built when Sway was still in use
- Dead code and dead config is a structural integrity risk
- The summer presentation runs on Niri — it must be clean

## What Sway References Exist
Before removing anything, audit every Sway reference:
```
grep -r "sway\|swaymsg\|swaybar\|swaybg" ~/0-core/ --include="*.rs" --include="*.toml" --include="*.md" --include="*.sh" --include="*.zsh" 2>/dev/null | grep -v ".git" | grep -v "target/"
```

## Approach

### Phase 1 — Audit
Find every Sway reference in the forest.
Classify each as: REMOVE | UPDATE | KEEP (with reason).
Nothing deleted until full audit is complete.

### Phase 2 — faelight-login Hardening
faelight-login currently supports both Sway and Niri.
After removal, it should:
- Launch Niri exclusively
- Remove Sway session option
- Read session config from /etc/faelight/SESSION (value: niri)
- Fail loudly if Niri binary not found

### Phase 3 — Config Cleanup
Remove all Sway references from:
- aliases.zsh
- config.fsh
- niri config (any swaymsg calls)
- autostart scripts
- doctor checks

### Phase 4 — Package Removal
After all configs are clean:
```bash
paru -Rns sway swaybar swaybg swaylock swaynag
```
Only after every reference is removed and system is verified.

### Phase 5 — Verify
- Boot into faelight-login → Niri launches correctly
- d shows 100% health
- No broken symlinks
- No missing binaries

## Gate Check
```
⬜ Phase 1 — full audit of all Sway references documented
⬜ Phase 2 — faelight-login hardened for Niri-only
⬜ Phase 3 — all config files cleaned of Sway references
⬜ Phase 4 — Sway packages removed
⬜ Phase 5 — system verified clean — d shows 100% health
⬜ faelight-login boots into Niri with zero errors
⬜ No Sway binary, config, or reference remains
```

## The Phrase
**"You cannot build forward
while carrying what you have left behind.
Remove Sway. Own Niri completely.
The forest commits to what it uses."**

---
*"Full commitment is not a risk.
Half commitment is."* 🌲
