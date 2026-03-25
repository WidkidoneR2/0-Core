# 🌲 faelight-shell — The Forest-Native Shell

> *"A forest deserves a shell that knows it is a forest."*

**Version:** v0.6.0 (Faelight Forest 11.2.0 — Will and Motion)  
**Status:** Active daily driver development — INT-146  
**Last updated:** 2026-03-25

---

## What is faelight-shell?

faelight-shell is not a POSIX shell. It is not bash. It is not fish. It is not Nu.

It is a **forest-native structured shell** — every command returns structured data,
every pipeline is composable, and the shell knows it is running inside a living system
that tracks its own health, goals, decisions, and history.

### The Model
```
Unix shells:       text | text | text
Nushell:           table | filter | transform
faelight-shell:    forest_data | judgment | wisdom
```

### The Compatibility Contract

faelight-shell is **NOT POSIX**. It does not run bash or zsh scripts.
For POSIX compatibility when needed, use the escape hatch:
```fsh
sh {
  awk '{print $1}' /etc/passwd | sort
}
```

---

## Getting Started

### Launch
```bash
fs          # alias for faelight-shell
faelight-shell
```

### Exit
```fsh
q
exit
```

### Help
```fsh
help        # show all commands
```

---

## Core Concepts

### 1. Structured Data
Every command returns a **table** — not text. Tables are pipeable,
filterable, sortable, and composable with unix tools.
```fsh
ps                          # processes as table
ps | sort cpu desc          # sorted by CPU
ps | sort cpu desc | first 5 # top 5
ps | where name contains niri # filtered
```

### 2. The Pipeline System
Pipe operators work on structured data:

| Operator | Description | Example |
|----------|-------------|---------|
| `| first N` | Take first N rows | `gc | first 10` |
| `| last N` | Take last N rows | `et | last 5` |
| `| where field op value` | Filter rows | `ps | where cpu > 5` |
| `| sort field` | Sort ascending | `tt | sort score` |
| `| sort field desc` | Sort descending | `ps | sort memory desc` |
| `| select field1 field2` | Select columns | `gc | select hash message` |
| `| count` | Count rows | `pkgs | count` |
| `| get field` | Extract a field value | `gc | first 1 | get hash` |
| `| group field` | Group by field | `et | group domain` |

### 3. Forest Awareness
The shell knows your system state at all times:
- Current health score
- Active intents
- Recent commits
- Session history
- Forest goals

---

## Command Reference

### Forest Commands
```fsh
health          # system health summary
d               # full doctor run (core doctor run)
forecast        # health trend and 24h/7d forecast
story           # 30-day forest narrative
advise          # judgment advisory from decision history
version         # system version
commits         # commit count and last commit
```

### Data Commands (return pipeable tables)
```fsh
gc              # git commits
gf              # git files changed
et [today|domain] # events
tt              # tools with audit scores
at              # audit scores
dt              # decisions
ht              # shell command history
ct              # checkpoints
ps              # processes
ports           # open ports
services        # systemd services
files [path]    # filesystem entries
net             # network interfaces
pkgs            # installed packages
logs [--follow] [--errors] # system logs
```

### Forest State Commands
```fsh
intents         # active intents
decisions       # open decisions
events [today]  # recent events
audit           # tool intelligence scores
tools           # tool deployment status
sandbox         # recent sandbox runs
checkpoint      # recent checkpoints
git             # git status and recent commits
```

### Analysis Commands
```fsh
histogram <field>   # frequency histogram of any field
domains             # event domain summary
watch <cmd>         # live-updating command
ps | watch          # live process monitor
ps | watch 5        # refresh every 5 seconds
```

### Shell Management
```fsh
alias name=command  # create alias
unalias name        # remove alias
plugins             # list loaded plugins
search <query>      # search command history
clear / c           # clear screen
cd ~/path           # change directory
```

---

## Pipelines to External Commands

Forest data flows directly into unix tools:
```fsh
gc | first 20 | grep feat       # filter commits by content
ps | sort cpu desc | first 5 | less  # paginate process table
gc | first 10 > commits.txt     # redirect to file
gc | first 10 >> commits.txt    # append to file
```

---

## Multi-Command Execution

Run multiple commands in sequence with `;`:
```fsh
health; d; git status
cd ~/0-core; gc | first 5; health
cargo --version; ls
```

---

## Shell Variables
```fsh
let NAME = "Faelight"       # define variable
let VERSION = "11.2.0"
let MSG = "Forest v$VERSION" # interpolation at assignment
echo $NAME                  # use variable
echo $MSG                   # Forest v11.2.0
export EDITOR = nvim        # set environment variable
echo $EDITOR                # nvim
```

---

## Background Jobs
```fsh
sleep 30 &          # run in background
cargo build &       # background build
jobs                # list running jobs
fg 1                # bring job 1 to foreground
kill %1             # kill job 1
```

When a background job completes, the forest announces it automatically:
```
✅ [1] cargo build — done (8.3s)
```

---

## Signals

- **Ctrl+C** — kills the foreground process, shell survives
- **Ctrl+D** — exit shell
- **Ctrl+L** — clear screen (or use `c`)

---

## Configuration File

Location: `~/.config/faelight-shell/config.fsh`
```fsh
# Aliases
alias ll = "ls"
alias gs = "git status"
alias gc5 = "gc | first 5"

# Settings
set history_limit = 10000
set prompt_style = forest
```

Loaded automatically on every startup. Edit and restart to apply.

---

## Natural Language Queries

Prefix any query with `?` to use natural language:
```fsh
?biggest files in this directory
?show me failing health checks
?memory hogs
?recent git commits
?what am I working on
```

The shell translates to a structured pipeline, shows you what it will run,
and asks for confirmation before executing.

---

## Interactive Features

- **↑/↓ arrows** — navigate command history
- **Ctrl+R** — reverse history search
- **Home/End** — jump to line start/end
- **Alt+Backspace** — delete word backwards
- History deduplication — no repeated consecutive entries
- Max history: 10,000 entries

---

## Real Examples
```fsh
# Find the 5 processes using most memory
ps | sort memory desc | first 5

# Show today's git activity
gc | where date contains "today"

# Find all feat commits this week
gc | first 50 | where message contains "feat"

# Check tool audit scores below 70
tt | where score < 70 | sort score

# Watch health live
health | watch 10

# Multi-step workflow
cd ~/0-core; d; gc | first 3

# Export results
ps | sort cpu desc | first 10 > top-processes.txt

# Pipe to unix tools
gc | first 20 | grep "INT-146"
gc | first 100 | wc -l

# Background job workflow
cargo build & jobs

# Variable workflow
let TOOL = "faelight-shell"
let VERSION = "0.6.0"
echo "Building $TOOL v$VERSION"
cargo build -p $TOOL
```

---

## What's Coming

| Phase | Feature | Impact |
|-------|---------|--------|
| Phase 17 | Prompt v2 — two-line, flow mode, error intelligence | Addictive daily driver feel |
| Phase 17b | Completion v1 — forest-aware tab completion | Stickiness |
| Phase 12 | Package helpers — `pkg install/remove/search/undo` | Composable package management |
| Phase 18 | Script ergonomics — `sh{}` escape hatch, typed lists | Full scripting capability |
| Phase 18b | Flow mode — intent continuity display | The unfair advantage |

---

## The Philosophy
```
faelight-shell is NOT trying to replace bash.
It is trying to replace the NEED for bash.
```

Every workflow you currently do in zsh, the forest can do better —
with structure, observability, and context that zsh will never have.

The shell is not a command runner. It is the forest's voice.

---

*"The shell that knows itself needs no other shell to complete it.
It grows until it becomes the ground you walk on."* 🌲

*This document grows with the shell. Last updated: 2026-03-25*
