---
id: 232
title: "faelight-term v2 -- The Terminal That Thinks in 2136"
status: planned
date: 2026-04-19
tags: [terminal, faelight-term, friday, rendering, intelligence, ux, wayland, v2, overhaul]
---
Every other terminal shows you text.
faelight-term shows you context, knowledge, and Friday.
Not an incremental upgrade. A complete rebuild from the ground up.
Built for 2136, deployed in 2026.
The only thing carried forward from v11:
- The welcome screen (version, intents, health, session context)
- Ctrl+Shift+S forest status strip toggle
Everything else is rebuilt with one question:
"What would a terminal look like if it was invented today, by someone who understands the forest?"
The terminal is not a text pipe.
The terminal is the interface between the human and the machine.
It should be intelligent, responsive, beautiful, and aware.
It should know what you are building and help you build it.
Flawless. No exceptions.
- Ctrl+Shift+C copies selected text, no mangling, no extra newlines
- Ctrl+Shift+V pastes exactly what was copied, including unicode, emojis, Rust code
- Middle-click paste works
- Bracketed paste mode supported natively
- Multiline paste handled in fsh without sh fallback
- No selection state loss on resize or scroll
True GPU-accelerated rendering via wgpu (not OpenGL legacy).
- 165Hz capable, no frame drops
- True 24-bit color everywhere
- Font ligatures: -> => != <= >= <> |> ... and more
- Powerline glyphs: no missing boxes, no fallback squares
- Bold, italic, dim, strikethrough all correct
- Correct rendering at every font size
- Resize: zero artifacts, instant, smooth
- Scrollback: 50,000 lines, smooth scroll, no tearing
- Wide character support: CJK, emoji, box-drawing all aligned
The current prompt is functional. v2 is beautiful.
A two-line prompt that feels modern and alive:
Line 1: context bar (zone icon · git branch · health · active intent · time)
Line 2: caret with subtle color based on last exit status
No visual noise. No powerline spam. Clean, readable, fast.
Color shifts:
- Green caret: last command succeeded
- Red caret: last command failed
- Amber caret: health below 95%
- Cyan caret: Friday has something to say
Ctrl+Shift+F: Friday panel slides in from right (30% width).
The panel shows:
- Friday's latest insight (from synthesis brief)
- Last 5 knowledge entries for current domain
- Active contradictions if any
- Session summary: commits, deploys, health arc
When build fails: panel automatically shows relevant knowledge entry.
When health drops: panel surfaces the cause.
Panel reads directly from state.db -- no network, no IPC, no daemon dependency.
Panel persists across commands. No cold start.
The terminal understands what it is displaying:
- Rust compiler errors: error codes highlighted, knowledge engine consulted
- File paths: clickable, opens in $EDITOR or navigates faelight-fm
- URLs: Ctrl+Click opens in faelight-browser
- HTTP status codes: colored by class (2xx green, 4xx yellow, 5xx red)
- JSON output: auto pretty-printed when detected
- Git diff: syntax-aware coloring
- Build timing: slow builds flagged inline
- Restores last working directory on open
- Shows active intent in context bar immediately
- Remembers last 20 commands per session in state.db
- Long command notification: >30s sends faelight-notify
- Session end: writes summary to state.db for Friday
- Split panes: Ctrl+Shift+H (horizontal), Ctrl+Shift+V (vertical)
- Each pane maintains its own working directory and context
- Navigation: Ctrl+Shift+Arrow to move between panes
- No tabs -- panes only, cleaner model
Must beat foot on:
- Cold start: under 50ms
- Input latency: under 2ms
- Scroll FPS: 165Hz sustained
- Memory: under 30MB idle
⬜ Complete rewrite -- wgpu renderer, clean slate Rust codebase
⬜ Copy/paste flawless -- all edge cases tested, multiline, unicode, Rust code
⬜ 165Hz rendering, zero artifacts on resize
⬜ Font ligatures and powerline glyphs correct
⬜ New prompt design -- two-line, zone-aware, exit-status color
⬜ Ctrl+Shift+F Friday panel live and reading from state.db
⬜ Build error triggers automatic knowledge lookup in Friday panel
⬜ File paths and URLs clickable -- Ctrl+Click opens correctly
⬜ JSON/output intelligence -- auto-detected and highlighted
⬜ Split panes working -- H/V split, Ctrl+Shift+Arrow navigation
⬜ Session memory -- directory and intent context restored on open
⬜ Long command notification via faelight-notify (>30s)
⬜ Cold start under 50ms measured
⬜ Multiline paste handled in fsh without sh fallback
⬜ 14 days as primary terminal without any regression
⬜ Welcome screen preserved exactly as designed
⬜ Ctrl+Shift+S forest status strip preserved and upgraded
"The terminal of 2136 knows what you are building.
It remembers what broke last time.
It surfaces the fix before you ask.
It is not a window. It is a partner.
faelight-term v2." 🌲
