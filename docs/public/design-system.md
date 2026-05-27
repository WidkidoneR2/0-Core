# Faelight Forest Design System
*The canonical visual identity for every forest tool.*

---

## Philosophy
The forest looks like it was grown, not assembled.
Every pixel is intentional. Every color has meaning.
Depth. Warmth. Forest intelligence made visible.

---

## Color DNA

### Core Palette
Background:     #0a0f0a  -- Deep Forest Black (primary surfaces)
Background Alt: #111810  -- Forest Floor (secondary surfaces, panels)
Foreground:     #a8c5b0  -- Soft Moss (primary text)
Foreground Dim: #4a6b52  -- Deep Moss (secondary text, hints)

### Accent Colors
Forest Green:   #2affd5  -- Aqua Mint (primary accent, active states)
Neon Azure:     #00bfff  -- Electric Blue (borders, focus rings)
Sharp Green:    #00ff88  -- Sharp Forest (success, health 100%)
Soft Amber:     #ffd43b  -- Warm Gold (warnings, selection)
Coral:          #ff6b6b  -- Alert Red (errors, destructive)

### Surface Hierarchy
Surface 0: #0a0f0a  -- Deepest (window backgrounds)
Surface 1: #111810  -- Base (panel backgrounds)
Surface 2: #1a2419  -- Raised (cards, sidebars)
Surface 3: #243323  -- Elevated (tooltips, menus)
Border:    #1e3a28  -- Subtle (default borders)
Border Active: #00bfff  -- Neon Azure (focused borders)

### Glow System
Active elements emit a subtle glow using their accent color:
Focused window border: 0 0 8px #00bfff44
Active intent in bar:  0 0 12px #2affd544
Friday signal:         0 0 6px #00ff8866

---

## Typography

### Terminal / Code
- Font: JetBrains Mono (primary)
- Fallback: Fira Code, monospace
- Size: 13px base, 11px secondary

### UI Labels
- Font: Inter (primary), system-ui (fallback)
- Weight: 400 (body), 600 (labels), 700 (headings)
- Letter spacing: 0.02em for UI labels

---

## Spacing System

### Base Unit: 4px
xs:  4px   -- tight spacing (icon padding)
sm:  8px   -- compact (list items)
md:  16px  -- standard (section padding)
lg:  24px  -- generous (panel margins)
xl:  32px  -- spacious (major sections)

### Border Radius
Sharp:  0px   -- terminal elements, code blocks
Subtle: 4px   -- small UI elements, tags
Soft:   8px   -- cards, panels
Round:  12px  -- buttons, dialogs
Full:   9999px -- pills, badges

---

## Icon System

### Principles
1. 3D candy aesthetic -- depth, not flat
2. Forest color palette -- no generic system icons
3. Consistent 24x24 base size (scalable to 16, 32, 48)
4. Rounded corners (4px radius)
5. Forest green base with accent highlights

### Tool Icons (planned, NixOS era)
faelight-term:   🖥  Terminal -- green glow, command prompt visual
faelight-bar:    📊  Bar -- horizontal strip with Friday eye
faelight-menu:   🌿  Menu -- leaf/launcher, expanding branches
faelight-lock:   🔒  Lock -- forest padlock, glowing keyhole
faelight-login:  🌲  Login -- full tree, dawn light
faelight-notify: 🔔  Notify -- bell with forest green pulse
faelight-fm:     📁  Files -- folder with forest floor texture
faelight-git:    🌿  Git -- branch with leaf nodes

---

## Component Library

### Status Indicators
Health 100%:  ✅ bright green  #00ff88
Health 90%+:  ✅ forest green  #2affd5
Health 80%+:  ⚠️  amber        #ffd43b
Health <80%:  ❌ coral         #ff6b6b

### Focus Ring
All focusable elements: 2px solid #00bfff, glow 0 0 8px #00bfff44

### Friday Signals
High confidence (>90%):  Aqua mint text, brief glow pulse
Medium (70-90%):         Soft moss, no glow
Low (<70%):              Dimmed, not shown

### Bar Zones
Left:   Lock status + workspace tags (compact, monospace)
Center: Active intent title (centered, slightly larger)
Right:  Time + Friday status (compact)
Mode:   Full-width when in resize/launcher mode (amber background)

---

## Application Guidelines

### Terminal (faelight-term)
- Background: Surface 0 (#0a0f0a)
- Cursor: Neon Azure (#00bfff), blinking block
- Selection: Surface 3 with amber tint
- Friday panel: Surface 2 background, Aqua Mint header

### Bar (faelight-bar v4)
- Background: Surface 1 with 95% opacity
- Height: 28px
- Border bottom: 1px solid Border Active when focused window present
- Font: JetBrains Mono 11px

### Menu / Launcher (faelight-menu v4)
- Background: Surface 2 with blur
- Width: 600px centered
- Item height: 48px
- Active item: Surface 3 + left border 3px Aqua Mint
- Icon: 24px candy icons (left)
- Search bar: Surface 0, Neon Azure focus ring

### Lock Screen (faelight-lock)
- Background: Forest Black + wallpaper at 40% opacity
- Clock: Aqua Mint, 48px, centered
- Input: Surface 2, Neon Azure focus
- Glow: Radial gradient from center

### Login Greeter (faelight-login)
- Minimal: dark background, centered forest tree logo
- Session selector: both Niri and Pinnacle visible until cutover
- Status panel: health, intent, Friday brief

---

## Implementation Status

| Tool | Colors | Glow | Icons | libcosmic |
|------|--------|------|-------|-----------|
| faelight-term v3 | ✅ | partial | ❌ | ❌ |
| faelight-bar v3 | ✅ | ❌ | ❌ | ❌ |
| faelight-menu v3 | ✅ | ❌ | ❌ | ❌ |
| faelight-lock | ✅ | ❌ | ❌ | ❌ |
| faelight-login | ✅ | ❌ | ❌ | ❌ |
| faelight-fm v2 | ✅ | ✅ | partial | ✅ |
| faelight-compositor | partial | ❌ | ❌ | ❌ |

Full visual unification: NixOS era with libcosmic throughout.

---

*"The forest that thinks in Rust
should also look like it was grown, not assembled.
Depth. Warmth. Intention in every pixel."* 🌲