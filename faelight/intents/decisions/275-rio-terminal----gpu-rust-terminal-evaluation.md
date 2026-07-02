---
id: 275
title: "Rio Terminal -- GPU Rust Terminal Evaluation"
status: decided
date: 2026-05-06
decided: 2026-05-06
verdict: not-adopted
tags: [rio, terminal, gpu, rust, evaluation, decided]
---
Rio v0.4.2 evaluated on 2026-05-06. Not adopted.
- Installed from Arch extra repo (official, signed, Orhun Parmaksız)
- Launched on Niri/Wayland -- opens correctly
- Forest colors configured -- rendered correctly
- fsh loaded as shell inside Rio
- Emoji not rendering -- box characters instead of glyphs
- `pick intent` fails -- skim TUI incompatible inside Rio
- `compare --git` fails -- alt screen issues
- Font size feels smaller than foot at same size setting
- `c` and `d` commands noticeably slower -- GPU warmup latency
- `d` output looks broken without emoji
Rio is a serious project -- 97.4% Rust, WebGPU, MIT license, active maintenance.
But it is not ready for the forest today.
faelight-term already does what the forest needs, correctly.
Rio does not handle emoji, skim TUIs, or alt screen in a way compatible with
the forest's daily workflow.
Revisit at Rio v1.0 if emoji and TUI compatibility improve.
"The forest evaluated Rio honestly.
Rio was not ready for the forest.
faelight-term remains the forest terminal." 🌲
