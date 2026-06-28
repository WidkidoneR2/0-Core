---
id: 097
date: 2026-06-28
type: future
title: "Fsh needs a clean Nix/Shell operator path"
status: in-progress
tags: [fsh, Nix, system, faelight]
---

## Vision
fsh handles the shell-operator-heavy operations of NixOS development -- globs,
quoting, redirects, file surgery -- as cleanly as bash does, so the forest never
has to "step out" to bash to get real work done. fsh is the daily driver for a
NixOS build environment; it should not concede the exact operations that NixOS
development demands most.

## The Problem
Documented from the 2026-06-28 session (INT-024 VM harness + INT-076 faelight-nix).
We dropped to bash repeatedly because fsh could not handle the operation. Each is a
concrete, reproducible failure:

1. NESTED QUOTES. `sh -c '...'` and any command embedding single quotes inside
   double (or vice versa) failed with "unexpected EOF". Forced all multi-line edits
   through `cat > /tmp/x.py << 'PYEOF' ... PYEOF; python3 /tmp/x.py`.

2. GLOBS IN OPERATIONS. `ls ~/.../home.nix.bak-*` returned "No such file or
   directory" -- the `*` was passed literally, not expanded. Forced python3 glob
   workarounds. (Friday surfaced the workaround as KNOWN knowledge -- a recurring,
   already-catalogued failure.) NOTE: this same glob class bit charter authoring --
   a `097*.md` glob matched the wrong directory and overwrote a decision doc.

3. OVER-EAGER SAFETY GUARD FALSE POSITIVE. `cp home.nix /tmp/home-test.nix` was
   blocked: "direct copy to core binary -- use deploy script instead." A plain
   scratch copy tripped a binary-protection rule. Guard is good; matcher too broad.

4. PAGER / RANGE INSTABILITY. `query <file> <range>` closed the terminal on
   out-of-range line numbers; paging forced repeated `q`. File inspection unreliable;
   used bash wc/head/tail instead.

5. CHAINS. `&&` and `;` chains unreliable; every multi-step op issued one at a time.

Net effect: the shell built FOR this NixOS project is bypassed for the operations
this project most needs.

## The Solution
A deliberate "shell-operator layer", stabilised before features:
- Quoting: correct tokenizer for nested single/double quotes + escapes.
- Globbing: expand `*` `?` `[...]` in args (wire the existing glob_match helper into
  external-command argument expansion).
- Redirects + chains: reliable `>` `>>` `2>&1` `&&` `||` `;`.
- Guard precision: exempt /tmp and scratch paths; keep real core-binary protection.
- `query` hardening: clamp out-of-range lines (never crash); predictable/no-page mode.

Stabilise-first: land the layer, daily-drive 1+ week, stress-test each change BEFORE
new features.

## Success Criteria
- [ ] `sh -c '...'` and nested-quote commands run without "unexpected EOF"
- [x] globs expand correctly; no-match now prints 'no matches for pattern: X' (failglob), no cryptic os-error
- [x] `cp <file> /tmp/<scratch>` allowed; `cp x ~/.cargo/bin/core` still blocked (proven both ways, gen 260)
- [ ] `query <file> <out-of-range>` clamps safely, never closes the terminal
- [ ] `&&` `||` `;` `>` `>>` `2>&1` behave correctly in a multi-step test
- [ ] A full INT-076-style build session (scaffold, edits, git, cargo) completes
      start-to-finish in fsh with ZERO drops to bash
- [ ] Stress test passes; 1-week daily-drive before new fsh features

## Notes
Evidence base: 2026-06-28 session transcript (INT-024 + INT-076). Acceptance test:
"ZERO drops to bash for a full build session." Tonight failed that test repeatedly.

---


## Investigation (2026-06-28 PM): globbing WORKS -- real bugs are no-match UX + Friday

Reproduced live. Glob expansion (expand.rs expand_globs / expand_globs_in_segment)
is CORRECT: `ls ~/.../something.bak-*` expands properly when files exist (verified
3x, including from a different cwd). Tilde-before-glob ordering is right.

The actual failure last night was a NO-MATCH glob:
  `ls ~/.../nonexistent.bak-*`  (or a real pattern before the file existed)
  -> glob matches nothing -> expand_globs_in_segment pushes the LITERAL pattern back
     (expand.rs ~line 630) -> `ls` receives a literal `*` path
  -> "No such file or directory (os error 2)"  [cryptic; looks like a glob failure]

This is bash-with-nullglob-unset behaviour, but the UX is misleading. TWO real fixes:

A. NO-MATCH UX. On zero matches, fsh should emit a clear message
   ("no matches for pattern: <pat>") instead of passing a literal `*` that produces a
   raw OS error. (Consider bash-like nullglob/failglob semantics.)

B. FRIDAY MIS-LEARNED PATTERN (higher priority). On this glob-no-match error Friday
   fires `fsh_multiline_python` at 99% confidence: "Write script to /tmp/script.py...".
   That advice is WRONG for this error and actively trained us into the python/bash
   workaround all of last session. Friday is reinforcing the very friction 097 targets.
   Fix: correct or remove this pattern association so the no-match error does not map
   to the multiline-python suggestion.

Revised globbing criterion: globs already expand. Ship (A) clearer no-match handling
and (B) the Friday pattern correction instead of "make globs work".


## Progress (2026-06-28 PM-2): three fixes deployed (gen 259-260)

1. FRIDAY MATCHER PRECISION (exec.rs) -- error->knowledge search now matches only
   error_signature + description, not resolution/id. Stops fsh_multiline_python (and
   other high-confidence entries) firing on unrelated errors. This was the root of
   last session's bash detours. (f628568f)
2. FAILGLOB (expand.rs find_unmatched_globs + main.rs guard) -- a glob matching
   nothing prints 'no matches for pattern: X' and skips exec, instead of passing a
   literal * that gave 'os error 2' + a bogus Friday suggestion. (f628568f)
3. SAFETY-GUARD FALSE POSITIVE (exec.rs Rule 3) -- was raw.contains('core'), which
   matched every path under ~/0-core and blocked cp/mv of any repo file. Now checks
   the DESTINATION is an actual core binary. Proven both ways.

Each fix: mapped callers -> edit -> build -> subshell test (never deployed unproven
over the live shell) -> commit -> deploy -> verify live.

### Still open
- Nested-quote tokenizer (#1): `sh -c '...'` "unexpected EOF".
- `query` out-of-range crash (#4): closes the terminal.
- Chains (#5): `&&`/`||`/`;` reliability (re-verify; may be edge cases).
- Friday: a *successful* grep/command with no matches exits 1 -> treated as error ->
  enters knowledge path. Consider: exit-1-with-no-output isn't always an error.
