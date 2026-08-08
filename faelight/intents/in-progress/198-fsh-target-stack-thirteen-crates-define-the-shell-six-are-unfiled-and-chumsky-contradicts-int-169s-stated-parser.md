---
id: 198
date: 2026-07-26
type: future
title: "fsh target stack: thirteen crates define the shell, six are unfiled, and chumsky contradicts INT-169's stated parser"
status: in-progress
tags: [fsh, chumsky, reedline, Crossterm, Nu-ansi-term, nix, Tracing+Miette]
---

## Vision
The document that governs fsh's future describes the shell fsh WAS BECOMING before the
spine existed. This intent records the architecture that actually defines it, judges each
piece against the roadmap's own filter, and settles the one decision that currently blocks
parser work.

## The Problem
1. **The roadmap has no architecture lane.** `docs/fsh-evolution-roadmap.md` is 170 lines
   across ten lanes and mentions the spine ZERO times -- not INT-169, not the parse/plan/
   execute rebuild, not the observation layer (189/191/192), not one of the thirteen crates
   below. It is a FEATURE roadmap, and the last month of work has been ARCHITECTURE. The
   two do not intersect anywhere in the document.
2. **Six of thirteen have no intent at all:** Chumsky, Tokio, Crossterm, Nu-ansi-term,
   Nucleo, Tracing + Miette. These are not imagined work -- they were stated as the target,
   so filing them is a record rather than a wish.
3. ⚠️ **Chumsky CONTRADICTS INT-169.** 169's own title is *"logos lexer + HANDWRITTEN
   recursive-descent parser -> AST"*, and `spine/lexer.rs` records WHY: quoting is
   context-sensitive, a regex alternation cannot express it (`foo"bar baz"` is one word and
   a non-whitespace-run regex already gets it wrong), so a hand-written stateful scanner
   finds word boundaries while logos stays for the structurally-regular operator tokens. A
   parser-combinator library is a DIFFERENT architecture from the one being built. Until
   this is settled every further hand-written parser step is potentially throwaway -- and
   blocker 4 step 1b, the parser half of command substitution, is exactly that step.

## THE STACK, AND WHERE EACH PIECE STANDS TODAY
    Reedline      interactive line editing      -> INT-168 (in-progress, HELD on INT-171)
    Logos         lexing                        -> INT-169 (partial: hybrid, operators only)
    Chumsky       parsing                       -> NOWHERE, and contradicts INT-169
    Tree-sitter   highlighting + semantics      -> INT-186 (planned)
    Tokio         async execution               -> roadmap "Async jobs with futures", unfiled
    Nix           signals, job control, pgroups -> INT-188 (planned)
    Crossterm     terminal I/O                  -> NOWHERE
    Nu-ansi-term  prompt styling                -> NOWHERE
    Nucleo        fuzzy completion              -> roadmap "Fuzzy command completion", unfiled
    Gix           git for prompt and commands   -> INT-187 (planned, "NOT a felt need yet")
    Serde + TOML  configuration                 -> partially exists (config.fsh, INT-060)
    Tracing       logging                       -> NOWHERE
    Miette        rich diagnostics              -> NOWHERE

★ REEDLINE IS WORTH MORE THAN ITS OWN INTENT SUGGESTS: it unlocks FIVE unchecked UX items
at once -- multi-line editing, vim mode, emacs mode, undo/redo, fish-style autosuggestions.
The roadmap lists them as five separate wishes; they are one dependency.

## The Solution
Decide, do not build. Every piece gets KEEP-with-lane-and-rough-order or CUT-with-the-reason
against the filter the roadmap already states:

> A feature earns a place only if it deepens understanding + authorized, reproducible
> control. Opaque convenience and auto-magic are cut.

The parser question gets a decision record stating the cost of BOTH paths honestly:
chumsky discards the hand-written parser and lexer-hybrid work already landed, while the
hybrid exists precisely because quoting is context-sensitive -- so the decision must show
that a combinator over a flat token stream does not hit the same wall, rather than assuming
it will not.

## Explicitly out of scope
Building any of it. Kept items spawn their own intents; this one produces a judged,
sequenced stack. If this charter starts growing implementation work, it has failed -- the
same fence INT-134 sets, for the same reason.
ALSO out of scope: reconciling the roadmap from v3.1.0 to v3.6.3. That is INT-134's standing
rule ("reconciled by Christian at each fsh version bump") and belongs there, not here --
though the five-version drift is why the architecture gap went unnoticed.

## THE RULINGS (each on evidence, recorded as it is made)

REEDLINE -- CUT. Decided 2026-08-06, against Christian's own earlier preference: "I am going to go
with cancelling this even though I fought hard to have it." INT-168's own gate 4 set the test -- at
least one thing reedline does that rustyline could not, demonstrated -- and the answer was nothing.
Multi-line editing shipped on rustyline the same day; hinting already existed and does more than
fish, with a history prefix plus a Friday-patterns fallback; a Highlighter is implemented; menus
exist as CompletionType::List with completion_show_all_if_ambiguous; and vi mode landed too, which
was not even on reedline's list. Set against a swap on the code between his fingers and the shell in
his daily driver, where the intent itself says a regression is not a bug report but an unusable
terminal. INT-168 is cancelled with the evidence recorded as the reason.

TREE-SITTER -- KEEP. His ruling, same day. Lane: INT-186, after the spine grammar settles, because
defining fsh's grammar a second time for highlighting while the first one is still moving means
chasing a target.

LOGOS -- CUT, and the code says why in its own words. The hybrid decision kept logos for
structurally-regular OPERATOR tokens while a hand-written stateful scanner owned words, because
quoting is context-sensitive. Operators have since landed -- in the scanner. spine/lexer.rs's
operator_at carries the finding: "Called ONLY in unquoted context. echo "|" is data, and the caller's
state machine is the only thing that knows the difference -- which is precisely why a stateless
pattern matcher cannot own this and logos stays deferred." The function is a fourteen-line match on a
character and its successor, covering all nine operators with no regex and no state of its own. So
logos would have to be called from INSIDE the quote state machine to match two characters, which is
absurd overhead for a match arm, or allowed to scan context-free, which breaks echo "|". The word
half of the hybrid won on merit and the operator half turned out not to need a lexer generator
either. logos is currently an unused dependency with zero source references.

⭐ AND THIS RULING SHARPENS THE PARSER QUESTION RATHER THAN COMPLICATING IT. The wall the hybrid was
built to avoid is at the LEXER level -- quote context -- not at the parser level. A combinator over a
TOKEN stream never meets it, because the lexer has already resolved quoting. That separates "lexing
is context-sensitive" from "parsing is not", which is the distinction the chumsky decision turns on.

⚠️ CONSEQUENCE FOR INT-169: its gate asks for "regular tokens via logos", which the design has since
rejected on evidence. That gate cannot be ticked honestly as written and must be reworded to describe
the hybrid that exists.

TREE-SITTER -- REVERSE / DEFER. This reverses the KEEP ruling of 2026-08-06, on a better reason:
two grammars, two sources of truth. Tree-sitter would introduce a second shell grammar alongside the
canonical one, creating synchronisation and semantic-drift costs across parsing, highlighting,
completion and diagnostics. INT-186 becomes AST/CST-driven SEMANTIC highlighting from the canonical
grammar -- git as a command, commit as a subcommand, --amend as an option, foo.txt as an argument,
which is more than syntax highlighting can give. Reconsider tree-sitter only if a concrete
incremental-editing requirement cannot be met by the canonical parser.

⚠️ AND THE GUARD THAT MATTERS MORE THAN THE RULING: "tree-sitter reversed" does NOT mean "incremental
parsing rejected." The thing being rejected is a SECOND GRAMMAR. Incremental parsing remains a
desirable property, and the canonical architecture must be evaluated on whether it can provide it
before another grammar is introduced. This paragraph exists so a future ticket cannot reintroduce
tree-sitter merely because the words "incremental parsing" appear in its justification.

CHUMSKY -- KEEP. A good fit for the canonical interactive parser, with recursive parsers, error
recovery and debugging support -- which are the properties an interactive language needs. It replaces
parser.rs, not the scanner: the context-sensitivity of quoting is resolved in the lexer, so a
combinator over a TOKEN stream never meets the wall the hybrid was built to avoid. Measured cost
today: roughly fifty lines discarded, and rising as the parser grows.

INT-169 -- REDESIGN, and it stops being a lexer ticket. Its responsibility becomes shell-aware
scanning and parsing producing a canonical ParseResult capable of representing complete, incomplete
and invalid interactive input: source, then a shell-aware scanner, then Chumsky, then a ParseResult of
Complete, Incomplete or Invalid.

⭐ THE INCOMPLETE STATE IS THE POINT, AND IT HAS A CONSUMER TODAY. fsh's multi-line Validator lexes
and reports Incomplete on an unterminated quote, then needs a special case for heredocs and a
comment-stripping pass, because apostrophes in ordinary English prose hung the prompt twice. Those
hacks exist because the parser can say "error" but not "expecting a closing quote, mode DoubleQuote".
An explicit Incomplete state replaces both, and highlighting, completion, diagnostics and heredoc
handling all fall out of the same fact.

GATE 115 -- REWORDED AS PART OF THE 169 REDESIGN, not as a wording cleanup. The old contract was that
the parser produces an AST or an error; the new one is that it produces a ParseResult explicitly
modelling complete, incomplete and invalid input. That changes how the gate evaluates the parser and
how downstream work understands parser completion. The five gates already done survive, because they
concern the spine existing and working rather than which crate lexes.

TOKIO -- OPTIONAL, NOT CORE. A shell is not naturally an async application: fork, exec, waitpid,
setpgid, tcsetpgrp, SIGCHLD, SIGINT, SIGTSTP, terminal input and PTY are OS process-control
operations, and nothing is gained by turning them into async fn. The execution architecture is the OS
process model, with a synchronous job-control core. Tokio stays available for asynchronous output,
network-backed commands, plugin APIs, background tasks and completion providers. fsh's only
async-shaped execution today is backgrounding, whose model is already explicit -- spawn, never wait,
JobTable::check_completed polls, and a background job gets no tee because there is nothing to tee
into. Any tokio decision inherits that model rather than starting fresh.

NIX -- KEEP INITIALLY, AND INVESTIGATE RUSTIX. The role was described as "signals, job control,
pgroups", which is architecturally misleading: nix is an implementation BACKEND, not the job-control
model. Define a ProcessBackend trait -- spawn, set_process_group, foreground, send_signal, wait --
and let a Linux backend sit behind it. The interesting direction is not which wrapper crate but the
Linux primitives: process groups, waitid, pidfds, signalfd, epoll, termios, controlling terminals.
Lane: INT-188, which got easier once backgrounding moved beside try_jobs, try_fg and try_kill.

CROSSTERM -- KEEP AS INFRASTRUCTURE, BEHIND AN ABSTRACTION. Do not build reedline, crossterm and a
styling crate as three independent terminal layers. One shell terminal abstraction, with the editor
and job control beneath it. Ratatui becomes interesting only if a full-screen UI actually exists --
not because it is appealing.

NU-ANSI-TERM -- REPLACE, and not with itself. The incumbent is `colored`, already in scope and used
for every marker in the shell, so this was never an addition. The better model is anstyle: a small
interoperable representation of styling, so a PromptSegment carries semantic style and the renderer
decides whether it becomes ANSI, plain text, JSON or a test snapshot. Prompt rendering stops
depending on one ANSI implementation.

NUCLEO -- KEEP, and go beyond fuzzy matching. Make completion a PROVIDER architecture -- command,
argument, option, path, environment, alias, history, git, plugin -- each receiving a CompletionContext
of source, cursor, ast, current node, cwd and environment. Completion stops being "find strings
starting with gi" and becomes "I am in the first argument of git checkout". Nucleo is then the
ranking engine rather than the feature.

GIX -- DEFER, and keep git out of the core. Its own ledger entry already says "NOT a felt need yet".
Git becomes a PromptProvider, which also opens the same seam for nix, docker, kubernetes, ssh and
direnv providers without any of them infecting the shell.

SERDE + TOML -- KEEP, with a layered configuration model. config.fsh must not stay the only
configuration concept: defaults, system, user, project, environment, runtime state. And distinguish
configuration (durable preference) from runtime state (cwd, jobs, history, last status) and session
state (temporary overrides). On NixOS the configuration should eventually be generatable declaratively
from Nix WITHOUT the shell depending on Nix.

TRACING -- ADD NOW, and this is the strongest live case in the stack. fsh is a process manager and
already needs observability: FSH_SPINE_TRACE is a bare eprintln behind an env check, and it was the
decisive tool twice in one week -- it proved the router claims a redirected background line, and it
proved jobs and kill are excluded as REPL-state commands. Spawning, pgid, job state, signals and
terminal foreground are exactly the things that become undebuggable without it.

MIETTE -- ADD NOW, AS A RENDERER ONLY. The internal diagnostic model stays independent: severity,
message, primary span, labels, help, code. Miette then renders it for a terminal, while the same
model can produce JSON, an editor-facing diagnostic, or a snapshot in a test. Judge it with INT-199,
whose thesis -- every failure answers what happened, what changed, why, and what to do next -- miette
is the mechanism for rather than a competitor to.

## Success Criteria
- [ ] Each of the thirteen judged against the filter: KEEP with lane and rough order, or CUT
      with the reason. No piece left unjudged
- [ ] The parser decision recorded, with the cost of BOTH paths stated -- including what
      chumsky would discard and whether a combinator meets the context-sensitivity wall the
      hybrid was built to avoid
- [ ] Every KEEP without an intent either gets one filed or is explicitly deferred with the
      reason. Six currently qualify
- [ ] The roadmap gains an architecture lane, or this intent records why architecture stays
      outside it
- [ ] Nothing built. The close condition is a decision anyone can act on, not a diff
