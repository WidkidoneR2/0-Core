---
id: 324
title: "faelight-term v4 -- Font and Text Identity"
status: planned
date: 2026-05-20
tags: [terminal, font, typography, rendering, hinting, subpixel, foot, cosmic-text, glyphon, wgpu]
---
---
THE PREMISE

faelight-term v3 proved the GPU rendering stack.
wgpu + cosmic-text + glyphon is fast, correct, and ours.
The terminal runs. The terminal is the daily driver.

But something is off.

You look at foot. The text feels right.
You look at faelight-term v3. Something is different.
Not wrong. Just not right yet.

This is not a performance problem.
This is an identity problem.

foot has spent years finding exactly the right way to render
a monospace font on a Wayland surface with fractional scaling.
faelight-term v3 is newer. It has not found its voice yet.

faelight-term v4 finds the voice.
Not foot's voice. The forest's voice.
The typography that is unmistakably faelight.
---
WHAT MAKES FOOT FEEL LIKE FOOT

Before changing anything, understand what foot does differently.

Font rendering pipeline in foot:
  foot uses fontconfig + FreeType directly.
  FreeType applies hinting (bytecode or autohint).
  Subpixel rendering (RGB or BGR component order).
  Gamma correction for subpixel blending.
  Custom LCD filter (FreeType's built-in or custom weights).

font rendering pipeline in faelight-term v3:
  cosmic-text uses fontdb for font discovery.
  swash (via glyphon) for rasterization.
  swash is a pure-Rust font renderer -- no FreeType.
  swash does its own hinting (TrueType hint interpreter).
  Subpixel: depends on swash config + glyphon TextArea settings.
  No explicit gamma correction currently.

The differences that matter:
  1. Hinting aggressiveness -- foot may hint more strongly
  2. Subpixel component order -- RGB vs BGRA affects perceived sharpness
  3. Gamma correction -- affects contrast and weight of strokes
  4. Line height and baseline positioning -- 1-2px off feels wrong
  5. Letter spacing -- monospace cell width calculation
  6. Font choice itself -- JetBrainsMono vs other feels different

The investigation process:
  Take a screenshot of foot rendering "Hello World" at 13px JetBrainsMono.
  Take a screenshot of faelight-term v3 rendering the same.
  Zoom in 4x. Compare pixel by pixel.
  The difference will be visible and specific.
  That specific difference is what we fix.
---
WHAT WE ARE BUILDING

Not a clone of foot's renderer.
The forest's own typographic identity.

The questions to answer:
  What font does the forest speak in?
  What size? What weight? What hinting level?
  What line height makes the forest readable?
  What letter spacing feels natural?
  What subpixel mode matches the Framework display?

The answers become constants in faelight-term v4.
They are not configurable by default -- they are the forest's choice.
One perfect configuration, owned completely.

The font decision:
  JetBrainsMono -- current, clean, good ligature support
  Iosevka -- narrow, information-dense, customizable
  Monaspace -- variable axis, modern, GitHub-backed
  Fira Code -- ligature-forward, readable
  Berkeley Mono -- premium, distinctive
  The forest chooses one. Not the user. The forest.
  (User override is possible but the default is the identity.)

The rendering decision:
  Hinting: slight (not none, not full -- slight is the sweet spot)
  Subpixel: RGB horizontal (Framework display is standard RGB)
  Gamma: 1.8 (slightly lighter than sRGB 2.2 -- better for dark backgrounds)
  Line height: 1.2x font size (breathing room without waste)
  Baseline: measured precisely per font, not assumed

The size decision:
  13.5px renders clean on Framework at 1.5x scale (physical 20.25px)
  14px renders slightly heavier, more readable for long sessions
  Measure both against foot at same size. Choose what feels right.
---
REFERENCE ARCHITECTURES TO STUDY

foot (the reference):
  Language: C
  Source: codeberg.org/dnkl/foot
  Key files:
    render.c        -- the actual pixel rendering, glyph cache
    font-shaping.c  -- font selection, shaping pipeline
    config.c        -- all font config options and defaults
    fcft/           -- foot's own font library (embedded)
  Study: exactly how foot applies hinting and subpixel rendering
  Key insight: foot uses fcft (its own C library wrapping FreeType)
               This is why foot looks like foot -- fcft is tuned specifically

alacritty (GPU terminal, closest to our stack):
  Language: Rust
  Source: github.com/alacritty/alacritty
  Key files:
    alacritty/src/renderer/   -- glyph rendering, atlas
    alacritty/src/config/font.rs -- font configuration
  Study: how alacritty handles font metrics, cell sizing, baseline
  Key insight: alacritty uses crossfont (wraps FreeType via C FFI)
               We use swash (pure Rust). The difference matters.

wezterm (feature-rich Rust terminal):
  Language: Rust
  Source: github.com/wez/wezterm
  Key files:
    wezterm-font/src/   -- font loading, shaping, rendering
    wezterm-render/src/ -- GPU rendering pipeline
  Study: how wezterm achieves consistent rendering across platforms
  Key insight: wezterm uses harfbuzz for shaping, FreeType for rasterization
               More complex than our stack but shows what is possible

swash documentation (our rasterizer):
  Source: github.com/dfrg/swash
  Study: all hinting modes, subpixel options, rendering context config
  This is the most important study -- we use swash, understand it completely
  Key structs: ScaleContext, Scaler, Image, Render
  Key options: hint: bool, subpixel rendering, offset handling

cosmic-text rendering path:
  Source: github.com/pop-os/cosmic-text
  Study: how cosmic-text calls swash, what options it passes through
  Key file: src/font/system.rs, src/swash.rs
  Question: can we pass custom swash rendering options through glyphon?
  If not: do we call swash directly for terminal rendering?
---
INVESTIGATION PLAN

Step 1 -- Side-by-side screenshot analysis:
  Open foot with JetBrainsMono 13 on left.
  Open faelight-term v3 with same font and size on right.
  Type identical text in both.
  Screenshot both. Zoom 4x in GIMP or similar.
  Document every visible difference: weight, spacing, sharpness, baseline.

Step 2 -- Metric comparison:
  Print font metrics from foot (via its debug mode or source reading):
    cell width, cell height, baseline offset, ascender, descender
  Print font metrics from faelight-term v3:
    Add a debug command: faelight-term --font-metrics
  Compare. Any difference > 0.5px is significant.

Step 3 -- Hinting experiment:
  Disable hinting in swash. Screenshot. Compare to foot.
  Enable full hinting. Screenshot. Compare.
  Find the setting that matches foot most closely.
  Then decide: do we want to match foot, or do we want our own look?

Step 4 -- Subpixel experiment:
  Framework laptop: IPS display, standard RGB subpixel order.
  Test swash subpixel rendering modes.
  Compare sharpness vs grayscale rendering.
  Document what looks best at 1.5x fractional scaling.

Step 5 -- Font comparison:
  Render the same text in JetBrainsMono, Iosevka, Monaspace at 13.5px.
  Live with each for 30 minutes of real work.
  Choose the forest's font. Document why.

Step 6 -- The decision document:
  forest-typography.md in docs/
  Records: chosen font, size, hinting, subpixel, line height, baseline
  This is the forest's typographic identity in writing.
  All future tools (bar, FM, boot splash) reference this document.
---
ARCHITECTURE CHANGES FOR V4

The font rendering path today:
  glyphon TextRenderer → cosmic-text Buffer → swash ScaleContext
  Options passed: font family, size, line height
  Missing: explicit hinting mode, explicit subpixel config, gamma

The font rendering path in v4:
  Option A -- Configure through glyphon/cosmic-text:
    Research whether glyphon exposes swash rendering options.
    If yes: add hinting and subpixel config to TextRenderer setup.
    Simpler. Stays within the current abstraction.

  Option B -- Direct swash rendering for terminal glyphs:
    Bypass glyphon's rendering for terminal cell glyphs.
    Use swash ScaleContext directly with full option control.
    More complex. Complete control.
    This is what alacritty does (via crossfont → FreeType).

  Decision: investigate Option A first. If insufficient, implement Option B.

New configuration constants in faelight-term v4:
  const FONT_FAMILY: &str  = "JetBrainsMono";  // or chosen font
  const FONT_SIZE: f32     = 13.5;
  const LINE_HEIGHT: f32   = 1.2;              // multiplier
  const HINTING: Hinting   = Hinting::Slight;
  const SUBPIXEL: Subpixel = Subpixel::Horizontal;
  const GAMMA: f32         = 1.8;
  const CELL_PADDING: u32  = 0;               // pixels around each cell

New debug command:
  faelight-term --font-metrics
  Prints: cell width, cell height, baseline, ascender, descender
  Useful for comparing against foot's metrics

Ligature support:
  fsh v3 deferred ligatures (per-cell rendering incompatible).
  v4 investigates: can ligatures work within the cell grid?
  If a ligature spans 2 cells: render as wide glyph across both cells.
  This is what wezterm does.
  Gate: at minimum, ->  => and != render as ligatures if font supports it.
---
THE TYPOGRAPHY DOCUMENT

docs/forest-typography.md created as part of this intent.
Contents:
  The chosen font and why
  The exact rendering parameters
  Screenshots showing the decision
  How this applies to: faelight-term, faelight-bar, faelight-fm, faelight-boot
  The principle: one typographic voice across the entire forest

This document is referenced by all future visual intents.
Every tool that renders text in the forest references this document first.
---
PHASES

Phase 0 -- Investigation (1 session):
  Side-by-side analysis: foot vs faelight-term v3
  Font metrics comparison
  Hinting and subpixel experiments
  Gate: specific differences documented with screenshots
        forest-typography.md started

Phase 1 -- Rendering fixes:
  Apply correct hinting mode
  Apply subpixel configuration
  Fix gamma if needed
  Correct baseline and line height
  Gate: side-by-side comparison shows clear improvement
        text weight and sharpness matches or exceeds foot

Phase 2 -- Font identity:
  Font comparison: JetBrainsMono vs Iosevka vs Monaspace vs others
  Choose the forest's font
  Document the choice in forest-typography.md
  Gate: font chosen, documented, applied to faelight-term v4

Phase 3 -- Cell metric precision:
  Measure cell width and height exactly per chosen font
  Ensure all ANSI art, box drawing, progress bars render correctly
  Gate: box drawing characters align perfectly
        no gaps or overlaps in cell grid

Phase 4 -- Ligature investigation:
  Research swash ligature support at cell level
  Implement if feasible without breaking cell grid
  Gate: at minimum -> and => render as ligatures if font supports them
        cell grid integrity maintained

Phase 5 -- Typography document + daily driver:
  forest-typography.md complete
  faelight-term v4 deployed and running as daily driver
  1 week comparison period against foot
  Gate: after 1 week, faelight-term v4 preferred over foot for daily use
---
GATES
[ ] Phase 0: foot vs v3 differences documented with screenshots
[ ] Phase 1: hinting, subpixel, gamma fixed -- measurable improvement
[ ] Phase 2: forest font chosen and documented
[ ] Phase 3: cell metrics precise -- box drawing characters correct
[ ] Phase 4: ligature investigation complete, implemented if feasible
[ ] Phase 5: forest-typography.md complete, daily driver 1 week
Final:
[ ] The forest has a typographic identity -- one font, one rendering config
[ ] faelight-term v4 text is immediately recognizable as the forest
[ ] forest-typography.md is the reference for all future visual tools
[ ] After 1 week daily use, foot is no longer opened for comparison
---
DEPENDS ON
faelight-term v3 (INT-286) -- COMPLETE -- GPU rendering foundation
swash documentation -- study required
foot source -- study required (C, but the concepts translate directly)

TIMELINE
Phase 0 (investigation): can start any time, independent of other intents
Phase 1-3: 2-3 sessions
Phase 4-5: 1-2 sessions
Target: forest-typography.md complete before NY presentation
        Linus Torvalds will see this terminal -- it should be unmistakable

"The terminal is the primary surface.
Every hour of work happens through it.
The font is the voice of that surface.
faelight-term v4 finds the voice.
Not foot's voice. Not alacritty's voice.
The forest's voice." 🌲
