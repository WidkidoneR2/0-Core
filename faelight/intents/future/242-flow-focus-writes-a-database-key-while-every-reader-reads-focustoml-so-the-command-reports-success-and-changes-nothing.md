---
id: 242
date: 2026-09-05
type: fix
title: "flow focus writes a database key while every reader reads focus.toml so the command reports success and changes nothing"
status: planned
tags: [fix, bugfix]
---

## Vision
A command that reports success has changed something. `flow focus` sets the
focus that every reader reads.

## The Problem
Found 2026-09-05 while migrating the shell's intent readers onto the INT-230
adapter. **`flow focus` writes one storage and every reader reads another.**

```
db::set_focus_intent   ->  INSERT INTO shell_state (key='focus_intent', ...)
db::get_focus_intent   ->  reads ~/.local/state/0-core/intent/focus.toml
```

A setter and a getter sharing a name and not a storage. Measured: `shell_state`
contains **no `focus_intent` key at all**, while `focus.toml` holds
`id = "230"`, written by `cistart` at 23:51:17.

### Three defects, not one
1. **`flow focus INT-231` prints `focus set -> INT-231` in green and changes
   nothing any reader observes.** A success message for a write nobody reads.
2. **`flow clear` clears a key nobody reads**, so focus survives being cleared.
3. **The formats differ.** `flow` requires and stores `INT-231`; `focus.toml`
   stores `230` bare, and `main.rs:2547` does
   `get_focus_intent().map(|i| format!("INT-{}", i))` -- confirming readers
   expect the bare form. Even sharing a storage, the values would not match.

### Who reads focus
Eleven call sites use `get_focus_intent`, including `db.rs:329`, which stamps
**every shell-history row** with it. So the value is not cosmetic; it is
attached to the historical record.

### The source of truth is not in doubt
`engine/src/domains/friday/attention.rs:98` says it outright: *"read focus.toml
(written by cistart, source of truth)"*. The file carries id, title, started and
workflow.

## The Solution
`focus.toml` wins -- it is what `cistart` writes, what the engine calls
canonical, and what every reader already reads.

⭐ **SO THE FIX IS MOST LIKELY A DELETION**, which is the shape that has been
right repeatedly in this codebase: remove `set_focus_intent` and
`clear_focus_intent`, and have `flow focus` / `flow clear` either write
`focus.toml` through one owner or refuse and point at `cistart`.

⚠️ **A RULING IS NEEDED BEFORE CODE**: should `flow focus` be able to set focus
at all, or is focus something only `cistart` sets? If the shell may set it, the
write needs the same owner the engine uses -- a second writer of `focus.toml`
would reproduce this defect in a new place.

📍 `db.rs:495` hand-builds the focus.toml path from `env::var("HOME")` and never
asks `paths.rs`. INT-230 added `core_integration::focus()` which routes through
`paths`, so there are currently THREE readers of that file. Consolidating them
is part of this.

## Success Criteria
- [ ] G1 RED FIRST: `flow focus INT-999` prints success, and `flow status`
      afterwards reports something else. Captured verbatim
- [ ] G2 THE RULING IS RECORDED: may the shell set focus, or is that `cistart`
      only? Written here before any code
- [ ] G3 ONE STORAGE. Whatever the ruling, exactly one location holds focus and
      `grep` proves no second writer exists
- [ ] G4 ONE READER. The three current readers of focus.toml
      (`db.rs:495`, `commands/mod.rs:14753`, `core_integration::focus`) become
      one, routed through `paths.rs`
- [ ] G5 THE ID FORMAT IS SETTLED and stated: bare `230` or prefixed `INT-230`,
      with every producer and consumer agreeing
- [ ] G6 `flow clear` demonstrably clears what `flow status` reads
- [ ] G7 If `flow focus` is retained, a regression test asserts that setting
      focus is observable by `get_focus_intent` in the same session
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- The focus feature's design. This is about a write and a read disagreeing.
- INT-230's adapter boundary. `core_integration::focus()` reads correctly today;
  it is one of the three readers this intent consolidates.
