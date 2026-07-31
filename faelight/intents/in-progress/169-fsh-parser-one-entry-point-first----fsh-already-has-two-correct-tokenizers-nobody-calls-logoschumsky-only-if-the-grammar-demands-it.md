---
id: 169
date: 2026-07-16
type: arch
title: "fsh spine: logos lexer + handwritten recursive-descent parser -> AST. The parse/plan/execute rebuild"
status: in-progress
tags: [fsh, parser, ast, logos, lexer, recursive-descent, spine, phase3, 171]
---

## Vision
Build the parse -> AST layer fsh never had. Source -> logos (regular tokens) + custom lexer state
(context-sensitive regions) -> handwritten recursive-descent parser -> AST. ExecContext holds the
AST, not cmd + args. This is the structural reframe of how fsh represents commands -- the thing that
ends the bug class the string-based approach keeps generating.

## GATE ZERO -- ANSWERED WITH EVIDENCE (2026-07-21, 171 complete)
The original gate: "name THREE things fsh cannot do because of its current parser, each a real thing
Christian wanted and could not have -- or CANCEL." Answered, and the evidence is stronger than three
feature-wishes: it is a RECURRING BUG CLASS, all the same shape (commands are cmd+args strings
re-inspected in scattered places, each place able to be wrong independently):
  - INT-172: redirect dropped the line after `2>` (a parser that did not understand redirect structure)
  - INT-174: `$(...)` executed inside single quotes (no quote-context in the "parse")
  - INT-143/171: FOUR divergent tokenizers, six bugs, incl. running a command twice and reporting
    success for a command that never ran
RECON 2026-07-21 (post-171): fsh has a TOKENIZER (commands::tokenize, works), NOT a parser/AST layer.
ExecContext = { cmd: String, args: Vec<String> } (exec.rs:23). from_line() is `raw.splitn(2, ' ')` +
lowercase + tokenize -- there is no parse-into-structure stage at all. Pipelines/redirects/conditionals
are handled by ad-hoc string re-inspection elsewhere. The ABSENCE of a parse->AST layer is the
limitation. Gate resolves YES -- but "add the missing layer", NOT "rewrite working code" (see no-big-bang).

## THE TOOL DECISION (2026-07-21, Christian's call after evaluating the options)
DECIDED: `logos` (lexer) + HANDWRITTEN recursive-descent parser (Pratt for expressions if/when needed).
chumsky is DROPPED. Reasoning:
  - A handwritten recursive-descent parser is DEBUGGABLE THE WAY THE WHOLE SHELL IS DEBUGGED -- greppable,
    steppable, readable as a straight call path. For a daily-driver shell where a parser bug = an unusable
    terminal, "easy to debug" is not a nice-to-have, it is the safety model. chumsky's combinator
    composition is a harder, more distributed kind of debugging -- wrong for this risk profile.
  - logos removes the repetitive MECHANICAL lexer boilerplate (regular tokens: words, numbers, operators,
    whitespace, punctuation) -- fast (compiles to a DFA), declarative, low-maintenance.
  - The CONTEXT-SENSITIVE regions (strings, heredocs, `$(...)` command sub, `${...}` var expansion,
    interpolation) get a CUSTOM STATEFUL LEXER layer -- full control exactly where the bugs live (172/174).
  - This directly RESOLVES the caution below (lines about `>` being redirect-vs-comparison, context fighting
    parser combinators): the custom lexer DISAMBIGUATES context into distinct token types BEFORE the parser
    sees them, so the parser reads a clean, already-disambiguated token stream. No combinator fights context.
FULLY-HANDWRITTEN LEXER was considered and set aside: only needed for complete char-level control or strict
existing-shell compatibility. fsh is its OWN thing (not POSIX-sh compatible by goal), so logos earns its
place -- the last 5% of lexer control is not needed.

## The hybrid lexer shape (Christian's design)
    Source
      -> logos:        words | numbers | operators | whitespace | punctuation
      -> custom state: strings | heredocs | command substitution | variable expansion | interpolation
      -> token stream
      -> recursive-descent parser (Pratt for expressions if needed)
      -> AST: Command / Pipeline / Redirect / Assignment / (If / While / Function as the grammar grows)
      -> ExecContext holds the AST

## Rides with the spine (small correctness fixes the new structure enables)
- Stop lowercasing the command name (from_line's `.to_lowercase()` -- wrong; a real parser preserves case).
- Store SystemTime, not a raw u64 unix timestamp.
- Give each execution a unique ID (UUID or incrementing) -- this also lights up the DEAD correlation_id
  column (INT-167), finally making cross-layer tracing real.
  DECIDED 2026-07-25, from an outside architecture review: a MONOTONIC COUNTER,
  not a UUID, for as long as execution is intra-process. `exec_id=42` on every
  log line reads and correlates well locally; UUIDs and trace IDs earn their
  cost only if remote execution or distributed tracing arrives, and can be
  introduced then. This settles the 'UUID or incrementing' choice above.

## Scoped OUT of 169 (separate sequenced intents -- do NOT smuggle in)
- tree-sitter = for SYNTAX HIGHLIGHTING (editor side), NOT the parser. chumsky/handwritten owns execution
  parsing; tree-sitter would define fsh's grammar a SECOND time for highlighting -- a real cost to weigh in
  its own intent, not a free add. Its own future intent.
- reedline (INT-168), gix (git migration off libgit2), nix (job control), tokio (async), nu-ansi-term
  (check vs existing `colored` first) -- each its own sequenced sibling intent. 169 is the SPINE ONLY.

## PRESERVED HISTORY (do not lose -- this is why the discipline exists)
### The four-parser measurement (INT-143, 2026-07-16)
  1. commands/mod.rs tokenize_args -- CORRECT, quote-aware
  2. exec.rs tokenize -- CORRECT, quote-aware, a byte-for-byte DUPLICATE of #1
  3. main.rs:2411 redirect branch -- WRONG (split_whitespace)
  4. main.rs:1956 inline-VAR loop -- WRONG (split_whitespace)
### THE TRAP THIS INTENT MUST NOT FALL INTO
Those bugs were NOT an argument for a new parser crate. fsh ALREADY HAD a correct tokenizer -- twice. The
bugs were code that did not CALL it. You can fail to call a handwritten parser exactly as easily as you can
fail to call tokenize(). 171 consolidated to ONE entry point FIRST so this could be judged honestly. The
spine REPLACES the string-reinspection with a single AST every path routes through -- that is the point,
not "a better parser."
### The chumsky context-sensitivity caution (2026-07-16) -- now RESOLVED by the hybrid
"Shell syntax is context-sensitive: `>` is a redirect here and a comparison there; a word is a command,
a filename, or a bare string depending on position. That is not chumsky's sweet spot." -> RESOLVED: the
custom stateful LEXER disambiguates context into distinct tokens before parsing, and the parser is
handwritten (no combinator fighting context at all).

## Scope guardrails (STILL BINDING)
NO BIG-BANG. It starts behind the 171 entry point, ONE CONSTRUCT AT A TIME, old path live until the new one
passes the SAME tests (fsh-test REPL-driven). fsh must boot, log in, and deploy at every step -- it is the
daily driver AND the demo. Honestly stated: the full spine is a long build (many nights), not a weekend.
Christian committed to it 2026-07-21 as active work, incrementally -- "rolling the dice" on the right shape,
built behind the discipline that makes it not-a-gamble.

## Success Criteria
- [x] INT-171 COMPLETE before this starts.
      <!-- DONE -- 171 complete, deployed gen 402, one tokenizer entry point exists. -->
- [x] Gate zero answered with evidence, or cancel.
      <!-- DONE 2026-07-21 -- answered YES. Evidence: the bug class (172 redirect, 174 quote-context,
      143/171 four-tokenizer divergence) + recon finding (tokenizer exists, no parse/AST layer;
      ExecContext = cmd+args, from_line = splitn(2,' ')). The absence of the layer IS the limitation. -->
- [x] RFC first: what problem, why logos+handwritten, alternatives (chumsky/fully-handwritten) & why not,
      trade-offs, how it fits fsh's philosophy. Written before code. (Largely drafted in the 2026-07-21
      discussion -- capture it as the RFC.)
      <!-- DONE 2026-07-21 -- docs/rfc-169-parser-spine.md, commit 86951398. Problem, decision,
      alternatives (chumsky + fully-handwritten, and why not), AST design, lexer design, execution
      interface, 9-step roadmap, first construct, rides-with fixes, Appendix A. Written before any
      spine code was cut. -->
- [ ] logos added + the hybrid lexer stands up: regular tokens via logos, custom stateful layer for
      strings/heredocs/$()/${}/interpolation. Demonstrated tokenizing a real line into typed tokens.
- [x] ONE CONSTRUCT end-to-end as proof-of-shape: a simple command -> logos+lexer tokens -> handwritten
      parse -> AST node -> execute, with the OLD path live beside it and the SAME REPL tests passing.
      <!-- DONE 2026-07-22 gen 427. The vertical: source -> custom stateful scanner (logos
      replaced on the word path at step 2) -> handwritten recursive-descent parse -> AST ->
      lower() -> ExecutionPlan -> argv -> std::process::Command -> process. NO sh, NO re-parse.
      Two opt-in entry points, live path untouched: spine exec (builtin) and spine-exec (REPL
      prefix handler, with session vars + preexec/postexec hooks).
      PROVEN on metal: spine-exec echo MY_VAR resolved to hello from real session state; the
      single-quoted form stayed literal while a resolver held that name -- the INT-174 bug class
      impossible by construction rather than avoided by convention; spine exec help reached a
      builtin; printf with a per-arg format proved quoting survives to argv as ONE argument; a
      failing command fired Friday's counter through postexec, so the hooks are not bypassed.
      OLD PATH LIVE THROUGHOUT: fsh-test 97/97 including all five REPL redirect tests and the
      INT-143 double-execution guards, plus 77 unit tests. Commits 12487484, c668dfce, d5dbffd0,
      97359732, c445345f, f9dca926. -->
- [ ] AST types defined (Command / Pipeline / Redirect / Assignment to start) + ExecContext holds the AST.
- [ ] The rides-with fixes: stop lowercasing cmd; SystemTime not u64; unique execution ID (lights up 167's
      correlation_id).
      <!-- NOT TICKED, and deliberately: three of four are done and deploy-validated, but the
      parenthetical is part of the acceptance condition, not commentary. Ticking would make the
      ledger say every prerequisite holds while one stated invariant does not.
      DONE: command identity preserves invocation case (ae081b82) -- case normalization moved to
      the consumers that need a lookup key, because identity and lookup were accidentally coupled.
      DONE: ExecContext.timestamp is SystemTime, with conversion to unix seconds at the DATABASE
      boundary rather than in the type (3e0f2f46).
      DONE: one process-local AtomicU64 across all three constructors, so every event, hook and
      trace referring to an execution carries the same id. INT-191 then found it insufficient
      ALONE -- it restarts at 1 in each shell -- so persistence keys on (session_id, execution_id).
      Deploy-validated gen 438, fsh-test 105/105.
      DEFERRED to INT-167: events.correlation_id is still a dead column. The id now EXISTS to put
      in it; nothing writes it. That is 167's contract, not the spine's, and this gate stays open
      until it lands so the dependency keeps its reason. -->
- [x] fsh still boots, logs in, deploys at EVERY step. No big-bang.
      <!-- DONE -- demonstrated across generations 411-421 (11 deploys this session). Every increment
      went to the daily driver and the shell kept booting, logging in, and deploying. The old from_line
      path stayed live throughout; nothing calls spine::plan::lower() on the live path. -->
- [ ] Each gate carries evidence per INT-158.

## PROGRESS 2026-07-21/22 -- the foundation is built, execution is NOT flipped

Built and deployed (all behind the old path, which still executes everything):
  src/spine/{ast,lexer,parser,render,plan,compare,migrate,migrate_audit,audit,golden,proptests}.rs
  - AST FROZEN (core-frozen, Redirect internals RESERVED): Span, Spanned<T>, AstNode, Command with
    per-word spans, Word, WordPart. Commit 25773f30.
  - ExecutionPlan FROZEN: argv Vec<OsString>, cwd, Environment enum, IoPlan reserved, lower() ->
    Result. Commit dbc94af2.
  - Tested three ways: unit, proptest torture (5 properties), goldens from real history. 49 tests.
  - `spine parse` and `spine migrate` builtins live.
  - Roadmap step 1 (bare commands), step 2 (quoted literals), and variable RECOGNITION landed.

Measured against 24,675 real single-line commands from shell_history:
  98.6% equivalent, 1.4% safe improvement, 0 language feature gaps, 3 unexpected (all paste debris).
  Found three real legacy bugs: cmd-word lowercasing corrupts case-sensitive env-assignment prefixes
  (SHELL= -> shell=); tokenize DROPS empty quoted args (`echo ""`); splitn mangles `VAR="a b"`.

⚠️ SCOPE CORRECTION found while measuring: the audit compares PARSERS on the same input string. The
live shell expands variables, subshells, globs and aliases BEFORE the legacy parser runs -- the spine
inverts that order (parse, then expand by walking words). Only recognition exists; nothing expands.
So parser equivalence is demonstrated; EXECUTION equivalence is not.

⚠️ GATE DISCREPANCY to resolve: the hybrid-lexer gate says "regular tokens via logos", but step 2
replaced logos on the word path with the hand-written stateful scanner (a regex alternation cannot
express quoting -- `foo"bar baz"` is one word). logos is currently an unused dependency. Either the
gate's wording needs revisiting or logos returns for the operator tokens at steps 5-7.
RESOLVED 2026-07-25 by an outside architecture review, which took the SECOND branch:
"logos isn't the lexer -- it's the engine for the easy pieces. The shell lexer is
still your state machine." So the split is by RESPONSIBILITY, not by coverage: the
stateful scanner owns context-sensitive regions (quoting, heredocs, command sub,
expansion) because that is where context-sensitivity lives and where the 172/174 bug
class came from; logos owns the regular operator tokens at steps 5-7. logos sitting
unused after step 2 is therefore EXPECTED MID-BUILD, not a failed gate. The gate's
wording stands as written and needs no revision.

## PROGRESS 2026-07-26/29 -- expansion and policy: blockers 4, 5 and 2
Six of nine flip blockers now done (1, 2, 3, 4, 5, 9), all with the same qualifier: implemented and
DEMONSTRATED, opt-in path only, not the default shell path. What remains is routing (6) and
dual-execution comparison (7) -- and those two ARE the flip.

COMMAND SUBSTITUTION (blocker 4). `$(...)` is lexed as a nested REGION, parsed into a nested AST,
lowered through its own entry point so the plan ASKS for captured output, run through a
`CommandRunner` capability, and spliced back into the surrounding word. No layer re-scans text.
Three things it deliberately did NOT do: no `WordPart::Glob`-style syntax variant for something
that is not syntax; no escaped-string pattern; no lifecycle row, because INT-191 ruled a
substitution is an EXPANSION, not a user command -- recording it would pollute history with text
nobody typed and inflate execution counts.

PATHNAME EXPANSION (blocker 5), which needed two prerequisites nobody had asked for.
First, QUOTING HAD TO SURVIVE PARSING: `*`, `'*'` and `"*"` all collapsed to the same Literal, so
glob eligibility could not be decided at expansion time at all. `WordPart::Literal` now carries the
`QuoteContext` as a FACT -- not a set of permission flags, because flags would encode policy for
expansions that do not exist yet, which this intent's own AST note forbids. Second, EXPANSION HAD TO
BE ONE-TO-MANY: a glob is not one word in, one argv entry out. The same signature now serves brace
expansion and sequences when they land. Only then did the glob itself arrive, as a third capability
alongside variables and substitution -- and the deciding argument was the audit: `spine migrate`
replays ~25,000 historical commands, and lowering that touched the filesystem would make an audit's
result depend on when and where it ran.

EXECUTION POLICY (blocker 2), and the scorecard entry for it was wrong. `from_plan` already existed,
deriving cmd and args from `plan.argv` with no tokenizing. What was missing was the POLICY half: a
nested command walked straight past preexec, so a substitution reached a process without meeting the
guard a typed command cannot avoid. Adding the call was a HALF FIX and would have been worse than
the gap -- the nested context has no source line, so the catastrophic-rm predicate substring-searched
an EMPTY string and silently passed everything, while the gate now looked closed. The real fix is
INT-196's thesis applied to the guard itself: the predicate takes an argument vector and must not
flatten it into text and re-parse it. It now reads `cmd` and `args` directly, which closed two
pre-existing holes as well -- separated flags (`rm -r -f /home`) were NEVER blocked by a substring
search, and a path containing a space could not be represented at all.

⚠️ THREE LIVE PROOFS WERE INVALID AND WERE RE-RUN. main.rs has THREE spine doors at three points in
the pipeline: `spine-exec` (hyphen) is intercepted BEFORE expand_vars, expand_subshells and
expand_globs, while `spine exec` (space) and `spine parse` are ordinary builtins dispatched AFTER all
of them. Probes through the wrong door measure LEGACY expansion and look identical when they
succeed. Blocker 4's original proof was measuring `expand_subshells`; blocker 5's was measuring
`expand_globs`. Both were re-verified through the hyphenated door, and blocker 1's was too. A debug
door's NAME is not evidence of where it sits.

★ AND THE STRONGEST EVIDENCE IN ALL OF IT WAS A HANG, not a passing test. A stale test fixture set an
empty argument vector while putting arguments in the display text -- a state no execution path can
produce. When the policy became structural it correctly found nothing to block, fell through to a
confirmation prompt, and waited forever. Narrowing the test filter would have hidden a live
regression behind a green run.

## PROGRESS 2026-07-29 -- THE SPINE IS CONNECTED (blockers 2, 6; operators; tilde)
Seven of nine flip blockers done. What remains is blocker 7, and its name promises something it
cannot deliver -- see below.

EXECUTION POLICY (blocker 2). A nested command walked past the only policy gate: `run_capture`
called dispatch, which performs no preexec, so a substitution reached a process without meeting the
guard a typed command cannot avoid. Adding the call was a HALF FIX and would have been worse than
the gap -- the nested context has no source line, so the catastrophic-rm predicate substring-searched
an EMPTY string and silently passed everything while the gate LOOKED closed. The real fix is
INT-196's thesis applied to the guard: it takes an argument vector and must not flatten it into text
and re-parse it. Reading `cmd` and `args` directly closed two pre-existing holes as well -- separated
flags (`rm -r -f /home`) were NEVER blocked by a substring search, and a path containing a space
could not be represented at all.

THE SPINE CAN NOW SAY NO. `TokenKind` had ONE variant, so `|` was ordinary word content and
`echo hi | grep h` parsed happily into five words and LOWERED -- a shell claiming a command it cannot
run. That mattered because routing needs something safe to route on, and neither candidate was:
`lower()` EXECUTES substitutions before it can discover what it does not support, so try-then-fall-
back would run one twice (the INT-143 class); `parse()` is pure but refused NOTHING. One refuses
nothing; the other executes before it refuses. The lexer now emits operator tokens and the parser
declines with a span. ★ This added OWNERSHIP VISIBILITY, not shell features -- nothing here implies a
pipeline will ever be executed by this path. An unsupported construct has THREE states, and the
middle one is what makes incremental replacement possible at all: invisible bug, then EXPLICIT
REFUSAL, then owned implementation.

THE ROUTER (blocker 6). After alias expansion and before `expand_vars`, a typed line is offered to
the spine: parse, lower with the full capability set, execute. NONE MEANS NOT MINE, and legacy
receives the original source exactly as if routing did not exist. Placement is the whole design --
routing below the legacy expansions would hand the spine already-transformed text and make its three
capabilities dead code. REFUSALS FALL BACK; DEFECTS SURFACE: parse errors and capability boundaries
go to legacy, but an invalid plan is REPORTED, because legacy must never be a recovery mechanism for
a spine bug. Behind `FSH_SPINE`, off by default.

★ THE TRACE FOUND A SILENT REGRESSION WITHIN MINUTES. Three probes had been spent trying to DEDUCE
which engine ran a command and every one was ambiguous. Four lines behind `FSH_SPINE_TRACE` turned
"did the behaviour differ?" into "which engine owned it?" -- and immediately showed the router
claiming `echo ~` and printing a literal tilde, because legacy expands one BELOW the routing point
and the spine had no tilde phase. No error, no crash, plausible output, wrong path. fsh-test has
SEVENTEEN tilde tests and not one would have caught it: they exercise the legacy path. Fixed in the
spine rather than by a router exception -- tilde is a word expansion and the spine owns word
expansion -- at word start only, unquoted only, HOME through the same resolver as the dollar form,
and the home arrives as a VALUE so a glob after it stays active.

MEASUREMENT. `spine migrate` now reports ZERO unexpected differences (was eight) and ZERO feature
gaps (was eighty) across 19,696 comparable commands, 99.5% equivalent. The eight became zero when
the AST learned to record whether a variable was written `$HOME` or `${HOME}` -- the information was
not wrong, it was absent. The eighty were never gaps: they are command substitutions the spine has
executed since blocker 4, which the audit cannot exercise because it lowers with no runner ON
PURPOSE. `LowerError` now distinguishes a construct it cannot run from a capability it was not given.

## STATUS 2026-07-29 -- THE SPINE IS THE DEFAULT PATH. 169 IS NOW *WAITING*, NOT WORKING.
`FSH_SPINE` stopped being an opt-in and became an escape hatch: unset routes to the spine,
`FSH_SPINE=0` restores legacy. Deployed gen 446, fsh-test 107/107 with routing on by default.

WHAT THE FLIP RESTS ON. The existing 107-test suite passes end to end through the router --
aliases, pipelines, redirects, heredocs, tilde expansion, SIGPIPE, the double-execution guards,
the recursion protection -- because fsh-test spawns the shell as a child and inherits the
environment, so one variable turned the whole compatibility contract into a spine integration test.
The migration audit reports ZERO unexpected differences and ZERO feature gaps across 19,696
comparable commands. Routing counters at the ownership boundary show the spine claiming what it
owns and declining exactly what it has refused since operator tokens landed. Both directions of the
toggle are proven: with the variable unset a tilde is claimed and expands, a pipeline is declined
and legacy runs it; with `FSH_SPINE=0` every command traces as disabled.

⚠️ ONE VISIBLE BEHAVIOUR CHANGE, stated rather than discovered: legacy never expanded a BARE tilde,
so `echo ~` printed the character. The spine expands it, which is what bash does. A correction
rather than a regression, in the same family as legacy dropping empty quoted arguments -- but users
notice output changes even when the new answer is the right one.

★ WHY THIS INTENT IS NOT CLOSING. Four gates are honestly unmet: logos is still an unused dependency
and the operator work argued it may never earn its place, since recognition must live inside the
scanner's quote state machine; `from_line` still has callers; the rides-with gate waits on INT-167's
correlation_id; and blocker 7 plus SIX legacy deletions remain. The deletions are what make the flip
PAY OFF, and their precondition was never "the audit says ready" -- it is the flip having HELD IN
REAL DAILY USE with the old path present and reversible.

★★ SO THE STATE CHANGED FROM WORKING TO WAITING. Nothing productive comes from writing more spine
code this week; what 169 needs is TIME with the default on and counters accumulating real numbers.
That does not block the other fsh intents -- most of them were waiting on this flip to unblock them.

## BLOCKER FOUND 2026-07-31 -- THE ROUTER NEVER SEES A BOOLEAN CHAIN

Proven live with the router trace on: a boolean chain produces correct output and NO trace line at
all, and a semicolon line produces TWO separate claims. main.rs splits the line on `;` (1294) and
then on `&&`/`||` (1332), and when the split yields more than one part it runs its own executor
(1333+) that tracks success and special-cases `cd`. The spine never receives a chain intact.

So `&&` is not a missing construct -- it is a PARALLEL EXECUTION PATH THAT PREDATES THE SPINE, and
the routing point sits below it. INT-200 built the Sequence AST and parser (7fa7056b) and they are
correct, but they have no live consumer until the router moves above the splitter.

WHY THIS BELONGS TO 169 rather than a new intent: 169 owns "connect the spine to fsh" and carries the
removal list of legacy execution paths. This splitter IS one of those paths. Filing it separately
would fragment ownership of the same seam.

⚠️ SIZED BEFORE, DELIBERATELY DEFERRED. INT-134 investigated exactly this, found the referenced
INT-267/322 were PHANTOM -- never filed -- and recorded: "the sh-routing fix landing -> major, since
it changes fsh's execution model." Those 378 boolean chains ALREADY WORK; moving them buys ownership
for milestone 2, not a fix. Judge it as a main.rs refactor of a working path, not as a spine feature.


## The Rule
"fsh already had a correct tokenizer -- twice -- and still broke, because nothing routed through one
structure. The spine is not a better parser. It is a single AST every path must go through, built one
construct at a time, in code Christian can read every line of." 🌲
