# Foot Config to Match

## From ~/.config/foot/foot.ini

**Font:**
- JetBrainsMono Nerd Font:size=12

**Colors:**
- background=0f1411
- foreground=d7e0da
- regular0=0f1411  
- regular1=e67e80

## Changes Needed in main.rs

1. Font size: 24.0 → 12.0
2. Background: 0x00,0x00,0x00 → 0x11,0x14,0x0f
3. Text color: 0xFF,0xFF,0xFF → 0xda,0xe0,0xd7

## Phase 4 Goals
- Match colors exactly
- Add PTY (shell spawning)
- Terminal grid
- Cursor rendering
- ANSI escape sequences
