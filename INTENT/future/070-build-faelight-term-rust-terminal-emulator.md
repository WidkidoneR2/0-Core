---
id: 070
date: 2026-01-22
type: future
title: "Build faelight-term: Rust terminal emulator mirroring Foot quality"
status: planned
priority: medium
tags: [rust, terminal, wayland, learning, systems-programming, tool-31]
---

## Vision

Build a minimal, fast, Wayland-native terminal emulator in Rust that mirrors Foot's excellence while embodying 0-Core philosophy. This is a long-term learning project demonstrating commitment to continued development beyond the presentation and pushing the ecosystem toward 100% Rust.

## The Problem

**Current State:**
- Ecosystem: 81.1% Rust, 10.5% Shell, 6.9% Lua, 1.1% Python, 0.4% Nu
- Using Foot (C) - perfect terminal, but not Rust
- Goal: Push toward ~95% Rust (keeping Lua for Neovim)
- Want deep understanding of terminal internals

**Why This Matters:**
- Complete the "built from scratch" story
- Demonstrates ongoing commitment (not one-time presentation)
- Learn terminal emulation, Wayland rendering, font rendering deeply
- Tool #31 - the ultimate flex

## The Solution

**Philosophy:**
- Minimal: Only essential features, no bloat
- Fast: Performance must match or approach Foot
- Wayland-only: No X11 legacy support
- Well-understood: Every line has a purpose
- Production-ready: Daily driver quality, not a toy

**Technical Approach:**

### Phase 1: Research & Architecture (Weeks 1-2)
- Study Foot's source code and architecture
- Evaluate VTE crates: `alacritty_terminal`, `vte-rs`, `termwiz`
- Evaluate Wayland: `smithay-client-toolkit`
- Evaluate font rendering: `fontdue`, `rusttype + harfbuzz`
- Design architecture document
- Make critical technology choices

### Phase 2: Minimal Window (Weeks 3-4)
- Create Wayland window using smithay-client-toolkit
- Render solid color background
- Handle window resize events
- Basic keyboard input
- Graceful exit

### Phase 3: Font Rendering (Weeks 5-6)
- Load font from TOML config
- Render monospace glyphs
- UTF-8 support
- Basic ANSI colors (16 colors)
- Cursor rendering

### Phase 4: Terminal Emulation (Weeks 7-10)
- Integrate VTE library (likely alacritty_terminal)
- PTY creation and management
- Process spawning (/bin/sh)
- ANSI escape sequence handling
- Basic shell interaction

### Phase 5: Essential Features (Weeks 11-14)
- Scrollback buffer
- True color (24-bit RGB) support
- Copy/paste (Wayland clipboard)
- Proper signal handling (SIGCHLD, etc.)
- TOML configuration system
- Integration with faelight ecosystem

### Phase 6: Performance & Polish (Weeks 15-18)
- Optimize rendering pipeline
- Memory profiling and optimization
- Reduce input latency
- Handle edge cases (resize, invalid input, etc.)
- Test with real programs (vim, htop, neofetch, etc.)

### Phase 7: Foot Parity (Weeks 19-24)
- URL detection and hints
- Scrollback search
- Mouse support
- All remaining Foot features
- Comprehensive testing
- Ready to replace Foot as daily driver

## Key Decisions

**VTE Library:** TBD - Research needed
- Option 1: alacritty_terminal (battle-tested)
- Option 2: vte-rs (GTK standard)
- Option 3: termwiz (WezTerm's library)

**Rendering:** Software rendering first (tiny-skia or similar)
- GPU rendering (wgpu) can come later if needed
- Foot uses software rendering and is blazingly fast

**Configuration:** TOML (consistent with ecosystem)
- Font, size, colors
- Keybindings
- Integration with faelight-bar, faelight-launcher

**Integration Points:**
- faelight-bar: Show terminal info
- faelight-launcher: Launch with custom profiles
- faelight-theme: Unified theming

## Success Criteria

- [ ] Can run as daily driver for 1+ week without issues
- [ ] Performance within 10% of Foot's latency
- [ ] Zero crashes during normal use
- [ ] All essential features working (scrollback, colors, copy/paste)
- [ ] Clean, maintainable, well-documented code
- [ ] TOML configuration working
- [ ] Integrated with faelight ecosystem
- [ ] Pushes Rust percentage to ~85-90%

## Non-Goals (Initially)

- ❌ Tabs/splits (use tmux or Sway layouts)
- ❌ Font ligatures (maybe Phase 8)
- ❌ Transparency/blur (focus on performance first)
- ❌ Image rendering (sixel/kitty graphics)
- ❌ Emoji rendering (can come later)
- ❌ X11 support (Wayland-only)

## Timeline

**Start:** Post-presentation (no deadline pressure!)
- **Months 1-2:** Phases 1-3 (foundations)
- **Months 3-4:** Phases 4-5 (functionality)
- **Months 5-6:** Phases 6-7 (polish & parity)
- **Month 6+:** Daily driver testing

**Realistic:** 6 months to Foot-quality terminal

## Technical Challenges Expected

**High Difficulty:**
- Terminal emulation complexity (ANSI escapes, edge cases)
- Performance optimization (matching Foot's C speed)
- Wayland protocol subtleties

**Medium Difficulty:**
- Font rendering and shaping
- PTY management
- Input handling (modifiers, special keys)

**Lower Difficulty:**
- TOML configuration
- Integration with existing tools

## Learning Goals

- Deep understanding of terminal emulation
- Wayland client protocol mastery
- Font rendering internals
- High-performance Rust patterns
- Systems programming best practices

## Research Links

**Terminals to Study:**
- Foot: https://codeberg.org/dnkl/foot
- Alacritty: https://github.com/alacritty/alacritty
- Rio: https://github.com/raphamorim/rio

**Key Crates:**
- alacritty_terminal: https://crates.io/crates/alacritty_terminal
- smithay-client-toolkit: https://github.com/Smithay/client-toolkit
- fontdue: https://github.com/mooman219/fontdue

## Path to 100% Rust

**Current:**
```
Rust:   81.1%  ← faelight-term boosts this
Shell:  10.5%  ← Can migrate to Rust CLIs or Nushell
Lua:    6.9%   ← Keep (perfect for Neovim)
Python: 1.1%   ← Can eliminate
Nu:     0.4%   ← Could expand
```

**Target:** ~95% Rust (keeping Lua for Neovim is fine)

**Additional opportunities:**
- Expand Nushell usage (replace bash scripts)
- Rewrite remaining Python scripts in Rust
- Consider Ion shell (Rust) alternative

## Notes

- This is Tool #31 in Faelight Forest
- Post-presentation priority: Learning > Speed
- No pressure to rush - quality matters
- Represents ongoing commitment to development
- Ultimate demonstration of 0-Core philosophy
- Will be presented as "future work" at presentation

## Status Updates

*Post-presentation updates will be tracked here*
