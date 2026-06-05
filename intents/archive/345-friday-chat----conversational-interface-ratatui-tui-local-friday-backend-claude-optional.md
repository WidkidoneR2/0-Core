---
id: 345
title: "Friday Chat -- Conversational Interface -- ratatui TUI, Local Friday Backend, Claude Optional"
status: complete
date: 2026-05-26
tags: [friday, chat, tui, ratatui, llm, conversation, backend, slash-commands]
depends_on: [336, 278, 251]
---

## Why This Intent Exists

INT-336 studied Oatmeal -- the best Rust LLM TUI in existence.
The patterns are clear. Friday Chat is not Oatmeal.
Friday Chat is Friday's voice in a conversation window.

## What Friday Chat Is

A ratatui TUI that opens when you run `friday chat` from fsh.
It talks to Friday's local intelligence first.
Claude API is optional (set ANTHROPIC_API_KEY).
┌─ Friday Chat ───────────────────────────────────────────────┐
│                                                             │
│  ╭─ Friday ──────────────────────────────────────────╮     │
│  │ Good afternoon. INT-326 is active. 4 gates remain. │     │
│  │ Health is 100%. No blocking issues detected.       │     │
│  ╰────────────────────────────────────────────────────╯     │
│                                                   ╭─ You ─╮│
│                                                   │ /intent││
│                                                   ╰────────╯│
│  ╭─ Friday ──────────────────────────────────────────╮     │
│  │ Active: INT-326 fsh Semantic Architecture          │     │
│  │ Phase 0-3 complete. Phases 4-7 pre-NixOS.         │     │
│  │ Next gate: verb taxonomy enforcement               │     │
│  ╰────────────────────────────────────────────────────╯     │
│                                                             │
│ /intent /status /decide /why /events /health /help /q      │
│ ┌─────────────────────────────────────────────────────┐    │
│ │ Type a message or /command...                        │    │
│ └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘

## Architecture (from Oatmeal patterns)

### File Structure
rust-tools/friday-chat/src/
main.rs              -- entry point
app.rs               -- AppState, event loop
ui.rs                -- ratatui layout, bubble rendering
backends/
mod.rs             -- Backend trait (from Oatmeal)
friday.rs          -- FridayBackend (local, state.db)
claude.rs          -- ClaudeBackend (optional, reqwest+SSE)
models/
message.rs         -- Message { author, text, timestamp }
session.rs         -- Session { id, timestamp, messages }
slash_command.rs   -- SlashCommand parser
event.rs           -- Event enum
action.rs          -- Action enum
services/
sessions.rs        -- state.db session persistence
bubble.rs          -- bubble rendering (Left/Right)

### Backend Trait (from Oatmeal)
```rust
#[async_trait]
pub trait Backend {
    fn name(&self) -> &str;
    async fn health_check(&self) -> Result<()>;
    async fn get_completion(
        &self,
        prompt: &str,
        context: &[Message],
        tx: &mpsc::UnboundedSender<Event>,
    ) -> Result<()>;
}
```

### FridayBackend (local, no internet)
Reads from state.db:
- friday_knowledge: facts Friday knows
- friday_patterns: behavioral patterns (confidence, frequency)
- friday_decisions: past decisions and outcomes
- events: recent forest activity
- shell_history (intent-tagged): what was done

Responds with:
- Current intent status
- Health summary
- Decision history
- Pattern analysis
- Proactive suggestions based on patterns

### ClaudeBackend (optional)
Uses Anthropic API with streaming (SSE).
System prompt: "You are Friday, the forest intelligence for Faelight Forest.
You know about: [current intent, recent events, health, patterns]"
Only available when ANTHROPIC_API_KEY is set.

### Two-Channel Architecture (from Oatmeal)
User types → Action channel → backend.get_completion()
Backend streams → Event channel → UI renders token by token

### Session Persistence
```sql
CREATE TABLE friday_chat_sessions (
    id TEXT PRIMARY KEY,
    timestamp INTEGER,
    backend TEXT,
    messages_json TEXT,  -- Vec<Message> as JSON
    context TEXT         -- conversation context for backend
);
```

### Forest Slash Commands
/status    -- core status summary
/intent    -- current active intent + open gates
/decide    -- record a decision (calls core friday decide)
/why       -- query decision record
/events    -- recent event bus activity (last 10)
/health    -- run fsh doctor summary
/rewind    -- show last 5 time-travel snapshots
/patterns  -- show Friday's top patterns with confidence
/sessions  -- list previous chat sessions
/help      -- available commands
/q /quit   -- exit

### UI Layout
```rust
Layout::vertical([
    Constraint::Length(1),   // status bar: intent + health + backend
    Constraint::Min(1),      // chat bubbles
    Constraint::Max(5),      -- input textarea (grows with content)
])
```

## Gates
- [x] Phase 1: friday-chat crate scaffolded, compiles -- delivered by INT-278 2026-05-27
- [x] Phase 2: FridayBackend reads friday_knowledge, friday_patterns, events, intent_commits -- INT-278 2026-05-27
- [x] Phase 3: ratatui TUI renders Friday + User bubbles with forest palette -- INT-278 2026-05-27
- [x] Phase 4: /status /intent /events /patterns /facts /why /recall /trace /explain /where /show all working -- INT-278+INT-279 2026-05-27
- [x] Phase 5: Conversations logged to events table (domain: friday, action: chat_message) -- INT-278 2026-05-27
- [x] Phase 6: Local intelligence only -- Claude backend deferred to future enhancement -- approved by: christian 2026-05-27
- [x] Phase 7: friday chat wired in fsh main.rs before alias expansion -- INT-278 2026-05-27
- [x] Phase 8: Header shows active intent + health + /help hint -- INT-278 2026-05-27
- [x] Final: Full conversation demonstrated -- /status /patterns /recall deploy all working with real state.db data -- INT-278 2026-05-27

## Note
MIT license for Oatmeal. Friday Chat is clean-room using same ratatui primitives.
No Oatmeal code copied. Patterns studied: Backend trait, two-channel arch, slash commands.

---
"Friday does not need the internet to know the forest.
The forest is the database.
The patterns are the knowledge.
The conversation is the interface." 🌲
