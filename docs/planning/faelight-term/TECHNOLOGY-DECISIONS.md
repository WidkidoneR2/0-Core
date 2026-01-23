# 🎯 faelight-term Technology Decisions

## Date: 2026-01-22
## Status: Phase 1 Research Complete

## VTE Library: ✅ DECIDED - alacritty_terminal

**Choice:** `alacritty_terminal` v0.25.2

**Why:**
- Battle-tested (powers Alacritty)
- Clean, standalone crate
- Handles all terminal emulation complexity
- Active development
- Uses standard `vte` crate underneath
- Cross-platform (Unix + Windows)

**What it provides:**
- Terminal grid management
- ANSI/CSI/OSC escape sequences
- PTY creation and I/O
- Scrollback buffer
- Selection/copy
- Event loop foundation

**Dependency:**
```toml
alacritty_terminal = "0.25.2"
```

---

## Wayland Client: ✅ DECIDED - smithay-client-toolkit

**Choice:** `smithay-client-toolkit`

**Why:**
- De facto standard for Wayland in Rust
- Well-documented
- Active community
- Handles protocols we need:
  - xdg-shell (windows)
  - text-input-v3 (IME)
  - clipboard
  - keyboard/mouse

**Dependency:**
```toml
smithay-client-toolkit = "0.18"
```

---

## Font Rendering: ✅ DECIDED - fontdue

**Choice:** `fontdue`

**Why:**
- Pure Rust (no C dependencies)
- Fast, simple
- Minimal API
- Perfect for monospace terminals
- No complex shaping needed initially

**If we need shaping later:**
- Can add `rusttype` + `harfbuzz_rs`

**Dependency:**
```toml
fontdue = "0.9"
```

---

## Configuration: ✅ DECIDED - serde + toml

**Choice:** `serde` + `toml`

**Why:**
- Already used in ecosystem
- Simple, clean syntax
- Type-safe with derive macros
- Much simpler than C parsing!

**Dependency:**
```toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

---

## Event Loop: ⏳ TBD

**Options:**
1. Use alacritty_terminal's event loop
2. Build custom with epoll/kqueue
3. Use tokio (async)

**Decision:** Will determine during Phase 2 implementation

---

## Rendering: ✅ DECIDED - Software rendering

**Choice:** Software rendering (like Foot)

**Why:**
- Foot proves it's fast enough
- No GPU overhead
- Simpler code
- Lower latency

**Can add GPU later if needed:**
- `wgpu` for GPU acceleration
- Only if benchmarks show need

---

## Summary

**Core Stack:**
```toml
[dependencies]
alacritty_terminal = "0.25.2"
smithay-client-toolkit = "0.18"
fontdue = "0.9"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

**Total dependencies:** 5 core crates (plus their deps)

**Estimated LoC for MVP:** ~500-1000 lines of glue code

**Why this works:**
- alacritty_terminal handles terminal complexity (147KB saved!)
- smithay handles Wayland complexity (87KB saved!)
- fontdue handles font complexity
- We write the glue: window, config, integration

**Comparison:**
- Foot (C): ~500K lines
- faelight-term (Rust): ~1K lines glue + libraries

This is the power of standing on giants' shoulders! 🦀
