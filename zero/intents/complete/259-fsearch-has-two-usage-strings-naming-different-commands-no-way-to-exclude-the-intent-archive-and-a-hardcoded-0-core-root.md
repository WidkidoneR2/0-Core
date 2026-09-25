---
id: 259
date: 2026-09-23
type: future
title: "fsearch has two usage strings naming different commands, no way to exclude the intent archive, and a hardcoded 0-core root"
status: complete
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

## ⚠️ FIVE DEFECTS, NOT THREE -- THE RECON FOUND TWO MORE, BOTH THE SAME SHAPE

```text
    FILED       two usage strings, one naming a command that does not exist
                no inverse of --intent
                --all and --scripts building $HOME/0-core by hand
    FOUND       --scripts pointed at 0-core/scripts, WHICH DOES NOT EXIST
                walk_dir skipped every EXTENSIONLESS file
```

★ THE TWO FOUND DEFECTS COMPOUND INTO ONE FACT: fsearch could not read a single file in
faelight/scripts. Wrong directory, and unable to open the files even when pointed at the right one
-- every file there (devshell, devshell-lib, dev) is extensionless, and the walk only opened files
whose extension was on an allow-list.

⚠️ AND IT REPORTED THAT AS AN ANSWER. This builtin's own comment defines an empty table as "the
files were read, the pattern did not appear". It never opened them. The same collapse as
/etc/faelight reading as empty, the health cache reading as 100, and the caret reading unknown as
success -- this time in the shell's own search.

## MEASURED, before and after

```text
    fsearch paths.rs                 141 rows        the noise
    fsearch paths.rs --live           21 rows        the answer -- 120 rows of history removed
    fsearch bwrap --scripts       0 -> 5 rows        the devshell scripts, previously invisible
    an absent 0-Core              refuses, as --intent already did, rather than searching the cwd
```

## ⭐ THE CASE FAILED THREE TIMES AND THE SHELL WAS RIGHT EVERY TIME

Recorded because the lesson outlives this intent:

```text
    1  asserted on "intents/"         the table TRUNCATES the path column, so rows read
                                      "/home/.../faelight/inten..." -- the string could not appear
    2  asserted on "faelight-core"    truncated away for the same reason
    3  probed with `tail -8`          the banner pushed the rows off the end; the search was fine
```

Each time the instinct was to suspect the code. Each time the code was correct and the assertion
was measuring something the system does not produce. The case now asserts on `faelight/inten` --
what the table actually prints.

## Success Criteria
- [x] WATCH IT FAIL FIRST: search for a term that appears in both live code and the archive, and
      record the counts. The ratio is the argument for the filter
<!-- evidence: 2026-09-23. `paths.rs`: 10 live, 111 archive -- ELEVEN times more history than code,
     so the answer to "where is this in code that runs" was 8% of the output. Also faelight
     1163/1526, runtime_dir 36/16, restore_sigpipe 19/4. The ratio is not uniform, which is itself
     the argument: you cannot know in advance whether a term is drowned. -->
- [x] ONE usage string, naming fsearch, from ONE place in the source. Proven by grep returning one
<!-- evidence: const FSEARCH_USAGE, used at both sites. grep "usage: fsearch" returns 1 and
     "usage: search" returns 1 -- the surviving one is inside the doc comment that quotes the old
     string to explain why it went. -->
- [x] a flag excludes the intent archive, and "live" is DEFINED here: what is excluded and why --
      history, generated output, or both
<!-- evidence: --live (alias --code). THREE exclusions, THREE reasons, named separately so a later
     change to one does not silently alter the others:
       faelight/intents/   HISTORY -- INT-247 forbids rewriting it, so it is noise for a live question
       meta/CHANGELOG.md   a record of what was, same reason
       docs/public/        GENERATED from docs/ -- every hit duplicates one you already have
     Excluded by PATH rather than by name, so a file called intents.rs in live code is not caught
     by a rule about the archive. Measured: 141 rows -> 21. -->
- [x] no root is built by hand: `$HOME/0-core` appears nowhere in the fsearch path. Every root
      comes from paths.rs
<!-- evidence: --all and --scripts now ask core_integration::tools_root(), which asks
     paths.rs::rust_tools_dir(). The only 0-core strings left in the fsearch block are inside the
     comment recording what was removed.
     ⚠️ AND --scripts WAS WRONG, NOT MERELY HAND-BUILT: it pointed at 0-core/scripts, which does
     not exist -- the scripts are in faelight/scripts. Sixteen OTHER hand-built 0-core paths remain
     elsewhere in commands/mod.rs, four of them naming that same absent directory. INT-240's work,
     not this intent's. -->
- [x] an absent root REFUSES rather than searching the working directory, as --intent already does.
      Proven by running it where the root is absent
<!-- evidence: with HOME pointed at an empty temporary directory, `fsearch anything --all` prints
     "fsearch: --all needs 0-Core, which is not present" and exits 1. The model was --intent's own
     comment: "assigning None for 0-Core is absent would make this silently search the working
     directory instead of refusing." -->
- [x] nsh-test gains a case for the exclusion filter, and the case is proven to have TEETH --
      red when the filter is removed
<!-- evidence: repl_259_live_excludes_the_archive. 200/200 against the new binary; RED against the
     deployed one, and the failure names the leaked rows.
     ⭐ PROVEN BY CONTRAST, NOT BY A COUNT: a row count drifts with every commit, while "the archive
     appears without the flag and not with it" stays true as the tree grows.
     ⭐ AND IT CARRIES ITS OWN RED-FIRST CHECK: if the unfiltered search does not reach the archive,
     the case FAILS rather than passing vacuously -- otherwise an empty result would satisfy the
     exclusion assertion while proving nothing. It also asserts the filtered search still finds
     live code, for the same reason. -->
