# faelight-shell Scripting Story — INT-162 Phase 6
**Date:** 2026-03-31
**Status:** Factual — documents what .fsh scripting actually implements today.
**Source:** rust-tools/faelight-shell/src/scripting.rs (402 lines)

## Philosophy
.fsh scripts are not shell scripts.
They are forest behavior expressions — structured, observable, deterministic.
"Not for automating tasks. For expressing forest behavior."

## Script Execution
```bash
run deploy.fsh              # execute a .fsh script
run deploy.fsh arg1 arg2    # with positional arguments ($1, $2)
```

Scripts are loaded, parsed, and executed by the scripting engine.
Each statement is executed in order. No hidden state. No ambient reads.

## Statement Grammar (all implemented)

### let — variable binding
```fsh
let name = value
let tool = "faelight-shell"
let count = 42
let path = $HOME
```
Variables are scoped to the script execution.
`$varname` syntax for interpolation in strings.

### if — conditional execution
```fsh
if condition {
    run some-command
    emit "event.name"
}
```
Condition is a truthy string expression.
Body executes only if condition is true.

### when — event handler
```fsh
when "tool.deployed" {
    warn "A tool was just deployed"
    run core doctor run
}
```
Triggers on forest events. Registered via the event system.

### run — execute a command
```fsh
run cargo build --release -p faelight-shell
run core doctor run
run deploy.fsh              # nested script execution
```
Runs any command available in PATH or fsh builtins.
Exit code propagated correctly.

### emit — fire a forest event
```fsh
emit "tool.deployed" { name: "faelight-shell", version: "0.6.0" }
emit "session.started"
```
Events are written to state.db and trigger any registered `when` handlers.

### warn — display a warning
```fsh
warn "This action is irreversible"
warn "Health is below 95%"
```
Prints a warning message. Does not halt execution.

### confirm — require user confirmation
```fsh
confirm "Are you sure you want to delete this?"
```
Prompts user for y/n. Halts script if user declines.

## Variable Interpolation
```fsh
let tool = "faelight-shell"
run cargo build --release -p $tool
warn "Deploying $tool"
```
`$varname` is interpolated in string arguments.
`$HOME`, `$1`, `$2` etc. from environment/args also available.

## Positional Arguments
```fsh
# deploy.fsh
let tool = $1
let version = $2
run cargo build --release -p $tool
emit "tool.deployed" { name: $tool, version: $version }
```
Invoked as: `run deploy.fsh faelight-shell 0.6.0`

## Design Constraints
These are intentional — not limitations:

**No for loops** — forest scripts express behavior, not iteration.
Use pipeline operators (`first`, `where`, `group`) for data processing.

**No function definitions** — scripts are linear behavior sequences.
Complex reuse belongs in core domains (Rust), not scripts.

**No POSIX compatibility** — deliberate. The forest is not trying to
replace bash. It replaces the NEED for bash.
For POSIX when needed: use the escape hatch via external commands.

**No hidden state** — scripts cannot read ambient forest state.
All state access is explicit via `run core ...` commands.

## Example Scripts

### health-gate.fsh
```fsh
confirm "Run full deployment?"
run core doctor run
warn "Deploying faelight-shell..."
run cargo build --release -p faelight-shell
emit "tool.deployed" { name: "faelight-shell" }
```

### intent-start.fsh
```fsh
let id = $1
confirm "Start INT-$id?"
run cistart $id
emit "intent.started" { id: $id }
```

## Known Gaps (future work)
```
for <item> in <list> { }   — iteration (not yet implemented)
fn name(args) { }          — function definition (not planned — by design)
return value               — early return (not yet implemented)
import "other.fsh"         — script imports (not yet implemented)
```

## Testing Scripts
Every .fsh script should be testable in isolation:
```bash
run script.fsh --dry-run   # planned — Phase 28 / INT-175
run script.fsh --trace     # planned — INT-175 Script Debug Mode
```

## DEC-005 Compliance
Scripts fire events into state.db for causal linkage.
Scripts call core via subprocess — no direct state reads.
Policy lives in core. Scripts express intent, not implementation.
