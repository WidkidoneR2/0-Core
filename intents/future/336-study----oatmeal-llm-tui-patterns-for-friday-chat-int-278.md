---
id: 336
title: "Study -- Oatmeal LLM TUI patterns for Friday Chat INT-278"
status: planned
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

⬜ Oatmeal source fully studied -- architecture documented in docs/oatmeal-patterns.md
⬜ Backend trait pattern understood and documented
⬜ Session persistence pattern understood
⬜ Streaming response rendering pattern understood
⬜ Bubble UI rendering pattern understood
⬜ Slash command parser pattern understood
⬜ Friday Chat scaffolded as a ratatui TUI
⬜ Friday Chat connects to Friday's local intelligence (no internet required)
⬜ /status, /intent, /decide, /why, /events slash commands working
⬜ Session persistence: friday chat sessions saved and resumable
⬜ Optional: Claude API backend when ANTHROPIC_API_KEY set in environment
⬜ Demonstrated: full conversation with Friday using Friday Chat
⬜ friday command from fsh opens Friday Chat
