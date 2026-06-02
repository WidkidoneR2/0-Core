---
id: 021
date: 2026-01-10
type: future
title: "faelight - Unified Binary (Core Spine)"
status: complete
tags: [rust, architecture, v6.4.0, critical]
---
## The Vision
Replace shell orchestration with a single unified binary. Shell becomes thin aliases only.

## Why
- No business logic in shell. Ever.
- Single binary to remember
- Centralized logging, config, dry-run, exit codes
- Gradual migration path

## Design
```
faelight
├── core
│   ├── lock
│   ├── unlock
│   └── status
├── profile
│   ├── list
│   └── switch <name>
├── intent
│   ├── list
│   ├── show <id>
│   └── search <term>
├── health
├── sway
│   ├── reload
│   └── status
├── git
│   ├── verify
│   └── install-hooks
└── explain
```

## Shell Becomes
```bash
alias lock-core="faelight core lock"
alias profile="faelight profile"
alias health="faelight health"
alias intent="faelight intent"
```

## Implementation Strategy
1. Phase 1: Wrap existing binaries (dot-doctor, profile, etc.)
2. Phase 2: Migrate logic inward incrementally
3. Phase 3: Deprecate standalone binaries

## Success Criteria
- [ ] Single `faelight` binary with subcommands
- [ ] All existing tools callable via faelight
- [ ] Centralized config loading
- [ ] Dry-run mode (`--dry-run`)
- [ ] JSON output mode (`--json`)
- [ ] Shell aliases updated

---
_The spine of the forest._ 🌲
