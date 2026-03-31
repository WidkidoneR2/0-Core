# faelight-shell Grammar — INT-162 Phase 4
**Date:** 2026-03-31
**Status:** Factual — describes what fsh actually does today, not aspirational.

## Command Grammar
```
input      := command ("|" pipeline_op)* (";" command)*
command    := name arg*
name       := identifier | path
arg        := string | number | flag
flag       := "--" identifier ("=" value)?
```

## Pipeline Operators (all implemented in value.rs)
```
pipeline_op := where_op
             | select_op
             | sort_op
             | first_op
             | last_op
             | count_op
             | get_op
             | group_op
             | watch_op
             | join_op
             | external_op

where_op    := "where" field op value
               field  := identifier
               op     := "==" | "!=" | ">" | "<" | ">=" | "<=" | "contains"
               value  := string | number | bool

select_op   := "select" field+

sort_op     := "sort" field ("desc")?

first_op    := "first" number

last_op     := "last" number

count_op    := "count"

get_op      := "get" field

group_op    := "group" field

watch_op    := "watch" number?        # repeat every N seconds (default 2)

join_op     := "join" command "on" field

external_op := any unix command       # implicit — structured data serialized to text
```

## Value Types
```
Value ::= String(String)
        | Int(i64)
        | Float(f64)
        | Bool(bool)
        | Table(Vec<Row>)      # structured table — primary pipeline type
        | List(Vec<Value>)
        | Null

Row   ::= HashMap<String, Value>
```

## Pipeline Semantics
```
# Structured pipeline — stays typed throughout
gc | first 10 | where message contains "feat" | count

# External boundary — explicit text conversion
gc | first 10 | to-text | grep feat      # planned (Phase 3)

# Current behavior — implicit serialization (lossy)
gc | first 10 | grep feat                # works but loses structure
```

## .fsh Scripting Grammar
```
script     := statement+
statement  := let_stmt | if_stmt | emit_stmt | warn_stmt
            | confirm_stmt | run_stmt | command

let_stmt   := "let" identifier "=" value
if_stmt    := "if" condition "{" statement+ "}"
            | "when" condition "{" statement+ "}"
emit_stmt  := "emit" string
warn_stmt  := "warn" string
confirm_stmt := "confirm" string
run_stmt   := "run" path arg*

condition  := string                    # truthy string
            | identifier "==" value
```

## Multi-Command Separator
```
input := command (";" command)*

# Example:
gc | first 3; health; d
```

## External Command Passthrough
```
# Any unrecognized command → PATH lookup → execvp
nvim main.rs          # opens neovim
cargo build           # runs cargo
git status            # runs git
```

## Known Grammar Gaps (Phase 3 targets)
```
to-text    — explicit external boundary operator (not yet implemented)
map        — transform each row (not yet implemented)
reduce     — aggregate to single value (not yet implemented)
unique     — deduplicate by field (not yet implemented)
flatten    — expand nested tables (not yet implemented)
```

## DEC-005 Compliance

The grammar enforces the layer boundary:
- Pipeline operators operate on Value::Table (Layer 2 — Data Model)
- External commands receive text (explicit boundary)
- Forest state reads happen via core subprocess calls (Layer 3)
- Shell grammar owns tokenization and dispatch (Layer 1)
