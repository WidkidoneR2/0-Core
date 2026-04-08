---
id: 210
date: 2026-04-08
type: planned
title: "Registry Stats Accuracy — Tools Count and Release Stats"
status: planned
tags: [registry, stats, tools, accuracy, faelight-release, housekeeping]
---
The forest reports inconsistent tool counts:
- `d` shows 44/44 tools deployed
- `tools` table shows 55 in registry
- faelight-release stats show "55 deployed"
- Actual deployed tools: ~54
Root cause: retired tools (archaeology-0-core, bin-doctor, entropy-check,
workspace-view, faelight-intent) remain in tools.toml with deployed=false
but still count toward the registry total.
1. tools.toml — remove retired tools entirely (they are in intents/cancelled)
2. faelight-release count_tools() — count only deployed=true tools
3. doctor path resilience check — align with actual deployed count
4. state.db tools table — sync with registry after cleanup
⬜ Retired tools removed from tools.toml
⬜ faelight-release count_tools() counts deployed=true only
⬜ Tool count consistent across d, tools, faelight-release
⬜ d passes 100% after cleanup
