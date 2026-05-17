---
id: 318
title: "fsh v3 -- tab completion, structured output, PowerShell ideas, startup improvement"
status: planned
date: 2026-05-18
tags: fsh, shell, tab-completion, structured-output, powershell, startup, v15
depends_on: []
blocks: []
---

## Why This Intent Exists

fsh v2.1.0 is a working daily driver shell.
But anyone watching it for the first time notices immediately:
no tab completion. That one gap makes it feel incomplete
no matter how powerful everything else is.

v3 fixes the gaps and borrows the best ideas from PowerShell
-- not the syntax, but the philosophy:
commands return structured data, not just text.

---

## The Missing Pieces

### 1. Tab Completion
The biggest gap. Every shell has it. fsh does not.

Tab completion in fsh must be forest-aware:
- Complete commands from the forest vocabulary (delete, find, search, show...)
- Complete fsh builtins (deploy, friday, fsearch, patch...)
- Complete file paths (standard behavior)
- Complete intent IDs for cistart/cicomplete/intent show
- Complete core domains for core <domain> <command>
- Complete git branches for gt checkout
- Context-aware: after `deploy` → show deployable tools
- After `cistart` → show planned intents
- After `intent show` → show intent IDs

Implementation: rustyline has a Completer trait. Build a ForestCompleter
that knows the vocabulary, builtins, and can query state.db for context.

### 2. Structured Output Objects
PowerShell's best idea: commands return objects, not strings.
`Get-Process | Where-Object CPU -gt 50` works because Get-Process
returns structured data.

fsh version:
tools | where score > 80        # already works (vocabulary)
intents | where status = active # should work the same way
friday | where confidence > 0.8 # structured Friday query
deploys | where tool = core     # deployment history query

The forest already has state.db. Every command that queries it
can return structured data that pipes naturally.

### 3. Better Error Messages
PowerShell gives context on errors. fsh currently gives raw errors.

fsh v3 error format:
✗ command not found: deply
→ did you mean: deploy?
→ run 'where deploy' to see what it does

Unknown command → fuzzy match suggestion
Failed command → show exit code + hint
Permission denied → show what permission is needed

### 4. Multi-Command Handling
Complex pipelines still have edge cases. Improve:
- Nested subshells work correctly
- Here-strings (<<<) supported
- Process substitution <(cmd) supported
- Proper job control (bg, fg, jobs)

### 5. Startup Speed + Cleanliness
Current startup loads all aliases, runs db queries, shows welcome.
Target: under 50ms to prompt.
- Lazy-load aliases (load on first use)
- Async db queries (don't block prompt)
- Clean welcome screen -- version, health, Friday status only

### 6. PowerShell Ideas Worth Stealing
- `$?` works correctly (exit code of last command)
- Error stream separate from output stream
- Structured pipeline: commands can emit typed records
- Help system: `help deploy` shows full documentation
- Tab completion shows descriptions, not just names

---

## Gates

Phase 1 -- Tab completion:
- [ ] rustyline ForestCompleter implemented
- [ ] Completes fsh vocabulary words (delete, find, search, show...)
- [ ] Completes fsh builtins (deploy, friday, fsearch, cistart...)
- [ ] Completes file paths correctly
- [ ] Context-aware: after `cistart` → intent IDs from ledger
- [ ] Context-aware: after `deploy` → deployable tool names
- [ ] Tab shows description alongside completion

Phase 2 -- Error messages:
- [ ] Unknown command shows fuzzy match suggestion
- [ ] Failed command shows exit code + context hint
- [ ] Permission denied shows helpful message
- [ ] Error format is consistent and forest-colored

Phase 3 -- Structured output:
- [ ] `intents | where status = active` works
- [ ] `deploys | where tool = core` works
- [ ] `friday | where confidence > 0.8` works
- [ ] Pipeline between structured commands works

Phase 4 -- Multi-command + job control:
- [ ] Here-strings (<<<) work
- [ ] bg/fg/jobs work correctly
- [ ] Nested subshells reliable
- [ ] Complex pipelines survive edge cases

Phase 5 -- Startup:
- [ ] Time to prompt under 50ms (measured)
- [ ] Welcome screen clean and fast
- [ ] Alias loading does not block startup

Final:
- [ ] Tab completion works in every context a watcher would try
- [ ] fsh feels complete to someone seeing it for the first time
- [ ] Structured output demonstrated live: intents | where status = active
- [ ] Startup is instant
- [ ] fsh v3 is presentation-ready for Linus Torvalds

---

"A shell that speaks human must also complete human.
The forest knows what you are about to say.
It has been listening." 🌲
