---
id: 283
title: "faelight-docs generates COMMAND-GUIDE -- auto-generated from core domains, never stale"
status: complete
date: 2026-05-07
tags: [faelight-docs, command-guide, automation, docs, core, generation]
depends_on: [282]
---
The COMMAND-GUIDE.md is hand-maintained.
Every time core adds a domain, the guide gets stale.
Every time a command is removed, the guide lies.
This intent makes the guide self-maintaining.
faelight-docs reads the core binary and generates COMMAND-GUIDE.md.
The guide is never stale because it cannot be stale.
It reflects exactly what core can do today.
---
HOW IT WORKS
faelight-docs runs: core --help (top level)
For each subcommand: core <subcommand> --help
Parses the clap-generated help output.
Generates structured markdown from the parsed output.
Writes to docs/COMMAND-GUIDE.md.
The guide structure mirrors core domains:
  core doctor -- health monitoring
  core friday -- intelligence layer
  core intent -- ledger v2
  core genealogy -- family tree
  core predict -- anticipation
  ...and so on for all 56+ domains.
Each domain section includes:
  Domain name and description (from --help)
  All subcommands with their descriptions
  Example usage where meaningful
  Current as of this version (auto-populated)
---
INTEGRATION
faelight-docs cmd_command_guide():
  Runs core --help to get domain list
  For each domain, runs core <domain> --help
  Parses output into structured sections
  Generates markdown with forest formatting
  Writes docs/COMMAND-GUIDE.md
  Records the generation in faelight-docs log
New command: docs command-guide
  Generates the guide on demand
  Dry-run mode shows what would change
  Integrated into release pipeline --
    every release regenerates the guide automatically
Release pipeline hook:
  faelight-release runs docs command-guide before bumping version
  Guide is always current on every release
---
COMMAND-GUIDE FORMAT
  Version: 13.x.x (auto-populated)
  Generated: 2026-05-07 (auto-populated)
  Domains: 56 (auto-counted)
  23-check health monitoring with forecast and early warning.
    core doctor run          -- full health check
    core doctor quick        -- critical checks only
    core doctor forecast     -- 24h and 7d health prediction
    core doctor trend        -- health trend over time
  Friday intelligence layer -- observe, suggest, recommend, challenge.
    core friday ask <q>      -- ask Friday a question
    core friday plan         -- session-aware planning
    core friday context      -- current session buffer
    ...
---
GATES
[ ] faelight-docs has cmd_command_guide() function
[ ] Function runs core --help and parses domain list
[ ] For each domain, subcommand help is parsed
[ ] Structured markdown is generated correctly
[ ] docs command-guide command works end-to-end
[ ] Generated guide matches actual core behavior (spot-checked)
[ ] docs command-guide --dry-run shows diff without writing
[ ] Release pipeline calls docs command-guide before version bump
[ ] Generated guide replaces hand-maintained COMMAND-GUIDE.md
[ ] Guide stays current after adding a new core domain (verified)
Final gate:
[ ] Christian adds a new core command, runs docs command-guide, guide updates automatically
[ ] The guide was never touched by hand
"The guide that writes itself
cannot lie about what the forest can do.
It reflects exactly what is there.
No more. No less.
Always current.
Always honest." 🌲
