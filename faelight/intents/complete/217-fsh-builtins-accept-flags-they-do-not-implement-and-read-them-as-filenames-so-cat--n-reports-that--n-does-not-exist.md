---
id: 217
date: 2026-08-09
type: future
title: "fsh builtins accept flags they do not implement and read them as filenames, so cat -n reports that -n does not exist"
status: complete
tags: [fsh, builtins, cat, dispatch]
---

## Vision
A builtin that does not implement an option says so by standing aside, not by reading the option as
a filename. The user asked for cat semantics; give them cat.

## The Problem
The fsh cat builtin is a RENDERER -- it numbers and dims code files for a human reading them -- and
it implemented no options at all. It took the first argument as a filename, so `cat -n file`
answered that -n does not exist. That reads as the shell claiming there is no such file, rather than
admitting it does not implement the option.

The defect had a second half that only a probe found. An alias bypass upstream held exactly six
flags, so those six reached the builtin while `cat -E` and `cat -T` expanded to bat and failed with
a clap error naming bat -- a shell reporting a tool the user never invoked. Two expressions of one
rule, disagreeing with each other, in a place nobody had looked.

## The Solution
Defer, do not implement. Real cat implements all eight GNU options correctly, including the compound
forms where -A is -vET. The builtin stands aside when it sees any leading dash, which is the shape
grep_cmd already uses for an unrecognised flag and the same conclusion the redirect bypass reached.

Deliberately NOT an allowlist of the eight options. An allowlist is another place to update when GNU
adds one, and it would get the compound forms wrong on the way. Any leading dash defers, including a
bare dash, which real cat reads as stdin.

The alias bypass upstream gets the same rule, so the two sites agree instead of disagreeing.

## Success Criteria
- [x] G1: the builtin defers on ANY leading dash rather than a list of known options
<!-- evidence: 44566184. commands/mod.rs cat arm, `args.iter().any(|a| a.starts_with("-"))` ->
     spawn_sh_with_leak_check. Same shape as grep_cmd:11686. -->
- [x] G2: the alias bypass uses the same rule, so the two sites cannot disagree
<!-- evidence: 44566184. engine.rs expand_aliases held exactly six flags, so cat -E and cat -T
     expanded to bat and failed with a clap error NAMING BAT -- a shell reporting a tool the user
     never invoked. Found by probe, not by reading. Now defers on any flag. -->
- [x] G3: verified against GNU BEHAVIOUR on all eight options, not against the absence of an error
<!-- evidence: measured on the debug build. -n numbers every line, -b numbers non-empty lines only,
     -E and -e show line ends, -T shows tabs, -A shows both, -v is quiet on ordinary text. Each
     matches coreutils on the same file. -->
- [x] G4: stdin works through a pipe
<!-- evidence: a piped cat -b numbers from the pipe, because spawn_sh_with_leak_check inherits
     stdin (db.rs:615). -->
- [x] G5: the RENDERER is not traded away -- plain cat still renders through bat
<!-- evidence: the control probe. `cat file` with no flag still produces the bat box. Without this
     the fix would have silently removed the reason the builtin exists. -->
- [x] G6: no recursion
<!-- evidence: sh resolves cat from PATH, which is coreutils, never this builtin. Confirmed:
     `sh -c "command -v cat"` returns the coreutils path. -->
- [x] G7: deployed and verified on the live shell
<!-- evidence: gen 489 onward. `cat -b` on a three-line file prints 1 a / 2 b, non-empty only. -->
- [x] G8: each gate carries evidence per INT-158
<!-- evidence: this block. The intent body was written AFTER the work, which is recorded here
     rather than hidden: it was filed as a placeholder with a single `- [ ] ...` gate line, built,
     and only then written up. That stub would have passed cicomplete unchallenged on a status
     flip, which is exactly the hole INT-212 exists to close. -->

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
