---
id: 303
title: "pastel evaluation -- color tool for forest color DNA workflow"
status: planned
date: 2026-05-14
type: eval
tags: [pastel, color, palette, design, tools, color-dna]
depends_on: []
---
## What Is pastel
pastel is a Rust command-line color tool.
It manipulates, converts, and displays colors in the terminal.
Source: https://github.com/sharkdp/pastel (sharkdp -- also wrote bat, fd, hyperfine)

Capabilities:
  - Display colors as terminal swatches
  - Convert between hex, RGB, HSL, ANSI
  - Lighten, darken, saturate, desaturate
  - Mix colors with configurable ratios
  - Generate color palettes
  - Check WCAG contrast ratios
  - Pick colors from named sets

## Why This Matters Now
The forest is redesigning its color DNA (2026-05-14).
New palette: Abyss Black-Blue base, Neon Azure/Electric Cyan accents,
Sharp Forest Green for Friday, full terminal palette defined.

pastel enables:
  - Verify contrast ratios before committing to colors
  - Preview colors in terminal without leaving the shell
  - Derive complementary shades (hover states, disabled states)
  - Generate the full 16-color terminal palette from 4-5 base colors
  - Export color values in any format needed by Rust/TOML configs

## Color DNA Workflow
  pastel color "#0a0f14"               -- preview background
  pastel contrast "#0a0f14" "#a9dfff"  -- verify foreground contrast
  pastel lighten 0.1 "#0a0f14"         -- derive secondary background
  pastel mix "#00bfff" "#00e0ff"       -- blend two accents
  pastel gradient "#00bfff" "#00ff88" 5 -- 5-step gradient from azure to friday green

## Evaluation Criteria
1. Does contrast checking work reliably (WCAG AA/AAA)?
2. Does terminal color preview render correctly in fsh + faelight-term?
3. Is it useful for deriving the full 16-color palette from base colors?
4. Can it export in formats useful for Rust TOML config files?
5. Does it integrate naturally as a fsh vocabulary word?

## Integration Plan (if evaluation passes)
- Add `color` vocabulary word in fsh:
    color "#00bfff"           -- preview a color
    color contrast "#bg" "#fg" -- check contrast ratio
    color palette             -- show current forest color DNA
- Store forest color DNA in state.db
- `color palette` reads from state.db and displays with pastel
- Register in command registry (INT-259)

## Forest Color DNA (to be stored in state.db)
Background:  #0a0f14  Abyss Black-Blue
Foreground:  #a9dfff  Soft Ice Blue
Secondary:   #101820  Deep Graphite
Primary:     #00bfff  Neon Azure
Accent:      #00e0ff  Electric Cyan
Success:     #2affd5  Aqua Mint
Warning:     #ffd43b  Soft Amber
Error:       #ff4c4c  Signal Red
Info:        #33d4ff  Sky Glow
Friday:      #00ff88  Sharp Forest Green

## Gates
- [ ] pastel installed and working in fsh
- [ ] color preview renders in both foot and faelight-term
- [ ] contrast ratios verified for all color DNA pairs
- [ ] full 16-color terminal palette derived and documented
- [ ] color vocabulary word implemented in fsh
- [ ] forest color DNA stored in state.db
- [ ] color palette command shows full DNA
- [ ] 3 days daily use confirms value

---
"Color is not decoration.
In the forest, color is signal.
Every shade carries meaning.
pastel helps the forest speak in the right colors." 🌲
