---
id: 193
date: 2026-04-04
type: planned
title: "Tool Retirement Sprint — The Forest Prunes Itself"
status: planned
tags: [retirement, cleanup, tools, registry, pruning, v13-prep]
---
The forest has grown to 55 tools. Not all of them deserve to stay.
A tool that is never used is not neutral — it is dead weight.
It consumes build time, confuses the registry, and dilutes the signal.
The forest's integrity requires that every deployed tool earns its place.
This is not deletion. This is pruning. The tree grows stronger.
A tool is a retirement candidate if it meets 2+ of:
- 0 actual usage in 30 days (reality-check confirms)
- Functionality fully superseded by a core command or another tool
- Was flagged for retirement in a previous audit (DEC-005, INT-163)
- Expected usage marked "rare" with no critical use case documented
These tools have 0 actual usage AND are superseded:
- Superseded by: core archaeology, core story, core why
- Last meaningful use: pre-v9 (before core intent engine)
- Action: retire + remove from deploy
- Superseded by: niri native workspace management, core events
- Last meaningful use: pre-Niri migration
- Action: retire + remove from deploy
- Superseded by: core doctor entropy
- Last meaningful use: pre-v10 doctor audit
- Action: retire + remove from deploy
- Superseded by: core doctor bins, deploy --verify
- Last meaningful use: pre-v10
- Action: retire + remove from deploy
- Superseded by: fsh find builtin, fd alias, core find
- Never reached production daily use
- Action: retire + remove from deploy
- Expected: rare, Actual: 0
- Functionality absorbed by: faelight-contextd (INT-185)
- Action: retire + remove from deploy
These need a conversation before retiring:
- Expected: medium, Actual: 0
- Was this ever used? Check git log before retiring.
- Expected: medium, Actual: 0
- Hook system — does anything depend on it?
- Expected: low, Actual: 0
- Superseded by core intent? Verify before retiring.
For each tool:
1. core registry retire <tool>
2. Remove from scripts/ (undeploy)
3. Mark source directory as archived (not deleted — history preserved)
4. Update registry TOML
5. Verify health check still passes after each retirement
⬜ archaeology-0-core retired and undeployed
⬜ workspace-view retired and undeployed
⬜ entropy-check retired and undeployed
⬜ bin-doctor retired and undeployed
⬜ faelight-search retired and undeployed
⬜ faelight-daemon retired and undeployed
⬜ faelight-gen decision made (retire or keep with justification)
⬜ faelight-hooks decision made (retire or keep with justification)
⬜ faelight-intent decision made (retire or keep with justification)
⬜ Health check passes after all retirements
⬜ Registry reality-check shows clean tool list
⬜ Deploy count reduced — fewer tools, faster builds
"A forest that cannot shed its dead branches cannot grow new ones.
Retirement is not failure — it is evolution.
The tools that remain are the tools that matter." 🌲
