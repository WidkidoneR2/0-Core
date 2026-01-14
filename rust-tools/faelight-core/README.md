# faelight-core v0.1.0

Shared foundation library for all Faelight tools.

## Features

- **GlyphCache**: 70-90% CPU reduction in text rendering through intelligent caching
- **Canvas**: Simple drawing primitives with alpha blending
- **Theme**: Consistent color schemes across all tools (3 variants included)
- **Wayland Helpers**: Pre-configured layer-shell setups for common use cases
- **Error Handling**: Proper Result types with thiserror

## Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
faelight-core = { path = "../faelight-core" }
```

### Example: Simple Bar
```rust
use faelight_core::{Canvas, GlyphCache, Theme, LayerSurfaceConfig};

fn main() {
    // Load font
    let font_data = std::fs::read("/usr/share/fonts/TTF/JetBrainsMono.ttf").unwrap();
    let mut cache = GlyphCache::new(&font_data).unwrap();
    
    // Create canvas
    let mut canvas = Canvas::new(1920, 32);
    let theme = Theme::faelight_default();
    
    // Draw
    canvas.clear(theme.bg_primary);
    canvas.draw_text(&mut cache, "Hello World", 10, 8, theme.font_size_normal, theme.text_primary);
    
    // Layer config for bar
    let config = LayerSurfaceConfig::top_bar(32);
}
```

### Example: Notification Popup
```rust
use faelight_core::{Canvas, Theme, LayerSurfaceConfig};

let config = LayerSurfaceConfig::notification(300, 100);
let mut canvas = Canvas::new(300, 100);
let theme = Theme::faelight_default();

canvas.clear(theme.bg_secondary);
canvas.draw_rect(0, 0, 4, 100, theme.accent); // Accent bar
```

## Performance

**Before faelight-core:**
- faelight-bar: ~5-10% CPU when updating
- faelight-launcher: ~15-20% CPU while typing

**After faelight-core:**
- faelight-bar: ~1-2% CPU (5-10x improvement)
- faelight-launcher: ~2-4% CPU (5-7x improvement)

## Philosophy

Built on 0-Core principles:
- **Understanding**: Shared code = single place to learn
- **Explicit**: Clear API, no magic
- **Performance**: Respect for system resources
- **Recovery**: Better error handling

## Modules

- `glyph`: Font rasterization with caching
- `canvas`: Drawing primitives and pixel buffer management
- `theme`: Color schemes and typography
- `wayland`: Layer-shell configuration helpers
- `error`: Error types and Result

## Testing
```bash
cargo test
```

## License

MIT
