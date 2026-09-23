---
id: 259
date: 2026-09-23
type: future
title: "fsearch has two usage strings naming different commands, no way to exclude the intent archive, and a hardcoded 0-core root"
status: planned
tags: [fsearch, novashell, search, paths]
---

## Vision
"Where is this named in code that still runs" is ONE FLAG, not a grep pipeline.

## The Problem
Three defects, measured 2026-09-23 in `novashell/src/commands/mod.rs`:

**1. TWO USAGE STRINGS, AND ONE NAMES A DIFFERENT COMMAND.**

```text
    4416   "usage: search <pattern> [--type ext] [--file name]"     <- not this builtin
    4524   "usage: fsearch <pattern> [--type ext] [--file name]"
```

One builtin, two owners, one wrong. The same shape as the two tokenizers and the two alias sites,
in miniature.

**2. THERE IS NO INVERSE OF `--intent`.** `--forest` and `--all` search everything, and the intent
archive alone holds 172 historical occurrences of `faelight/`. Every question INT-247 and INT-252
ask is about LIVE code, and the archive drowns the answer. `--intent` selects the archive; nothing
excludes it.

**3. `--all` AND `--scripts` BUILD `$HOME/0-core` BY HAND.** That is INT-240's class exactly: a
path typed rather than asked for. INT-252 renames `faelight/`, and INT-247 Layer 3b moves the
state paths -- a hardcoded root in the shell's OWN search builtin is a stale path waiting to
happen.

⭐ AND `--intent` IS ALREADY THE MODEL FOR THE FIX. It REFUSES when 0-Core is absent rather than
falling back to the working directory, and the comment says why: "None here already means the user
did not pick a root -- assigning None for 0-Core is absent would make this silently search the
working directory instead of refusing." A filter that quietly searches the wrong tree is the same
collapse this project keeps finding.

## The Solution
One usage string from one place. One flag that excludes history. Every root from paths.rs.

```text
    usage      ONE string, naming fsearch, defined once
    the filter live code only -- the archive excluded, not merely deprioritised
    roots      paths.rs, and an absent root REFUSES as --intent already does
```

⚠️ THE FILTER IS THE FEATURE; the other two are correctness. And "live" needs defining IN THE
INTENT rather than in the code: completed intents are history, CHANGELOGs are history, and
docs/public/ is generated -- three different reasons to exclude, and a flag that conflates them
will be wrong for one of them later.

## Success Criteria
- [ ] WATCH IT FAIL FIRST: search for a term that appears in both live code and the archive, and
      record the counts. The ratio is the argument for the filter
- [ ] ONE usage string, naming fsearch, from ONE place in the source. Proven by grep returning one
- [ ] a flag excludes the intent archive, and "live" is DEFINED here: what is excluded and why --
      history, generated output, or both
- [ ] no root is built by hand: `$HOME/0-core` appears nowhere in the fsearch path. Every root
      comes from paths.rs
- [ ] an absent root REFUSES rather than searching the working directory, as --intent already does.
      Proven by running it where the root is absent
- [ ] nsh-test gains a case for the exclusion filter, and the case is proven to have TEETH --
      red when the filter is removed
