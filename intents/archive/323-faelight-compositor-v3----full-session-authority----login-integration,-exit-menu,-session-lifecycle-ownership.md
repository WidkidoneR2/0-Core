---
id: 323
title: "faelight-compositor v3 -- Full Session Authority"
status: planned
date: 2026-05-20
tags: [compositor, niri, session, login, menu, lifecycle, wayland, ipc, authority]
---
version: TBD
---

## Vision

<!-- What is this intent trying to achieve? -->

## Why Now

<!-- Why is this the right time for this intent? -->

## Approach

<!-- How will this be implemented? -->

## Success Criteria

- [ ] <!-- First criterion -->
- [ ] <!-- Second criterion -->

## Gate Check
```
⬜ Not started
```

---

*\"The forest grows with intention.\"* 🌲

## Migration Strategy (2026-05-26)
faelight-login dual-session approach:
1. Build Pinnacle in R&D VM (INT-328) -- validate before touching daily system
2. Update faelight-login to show both: [Niri] and [Pinnacle] as session choices
3. Run side by side -- Niri as fallback, Pinnacle as primary
4. Once Pinnacle is daily-driver stable -- remove Niri from login options
5. NixOS migration with Pinnacle as the only compositor

This prevents any hard cutover. Gradual confidence building.
The forest never bets everything on one session.
