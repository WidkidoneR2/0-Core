# Known Issues - faelight-term v10.0.0

## Critical Issues

### 1. Ctrl+C Signal Handling (SIGINT)

**Status:** Under investigation  
**Priority:** High  
**Affects:** Interactive programs (vim, htop, etc.)

#### Symptoms
- Ctrl+C doesn't send SIGINT to child process
- Programs don't respond to interrupt signal
- Must use `:q` or other exit commands instead

#### Technical Analysis

**What we know:**
1. Keystroke is captured correctly in Wayland event handler
2. `\x03` (ETX/Ctrl+C) is written to PTY master fd
3. PTY echo shows the character being sent
4. Child process doesn't receive SIGINT

**What we've tried:**
```rust
// Verified PTY writes succeed
let bytes_written = write(pty_fd, b"\x03", 1);
assert!(bytes_written == 1);

// Verified termios ISIG is enabled
termios.c_lflag |= ISIG;

// Verified VINTR is set to 0x03
termios.c_cc[VINTR] = 0x03;
```

**Current hypothesis:**
- Terminal is in raw mode when it should be cooked
- ISIG flag may not be propagating to child process
- PTY layer might need explicit signal generation
- Possible `tcsetattr()` timing issue

**Workarounds for users:**
- Use program-specific exit commands
- Alt+F4 still works for window close
- Most programs have alternatives (`:q`, `q`, Ctrl+D)

**Next steps:**
- Study Foot's PTY signal handling implementation
- Test with `strace` to see exact syscalls
- Verify child process termios state
- Consider explicit `kill()` on detecting Ctrl+C

#### References
- alacritty/alacritty PTY implementation
- Foot terminal source (PTY handling)
- "The TTY demystified" (Linus Akesson)

---

## Minor Issues

### 2. Emoji Positioning

**Status:** Cosmetic  
**Priority:** Low

Some wide emoji render slightly off-center vertically. Doesn't affect functionality, just aesthetics.

**Affected:** Mostly flags and multi-codepoint emoji  
**Workaround:** None needed, text is still readable

---

### 3. Missing Scrollbar

**Status:** Planned feature  
**Priority:** Medium

No visual scrollbar indicator. Scrolling works via mouse wheel and Page Up/Down, but position isn't shown visually.

**Workaround:** Use Shift+Page Up/Down to see scroll position change

---

### 4. No Configuration File

**Status:** Planned feature  
**Priority:** Medium

Font, colors, and behavior are currently hardcoded in source.

**Workaround:** Edit `src/main.rs` and rebuild

**Planned:** TOML config in `~/.config/faelight-term/config.toml`

---

## Not Bugs (By Design)

### Limited Feature Set

faelight-term intentionally omits:
- Tabs (use window manager workspaces)
- Ligatures (prefer explicit characters)
- Hyperlinks (use launcher)
- Sixel graphics (complexity vs utility)

These aren't bugs - they're philosophical choices to keep the codebase understandable.

---

## Reporting Issues

This is a personal learning project. Issues are tracked in the Intent Ledger system, not GitHub Issues.

**If you find something:**
1. Check if it's listed here
2. Test if it reproduces in Foot
3. If Foot also has the issue, it's probably not faelight-term
4. Document reproduction steps

**Philosophy:** Better to understand one bug deeply than fix ten bugs superficially.

---

## Development Approach

**Priority order:**
1. Safety (no crashes, no data loss)
2. Correctness (proper terminal behavior)
3. Features (only when needed)
4. Polish (last)

The Ctrl+C issue falls under "correctness" - it will be fixed before v1.0.0.

---

Last updated: 2026-01-27
