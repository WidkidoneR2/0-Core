---
id: 097
date: 2026-06-28
type: future
title: "Fsh needs a clean Nix/Shell operator path"
status: planned
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
- [ ] `ls *.bak-*` and other globs expand correctly (no literal-`*` failures)
- [ ] `cp <file> /tmp/<scratch>` NOT blocked; real core-binary writes still blocked
      (guard precision proven both ways)
- [ ] `query <file> <out-of-range>` clamps safely, never closes the terminal
- [ ] `&&` `||` `;` `>` `>>` `2>&1` behave correctly in a multi-step test
- [ ] A full INT-076-style build session (scaffold, edits, git, cargo) completes
      start-to-finish in fsh with ZERO drops to bash
- [ ] Stress test passes; 1-week daily-drive before new fsh features

## Notes
Evidence base: 2026-06-28 session transcript (INT-024 + INT-076). Acceptance test:
"ZERO drops to bash for a full build session." Tonight failed that test repeatedly.

---
