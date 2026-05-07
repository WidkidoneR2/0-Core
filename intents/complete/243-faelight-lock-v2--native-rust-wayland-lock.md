---
id: 243
title: faelight-lock v2 -- Native Rust Wayland Lock
status: complete
date: 2026-04-19
tags: [faelight-lock, wayland, ext-session-lock, niri, security, clock, friday, pam, v2]
---
faelight-lock v1 wrapped swaylock.
swaylock is gone (INT-180).
v2 is native Rust via ext-session-lock-v1 Wayland protocol.
No wrapper. No external dependency. The forest locks itself.
When you lock, the forest does not go dark.
It breathes.
It shows you a clock, a Friday quote, and forest stats.
The screen is alive but locked.
Unlocking feels intentional.
- Wrapped swaylock with forest theme colors
- PAM authentication via swaylock
- Health check to verify swaylock installed
- Implements ext-session-lock-v1 Wayland protocol directly in Rust
- No swaylock dependency -- ever again
- PAM authentication via pam crate
- Works on any compositor supporting ext-session-lock-v1
- Niri supports it natively
Top zone -- Clock:
- Large ASCII digits: HH:MM
- Updates every second
- Day and date below: Sunday, April 19 2026
Center zone -- Forest identity:
- Faelight Forest title
- Version + health percentage
- Total commit count
- Active intent (if any)
Bottom zone -- Friday quote:
- Rotates through friday_language named abstractions
- Falls back to last friday_brief from synthesis_snapshots
- One line at a time, rotates every 30 seconds
Password zone -- appears on keypress:
- Clean minimal input field, bullets for chars
- Disappears when idle for 5 seconds
- Error message on failed auth, clears after 3 seconds
- Max 3 rapid attempts then 5 second cooldown
On successful authentication:
- Screen flashes forest green for ~200ms
- ASCII tree collapses upward (reverse of login animation)
- Total duration ~500ms
- Returns to desktop
- PAM authentication stack
- Lock surface covers all outputs simultaneously
- Cannot be dismissed without correct password
- All unlock attempts logged to state.db with timestamp
- Lock state written to state.db on activate
wayland-client + wayland-protocols for ext-session-lock-v1.
pam crate for PAM authentication.
ratatui + crossterm for TUI on the lock surface.
rusqlite for reading friday data and logging.
chrono for clock display.
ext-session-lock-v1 protocol flow:
1. Connect to Wayland display
2. Bind ext_session_lock_manager_v1 global
3. Call lock() -- get ext_session_lock_v1
4. Create lock surface for each output
5. Render ratatui TUI on each lock surface
6. On correct password: send unlock_and_destroy()
⬜ ext-session-lock-v1 Wayland protocol implemented in Rust
⬜ Lock surface renders correctly on all outputs
⬜ PAM authentication working via Rust pam crate
⬜ Clock display -- large ASCII digits, updates every second
⬜ Day and date shown below clock
⬜ Forest identity panel -- version, health, commits, intent
⬜ Friday quote rotates from vocabulary and synthesis
⬜ Password field appears on keypress, hides when idle
⬜ Failed auth shows error, clears after 3 seconds
⬜ 3-attempt cooldown working
⬜ Unlock animation -- forest green flash + tree collapse
⬜ All outputs locked simultaneously
⬜ Unlock attempts logged to state.db
⬜ No swaylock dependency -- fully native Rust
⬜ Tested lock/unlock on Niri real hardware
⬜ faelight-lock command replaces binary cleanly
The forest does not go dark when it locks.
It breathes.
It shows you what it knows.
And when you return,
it welcomes you back
with a flash of green. 🌲

Evaluate **slint** for the lock screen UI. Slint's declarative DSL compiles to
Rust and forces clean separation of UI and business logic. Good fit for a
one-shot declarative UI like a lock screen.
Same time-box rule as INT-239: if slint + Wayland/greetd handoff proves painful,
keep current ratatui approach and upgrade it instead.
**Rule: if you don't understand every line, it doesn't ship.**
