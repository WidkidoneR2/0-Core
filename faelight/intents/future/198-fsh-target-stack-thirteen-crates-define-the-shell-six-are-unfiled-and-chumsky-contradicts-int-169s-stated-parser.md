---
id: 198
date: 2026-07-26
type: future
title: "fsh target stack: thirteen crates define the shell, six are unfiled, and chumsky contradicts INT-169's stated parser"
status: planned
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
