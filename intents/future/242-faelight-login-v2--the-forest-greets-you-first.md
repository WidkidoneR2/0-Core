---
id: 242
title: faelight-login v2 -- The Forest Greets You First
status: planned
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
⬜ Animated ASCII forest boot sequence -- tree grows line by line
⬜ Animation skippable with any keypress
⬜ Typography upgraded -- bold title, version, forest name prominent
⬜ Forest status panel -- version, commits, health, active intent
⬜ Friday morning brief -- last synthesis shown with confidence gate
⬜ Reads state.db directly for live data
⬜ Color palette consistent with forest theme
⬜ Login fields clean and readable
⬜ Error messages styled properly
⬜ Boots into Niri correctly on real hardware
⬜ No regression from v1 functionality
The forest greets you first.
Before you type a word,
it shows you where you are,
what you have built,
and what Friday is thinking.
Login is not a barrier.
It is a welcome. 🌲
