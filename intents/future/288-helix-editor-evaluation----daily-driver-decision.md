---
id: 288
title: "helix editor evaluation -- daily driver decision"
status: in-progress
date: 2026-05-15
type: evaluation
tags: [helix, editor, evaluation, daily-driver, faelight-term]
depends_on: [286]
---
## The Question
Is helix the forest's daily editor?
Not "is helix good" -- helix is good.
The question is: does helix fit the forest philosophy?
Understanding over convenience. Manual control over automation.
Intentional design. The forest speaks human first.

---
## What Was Proven (2026-05-15 in faelight-term)
- Helix launches and renders correctly in faelight-term ✅
- Syntax highlighting works (Rust, confirmed) ✅
- Arrow keys, hjkl movement ✅
- Normal/insert/goto modes ✅
- :w save, :q quit ✅
- Space+y yanks to Wayland clipboard (OSC 52) ✅
- wl-paste confirms clipboard reaches Wayland ✅
- Mouse disabled cleanly (no gutter artifacts) ✅
- gg top, G bottom, g goto menu ✅
- No spawn hesitation ✅

---
## What Still Needs Testing
- Rust LSP (rust-analyzer) -- does it start automatically?
- Error diagnostics -- do they render correctly?
- Multi-file workflow -- :open, buffers, pickers
- ripgrep integration -- Space+/ global search
- fzf-style file picker -- Space+f
- Git diff in gutter
- Ctrl+arrows word movement (needs kitty protocol)
- Helix theme -- Faelight Forest colors

---
## Forest Integration Plan
### Helix + Claude Code
Claude Code runs in one faelight-term.
Helix runs in another faelight-term.
No plugins. No LSP AI. Clean separation.
Claude handles architecture. Helix handles editing. Forest handles context.

### Helix Theme (faelight.toml)
~/.config/helix/themes/faelight.toml
Colors from forest DNA:
  background: #0a0f14 (Abyss Black-Blue)
  foreground: #a9dfff (Soft Ice Blue)
  primary: #00bfff (Neon Azure)
  accent: #00e0ff (Electric Cyan)
  success: #2affd5
  warning: #ffd43b
  error: #ff4c4c

### Stow Integration
Once decision is made:
  ~/.config/helix/ → managed by stow
  Part of forest dotfiles
  Versioned with 0-core

---
## Gates
- [ ] rust-analyzer starts in helix for Rust files
- [ ] Error diagnostics render correctly
- [ ] Space+f file picker works
- [ ] Space+/ global search works
- [ ] Faelight Forest theme applied
- [ ] Helix config stowed into 0-core
- [ ] 1 week daily driving helix exclusively
- [ ] Claude Code + helix workflow validated
- [ ] Decision documented: adopt or reject

---
## The Standard
Helix earns its place by being used, not evaluated.
Start using it. Every file edit this session: helix.
If it gets in the way, that is the answer.
If it disappears and lets you think, that is the answer.

"The best editor is the one you stop thinking about." 🌲
