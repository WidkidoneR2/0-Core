---
id: 201
date: 2026-04-05
type: planned
title: "faelight-term v11 — The Terminal That Thinks"
status: in-progress
tags: [terminal, faelight-term, daily-driver, ux, intelligence, v11]
---
Foot is a great terminal. kitty is a great terminal.
faelight-term must be something they cannot be:
a terminal that understands the forest.
Not just rendering text.
Not just handling input.
A terminal that knows what you are doing,
shows what matters, hides what does not,
and gets out of the way when you are in flow.
Every other terminal is context-blind.
They show you text. They do not understand it.
faelight-term knows:
- Which intent is active
- Whether the last command succeeded or failed
- How long commands take vs their historical average
- When you are in a flow state vs stuck
- What the forest health is right now
This context shapes what the terminal shows and how it behaves.
Current state: PATH fix deployed but needs verification across:
- Cold start (fresh login)
- Niri autostart launch
- Nested terminal (term inside term)
- After su/sudo
Run this audit: launch term and foot side by side.
Compare: PATH, HOME, SHELL, TERM, XDG vars, WAYLAND_DISPLAY
Every variable foot has, term must have.
If term is missing any — add it to pty.rs spawn.
Currently: basic scrollback exists but behavior differs from foot.
Fix: Shift+PageUp/PageDown for scrollback navigation
Fix: scrollback should survive terminal resize
Fix: scrollback history limit configurable (default: 10000 lines)
Current: basic font rendering.
Improve: ligature support for common coding ligatures (-> => != <=)
Improve: bold/italic rendering for bat/syntax highlighting
Improve: powerline glyph support (for prompt segments)
Ctrl+Shift+C/V for clipboard (done)
Fix: primary selection auto-copy on mouse release (done)
Add: middle-click to paste from primary selection
Add: double-click to select word
Add: triple-click to select line
Currently: URL detection exists but click behavior unreliable.
Fix: Ctrl+Click reliably opens URL in faelight-browser
Fix: URL underline on hover
Add: URL preview on hover (show full URL in status bar)
Current: resize sometimes corrupts display.
Fix: proper SIGWINCH on resize
Fix: PTY window size update on resize (TIOCSWINSZ)
Fix: content reflow on resize
The title bar is not just a window name.
It shows the active intent, health, and current directory:
"🌲 INT-194 | ~/0-core | 100% | 14:23"
Updates live as you work.
After every command:
- Green pulse on success
- Red pulse on failure
- Duration shown for commands > 2 seconds
Not intrusive — a subtle color shift in the border or title.
A minimal 1-line status at the bottom of the terminal:
"INT-178 · 100% · 6 commits today · contextd: 3 insights pending"
Toggleable with Ctrl+Shift+S.
Disappears during typing to stay out of the way.
Ctrl+F opens inline search — not a separate mode.
Searches as you type. Highlights all matches.
Enter to jump to next. Escape to exit without moving cursor.
Search persists as you type new commands.
Ctrl+Shift+H — horizontal split
Ctrl+Shift+V — vertical split  
Ctrl+Shift+W — close current pane
Ctrl+Shift+Arrow — navigate between panes
Each pane is a full terminal. Independent PTYs.
This is the single biggest daily workflow improvement.
When you close and reopen faelight-term, it remembers:
- Working directory
- Active intent context
- Recent command history (shown faded in new session)
Not full session restore — just enough context to orient quickly.
If a command runs for > 30 seconds in a background pane:
Send a faelight-notify notification when it completes.
"cargo build --release: completed in 47s"
Never miss a long build finishing.
Phase 1 — Parity (fix all foot parity bugs)
PATH inheritance verified across all launch contexts
Environment variable audit and fix
Scrollback navigation
Copy/paste parity (double/triple click, middle click)
Resize fix
URL click reliability
Phase 2 — Intelligence (what makes it different)
Intent-aware title bar (live updates)
Command success/failure indicators
Forest status strip
Phase 3 — Power Features (what makes it better)
Split panes
Smart scrollback search
Notification on long commands
Session memory
Phase 4 — Polish
Font ligatures
Powerline glyph support
Performance profiling (must be as fast as foot)
faelight-term replaces foot as primary terminal when:
- All Phase 1 bugs fixed and verified
- Intent-aware title bar live
- Split panes working
- No regressions vs foot for 14 days of daily use
✅ Phase 1 — PATH verified — 0-core/scripts in PATH, deduplicated
✅ Phase 1 — .zshrc PATH additions made idempotent, no duplicates
✅ Phase 1 — Scrollback navigation — Shift+PageUp/Down already working
✅ Phase 1 — Double-click word select, triple-click line select
✅ Phase 1 — Paste from browser via Ctrl+Shift+V working (wl-paste fallback)
⬜ Phase 1 — URL click reliable — Ctrl+Click exists, reliability needs verification
⬜ Phase 1 — Resize without display corruption — known issue, pending fix
✅ Phase 2 — Intent-aware title bar live — 🌲 INT-XXX | ~/dir | 100%, updates every 5s
✅ Phase 2 — Command success/failure indicators — ✓/✗ in title bar
⬜ Phase 2 — Forest status strip (toggleable)
⬜ Phase 3 — Split panes (H/V split, navigation)
✅ Phase 3 — Smart scrollback search — Ctrl+F search mode, Enter=next, Esc=exit
⬜ Phase 3 — Notification on long commands (>30s)
⬜ Phase 3 — Session memory (directory + intent context)
⬜ Phase 4 — Font ligatures and powerline glyphs
⬜ Phase 4 — Performance parity with foot
⬜ 14 days as primary terminal without regressions
**"Every other terminal shows you text.
faelight-term shows you context.
Not what the computer is doing —
what YOU are doing.
The terminal that thinks
is the terminal you never want to leave."** 🌲
