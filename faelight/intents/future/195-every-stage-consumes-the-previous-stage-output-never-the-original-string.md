---
id: 195
date: 2026-07-25
type: arch
title: "every stage consumes the previous stage output, never the original string"
status: planned
tags: [architecture, rust, design]
---

## Vision
Within the execution pipeline there is ONE canonical derivation of the command
word. Nobody re-derives it.

## Scope
IN: the execution path -- main.rs, commands/mod.rs, exec.rs, expand.rs -- and any
component whose decision directly GOVERNS execution, even if it is not adjacent
in the pipeline. safety_guard.rs is the motivating case: architecturally outside
execution, semantically inside it, because it decides whether execution may
proceed. A PRIVILEGED EXECUTION CONSUMER is in scope.

OUT: independent consumers of raw user text -- completion, natural-language
parsing, semantic analysis, health UI, scripting utilities, value and db helpers.
They are not successive stages of one pipeline; completion runs on a partial line
before any parse exists. They may benefit from command_word(), but using
split_whitespace() there is not automatically a violation. This exemption is what
keeps the intent actionable instead of "replace every split_whitespace in the
repository".

CANONICAL: commands::command_word(). It is quote-aware per INT-171 gate 2, so
`"ll" foo` resolves to `ll`. Execution-governing code must not derive the command
word by other means.

TESTS: a test may bypass the canonical helper when it deliberately generates
malformed input or checks equivalence between implementations, but it must say so
in a comment -- otherwise the next reader treats it as precedent.

BY ROLE, NOT BY FILE. commands/mod.rs is over 14,000 lines and holds the
dispatcher, every builtin's body, and unrelated helpers. Scoping by filename turns
eight real sites into twenty false positives. A function is in scope because of
what it DECIDES, not because of where it lives.
THE TEST, as a question to ask of any site: does this code derive the command word
to influence EXECUTION, PROTECTION, or EXECUTION-DERIVED STATE? That question
explains why safety_guard.rs is in scope and why db.rs's snapshot naming is in
scope, while display-only builtins stay out. It is stronger than a list of
filenames, which drifts.

## The Problem
fsh's documented bug class is not "the parser was wrong". Twice the parser was
RIGHT and something downstream ignored it: INT-143 was four tokenizers with no
pipeline between them, six bugs on 2026-07-16; INT-171 consolidated to one
parsing entry point precisely because the entry points were never the problem,
the bypasses were.

A live instance, found 2026-07-25 while scoping this intent. safety_guard.rs:12
derives the command word with `trimmed.split_whitespace().next()`, which is not
quote-aware, while the executor uses the quote-aware command_word(). On
`"rm" -rf /` the guard sees `"rm`, matches no deny entry, no allow entry, no safe
entry, and fails its own `first_word == "rm"` test, so it returns None and gates
nothing -- while the executor sees `rm` and runs it. The guard's first-word-only
design is deliberate and correct; only the instrument is wrong.

## The Solution
Name the scope, name the canonical derivation, enumerate the sites inside the
scope that bypass it, and make the check mechanical. The fix at each site is a
call substitution, not new logic, because the correct implementation already
exists -- which is INT-143's lesson applied rather than restated.

## Census -- 12 in-scope sites (2026-07-25, regenerated)
METHOD CORRECTION, recorded rather than quietly fixed: the first pass searched for a
single-line spelling of .split_whitespace().next() and therefore MISSED rustfmt-wrapped
method chains. It enumerated what a line-oriented grep could see, which is not the same
as what exists. The census was regenerated with a whitespace-spanning pattern before any
further classification, and grew from 9 to 12. DISCOVERY and CLASSIFICATION are separate
steps: the pattern answers where the derivations are, mechanically and reproducibly;
which ones are in scope remains a documented architectural judgement.
The ORIGINAL census was recorded before any code change, so later commits could
reference a written inventory rather than reconstruct the investigation; this
regeneration happened mid-migration, with six sites already converted. Classified by
ROLE, because role determines both the risk and the order.
LINE NUMBERS ARE INDICATIVE ONLY. They drift as fixes land -- 1504 is now 1509, 3169 is
now 3181, safety_guard 12 is now 13. Sites are identified unambiguously by enclosing
FUNCTION plus the exact expression, not by line.

GOVERNING -- decides whether a protection activates. Blockers, not migrations.
  safety_guard.rs:12  fn check()
      The safety gate. `"rm" -rf /` presents first_word `"rm`, which matches no
      deny, allow or safe entry and fails the `first_word == "rm"` test, so the
      gate returns None and never fires.
  main.rs:1484  fn handle()
      INT-322 Phase 4 auto-snapshot before destructive commands. Same bypass, same
      input: a quoted command word means no recovery snapshot is taken. Both
      protective mechanisms are bypassed by the same quoted command-word input,
      which is why these two rank above everything else here.

DISPATCH -- decides what runs, or where it runs.
  main.rs:1532  fn handle()  flow mode, earliest intercept (`ftok == "flow"`)
  main.rs:2253  fn handle()  shell control structures (for/while/until/if/case).
      Routes the line to sh WITH VARIABLES UNEXPANDED, so a mis-read changes
      expansion semantics rather than only which branch is taken.
  main.rs:2939  fn handle()  job control (`first_tok == "jobs"`)

BEHAVIOURAL -- changes auxiliary runtime behaviour without changing what executes.
  main.rs:1504  fn handle()
      INT-307 Friday power switching on `cargo`. A mis-read costs a performance
      profile, not correctness.
  main.rs:3072  fn handle()
      Decides is_fm_cmd (yazi, faelight-fm) to inject --cwd-file. Does not gate
      execution and does not dispatch elsewhere; it changes HOW a known command is
      launched. Also lowercases, so the fix preserves normalization.

TELEMETRY -- wrong derivation produces wrong DATA. Lower operational risk, but not
cosmetic: record_failure was fixed for exactly this reason, because Friday reads
what it writes.
  main.rs:3169  fn handle()    INT-194 command-timing key, INSERTed into the db
  exec.rs:368   fn postexec()  derives from `cmd_lower`, so this site is ALSO flip
      blocker 8's "stop lowercasing the command name"
  main.rs:3233  fn handle()  INT-194 prediction-aware suggestions. Execution-derived
      state under the scope test above.
  main.rs:3299  fn handle()  INT-296 consecutive-failure detection. The clearest of the
      late finds: derive command identity, then store and query execution-derived
      history -- identical in class to record_failure.
  db.rs:451     fn capture_snapshot()
      Names the auto-snapshot after the command word. In scope ONLY because
      capture_snapshot is reached solely from main.rs:1500, i.e. only from the
      execution path -- if a second caller appears, revisit this. Composes with
      main.rs:1484: that site governs WHETHER a snapshot is created, this one governs
      HOW it is attributed. Fix one and you get either no snapshot at all, or a
      snapshot filed under auto-"rm.

EXCLUDED, and why -- recorded so the next reader does not re-litigate them:
  - Display and reporting builtins that happen to live in commands/mod.rs:
    debug_cmd, explain_cmd, history_stats, dev_cmd, histogram_cmd,
    semantic_ambiguous_cmd, semantic_why_cmd. Separate consumers by role.
    NOTE explain_cmd reports what an alias resolves to, so a quote-blind read is
    user-facing misinformation about the user's own aliases. Worth fixing
    opportunistically; not a gate.
  - Not command words at all: print_welcome (an intent id), walk_dir (a SQL LIKE
    pattern), shell_handoff_cmd (a shell NAME, defaulting to zsh).
  - INTENTIONAL TOKENIZATION THAT MUST NOT BE CONVERTED -- the check will flag these
    forever, so each carries its reason and gate 5 needs suppression-with-reason rather
    than a bare hit list:
      main.rs:1203       heredoc DELIMITER extraction. For cat << EOF the command word
                         is cat and the delimiter is EOF. command_word() would return
                         the wrong semantic object.
      commands/mod.rs:7301  intent id parsed from intent-show OUTPUT, then INT- stripped.
                         Structured command output, not user command input.
      value.rs:750       parse_pipe_op, INT-162's structured-data pipeline operators.
  - completion.rs and value.rs -- exempt consumers under the scope above.
    db.rs was ORIGINALLY excluded here as a helper file. That was wrong: it scoped by
    FILE, which is the mistake the by-role rule exists to prevent, made in the
    opposite direction. db.rs:451 is in scope and is listed under TELEMETRY above.

## Narrowed 2026-07-25, and why (recorded, not silently rewritten)
This intent was FILED covering four banned calls: ad-hoc splitting, a second
tokenizer, raw-text operator scans, and re-parsing. Scoping recon showed the
first has a canonical replacement TODAY while the others do not -- the answer for
operator and structure derivation is "the parser", and the parser does not own
execution until it is driven from canonical parser output (INT-169's "spine
flip"). An intent must not be made responsible for
enforcing something the codebase cannot yet satisfy, so the clauses were split by
their PREREQUISITES rather than their similarity:
  - THIS intent: nobody re-derives the command word. Enforceable now.
  - Operator/structure derivation: nobody re-derives shell structure. Enforceable
    only once the parser is authoritative. Its own intent, tied to the flip.
  - Guard placement: whether safety evaluates the typed, canonical or expanded
    command. A policy question, independent of how the word is derived. Its own
    intent.

## Success Criteria
- [x] The scope above is recorded where code can be checked against it: execution
      path plus privileged execution consumers IN, independent consumers OUT,
      command_word() named as canonical, test bypasses requiring justification
      <!-- DONE 2026-07-25. The Scope section above states all four clauses: what is IN
      (execution path plus privileged execution consumers), what is OUT (independent
      consumers of raw user text), commands::command_word() as THE canonical derivation,
      and the test-bypass justification rule -- plus the by-role-not-by-file boundary that
      made the census tractable. The intent document IS where code is checked against the
      rule; automating that check is gate 5. Requiring gate 5 first would be circular --
      you cannot know what to check until the scope is defined, but the scope would not
      count as defined until the check existed. Gate 1 defines the contract, gate 2
      enumerates the violations, gate 5 automates enforcement. -->
- [x] Every execution-governing site that derives the command word independently
      is enumerated with file:line -- a census, not a fix
      <!-- DONE 2026-07-25. Census section above: 9 in-scope sites with file:line and enclosing
      function, classified GOVERNING / DISPATCH / BEHAVIOURAL / TELEMETRY, plus an EXCLUDED list
      with reasons so the exemptions are not re-litigated. Method: grep for
      `split_whitespace().next()` rather than every `split_whitespace()` (31 hits), drop comment lines (6 were the rule itself,
      already written in prose in two files), drop out-of-scope consumers, then resolve each
      remaining hit to its enclosing fn -- which classified most of them without reading bodies. -->
- [x] Each enumerated site either routes through command_word(), or is recorded
      as a known exception with a stated reason
      <!-- DONE 2026-07-25 gen 433. All 12 in-scope sites route through commands::command_word():
      safety_guard::check, main.rs 1484/1532/2253/2939/1509/3072/3181/3233/3299, exec.rs postexec,
      db.rs capture_snapshot. Commits ebe9dccf, 65eed5c5, 391b0a15, 7331b562. THREE recorded as
      known exceptions with stated reasons, because converting them would be WRONG rather than
      merely unnecessary: main.rs:1203 extracts a heredoc DELIMITER (for a cat heredoc the command
      word is cat and the delimiter is EOF), commands/mod.rs:7301 parses an intent id out of
      command OUTPUT, and value.rs:750 parses structured-data pipeline operators. -->
- [x] safety_guard.rs uses command_word(). Named explicitly because it is the
      privileged consumer that motivated the scope, and because a guard that
      disagrees with the executor about what is running is the worst instance of
      the class
      <!-- DONE 2026-07-25 gen 433, on the DEPLOYED binary per INT-110. WATCHED FAILING FIRST at
      gen 432: rm -rf on a nonexistent path raised CHALLENGE and blocked, while the same line with
      the command word quoted produced NO guard output at all and simply ran. After the fix, on the
      deployed shell, BOTH forms raise CHALLENGE and block. Commit ebe9dccf. The first-word-only
      design is unchanged; only the derivation moved. -->
- [x] A check exists that returns the violations, runnable on demand
      <!-- DONE 2026-07-26 gen 436. faelight-deadwood gained syntax-aware analysis: a syn 2 visitor
      finds whitespace-derived first tokens, a coarse file filter narrows to in-scope files, and a
      bounded adjacent `// deadwood: exempt` declaration resolves author intent. Commits a66171d1
      (detector), f9a69e93 (declarations), 55f85558 (resolver tests). Runnable on demand via
      `faelight-deadwood --only cmdword`. 36 raw hits narrowed to 10 candidates, every one already
      classified in the census and none a genuine unfixed violation -- the check and the census
      validate each other. Text searching was tried first and missed three classes: rustfmt-wrapped
      chains, by-file scope exclusion, and alternate spellings such as splitn.
      DOCUMENTED LIMIT: Rust macro bodies are not recursively analysed, so four sites inside
      format!() are invisible. All four are string-building rather than execution-governing, so the
      boundary does not currently overlap the rule; an execution-path violation inside a macro would
      be the evidence that it is too restrictive. -->
- [x] The check runs somewhere it will be seen (pre-commit or fsh-test), not only
      by hand
      <!-- DONE 2026-07-26 gen 436. `--strict` exits non-zero on reported findings (e0e49049),
      orthogonal to `--summary` so the health doctor's positional contract is untouched; fsh-test
      invokes it through a DEADWOOD_BIN seam (ad5ea790). PROVEN BY DISAGREEMENT rather than by two
      green runs: before deploying, the same case passed against the debug build at 105/105 and
      FAILED against the deployed binary at 104/105, because gen 434 predates --strict entirely.
      After deploying, bare fsh-test with no override reports 105/105 on gen 436. The check now runs
      on the normal test path rather than when someone remembers to type it. -->
- [x] RETRO-VALIDATION: the check is confirmed to catch safety_guard.rs's
      split_whitespace derivation, watched failing before it is trusted. A check
      that would have missed the violation it was written for is the wrong check.
      NOTE: the original wording named INT-172's detect_redirect, which is an
      OPERATOR scan and moved out of scope in the narrowing above
      <!-- DONE 2026-07-26 (03d1dc93). Five resolver tests bound the mechanism from both sides: bare
      derivation reported, adjacent declaration accepted, DISPLACED declaration rejected, prefix
      collision (`deadwood: exempted`) rejected, and safety_guard.rs's EXACT pre-fix line reported.
      The retro fixture is named safety_guard.rs so the scope filter is exercised as well as the
      detector, keeps the original `// Check first word only` comment to prove an ordinary comment
      does not exempt, and asserts the finding NAMES the file so a refactor cannot satisfy it with a
      finding pointing elsewhere. Also watched failing by hand on the real tree: inserting one line
      between a declaration and its candidate gave 9 author-exempt / 1 flagged / exit 1, and
      removing it restored 10 author-exempt / clean / exit 0. -->
