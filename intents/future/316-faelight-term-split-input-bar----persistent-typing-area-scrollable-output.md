---
id: 316
title: "faelight-term split input bar -- persistent typing area, scrollable output above"
status: planned
date: 2026-05-17
tags: faelight-term, UX, input, split, scrollback, AOL-pattern
depends_on: [313]
blocks: []
---

## Why This Intent Exists

The terminal UX problem: when you scroll up to read output, you lose your cursor.
You have to scroll back down to type. This breaks flow.

AOL Instant Messenger solved this in 1997 with a split window:
- Top area: conversation history, scrolls freely
- Bottom area: your input, always visible and focused

faelight-term can do this natively -- we own the wgpu renderer.
We draw the split line ourselves. The input area is a fixed N lines at the bottom.
Output renders above it. Scrollback only affects the top area.
The typing area never moves.

---

## Design
┌─────────────────────────────────────┐
│  [output area - scrollable]         │
│  $ cargo build --release            │
│     Compiling faelight-term v3.0.0  │
│     Finished in 1.36s               │
│  $ d                                │
│  🏥 Faelight Forest 14.0.0 100%    │
│  ...                                │
├─────────────────────────────────────┤  ← split line (forest green)
│  fsh ❯ _                            │  ← persistent input area
└─────────────────────────────────────┘

The split line is a thin forest-colored horizontal rule.
The input area is 1-3 lines tall (configurable).
Ctrl+Shift+Up/Down resizes the split.
PageUp/Down scrolls the output area without moving the cursor.

---

## Gates

Phase 1 -- Split rendering:
- [ ] Output area and input area rendered as separate regions
- [ ] Split line drawn with forest color (#11140f accent)
- [ ] Input area always at bottom, fixed height
- [ ] Output area scrolls independently

Phase 2 -- Input persistence:
- [ ] Typing never interrupted by new output
- [ ] Output appends above the split line
- [ ] fsh prompt always visible in input area

Phase 3 -- Resize and polish:
- [ ] Ctrl+Shift+Up/Down resizes split
- [ ] Input area can be 1, 2, or 3 lines
- [ ] Split line shows active intent (Friday integration)
- [ ] Smooth scroll in output area

Final:
- [ ] faelight-term split input bar is the daily driver UX
- [ ] Never lose cursor position while reading output
- [ ] The split line is Friday's face in the terminal

"The AOL pattern, reborn in Rust.
Scroll freely. Type without losing your place.
The forest remembers where you were." 🌲
