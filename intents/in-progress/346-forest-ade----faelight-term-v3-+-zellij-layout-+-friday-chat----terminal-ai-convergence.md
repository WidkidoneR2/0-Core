---
id: 346
title: "Forest ADE -- faelight-term v3 + Zellij Layout + Friday Chat -- Terminal-AI Convergence"
status: in-progress
date: 2026-05-26
tags: [ade, faelight-term, zellij, friday-chat, terminal, convergence, layout]
depends_on: [338, 345]
---

## The ADE Vision

One Super+Enter. One environment. Terminal + AI + Forest intelligence.
┌─ Faelight Forest ADE ───────────────────────────────────────────────┐
│ ┌─ fsh terminal ─────────────────┐ ┌─ Friday Chat ───────────────┐ │
│ │                                 │ │                              │ │
│ │  ~/0-core (main)                │ │  ▸ INT-346 active            │ │
│ │  → 100%                         │ │  Health: 100% · 14 patterns  │ │
│ │  fsh ❯ _                        │ │                              │ │
│ │                                 │ │  You: /intent                │ │
│ │                                 │ │                              │ │
│ │                                 │ │  Friday: Building Forest ADE │ │
│ │                                 │ │  faelight-term v3 + Zellij   │ │
│ │                                 │ │  + Friday Chat. 3 intents    │ │
│ │                                 │ │  depend on this. Next gate:  │ │
│ │                                 │ │  Zellij layout config.       │ │
│ └─────────────────────────────────┘ └──────────────────────────────┘ │
│ 🔒 INT-346: Forest ADE         100%  ·  14 patterns  ·  19:42        │
└─────────────────────────────────────────────────────────────────────┘

## Why This Approach (from INT-338 study)

Terax (the benchmark) uses webview + xterm.js. Result: 300ms cold start, GTK overhead.
faelight-term v3 uses wgpu + cosmic-text. Result: ~50ms cold start, pure Rust.

Building multi-pane from scratch = 3-4 months.
Zellij exists, is mature, runs native terminals, handles sessions/panes/tabs.
Forest ADE = faelight-term v3 (rendering) + Zellij (layout) + Friday Chat (AI pane).

## Architecture

### Component 1: faelight-term v3 (existing)
- GPU-accelerated with wgpu + cosmic-text
- 14MB RSS, ~50ms startup
- Friday panel already exists (Ctrl+Shift+F toggles it)
- fsh runs inside it as default shell

### Component 2: Zellij Layout
Zellij layout file defines the ADE:
```kdl
layout {
    tab name="Forest ADE" {
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
        pane split_direction="vertical" {
            pane name="Terminal" size="65%" {
                command "faelight-term"
            }
            pane name="Friday" size="35%" {
                command "friday"
                args "chat"
            }
        }
    }
}
```

### Component 3: Friday Chat (INT-345)
ratatui TUI, runs in right pane.
Knows about forest state via state.db.
FridayBackend (local) + optional ClaudeBackend.

### Flush Coalescing (from Terax, adopt in faelight-term v3)
```rust
const FLUSH_COALESCE: Duration = Duration::from_millis(4);
const FLUSH_MAX_IDLE: Duration = Duration::from_millis(50);
const READ_BUF: usize = 16 * 1024;
const MAX_PENDING: usize = 4 * 1024 * 1024;
// On overflow: write ESC c + notice, not corrupted data
```

### FAELIGHT_TERMINAL env var
When running inside the ADE, fsh sets `FAELIGHT_TERMINAL=1`.
Friday Chat can detect this and show terminal-aware suggestions.
Mirrors Terax's `TERAX_TERMINAL` pattern.

### Agent Approval Gating (from Terax, maps to INT-186)
When Friday suggests a command:
- confidence >= 0.9 + reversible=true → execute with note
- confidence >= 0.7 → show plan, confirm
- destructive verb → always confirm + reason
- confidence < 0.5 → don't suggest execution

## Architecture Decision (2026-05-29)
Zellij approach abandoned -- Zellij uses its own terminal renderer, not faelight-term.
Alacritty selected as primary terminal (better rendering than foot and faelight-term v3).
faelight-ade v1 will be built as a single Rust binary:
  ratatui -- layout (left PTY pane + right Friday Chat pane)
  portable-pty -- real shell embedding with PTY
  crossterm -- terminal input/event handling
  tokio -- async streaming (PTY output + Friday state.db)
  friday-chat -- right pane already exists, reuse directly

## Gates
- [x] Phase 1: faelight-ade crate scaffolded, compiles, launches standalone 2026-05-29
- [x] Phase 2: fsh runs via portable-pty with full ANSI color parsing 2026-05-29
- [x] Phase 3: Friday right pane reads state.db -- /status /patterns /why working 2026-05-29
- [x] Phase 4: FAELIGHT_ADE=1 set on launch, friday-chat detects it 2026-05-29
- [~] Phase 5: PTY output streamed, Friday detects error/warning lines -- full command awareness deferred to INT-320 Friday v3
- [ ] Phase 6: faelight-bar ADE indicator -- not built yet
- [x] Phase 7: Mod+Alt+Return launches faelight-ade directly -- no Alacritty wrapper 2026-05-29
- [~] Final: ADE ships and works -- daily driver period begins 2026-05-29. Graydon Hoare saw it working.

## Note
This does NOT require building faelight-term v4 from scratch.
faelight-term v3 is already excellent. Zellij handles the pane management.
The differentiator is Friday Chat -- the intelligence layer.
That is what Terax cannot match.

---
"Terax has the terminal.
The forest has the mind.
One understands commands.
The other understands intent.
The ADE is where they converge." 🌲
