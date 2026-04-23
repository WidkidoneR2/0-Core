---
id: 232
title: "faelight-term v2 -- The Terminal That Thinks in 2136"
status: in-progress
date: 2026-04-19
tags: [terminal, faelight-term, friday, wgpu, cosmic-text, rendering, intelligence, ux, wayland, v2, overhaul]
---
Every other terminal shows you text.
faelight-term shows you context, knowledge, and Friday.
Not an incremental upgrade. A complete rebuild from the ground up.
Built for 2136, deployed in 2026.
The only thing carried forward from v11:
- The welcome screen (version, intents, health, session context)
- Ctrl+Shift+S forest status strip toggle
- The Faelight color palette
Everything else is rebuilt with one question:
"What would a terminal look like if it was invented today,
by someone who understands the forest?"
The terminal is not a text pipe.
The terminal is the interface between the human and the machine.
It should be intelligent, responsive, beautiful, and aware.
It should know what you are building and help you build it.
Flawless. No exceptions.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE DECISION (LOCKED)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHY THE CURRENT TERM IS BROKEN:
  The current faelight-term uses smithay SHM (shared memory) software rendering.
  Every frame redraws the entire pixel buffer in CPU memory.
  This is why:
    - Copy/paste breaks beyond what fits on screen (buffer limit)
    - cat of large files chokes or corrupts
    - Resize causes artifacts (full redraw on every resize event)
    - 165Hz is impossible (CPU can't redraw fast enough)
    - Large scrollback degrades performance
THE FIX -- GPU PIPELINE:
  wgpu replaces the SHM renderer.
  smithay stays -- it handles the Wayland window, seat, keyboard, pointer.
  The split is clean:
    smithay  = Wayland protocol layer (window, input, surface)
    wgpu     = GPU render pipeline (glyph atlas, cells, colors, cursor)
  wgpu renders to a wgpu::Surface backed by the smithay WlSurface.
  This is the correct Wayland + wgpu architecture.
  AMD Radeon 780M handles 165Hz trivially with this approach.
WHY cosmic-text REPLACES swash:
  swash is a low-level font engine -- powerful but requires manual glyph
  pipeline management. In the current term this causes:
    - Nerd Font powerline glyphs misaligned
    - Ligatures partially broken (-> renders, => does not)
    - Emoji fallback unreliable
    - CJK wide characters misaligned
  cosmic-text is a higher-level text shaping engine built on swash.
  It handles:
    - Correct ligature rendering out of the box
    - Nerd Font glyph metrics correctly
    - Emoji with correct fallback to NotoColorEmoji
    - Bidirectional text and CJK alignment
    - Font fallback chains (JetBrainsMono -> NotoColorEmoji -> system)
  cosmic-text is actively maintained by System76 for COSMIC desktop.
  It is production-proven on Wayland.
DEPENDENCY STACK (LOCKED):
  smithay-client-toolkit  -- Wayland window + input (keep)
  wgpu                    -- GPU renderer (replaces SHM)
  cosmic-text             -- text shaping (replaces swash)
  vte                     -- VTE/ANSI parser (keep, it is correct)
  nix                     -- PTY (keep, PTY layer works)
  calloop                 -- event loop (keep)
  wl-clipboard-rs         -- clipboard (upgrade to full scrollback support)
  rusqlite                -- state.db connection (keep)
MODULE STRUCTURE (NEW):
  src/
    main.rs           -- entry point, event loop setup
    renderer/
      mod.rs          -- wgpu pipeline, surface management
      atlas.rs        -- glyph texture atlas
      grid.rs         -- terminal cell grid rendering
      cursor.rs       -- cursor rendering
    terminal/
      mod.rs          -- terminal state, grid, scrollback
      vte.rs          -- VTE parser integration
      grid.rs         -- cell grid (char, fg, bg, attrs)
      scrollback.rs   -- 50k line ring buffer
    input/
      mod.rs          -- keyboard, mouse, clipboard
      keyboard.rs     -- key mapping, modifier handling
      clipboard.rs    -- full scrollback copy/paste
      mouse.rs        -- selection, clicks, scroll
    friday/
      mod.rs          -- Friday panel, state.db reader
      panel.rs        -- sliding panel UI
      knowledge.rs    -- knowledge entry display
      inline.rs       -- inline output intelligence
    prompt/
      mod.rs          -- two-line context prompt
      context.rs      -- intent, health, git, time
    panes/
      mod.rs          -- split pane management
      layout.rs       -- H/V split, navigation
    config.rs         -- configuration, colors
    session.rs        -- session memory, state.db writes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
FEATURE SPECIFICATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
COPY/PASTE (FIXED):
  - Ctrl+Shift+C copies full selection including beyond screen
  - Ctrl+Shift+V pastes exactly -- unicode, emoji, Rust code, multiline
  - Middle-click paste works
  - Bracketed paste mode supported natively
  - No selection state loss on resize or scroll
  - Scrollback selection works across 50k lines
GPU RENDERING:
  - 165Hz capable, no frame drops on AMD Radeon 780M
  - True 24-bit color everywhere
  - Font ligatures: -> => != <= >= <> |> ... all correct via cosmic-text
  - Powerline/Nerd Font glyphs: no missing boxes, no fallback squares
  - Bold, italic, dim, strikethrough all correct
  - Resize: zero artifacts, instant redraw via GPU dirty rects
  - Scrollback: 50,000 lines, smooth GPU-accelerated scroll
  - Wide character support: CJK, emoji, box-drawing all aligned
PROMPT:
  Two-line prompt. Modern. Context-aware. Alive.
  Line 1: zone-icon · git-branch · health% · active-intent · time
  Line 2: caret with color based on context
  Caret colors:
    Green  -- last command succeeded
    Red    -- last command failed
    Amber  -- health below 95%
    Cyan   -- Friday has something to say
  No powerline spam. No visual noise. Clean and readable.
FRIDAY PANEL (Ctrl+Shift+F):
  Slides in from right. 30% width. No cold start.
  Shows:
    - Friday's latest synthesis brief
    - Last 5 knowledge entries for current domain
    - Active contradictions if any
    - Session arc: commits, deploys, health trend
  Auto-triggers:
    - Build fails: panel shows relevant knowledge entry immediately
    - Health drops: panel surfaces cause
    - Pattern detected: panel shows Friday's insight
  Reads directly from state.db. No network. No daemon dependency.
OUTPUT INTELLIGENCE:
  The terminal understands what it is displaying:
    - Rust compiler errors: E-codes highlighted, knowledge engine consulted
    - File paths: Ctrl+Click opens in $EDITOR or faelight-fm
    - URLs: Ctrl+Click opens in faelight-browser
    - HTTP status codes: colored by class (2xx green, 4xx yellow, 5xx red)
    - JSON output: auto pretty-printed when detected
    - Git diff: syntax-aware coloring beyond ANSI
    - Build timing: slow builds (>10s) flagged inline
    - Deploy output: structured display with timing
SESSION INTELLIGENCE:
  - Restores last working directory on open
  - Shows active intent in context bar immediately
  - Remembers last 20 commands per session in state.db
  - Long command notification: >30s sends faelight-notify
  - Session end: writes summary to state.db for Friday
SPLIT PANES:
  - Ctrl+Shift+H: horizontal split
  - Ctrl+Shift+V: vertical split (different from paste to avoid conflict)
  - Ctrl+Shift+Arrow: navigate between panes
  - Each pane: own working directory, own context
  - No tabs -- panes only, cleaner model
PERFORMANCE TARGETS (must beat foot):
  - Cold start:    < 50ms measured
  - Input latency: < 2ms measured
  - Scroll FPS:    165Hz sustained
  - Memory:        < 30MB idle
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BUILD PHASES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PHASE 0 -- FOUNDATION (Sessions 1-2)
The hardest phase. Must be done before anything else.
  - New Cargo.toml: wgpu + cosmic-text + smithay + vte + nix + calloop
  - Clean module structure created (all files, empty impls)
  - Wayland window opens via smithay
  - wgpu surface attached and clearing to background color
  - PTY spawns faelight-shell
  - Characters render on screen via cosmic-text glyph atlas
  - Keyboard input reaches the shell
  - Nothing else. Just: window opens, shell runs, you can type.
  Gate: type a command and see the output. That is all.
PHASE 1 -- BASELINE (Sessions 2-3)
Foot-level functionality. This proves the renderer is correct.
  - Full VTE/ANSI compliance via vte parser
  - Copy/paste flawless -- full scrollback, unicode, multiline
  - Scrollback 50k lines, smooth GPU scroll
  - Resize: zero artifacts
  - Bold, italic, dim, all attributes correct
  - Colors: all 256 + true color correct
  - Cursor: block, beam, underline, blinking
  Gate: everything foot does, faelight-term v2 does correctly.
        Run cat on a large file. Copy text beyond screen. Paste Rust code.
        All must work.
PHASE 2 -- IDENTITY (Sessions 3-4)
This is when it starts looking like faelight-term.
  - Two-line context prompt implemented
  - Faelight color palette applied
  - JetBrainsMono Nerd Font via cosmic-text -- ligatures correct
  - Powerline glyphs correct -- no missing boxes
  - Welcome screen preserved exactly
  - Ctrl+Shift+S forest status strip
  - Caret color shifts by context
  Gate: visually indistinguishable from the faelight-term you know,
        but everything actually works.
PHASE 3 -- INTELLIGENCE (Sessions 4-6)
What no other terminal does.
  - state.db connection live
  - Friday panel: Ctrl+Shift+F, slides in, reads state.db
  - Build error detection: E-code highlighted, knowledge entry shown
  - File path click: Ctrl+Click opens in $EDITOR
  - URL click: Ctrl+Click opens in faelight-browser
  - JSON auto-detection and pretty-print
  - Git diff enhanced coloring
  - Build timing inline
  - Session memory: directory and intent restored on open
  Gate: run a failing build. Friday panel shows the fix automatically.
PHASE 4 -- POWER (Sessions 6-8)
  - Split panes: H/V, navigation
  - Long command notification via faelight-notify
  - Performance targets measured and met
  - Memory profiled under 30MB idle
  Gate: cold start measured < 50ms. Input latency < 2ms.
        Split panes stable across resize and scrollback.
PHASE 5 -- VALIDATION
  - Replace foot as primary terminal
  - 14 days continuous use without regression
  - Friday panel used daily, knowledge entries accurate
  - All gates re-verified after 14 days
  Gate: foot is uninstalled. faelight-term v2 is the only terminal.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GATES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase 0 -- Foundation:
✅ New Cargo.toml with wgpu + cosmic-text locked in (2026-04-19)
✅ Clean module structure created -- 14 files, all modules compile clean (2026-04-19)
✅ Wayland window opens via smithay + SHM surface -- wgpu swap in Phase 1 (2026-04-20)
✅ PTY spawns faelight-shell correctly -- libc read/write, non-blocking (2026-04-20)
✅ Characters render via cosmic-text glyph atlas -- PhysicalGlyph + with_pixels pipeline (2026-04-20)
✅ Keyboard input reaches shell -- can type and see output (2026-04-20)
Phase 1 -- Baseline:
✅ Full VTE/ANSI compliance verified (2026-04-22)
✅ Copy/paste working -- paste via wl-paste, mouse select + copy to clipboard (2026-04-22)
✅ Scrollback 50k lines -- mouse scroll working, better than v1 (2026-04-22)
✅ Resize preserves grid content, no artifacts (2026-04-22)
✅ Bold, italic, dim correct via cosmic-text attrs (2026-04-22)
✅ cat large file works -- no corruption (2026-04-22)
Phase 2 -- Identity:
✅ Two-line context prompt -- fsh provides natively (2026-04-22)
✅ Caret color shifts -- green/red by exit code (2026-04-22)
⚠️  Font ligatures -- deferred, per-cell rendering prevents ligatures (future intent)
✅ Nerd Font glyphs working -- JetBrainsMono Nerd Font Mono (2026-04-22)
✅ Welcome screen preserved exactly (2026-04-22)
✅ Ctrl+Shift+S status strip -- multi-color, live state.db data (2026-04-22)
Phase 3 -- Intelligence:
✅ state.db connection live -- Friday panel + session memory read state.db (2026-04-22)
✅ Friday panel (Ctrl+Shift+F) -- slides in, rust-prioritized knowledge entries (2026-04-22)
✅ Build error detection -- Rust E-codes highlighted red in output (2026-04-22)
✅ File path Ctrl+Click -- opens in $EDITOR (nvim) (2026-04-22)
✅ URL Ctrl+Click -- opens in faelight-browser (2026-04-22)
✅ JSON auto-detection and pretty-print -- colored output (2026-04-22)
✅ Session memory -- restores last working directory on open (2026-04-22)
✅ Long command notification -- faelight-notify fires after >30s (2026-04-22)
Phase 4 -- Power:
⬜ Split panes -- Ctrl+Shift+H horizontal, Ctrl+Shift+B vertical
⬜ Pane navigation -- Ctrl+Shift+Arrow
⬜ Cold start under 50ms measured
⬜ Input latency under 2ms measured
⬜ Memory under 30MB idle measured
⬜ 165Hz sustained scroll verified on AMD Radeon 780M
Phase 5 -- Validation:
⬜ 14 days as primary terminal without any regression
⬜ foot uninstalled -- faelight-term v2 is the only terminal
⬜ All Phase 0-4 gates re-verified after 14 days daily use
"The terminal of 2136 knows what you are building.
It remembers what broke last time.
It surfaces the fix before you ask.
It is not a window. It is a partner.
The current term showed what was possible.
v2 shows what was always the goal.
wgpu renders the pixels.
cosmic-text shapes the words.
Friday understands the meaning.
faelight-term v2." 🌲
