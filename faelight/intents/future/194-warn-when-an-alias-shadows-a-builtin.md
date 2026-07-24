---
id: 194
date: 2026-07-24
type: feature
title: "warn when an alias shadows a builtin"
status: planned
tags: [feature, fsh, aliases, ergonomics, discoverability]
---

## Vision

Shadowing a builtin stays allowed. It stops being silent.

## The Problem

`shell_aliases` contains `gc` = `git commit -m`. `help` advertises a BUILTIN `gc`
-- "git commits as table -- pipeable". Alias expansion runs before builtin dispatch
on every path, so the alias wins and the builtin becomes unreachable.

Confirmed live: invoking it produced `error: switch 'm' requires a value`, which
is `git commit -m` running.

### It silently broke a second alias

`gc5` = `gc | first 5` was clearly written expecting the TABLE builtin -- piping
into `first 5` only makes sense against a table. It expands to
`git commit -m | first 5`, which is nonsense. A user-authored alias, permanently
broken by a name collision, with NOTHING anywhere reporting that a builtin was
displaced.

## The Solution

⚠️ NOT a precedence change. Aliases taking precedence over builtins is standard
shell behaviour -- bash resolves aliases -> functions -> builtins -> PATH, and
matching that is defensible. Changing precedence would break every alias that
deliberately overrides a builtin.

The problem is DISCOVERABILITY, so the fix is a warning at definition time:

    alias gc="git commit -m"
    ⚠️  alias 'gc' shadows the builtin 'gc' (git commits as table -- pipeable)

Once, when the alias is created. Ignorable. Behaviour unchanged.

Open questions for implementation:
  - warn only at `alias` creation, or also when loading config.fsh at startup?
    (config.fsh defines 285 aliases; warning for each on every start would be
    noise -- probably a startup SUMMARY if any shadow, or nothing)
  - is there a `shadow` / `builtins` listing command that should show collisions?

## Why this is separate from INT-193

INT-193 consolidates alias expansion so every alias expands exactly once. This
warning is NOT required to fix that bug, and folding them together would let it
quietly not get done during implementation -- the correctness work would land and
the ergonomics would be dropped as optional.

Different class of work: one is a correctness invariant, one is discoverability.

## Success Criteria

- [ ] Behaviour is UNCHANGED -- aliases still take precedence over builtins
- [ ] A warning is emitted when an alias is created whose name matches a builtin
- [ ] The warning names the builtin being shadowed, so the user knows what they lost
- [ ] Emitted ONCE at definition, not on every invocation
- [ ] Decided and recorded: what happens at startup when config.fsh defines a
      shadowing alias (per-alias warning, a summary, or silence)
- [ ] Existing shadowing aliases surveyed once, so known collisions are visible
      rather than discovered accidentally (`gc` is the one found; there may be more)
