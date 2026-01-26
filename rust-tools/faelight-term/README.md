# faelight-term v10.0.0

🚀 **Custom Wayland terminal emulator with color emoji support**

A minimal, fast terminal built from scratch in Rust for the Faelight Forest environment.

**Status:** Beta - Core functionality stable, some edge cases in development

---

## Why This Exists

**Problem:** Existing terminals either lack color emoji support or carry unnecessary complexity.

**Solution:** Purpose-built terminal emulator that does exactly what's needed - nothing more, nothing less.

---

## Features

### ✅ Working & Stable

- **Color emoji rendering** - Full Unicode emoji support with proper colors
- **VTE parsing** - Standard terminal escape sequences
- **Scrollback buffer** - 10,000 line history
- **Copy/paste** - Wayland clipboard integration
- **Mouse support** - Click, drag, scroll
- **UTF-8 text** - Full Unicode support
- **Basic ANSI colors** - 16-color palette
- **Cursor rendering** - Block cursor with blink

### 🧪 Beta Features

- **Signal handling** - Most signals work (see Known Issues)
- **Smooth scrolling** - Works but being refined

---

## Usage

### Launch
```bash
faelight-term
```

### Keybindings
```
Ctrl+Shift+C    Copy selection to clipboard
Ctrl+Shift+V    Paste from clipboard
Scroll Up/Down  Navigate scrollback
Mouse Select    Select text for copying
```

---

## Configuration

Currently minimal configuration - hardcoded for speed and simplicity:

**Font:** JetBrainsMono Nerd Font, 11pt  
**Colors:** Faelight Forest theme (tropical sunset)  
**Scrollback:** 10,000 lines  
**Shell:** $SHELL or /bin/zsh

Future versions will support config files.

---

## Technical Details

### Architecture
- **PTY handling:** Linux `openpty()` + `termios`
- **VTE parsing:** Custom state machine
- **Rendering:** Wayland + Cairo + Pango
- **Text layout:** Pango for proper emoji rendering

### Performance
- **Startup time:** ~50ms
- **Input latency:** <10ms
- **Memory:** ~15MB base + scrollback
- **Emoji rendering:** Hardware-accelerated via Cairo

### Why Rust?
- Memory safety for PTY operations
- Zero-cost abstractions
- Learning modern systems programming
- Part of 84% Rust ecosystem

---

## Known Issues

See [KNOWN_ISSUES.md](KNOWN_ISSUES.md) for detailed technical issues.

**Major:**
- Ctrl+C signal handling incomplete (being investigated)

**Minor:**
- Some emoji render slightly off-center
- Scrollbar not yet implemented

---

## Philosophy

**"Understanding over convenience"**

This terminal was built from scratch to understand:
- How terminals actually work
- PTY mechanics and signal handling
- Wayland protocol internals
- Text rendering complexity

It's not meant to replace Foot (which remains my daily driver for production work). It's a learning project that became useful.

---

## Comparison to Other Terminals

| Feature | faelight-term | Foot | Alacritty |
|---------|---------------|------|-----------|
| Color emoji | ✅ Native | ❌ No | ❌ No |
| Wayland-native | ✅ Yes | ✅ Yes | ✅ Yes |
| Config files | ⚠️ Planned | ✅ Yes | ✅ Yes |
| GPU acceleration | ❌ No | ✅ Yes | ✅ Yes |
| Signal handling | 🧪 Beta | ✅ Full | ✅ Full |
| Startup time | ✅ Fast | ✅ Fast | ✅ Fast |

**Use faelight-term when:** You want color emoji and don't mind beta status  
**Use Foot when:** You need rock-solid production reliability

---

## Building

Part of the 0-Core workspace:
```bash
cd ~/0-core
cargo build --release -p faelight-term
```

Binary outputs to: `target/release/faelight-term`

---

## Development Status

**v10.0.0 Changelog (Latest):**
- Fixed scrollback rendering (no more jitter)
- Fixed mouse wheel scrolling direction
- Fixed clipboard copy/paste integration
- Fixed emoji rendering crashes
- Fixed selection visual feedback
- Fixed text wrapping edge cases
- Improved PTY signal handling
- Improved mouse selection accuracy

**Next priorities:**
- Complete Ctrl+C signal handling
- Add configuration file support
- Implement scrollbar
- Refine emoji positioning

---

## Part of Faelight Forest

One of 35 Rust tools in the ecosystem. Built as a learning exercise in terminal emulation.

**Philosophy:** Build to understand, not just to use.

See: https://github.com/WidkidoneR2/0-Core
