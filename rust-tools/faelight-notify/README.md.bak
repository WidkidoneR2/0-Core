# faelight-notify v0.9.0

**Wayland notification daemon with urgency-based visual feedback**

## Features

- ✅ **Urgency Colors**: RED for critical, GREEN for normal, DIM for low
- ✅ **Wayland Native**: Pure Rust, layer-shell rendering
- ✅ **Click to Dismiss**: Left-click removes notification
- ✅ **D-Bus Compatible**: Implements org.freedesktop.Notifications

## Usage
```bash
# Start daemon (add to sway config)
faelight-notify &

# Send notifications
notify-send "Title" "Message"                    # Green border (normal)
notify-send -u critical "Alert" "Important!"     # Red border (critical)
notify-send -u low "Info" "Background info"      # Dim border (low)
```

## Installation
```bash
cargo build --release -p faelight-notify
sudo cp target/release/faelight-notify /usr/local/bin/
```

Add to `~/.config/sway/config`:
```
exec faelight-notify
```

## Technical Details

- **Rendering**: Direct pixel manipulation with faelight-core GlyphCache
- **Protocol**: Wayland layer-shell (overlay layer, top-right anchor)
- **Format**: ARGB8888 for correct color display
- **Timeout**: 5 seconds default (configurable via notify-send)

## Color Reference
```rust
CRITICAL: [0x6b, 0x6b, 0xe3, 0xFF]  // Red (ARGB)
NORMAL:   [0x00, 0xd0, 0x00, 0xFF]  // Green (ARGB)
LOW:      [0x77, 0x8f, 0x7f, 0xFF]  // Dim (ARGB)
```

---

**🌲 Faelight Forest** - Rust-native Wayland desktop environment
