---
id: 170
date: 2026-07-16
type: arch
title: "fsh plugin API: define initialize/execute/shutdown/metadata BEFORE picking a runtime -- plugins are text expansion today"
status: planned
tags: [fsh, plugins, api, wasmtime, rhai, mlua, libloading, phase4]
---

## Vision
Define a STABLE plugin interface. Then the runtime becomes a swappable detail instead of a decision.

    initialize()
    execute()
    shutdown()
    metadata()

"If that interface is stable, you can later support native Rust plugins, WebAssembly, Rhai, or other
runtimes WITHOUT CHANGING HOW THE SHELL INTERACTS WITH PLUGINS." (advisory, 2026-07-16)

## THIS DISSOLVES THE QUESTION IT WAS FILED TO ANSWER
The original framing was "pick one: wasmtime, rhai, or mlua". That is the wrong question and asking it
first is how you end up with a runtime looking for a use. Define the API; the runtime is an
implementation detail behind it. Christian was unsure which to pick -- correctly, because the choice
does not need making yet.

## MEASURED: what fsh has today (2026-07-16)
db.rs:192 `load_plugins()` returns `Vec<(command_name, expansion, description)>`, read from
~/.config/faelight-shell/plugins, resolved inside execute_impl right next to alias expansion (with
INT-057s cycle guard).

FSHS PLUGIN SYSTEM IS A TEXT EXPANDER. command -> string. That is not a criticism -- it works, and it
is the same shape as aliases, which are 285-strong and used constantly. But it means a plugin can
SUBSTITUTE and cannot COMPUTE.
THAT is the gap a runtime would fill. Not "wasmtime is cool" -- "a plugin cannot make a decision".

## Gate zero: name three things
Before any runtime is chosen, name THREE plugins Christian actually wants that a text expansion
CANNOT do. Real ones, not hypotheticals. If three cannot be named, CANCEL THIS INTENT -- a runtime
with no use is the definition of scope creep, and cancelling is a legitimate outcome (INT-110 is the
precedent: correctly cancelled, and the record of WHY is the value).
If three CAN be named, they are the API spec. The interface should be the smallest thing that serves
them.

## Then, and only then, the runtime -- decided by GOAL, not by fashion
The advisory frames it as three different goals, not three competing tools:
  scripting          -> Rhai      (users write logic; Rust-native, no C deps, easy embedding)
  sandboxing         -> Wasmtime  (untrusted or language-agnostic plugins, real isolation)
  native performance -> libloading / cdylib (fast, no isolation, full trust)
  (mlua              -> Lua, if the goal is an existing ecosystem of Lua authors -- fsh has none)
"Support additional runtimes only when they solve distinct use cases."
NOTE the existing safety layer: faelight-sandbox is deployed with 5 policies. Wasmtimes isolation
claim must be measured AGAINST that, not assumed on top of it -- the same question INT-165 asks of
AppArmor.

## Sequencing
AFTER INT-171 (one parsing entry point) and AFTER INT-168/169 settle. A plugin API that inspects
commands wants to know what a command IS -- which is the AST question (INT-169). Defining the API
before that is fine; IMPLEMENTING inspection before it is premature.
NOT PRE-OCTOBER.

## Success Criteria
- [ ] THREE real plugins named that text expansion cannot do. Written down. If not: CANCEL, and
      record why -- that record is the deliverable
- [ ] The API is defined and written down BEFORE any runtime is added. initialize/execute/shutdown/
      metadata, or whatever those three plugins actually need
- [ ] The EXISTING text-expansion plugins keep working through the new API -- it is a superset, not
      a replacement. Prove it with the existing plugin dir
- [ ] The runtime choice names the GOAL it serves (scripting / sandboxing / performance) and says
      what it measured, not what it read
- [ ] If wasmtime: its isolation is measured against faelight-sandboxs 5 existing policies. What
      does it add that we do not have?
- [ ] Exactly ONE runtime lands. A second requires a distinct use case, in writing
- [ ] Each gate carries evidence per INT-158

## The Rule
"Define the interface and the runtime becomes a detail. Pick the runtime first and the interface
becomes its shadow." 🌲
