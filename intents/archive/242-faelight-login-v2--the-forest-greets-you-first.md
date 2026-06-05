---
id: 242
title: faelight-login v2 -- The Forest Greets You First
status: complete
date: 2026-04-19
tags: [faelight-login, greeter, greetd, niri, animation, friday, typography, v2]
---
faelight-login v1 was functional. Niri-only after INT-180.
v2 is the forest's face to the world.
The first thing you see when you sit down.
It should feel like the forest is alive.
- Username + password fields
- Health + commit count display
- Forest green color palette
- Forest tree grows upward line by line (50-100ms per line)
- Rendered via ratatui frame-by-frame loop
- Once tree fully rendered, login fields fade in
- Skippable with any keypress
- Animation runs once, settles into static login state
- Larger bolder title: Faelight Forest + version
- Subtle color cycling on title using ratatui
- Field labels clean, aligned, properly spaced
- Error messages in warm red, not harsh white
Below login fields:
- Version and theme name
- Total commits + today count
- Health percentage
- Active intent title
- Hours since last session
- Single line below status panel
- Pulls from synthesis_snapshots last friday_brief
- Only shown if brief confidence >= 0.70
- Example: Strong momentum. 45 commits. Focus on INT-203.
- Full dark background
- Forest green accents
- Powerline-style separators between sections
- No visual noise -- everything intentional
- Consistent with the whole forest aesthetic
Reads state.db directly for live health, commits, friday brief.
Animation is ratatui frame loop with ASCII tree rendered line by line.
Launches niri-session directly -- no session selector ever again.
✅ Animated ASCII forest boot sequence -- tree grows line by line at 60ms/frame (2026-04-19)
✅ Animation skippable with any keypress (2026-04-19)
✅ Typography upgraded -- bold title, pulse color, version displayed (2026-04-19)
✅ Forest status panel -- health, commits, active intent (2026-04-19)
✅ Friday morning brief -- reads synthesis_snapshots, confidence gate 0.70 (2026-04-19)
✅ Reads state.db directly for health, commits, friday brief (2026-04-19)
✅ Color palette consistent with forest theme (2026-04-19)
✅ Login fields clean, focused field highlighted (2026-04-19)
✅ Error messages styled in warm red (2026-04-19)
✅ Boots into Niri correctly -- niri-session, no session selector (2026-04-19)
✅ No regression from v1 -- greetd_ipc auth preserved, builds clean (2026-04-19)
The forest greets you first.
Before you type a word,
it shows you where you are,
what you have built,
and what Friday is thinking.
Login is not a barrier.
It is a welcome. 🌲
