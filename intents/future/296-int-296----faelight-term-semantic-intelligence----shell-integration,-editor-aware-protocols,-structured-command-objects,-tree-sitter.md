---
id: 296
title: "faelight-term semantic intelligence -- shell integration, editor-aware protocols, structured command objects, tree-sitter"
status: planned
date: 2026-05-12
tags: [faelight-term, semantic, tree-sitter, shell-integration, editor-aware, command-objects, friday, intelligence, warp, kitty, ghostty]
priority: high
depends_on: [292, 261, 286]
---

The terminal and the editor are converging.
Most developers have not noticed yet.
Faelight Forest will be ahead of the curve.

faelight-term v3 is the foundation.
INT-296 is the intelligence layer on top of it.
This intent turns a terminal into a semantic surface.

---

THE VISION

Today a terminal is a dumb pipe:
  PTY in -> VT100 parser -> character grid -> render

Tomorrow (faelight-term with INT-296):
  PTY in -> semantic parser -> command objects -> state.db
                            -> tree-sitter highlight -> render
                            -> Friday context -> suggestion
                            -> structured selection -> copy

The terminal knows what it is displaying.
Not just characters. Structured meaning.

---

FEATURE 1: SEMANTIC SHELL INTEGRATION

What it is:
  The terminal knows where commands begin and end.
  Every command has a structured boundary in the buffer.
  Output belongs to the command that produced it.

How it works:
  fsh emits OSC sequences at command boundaries:
    OSC 133 ; A ST  -- prompt start
    OSC 133 ; B ST  -- command start (after user types)
    OSC 133 ; C ST  -- command output start
    OSC 133 ; D ; <exit_code> ST  -- command end
  
  faelight-term v3 parses these OSC sequences.
  Each command becomes a CommandBlock in state.db:
    struct CommandBlock {
        id: u64,
        session_id: String,
        prompt: String,
        command: String,
        output: String,
        exit_code: i32,
        duration_ms: u64,
        working_dir: String,
        timestamp: DateTime,
        line_start: usize,
        line_end: usize,
    }

Benefits:
  Click anywhere in output -> selects entire command block
  Scroll to previous command with keyboard shortcut
  Friday can see every command + output as structured data
  Copy a command block = copy command + output together
  Replay any previous command block

Study sources:
  kitty shell integration (OSC 133)
  iTerm2 shell integration protocol
  Warp terminal's block model
  ghostty's shell integration docs

---

FEATURE 2: EDITOR-AWARE PROTOCOLS

What it is:
  When an editor (nvim, evil-helix) is running inside the terminal,
  the terminal changes its behavior to support the editor.

How it works:
  Editors emit a mode signal via OSC or DCS sequences.
  faelight-term detects editor mode and:
    - Enables pass-through for all key sequences (no terminal intercepts)
    - Disables mouse selection (editor owns the mouse)
    - Adjusts cursor rendering (block/beam/underline per editor mode)
    - Enables synchronized rendering (no flicker on screen clears)
    - Supports kitty keyboard protocol (full modifier key disambiguation)

  When editor exits:
    - Terminal mode restored automatically
    - Cursor returns to terminal style
    - Mouse selection re-enabled

Specific protocols to implement:
  Kitty keyboard protocol -- full key encoding with modifiers
    Every key sends: key + modifiers + action (press/repeat/release)
    No more ambiguity between Escape and Alt+key
    evil-helix supports this natively
  
  Synchronized output (DEC 2026):
    Editor wraps screen updates in begin/end markers
    Terminal only renders complete frames
    Zero flicker during large screen redraws
  
  In-band resize notification (DEC 2048):
    Terminal tells editor about resize via escape sequence
    Editor reflows immediately without SIGWINCH polling

---

FEATURE 3: STRUCTURED COMMAND OBJECTS

What it is:
  Every command run in faelight-term becomes a persistent object.
  Not just history text. A structured record in state.db.

Schema:
  CREATE TABLE term_commands (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    sequence_num INTEGER,  -- command # in this session
    timestamp TEXT,
    working_dir TEXT,
    command TEXT,          -- raw command text
    command_parsed TEXT,   -- JSON: {binary, args, flags}
    exit_code INTEGER,
    duration_ms INTEGER,
    output_lines INTEGER,
    output_preview TEXT,   -- first 500 chars of output
    friday_context TEXT,   -- what Friday thinks about this command
    intent_id INTEGER      -- which forest intent this relates to
  );

Benefits:
  Friday can query: "what commands took longest this week?"
  Friday can query: "what commands failed most often?"
  fsh can recall: "show me the output of the deploy from 2 days ago"
  Atuin integration: term_commands feeds atuin's history
  Pattern detection: Friday sees command sequences, not just commands

Forest integration:
  When a command matches a known pattern (Friday patterns):
    Friday annotates the command object
    Suggests optimizations or alternatives
  When a deploy command runs:
    Links to deploy record in state.db
  When a git command runs:
    Links to commit record

---

FEATURE 4: TREE-SITTER POWERED TERMINAL

What it is:
  Syntax highlighting in the terminal itself.
  Not just in the editor. In the terminal output.

Use cases:
  cat src/main.rs    -> Rust syntax highlighting
  cat config.toml    -> TOML highlighting
  core intent show X -> highlighted intent markdown
  json output        -> colored JSON (like jq but in the terminal)
  diff output        -> enhanced diff with tree-sitter context

How it works:
  When output is written to the terminal:
    Detect content type (file extension, shebang, content sniffing)
    Run tree-sitter parser for detected language
    Map syntax nodes to ANSI color sequences
    Write highlighted output to PTY

  Languages to support first:
    Rust (primary)
    TOML (config files)
    JSON (API output, state.db queries)
    Markdown (intent files)
    Diff (git output)
    Shell (scripts)

Integration with cosmic-text:
  cosmic-text already handles complex Unicode
  Add per-span color from tree-sitter node type
  No additional rendering cost -- same glyphon spans

Study sources:
  helix editor tree-sitter integration (Rust, same crate)
  bat (syntax highlighting for terminal output) -- study the approach
  tree-sitter-highlight crate

---

FEATURE 5: PERSISTENT COMMAND OBJECTS + FRIDAY REASONING

What it is:
  Friday can see every command you have ever run.
  Not just patterns. The actual structured history.
  Friday reasons about your workflow at the command level.

Friday capabilities with command objects:
  "You always run cargo build before deploy -- want me to chain them?"
  "This command failed 3 times this week with the same error"
  "You ran d 47 times today -- health obsession detected"
  "Last time you ran this command in this directory it took 3 minutes"
  "You haven't committed in 2 hours -- you usually commit every 45 min"

Friday annotations on commands:
  Every command object gets a Friday annotation:
    risk_level: low/medium/high/critical
    pattern_match: which known pattern this triggers
    suggestion: what Friday would recommend instead
    anomaly: true if this is unusual for this context

Command replay:
  From fsh: replay cmd:2587
  Terminal re-runs the command with the same context
  Friday compares new output to stored output
  Flags any differences as potential issues

---

FEATURE 6: AUTO-SCROLL SELECTION

What it is:
  When dragging a selection beyond the visible viewport,
  the viewport scrolls automatically to follow the drag.

How it works:
  In pointer_frame Motion handler:
    if mouse_y < 0 {
        // dragging above viewport
        scroll up by abs(mouse_y) / CELL_H lines
        extend selection in global coordinates
    }
    if mouse_y > window_height {
        // dragging below viewport
        scroll down
        extend selection in global coordinates
    }

  Selection stored in GLOBAL coordinates (viewport_offset + mouse_y)
  NOT viewport-relative coordinates

  This is the fix for the current copy-beyond-screen bug.

---

FEATURE 7: ATUIN INTEGRATION

What it is:
  Atuin is a shell history replacement with search and sync.
  faelight-term feeds command objects directly to atuin.
  atuin feeds history back to fsh.

How it works:
  faelight-term writes to atuin's history database directly
  OR uses atuin's import format
  fsh queries atuin for history (Ctrl+R = atuin search TUI)
  Friday reads from atuin history for pattern detection

Why this matters:
  Atuin has server sync -- history available across machines
  Atuin has full context: directory, exit code, duration
  Atuin + Friday = the most powerful shell history system ever built
  fsh + atuin + Friday = remember everything, understand everything

---

TOOLING NEEDED

cargo-flamegraph:
  Profile the 60fps render loop
  Find hotspots in sync_terminal (rebuilding spans every frame)
  Find hotspots in tree-sitter parsing
  Gate: render loop < 2ms per frame at 80x50 grid

cargo-bloat:
  Analyze binary size of faelight-term release build
  wgpu + cosmic-text + alacritty_terminal = large binary
  Find what's eating the most space
  Goal: release binary < 30MB

winnow (parser combinator):
  Replace manual OSC/DCS sequence parsing
  Parse tree-sitter queries
  Parse fsh natural language (INT-261 vocabulary layer)
  Parse structured terminal output

nom (alternative to winnow):
  Similar to winnow -- choose one
  nom is more mature, winnow is more ergonomic
  Recommendation: winnow for new code, matches Rust 2024 idioms

---

RELATIONSHIP TO OTHER INTENTS

INT-261 (fsh Vocabulary):
  "forest speaks human first, UNIX as fallback"
  Structured command objects make vocabulary searchable
  Friday can suggest vocabulary based on command history

INT-246 (Friday Architecture v2):
  Command objects are prime Friday training data
  Every command = a labeled decision for Friday to learn from

INT-288 (evil-helix):
  Editor-aware protocols built specifically for evil-helix
  Kitty keyboard protocol = evil-helix fully functional in v3
  tree-sitter in terminal = helix-quality highlighting everywhere

INT-292 (v3 daily driver):
  OSC 133 shell integration fixes the copy-beyond-screen bug
  Structured selection = no more viewport-relative coordinates

---

IMPLEMENTATION ORDER

Phase 0 -- Study (2 weeks, parallel with INT-292):
  Read kitty shell integration protocol docs
  Read OSC 133 specification
  Study helix tree-sitter integration
  Study Warp's block model (from their blog posts)
  Gate: architecture document complete

Phase 1 -- OSC 133 shell integration:
  fsh emits OSC 133 A/B/C/D sequences
  faelight-term v3 parses and stores CommandBlocks
  Gate: every fsh command creates a record in state.db

Phase 2 -- Structured selection:
  Selection uses global coordinates
  Copy works across any number of scrollback lines
  Auto-scroll during drag
  Gate: can select and copy 1000 lines of scrollback

Phase 3 -- Tree-sitter highlighting:
  Detect content type from terminal output
  Apply tree-sitter highlighting for Rust, TOML, JSON
  Gate: cat main.rs shows Rust syntax highlighting in v3

Phase 4 -- Kitty keyboard protocol:
  Full modifier key support
  evil-helix works perfectly inside v3
  Gate: evil-helix inside v3 with full keybinding support

Phase 5 -- Friday command intelligence:
  Friday reads command objects
  Friday suggests based on command patterns
  Gate: Friday suggests after 3 consecutive failures

Phase 6 -- Atuin integration:
  faelight-term feeds atuin history
  fsh queries atuin for search
  Gate: Ctrl+R opens atuin search inside v3

---

GATES

[ ] Phase 0: OSC 133, kitty protocol, tree-sitter, Warp model all studied
[ ] Phase 1: every fsh command creates a CommandBlock in state.db
[ ] Phase 2: copy works across 1000+ lines of scrollback
[ ] Phase 3: cat main.rs shows Rust syntax highlighting
[ ] Phase 4: evil-helix runs inside v3 with full key support
[ ] Phase 5: Friday suggests based on command object patterns
[ ] Phase 6: atuin integration complete
[ ] faelight-term v3 with INT-296 = most intelligent terminal in the forest

---

THE BIG PICTURE

Warp built a $50M company around the idea that the terminal should be smarter.
They built it with Rust and a proprietary renderer.
We are building something more ambitious:
  Open source
  Forest-aware (no other terminal knows your intent ledger)
  Friday-integrated (no other terminal has an AI that knows YOUR patterns)
  Fully owned (every line of code is understood)

The convergence of terminal + editor + AI + forest context
is something no other project is doing.

faelight-term v3 + INT-296 + evil-helix + Friday
= the terminal that thinks.

That is what we are building. 🌲
