---
id: 011
date: 2026-01-05
type: future
title: "Profile Tab Completions - v4.0.1"
status: planned
tags: [profile, completions, zsh, polish]
---

## Why

Profile system works but tab completions don't trigger properly.
Something in zsh is overriding the custom completion function.

## What

Debug and fix `_profile` completion function.

- Completions registered but not triggering
- Shows file completions instead of profile commands
- May be PATH duplication issue (profile appears 7 times)

## Success Criteria

- `profile <TAB>` shows: list, status, gaming, work, low-power, default
- `profile edit <TAB>` shows available profiles

## Notes

- Function is valid (syntax check passes)
- `$_comps[profile]` shows `_profile`
- Likely environment/cache issue

---

_Part of the 0-Core Intent Ledger_ 🌲
