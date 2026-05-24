---
id: 338
title: "Study -- Terax ADE patterns for faelight-term v4 and Friday Chat convergence"
status: planned
date: 2026-05-25
tags: [study, terax, terminal, ai, ade, friday-chat, faelight-term, convergence]
---

## What Is Terax

Terax (https://terax.app) is a lightweight AI terminal (ADE -- AI Development
Environment). v0.7.1, Apache-2.0, built by one person (crynta).

Stack: Rust backend (PTY, filesystem, IPC) + React/TypeScript frontend + Tauri
+ xterm.js (WebGL renderer) + CodeMirror 6 + Vercel AI SDK.

7MB binary. 300ms cold start. No account, no telemetry, no hosted service.
BYOK (bring your own keys) or fully local via LM Studio.

Features:
- Terminal + code editor + file explorer + web preview + AI in one binary
- Voice input
- AI agents that can run commands, edit files
- Supports Claude, OpenAI, Google, Groq, Ollama, LM Studio

## Why Study It

The forest is not 100% Rust and does not need to be. What matters is
understanding what you depend on and why.

Terax represents the product vision that faelight-term v4 + Friday Chat
should converge toward: one environment where the terminal, the editor,
and the AI intelligence layer are unified rather than separate tools.

Terax is not the model to copy. It is the benchmark to surpass.

faelight-term v4 + Friday Chat, done correctly, will be:
- Smaller (pure Rust, no Tauri/webview overhead)
- Faster (wgpu GPU rendering, not xterm.js WebGL through a webview)
- More integrated (Friday knows the forest, not a generic LLM wrapper)
- More principled (local-first, no internet required, no BYOK needed)

## What To Study

1. **The ADE concept** -- terminal + editor + AI as one surface, not three apps
2. **PTY + AI integration** -- how Terax routes commands through the AI layer
3. **Voice input architecture** -- how voice feeds into the terminal workflow
4. **Agent design** -- how AI agents observe terminal state and propose actions
5. **7MB binary** -- what they include vs exclude to achieve this size
6. **300ms cold start** -- startup optimization patterns

## What We Build (After Study)

The convergence thesis for the forest:

faelight-term v4 = terminal + editor pane (helix integration) + Friday panel
Friday Chat = conversational AI layer aware of forest state
Together = the forest's ADE

One Super+Enter opens the forest ADE:
- Left: terminal running fsh
- Right: Friday Chat panel
- Bottom: status bar showing intent, health, active signals

No web tech. Pure Rust. Forest-aware. Local-first.
Faster than Terax by design.

## Gates

⬜ Terax source studied -- architecture documented in docs/terax-patterns.md
⬜ ADE concept formally defined for the forest -- what it means without web tech
⬜ PTY + AI integration pattern documented
⬜ Agent observation pattern documented -- how AI watches terminal state
⬜ faelight-term v4 design incorporates ADE convergence vision
⬜ Friday Chat design incorporates ADE convergence vision
⬜ Combined layout design documented -- terminal + Friday panel in one window
⬜ Startup time baseline measured for faelight-term v3 (comparison target)
⬜ faelight-term v4 cold start <= 300ms (matching Terax benchmark)
⬜ Forest ADE concept demonstrated -- terminal + Friday panel running together
