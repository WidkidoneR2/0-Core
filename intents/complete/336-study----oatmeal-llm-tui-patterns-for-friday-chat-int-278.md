---
id: 336
title: "Study -- Oatmeal LLM TUI patterns for Friday Chat INT-278"
status: complete
date: 2026-05-25
tags: [study, oatmeal, llm, tui, friday-chat, ratatui, conversation]
---

## What Is Oatmeal

Oatmeal (https://github.com/dustinblackman/oatmeal) is a Rust TUI application
for chatting with LLMs. 754 stars, 257 commits, last release v0.13.0 March 2024.
Supports Claude, OpenAI, Ollama, Gemini, LangChain backends.
MIT license.

Key features:
- Slash commands (/model, /append, /replace, /copy)
- Fancy chat bubbles with syntax highlighting
- Session persistence -- all chats saved and resumable
- Editor integration (Neovim, clipboard)
- Backend agnostic -- swap between LLMs without changing workflow

## Why Study It

Friday Chat (INT-278) is the intent for making Friday conversational.
Oatmeal is the most complete Rust TUI implementation of exactly this concept.

It stopped being maintained not because the idea is wrong but because
the maintainer moved on. The codebase is clean, well-structured Rust.

We study it to understand:
1. How it renders conversation bubbles in ratatui
2. How it handles streaming LLM responses
3. How it manages slash command parsing
4. How it persists and resumes sessions
5. How the Backend trait is designed (agnostic to model provider)

## What We Build (After Study)

Friday Chat is NOT a general LLM TUI. It is Friday's conversational layer.
The difference:
- Oatmeal talks to any LLM
- Friday Chat talks to Friday (local, no internet) OR optionally to Claude API
- Friday Chat knows about the forest -- intents, events, decisions
- Friday Chat is available via `friday` command from fsh
- Friday Chat uses forest vocabulary, not generic chat interface

The slash commands map to forest concepts:
  /status    -- show core status in chat
  /intent    -- show current intent
  /decide    -- record a decision (calls core friday decide)
  /why       -- query decision record
  /events    -- show recent event bus activity

## Key Patterns To Borrow

- Backend trait design: clean abstraction over model providers
- Session persistence: YAML-based session files with metadata
- Streaming response rendering: token-by-token display
- Bubble UI: user vs assistant visual distinction
- Slash command parser: clean separation of command vs content

## Gates

✅ Oatmeal source fully studied -- findings documented in intent file 2026-05-26
✅ Backend trait: async_trait, streaming via mpsc channel, done bool, context string 2026-05-26
✅ Session: id+timestamp+state, serde YAML -- forest version uses state.db 2026-05-26
✅ Streaming: get_completion sends tokens via UnboundedSender<Event>, done=true ends stream 2026-05-26
✅ Bubble: Left/Right alignment, syntect highlighting, BubbleConfig layout constants 2026-05-26
✅ SlashCommand::parse() returns Option -- None=text, Some=command+args 2026-05-26
⏸ Friday Chat scaffold -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ FridayBackend -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ Forest slash commands -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ Session persistence in state.db -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ Claude backend -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ Demonstration -- deferred: INT-345 -- approved by: christian 2026-05-26
⏸ fsh integration -- deferred: INT-345 -- approved by: christian 2026-05-26

## Study Findings (2026-05-26)

### Oatmeal Overview
v0.13.0 (Mar 2024), 754 stars, MIT license, 95.4% Rust
Domain-driven architecture: domain/ + infrastructure/ + application/
Total: ~2,500 lines. Clean, well-tested, minimal.

### Pattern 1: Backend Trait (THE key pattern)
```rust
#[async_trait]
pub trait Backend {
    fn name(&self) -> BackendName;
    async fn health_check(&self) -> Result<()>;
    async fn list_models(&self) -> Result<Vec<String>>;
    async fn get_completion(
        &self,
        prompt: BackendPrompt,
        tx: &mpsc::UnboundedSender<Event>,
    ) -> Result<()>;
}
pub type BackendBox = Box<dyn Backend + Send + Sync>;
```
Streaming via mpsc channel -- each token is an Event sent through tx.
`done: bool` in BackendResponse signals end of stream.
`context: Option<String>` carries conversation history between turns.

Friday Chat backends:
- FridayBackend: reads state.db (patterns, knowledge, decisions) -- local, no internet
- ClaudeBackend: ANTHROPIC_API_KEY env var -- optional external

### Pattern 2: Two-Channel Architecture
User input → tx: mpsc::UnboundedSender<Action>
↓
Backend.get_completion(prompt, tx)
↓
Backend streams → rx: mpsc::UnboundedReceiver<Event>
↓
UI renders each token as it arrives
This pattern is universal -- works for:
- Friday Chat (LLM responses)
- faelight-compositor (keybind → state change → bar update)
- faelight-bar (IPC events → render)

### Pattern 3: Slash Command Parser
```rust
pub struct SlashCommand {
    command: String,
    pub args: Vec<String>,
}
impl SlashCommand {
    pub fn parse(text: &str) -> Option<SlashCommand>
    // returns None for regular text, Some for commands
}
```
Forest slash commands:
  /status   -- show core status
  /intent   -- show current intent
  /decide   -- record decision
  /why      -- query decision record
  /events   -- recent event bus activity
  /health   -- fsh doctor
  /rewind   -- show time-travel snapshots
  /help     -- available commands
  /q        -- quit

### Pattern 4: Session Persistence
```rust
pub struct Session {
    pub id: String,
    pub version: String,
    pub timestamp: String,
    pub state: State,
}
pub struct State {
    pub backend_name: String,
    pub backend_model: String,
    pub backend_context: String,
    pub messages: Vec<Message>,
}
```
Oatmeal saves to YAML files. Friday Chat stores in state.db:
friday_chat_sessions table (id, timestamp, backend, messages_json, context)

### Pattern 5: Bubble UI
- BubbleAlignment::Left (Friday), BubbleAlignment::Right (User)
- syntect for code syntax highlighting in responses
- BubbleConfig { bubble_padding: 8, outer_padding_percentage: 0.04 }
- Minimum width check before rendering (graceful degradation)
- codeblock_counter for numbered code blocks (/copy 1, /copy 2..5)

### Pattern 6: Layout
```rust
Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![
        Constraint::Min(1),          // chat bubbles -- fills available space
        Constraint::Max(textarea_len), // input -- grows with content
    ])
```
Simple two-panel vertical split. Forest version adds:
- Ctrl+Shift+F toggle (already in faelight-term)
- Status bar at top (active intent, Friday confidence)

### Claude Backend Pattern
reqwest + SSE streaming:
- `stream: true` in CompletionRequest
- Reads SSE stream line by line with AsyncBufReadExt
- Parses JSON delta events
- Sends each token as Event through tx channel
This is exactly how Friday Chat's Claude backend would work.

### What Friday Chat is NOT
- Not a general LLM TUI (Oatmeal already does that)
- Not for browsing the internet
- Not for random questions

### What Friday Chat IS
- Friday's conversational interface
- Knows about the forest: intents, events, decisions, health
- FridayBackend is purely local -- no API key needed
- Slash commands map to forest concepts not editor concepts
- Sessions stored in state.db not in files
- Available via `friday chat` from fsh (already registered)

### New Intent: INT-345 Friday Chat
See INT-345 for the detailed build plan.

## Split Pane Deferral (approved by: christian 2026-05-27)
The original intent specified "friday chat command opens right split pane in faelight-term".
What shipped: standalone ratatui TUI launched via `friday chat` from fsh. Fully working.
The split pane integration is formally deferred to INT-346 (Forest ADE).
Both will coexist:
- Standalone: `friday chat` opens full-screen TUI in any terminal
- Split pane: INT-346 with Zellij layout -- left=fsh terminal, right=Friday Chat pane
- Compositor split: when faelight-compositor v3 (Pinnacle) is built, native split surfaces
This is not a shortcut. This is two valid delivery modes.
The standalone ships first. The split pane ships with the ADE.
