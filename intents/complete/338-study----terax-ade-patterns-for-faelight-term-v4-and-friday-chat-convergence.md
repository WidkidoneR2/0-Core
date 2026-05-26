---
id: 338
title: "Study -- Terax ADE patterns for faelight-term v4 and Friday Chat convergence"
status: complete
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

✅ Terax source studied -- findings documented in intent file 2026-05-26
✅ ADE defined: faelight-term v3 + Zellij layout + Friday Chat -- no webview 2026-05-26
✅ portable-pty (same as faelight-shell), flush coalescing, AgentDetector OSC pattern 2026-05-26
✅ OSC 777 sequences, TERAX_TERMINAL env var, working/attention/finished signals 2026-05-26
✅ Decision: faelight-term v3 + Zellij layout (not v4 from scratch) -- INT-346 2026-05-26
✅ Friday Chat as Zellij pane -- INT-345 + INT-346 2026-05-26
✅ Zellij layout: left=fsh in faelight-term, right=Friday Chat pane 2026-05-26
✅ faelight-term v3 ~50ms vs Terax 300ms -- forest already wins 2026-05-26
⏸ v4 cold start target -- deferred: staying with v3 on NixOS -- approved by: christian 2026-05-26
⏸ ADE demonstration -- deferred: INT-346 -- approved by: christian 2026-05-26

## Study Findings (2026-05-26)

### Terax Overview
5,100+ stars, Apache-2.0, one developer (crynta), actively maintained (May 2026)
Stack: Tauri 2 + Rust (portable-pty) + React 19 + xterm.js WebGL + CodeMirror 6 + Vercel AI SDK
Total Rust backend: ~5,000 lines. Total TypeScript frontend: much larger.

### THE KEY FINDING: portable-pty
Terax uses `portable-pty` -- THE SAME CRATE faelight-shell already uses.
The forest already has the PTY layer. This is not a gap to fill.
faelight-term v3 is ALREADY more sophisticated than Terax's Rust backend.

### Pattern 1: Flush Coalescing (directly adoptable by faelight-term)
```rust
const FLUSH_COALESCE: Duration = Duration::from_millis(4);  // batch window
const FLUSH_MAX_IDLE: Duration = Duration::from_millis(50); // safety timeout
const READ_BUF: usize = 16 * 1024;                          // 16KB read buffer
const MAX_PENDING: usize = 4 * 1024 * 1024;                 // 4MB overflow guard
```
When buffer overflows, write ESC c (hard reset) + notice instead of corrupting state.
This prevents partial CSI sequence splits that corrupt terminal rendering.
ADOPT: faelight-term v3 should use this coalescing pattern.

### Pattern 2: TERAX.md = Project Memory
Terax loads TERAX.md from workspace root as agent context (like AGENTS.md/CLAUDE.md).
Forest already has something BETTER:
- focus.toml: active intent (real-time)
- state.db:friday_knowledge: 369 structured facts
- state.db:friday_decisions: past decisions with outcomes
- state.db:friday_patterns: 14 behavioral patterns with confidence
- intent-tagged shell history: what was actually done
The forest's project memory is queryable, structured, and always current.
TERAX.md is a flat markdown file. state.db is a database. Forest wins.

### Pattern 3: AgentDetector (OSC sequences for AI state)
Terax uses OSC 777 sequences: `notify;Terax;working|attention|finished`
Sets TERAX_TERMINAL env var so shell hooks can signal AI state.
Hook events: UserPromptSubmit→working, Notification→attention, Stop→finished
Forest equivalent: Friday signals already exist (confidence-gated suggestions in bar).
The `FAELIGHT_TERMINAL` env var could signal Friday state to fsh.

### Pattern 4: Approval Gating Pattern
AI agents can run bash commands but with approval gating.
Matches exactly INT-186 (Delegation Engine):
- reversible=false → always confirm
- destructive verbs → confirm + reason
- confidence < threshold → confirm
The INT-186 trust contract pattern is the right implementation.

### Pattern 5: DaFilter (Device Attributes filtering)
Filters DA1/DA2 queries from terminal -- intercepts capability queries.
Important for terminal compatibility with tools that probe capabilities.
faelight-term v3 should handle DA responses properly.

### Pattern 6: Security Lesson (OSC 8888 removed)
Critical vulnerability: OSC 8888 allowed PTY to open arbitrary local files.
SSH server (or any process) could exploit this to expose secrets.
Forest lesson: NEVER trust escape sequences from remote processes.
faelight-term must validate all OSC sequences before acting on them.

### THE ADE DECISION

Terax proves: the ADE concept is RIGHT but web tech is the WRONG foundation.
Terax's 7MB includes a webview (GTK + WebKit) -- that is NOT lightweight.
faelight-term v3 (wgpu + cosmic-text): 14MB RSS, instant startup, pure Rust.

The forest ADE verdict:

OPTION A: faelight-term v4 ADE (full custom)
Pros: 100% forest, tightest integration
Cons: multi-pane is complex, 3-4 month build, risk during NixOS migration

OPTION B: Rio + Zellij + fsh (adopted)
Pros: mature pane management, proven on real hardware, immediate
Cons: two dependencies, less forest-native
Stack: Rio (GPU terminal) + Zellij (multiplexer) + fsh (shell) + Friday Chat (AI pane)
Zellij layout file defines: left=fsh terminal, right=Friday Chat, bottom=status

OPTION C (RECOMMENDED): faelight-term v3 + Zellij layout
Keep faelight-term v3 as the primary terminal.
Use Zellij ONLY for layout management (panes/sessions).
Friday Chat is a Zellij pane (native terminal TUI, not a separate app).
fsh already runs inside faelight-term v3.
This is the lowest-risk path to the ADE vision.

### New Intent: INT-346 -- Forest ADE
The convergence intent: faelight-term v3 + Zellij layout + Friday Chat = Forest ADE.

### Startup Time Baseline
faelight-term v3: ~50ms measured (vs Terax's 300ms claim with webview overhead)
Forest already wins on startup time.
