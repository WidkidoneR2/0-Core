# Forest Typography
## The Forest's Typographic Identity
*Established: 2026-05-30 -- INT-324*

---

## The Font

**JetBrainsMono Nerd Font**
- Weight: Regular (400)
- Size: 12px (runtime adjustable via Ctrl+=/-)
- Variant: Nerd Font (full glyph coverage for icons, glyphs, powerline)

### Why JetBrainsMono

The forest investigated four candidates:
- **Noto Sans Mono** -- was the default (via Family::Monospace). Generic. No identity.
- **JetBrainsMono** -- clean, designed for code, excellent legibility at small sizes
- **Iosevka** -- narrow, information-dense. Considered for ADE pane contexts.
- **Hack Nerd Font** -- good fallback, less distinctive

JetBrainsMono was chosen because:
1. Designed specifically for long coding sessions -- reduced eye strain
2. Clean distinction between similar characters (0/O, 1/l/I)
3. Nerd Font variant covers all forest glyphs (🌲, icons, powerline symbols)
4. Already installed system-wide on Arch and will carry to NixOS
5. Matches foot's configuration -- continuity of experience

The forest does not configure this per-user. One font. The forest's voice.

---

## Rendering Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Font family | JetBrainsMono Nerd Font | via cosmic-text Family::Name |
| Base size | 12.0px | Ctrl+0 resets to this |
| Line height | 12.0 × 1.286 = ~15.4px | breathing room without waste |
| Cell width | font_size × 0.6 | scales with runtime size |
| Cell height | line_height | matches LINE_HEIGHT |
| Min size | 8px | Ctrl+- floor |
| Max size | 32px | Ctrl+= ceiling |

### Runtime Adjustment
- `Ctrl+=` -- increase font size (1px increments)
- `Ctrl+-` -- decrease font size (1px increments)  
- `Ctrl+0` -- reset to 12px default

---

## Hinting and Subpixel

**Current state (2026-05-30):** Not formally investigated.

faelight-term uses cosmic-text → swash for rasterization.
swash is a pure-Rust font renderer (no FreeType).

Planned investigation (INT-324 Phase 1 remaining):
- Compare swash hinting modes (slight vs full vs none)
- Framework 16 display: IPS, standard RGB subpixel order
- Target: slight hinting, RGB horizontal subpixel

---

## The ADE Context

When faelight-ade launches, faelight-term uses the same 12px default.
The terminal pane in faelight-ade targets 12px for readability in the
60% width left pane.

Alacritty (primary terminal, Mod+Alt+Return) also uses:
- JetBrainsMono Nerd Font
- size = 12.0
- Forest color palette

This ensures visual continuity between faelight-term and Alacritty.

---

## Color Palette (Terminal Context)

| Role | Color | Hex |
|------|-------|-----|
| Background | Forest dark | #0a0f0a |
| Foreground | Forest mist | #a8c5b0 |
| Accent | Forest cyan | #2affd5 |
| Blue | Sky | #00bfff |
| Amber | Warning | #ffd43b |
| Red | Error | #ff5555 |
| Magenta | Friday | #ff79c6 |

---

## Application to Forest Tools

Every forest tool that renders text references this document first.

| Tool | Font | Notes |
|------|------|-------|
| faelight-term | JetBrainsMono NF 12px | primary terminal |
| faelight-ade | JetBrainsMono NF 12px | via faelight-term |
| Alacritty | JetBrainsMono NF 12px | primary terminal |
| faelight-bar | JetBrainsMono NF | via cosmic-text |
| faelight-login | JetBrainsMono NF | via slint |
| faelight-menu | JetBrainsMono NF | via ratatui |
| friday-chat | JetBrainsMono NF | via ratatui |

---

## The Principle

The terminal is the primary surface.
Every hour of work happens through it.
The font is the voice of that surface.

faelight-term found the voice.
Not foot's voice. Not alacritty's voice.
The forest's voice.

One font. One size. One rendering config.
Owned completely. 🌲
