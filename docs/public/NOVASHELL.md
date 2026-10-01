# NovaShell

**Binary:** `nsh` -- the interactive shell of Project 0. `bash` stays the login shell.

> **Not the login shell, deliberately.** `/etc/passwd` names bash, and `~/.bashrc` starts nsh
> for interactive sessions. A broken nsh build costs a prompt, not a session.

---

## What NovaShell is

Not POSIX, not bash. Builtins return tables, and pipelines filter values instead of
re-parsing text. Where nsh has not modelled a construct, it hands the line to `sh` rather
than guessing.

```
POSIX shells   text  -> text   -> text
Nu shell       table -> filter -> transform
nsh            table -> filter -> judgment, and a refusal when it cannot answer
```

---

## Getting Started

```bash
nsh            # start the shell
nsh --help     # what it takes
```

```nsh
help           # what the shell knows
cheat          # every alias, read live from config.nsh
reload         # restart into the newly deployed nsh
exit           # or q
```

---

## Pipelines

Builtins return tables. These verbs work on them:

| Verb | Does | Example |
|------|------|---------|
| `where field op value` | keep matching rows | `ps \| where cpu > 5` |
| `select field1 field2` | keep columns | `ps \| select name pid` |
| `sort field` / `sort field desc` | order rows | `ps \| sort memory desc` |
| `first N` / `last N` | take rows | `et \| first 10` |
| `count` | count rows | `pkgs \| count` |
| `get field` | one field's values | `ps \| first 1 \| get name` |
| `group field` | group rows | `et \| group domain` |

Tables also pipe into ordinary tools and redirect to files:

```nsh
et | first 20 | grep doctor
ps | sort cpu desc | first 10 > top-processes.txt
```

Typed on their own, `where` and `select` are different builtins: `where <name>` locates a
command or file, and `select ...` runs a SQL query against state.db.

---

## Builtins That Return Tables

```nsh
ps              # processes
services        # systemd services
files [path]    # filesystem entries
net             # network interfaces
pkgs            # installed packages
logs            # system logs
et              # events
tt              # tools with audit scores
at              # audit scores
dt              # decisions
ht              # shell history
ct              # checkpoints
```

## Project 0 State

```nsh
health          # health summary from the last d
intents         # active intents
events          # recent events
tools           # tool deployment status
sandbox         # recent sandbox runs
checkpoint      # recent checkpoints
commits         # commit count and the last commit
version         # versions
histogram <f>   # frequency of a field
domains         # event domains
watch <cmd>     # re-run a command live
```

---

## Names Your Aliases Own

Aliases expand before builtins, so an alias with a builtin's name wins. Five do today:

| Name | Runs | Builtin it hides |
|------|------|------------------|
| `gc` | `git commit -m` | git commits table |
| `gf` | `git fetch` | git files changed |
| `ports` | `sudo ss -tulanp` | open ports table |
| `decisions` | `core decision list` | decisions table |
| `audit` | `core audit scan` | audit scores |

`d`, `forecast`, `story` and `advise` are aliases too, each for the matching `core` command.

---

## Job Control

nsh owns process groups and the terminal.

```nsh
sleep 300 &     # run in the background
jobs            # list jobs
fg 1            # resume job 1 in the foreground
bg 1            # continue job 1 in the background
```

Ctrl+Z suspends the foreground job, not the shell. A pipeline is one job in one process
group. Bare `fg` with no job number prints its usage.

---

## Sequences, Variables, and the Escape Hatch

```nsh
cd ~/0-core; d; et | first 3
let NAME = "value"
echo $NAME
export EDITOR = nvim
sh {
  awk '{print $1}' /etc/passwd | sort
}
```

---

## Natural Language

Prefix a question with `?`:

```nsh
? show health
? memory hogs
```

nsh translates it into a pipeline, shows it with a confidence level, and runs nothing until
you say yes. When it has no pattern, it says so instead of guessing.

---

## Configuration

`~/.config/nsh/config.nsh` is read at every start, and runtime aliases absent from it are
pruned. Edit it, then `reload`.

```nsh
alias ll = "ls"
set prompt_style = zero
```

---

## Signals

- **Ctrl+C** -- stops the foreground job; the shell survives
- **Ctrl+D** -- exits the shell
- **Ctrl+L** -- clears the screen (or `c`)
- `signals` shows what the shell does with each signal

---

## The Philosophy

NovaShell is not trying to replace bash. It is trying to replace the need for bash.

A tool that cannot answer must say so, rather than reporting an answer it never established.