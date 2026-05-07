---
id: 275
title: "Rio Terminal -- GPU Rust Terminal Evaluation"
status: planned
date: 2026-05-06
tags: [rio, terminal, gpu, rust, evaluation, study, faelight-term, wgpu]
---
The forest built its own terminal: faelight-term.
It works. It is fast. It is ratatui-based with VTE rendering.
Rio is a GPU-accelerated terminal emulator written entirely in Rust.
It uses wgpu for rendering -- the same GPU pipeline considered for faelight-bar.
It is worth understanding what Rio does and how it does it.
Not to replace faelight-term.
To learn from it.
To understand what GPU-accelerated terminal rendering looks like in production Rust.
To inform faelight-term Phase 4 (wgpu) if that path is ever taken.
---
WHAT RIO IS
Rio is a terminal emulator built from scratch in Rust.
GPU rendering via wgpu -- no VTE, no PTY wrappers from C libraries.
Its own ANSI parser, its own glyph renderer, its own font system.
Sugarloaf: Rio's own rendering engine, designed for terminal typography.
Ligature support. Pixel-perfect font rendering. True GPU acceleration.
It is what faelight-term could become in Phase 4.
Rio GitHub: https://raphamorim.io/rio/
License: MIT
---
WHAT WE WANT TO LEARN
1. How does Rio handle the PTY? Compare to faelight-term's approach.
2. How does Sugarloaf render glyphs? What can faelight-term learn?
3. What is the startup time vs faelight-term?
4. Does Rio work cleanly on Niri/Wayland?
5. What is the memory footprint at idle vs faelight-term?
6. Does Rio support the features faelight-term needs: alt screen, resize, colors?
7. What would a GPU rendering phase for faelight-term actually require?
---
EVALUATION CRITERIA
Performance:
  Start time: hyperfine "rio" vs "faelight-term" -- cold start comparison
  Memory: ps aux idle comparison
  Rendering: scrollback speed, large file cat performance
Features:
  Wayland/Niri: does it work, does it support Wayland protocols?
  Font rendering: does HackNerdFont render correctly?
  Color: full 24-bit truecolor, forest palette correct?
  Alt screen: fsh, vim, faelight-fm all require alt screen
  Resize: terminal resize handled gracefully?
Code quality:
  Read Sugarloaf source -- understand the GPU rendering approach
  Read PTY handling -- compare architecture to faelight-term
  Read wgpu integration -- understand what Phase 4 would require
---
WHAT THIS IS NOT
This is not a decision to replace faelight-term.
faelight-term is the forest terminal. We built it. We own every line.
Rio is a reference implementation to learn from.
If Rio reveals a technique that improves faelight-term -- adopt the technique.
If Rio reveals that wgpu is the right next step -- that becomes Phase 4.
If Rio reveals that VTE + ratatui is the correct long-term approach -- confirmed.
The forest learns from everything. It replaces nothing without reason.
---
GATES
[ ] Rio installed and running on Niri
[ ] hyperfine comparison: Rio vs faelight-term cold start
[ ] Memory comparison at idle
[ ] HackNerdFont renders correctly in Rio
[ ] Forest color palette verified in Rio
[ ] Alt screen tested: fsh, vim, faelight-fm all work correctly
[ ] Sugarloaf source read -- GPU rendering approach understood
[ ] PTY handling in Rio compared to faelight-term architecture
[ ] Key findings documented in this intent
[ ] Decision recorded: what (if anything) faelight-term should learn from Rio
Final gate:
[ ] This intent moves to decisions/ with a clear conclusion:
    adopt wgpu for faelight-term Phase 4 / confirm VTE path / defer indefinitely
"The forest does not adopt tools blindly.
It studies them.
It understands them.
Then it decides." 🌲
