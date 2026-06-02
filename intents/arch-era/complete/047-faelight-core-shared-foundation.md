---
id: 047
date: 2026-01-14
type: future
title: "faelight-core - Shared Foundation Library"
status: complete
tags: [architecture, v7.0, rust, performance, library]
---

## The Problem

Current state across 4 tools:
```
faelight-bar/      ← Rasterizes glyphs every frame
faelight-launcher/ ← Rasterizes glyphs every frame
faelight-notify/   ← Rasterizes glyphs every frame
faelight-menu/     ← Rasterizes glyphs every frame

faelight-bar/      ← Recreates SHM buffers
faelight-launcher/ ← Recreates SHM buffers
faelight-notify/   ← Recreates SHM buffers
faelight-menu/     ← Recreates SHM buffers

faelight-bar/      ← draw_rect, draw_text, colors
faelight-launcher/ ← draw_rect, draw_text, colors
faelight-notify/   ← draw_rect, draw_text, colors
faelight-menu/     ← draw_rect, draw_text, colors
```

**Result:** 70-90% wasted CPU, code duplication, inconsistent behavior

## The Solution

Shared library: `faelight-core`

## Architecture

### Module Structure
```
faelight-core/
├── src/
│   ├── lib.rs           # Public API
│   ├── glyph.rs         # Glyph cache
│   ├── canvas.rs        # Drawing primitives
│   ├── theme.rs         # Colors, fonts, spacing
│   ├── wayland.rs       # Layer-shell helpers
│   └── error.rs         # Error types
├── Cargo.toml
└── README.md
```

### Key Components

#### 1. Glyph Cache (70%+ CPU Reduction)
```rust
// glyph.rs
use fontdue::{Font, Metrics};
use std::collections::HashMap;

pub struct Glyph {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
}

type GlyphKey = (char, u32); // (character, size * 10)

pub struct GlyphCache {
    font: Font,
    cache: HashMap<GlyphKey, Glyph>,
}

impl GlyphCache {
    pub fn new(font_data: &[u8]) -> Self {
        Self {
            font: Font::from_bytes(font_data, Default::default()).unwrap(),
            cache: HashMap::new(),
        }
    }
    
    pub fn rasterize(&mut self, ch: char, size: f32) -> &Glyph {
        let key = (ch, (size * 10.0) as u32);
        self.cache.entry(key).or_insert_with(|| {
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            Glyph { metrics, bitmap }
        })
    }
}
```

**Impact:** Single shared cache across all tools, 70-90% CPU reduction

#### 2. Canvas Abstraction
```rust
// canvas.rs
use wayland_client::protocol::wl_shm_pool::WlShmPool;

pub struct Canvas {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    shm: Option<WlShmPool>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self;
    
    pub fn clear(&mut self, color: u32);
    
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32);
    
    pub fn draw_text(
        &mut self,
        cache: &mut GlyphCache,
        text: &str,
        x: u32,
        y: u32,
        size: f32,
        color: u32,
    ) -> u32; // Returns text width
    
    pub fn create_shm_buffer(&mut self, pool: &WlShmPool) -> Buffer;
    
    pub fn resize(&mut self, width: u32, height: u32);
}
```

**Impact:** No more SHM buffer recreation, consistent drawing API

#### 3. Theme System
```rust
// theme.rs
pub struct Theme {
    // Background colors
    pub bg_primary: u32,
    pub bg_secondary: u32,
    pub bg_tertiary: u32,
    
    // Text colors
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    
    // Accent colors
    pub accent: u32,
    pub accent_hover: u32,
    pub danger: u32,
    pub warning: u32,
    pub success: u32,
    
    // Spacing
    pub padding: u32,
    pub gap: u32,
    pub border_width: u32,
    
    // Typography
    pub font_size_small: f32,
    pub font_size_normal: f32,
    pub font_size_large: f32,
}

impl Theme {
    pub fn faelight_default() -> Self;
}
```

**Impact:** Consistent theming, single source of truth

#### 4. Wayland Helpers
```rust
// wayland.rs
use smithay_client_toolkit::shell::layer::{Anchor, Layer, LayerShell};

pub struct LayerSurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub anchor: Anchor,
    pub layer: Layer,
    pub margin: (i32, i32, i32, i32), // top, right, bottom, left
    pub exclusive_zone: i32,
    pub keyboard_interactivity: bool,
}

pub fn create_layer_surface(
    config: LayerSurfaceConfig,
    layer_shell: &LayerShell,
) -> LayerSurface;

pub fn request_frame(surface: &WlSurface, qh: &QueueHandle);
```

**Impact:** Consistent layer-shell setup, proper frame callbacks

#### 5. Error Handling
```rust
// error.rs
#[derive(Debug, thiserror::Error)]
pub enum FaelightError {
    #[error("Wayland connection failed: {0}")]
    WaylandConnection(String),
    
    #[error("Rendering error: {0}")]
    Rendering(String),
    
    #[error("Font loading failed: {0}")]
    Font(String),
}

pub type Result<T> = std::result::Result<T, FaelightError>;
```

**Impact:** Proper error propagation, better debugging

## Implementation Plan

### Phase 1: Core Library (3-4 days)

**Day 1: Project setup + Glyph cache**
- Create `faelight-core` crate
- Implement `GlyphCache`
- Unit tests

**Day 2: Canvas abstraction**
- Implement `Canvas` with basic primitives
- SHM buffer management
- Integration tests

**Day 3: Theme + Wayland helpers**
- Define `Theme` constants
- Layer-shell config helpers
- Frame callback utilities

**Day 4: Polish + Documentation**
- Error types
- API documentation
- Example usage

### Phase 2: Dependency Setup
```toml
# faelight-core/Cargo.toml
[package]
name = "faelight-core"
version = "0.1.0"
edition = "2021"

[dependencies]
fontdue = "0.9"
wayland-client = "0.31"
smithay-client-toolkit = "0.19"
thiserror = "1.0"

[dev-dependencies]
# For testing without Wayland
```

### Phase 3: Tool Migration (Intent 048)

Each tool migrates in order:
1. bar (simplest, already stable)
2. menu (small, good test case)
3. notify (medium complexity)
4. launcher (most complex, benefits most)

## Success Criteria

**Core library:**
- [ ] GlyphCache working with 70%+ CPU reduction measured
- [ ] Canvas API complete (clear, draw_rect, draw_text)
- [ ] Theme constants defined
- [ ] Wayland helpers functional
- [ ] Error types defined
- [ ] Documentation complete
- [ ] Unit tests passing

**Integration:**
- [ ] All 4 tools compile with faelight-core
- [ ] No rendering regressions
- [ ] Performance improvement verified
- [ ] Code duplication eliminated

## Performance Target

**Before (current):**
- bar: ~5-10% CPU when updating
- launcher: ~15-20% CPU while typing
- notify: ~8-12% CPU per notification
- menu: ~10-15% CPU when open

**After (with faelight-core):**
- bar: ~1-2% CPU when updating (5-10x improvement)
- launcher: ~2-4% CPU while typing (5-7x improvement)
- notify: ~1-3% CPU per notification (5-8x improvement)
- menu: ~2-3% CPU when open (5-7x improvement)

**Measurement method:**
```bash
# Before
top -b -n 1 | grep faelight-bar

# After
top -b -n 1 | grep faelight-bar

# Compare
```

## Dependencies

- fontdue 0.9 (font rasterization)
- smithay-client-toolkit 0.19 (Wayland)
- thiserror 1.0 (error handling)

## Philosophy Alignment

**Does this advance 0-Core principles?**

✅ **Understanding:** Shared code = single place to learn  
✅ **Explicit:** Clear API, no magic  
✅ **Performance:** Respect for system resources  
✅ **Recovery:** Better error handling  

**Quote:** "Build the foundation once. Use it everywhere."

## Timeline

**Week 1 of v7.0.0:** 3-4 days to ship faelight-core v0.1

---

_"Duplication is waste. A shared foundation is wisdom."_ 🌲
