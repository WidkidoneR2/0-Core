---
id: 109
date: 2026-07-02
type: future
title: "Improve fsh to handle muiltiple/bundle commands"
status: planned
tags: [fsh, commands]
---

## Vision
fsh reliably executes multiple/bundled commands in one line -- `&&` chains,
`;` sequences, and piped multi-stage commands -- so the shell stops forcing
one-command-at-a-time workflows.

## The Problem
fsh cannot currently handle several common multi-command forms. Observed
repeatedly during INT-061 (2026-07-02), where nearly every step had to be
split into single commands:
- `&&` chains: `cmd1 && cmd2` -- second command drops or misbehaves.
- `;` multi-statement: `cmd1; cmd2; cmd3` -- unreliable, esp. after redirects.
- Piped multi-stage: `... 2>&1 | tail -N` -- broken-pipe panics / hangs; the
  `| tail`/`| head` buffering makes long builds LOOK hung (no streamed output).
- Chaining a `cd` with a following command in one line.
This forces a one-command-per-line discipline that slows multi-step work
(the entire 061 restructure was executed this way).

## The Solution (high-level -- design at start)
Proper command-sequence parsing in fsh:
- Tokenize and execute `&&` / `||` / `;` operator-separated command lists with
  correct short-circuit semantics.
- Fix pipe handling so `| tail`/`| head` don't panic on SIGPIPE and don't
  swallow/buffer in a way that appears hung.
- Ensure builtins (`cd`, aliases) compose correctly inside chains.
- Cross-check against the known fsh quirks already tracked (INT-298 shell
  issues: tilde expansion, heredoc hanging, `$(cmd | pipe)` subshell awareness).

## Prior Art / Related
- INT-291 (shell stabilization) fixed many parser bugs (`&&`/`||` logic,
  semicolon chains after redirects, `seq | while read`). This intent continues
  that line for the multi-command cases still broken.
- INT-298 tracks remaining shell issues; coordinate scope to avoid overlap.

## Success Criteria
- [ ] `cmd1 && cmd2` executes both with correct short-circuit
- [ ] `cmd1; cmd2; cmd3` runs reliably (incl. after redirects)
- [ ] `... 2>&1 | tail -N` works without broken-pipe panic and streams sanely
- [ ] `cd dir && cmd` composes correctly
- [ ] Multi-command pastes (like this session's) run as-is, no manual splitting

---
