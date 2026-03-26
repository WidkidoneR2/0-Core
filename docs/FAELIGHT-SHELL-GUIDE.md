# 🌲 faelight-shell — User Guide
> *"Not a POSIX shell. Not bash. Not Nu. The forest's own voice."*

**Version:** v0.6.0 | Faelight Forest 11.3.0 | Last updated: 2026-03-26

---

## What is faelight-shell?

faelight-shell is a forest-native shell environment. It is not a POSIX shell replacement — it is a structured, observable, self-aware computing environment expressed through its own native language.
```
text    → Unix shells (bash, zsh)
tables  → Nushell
forest  → faelight-shell
```

Every command is forest-aware. History is structured data. The prompt is a live instrument.

---

## Starting faelight-shell
```bash
fs          # launch via alias
faelight-shell  # direct
```

---

## The Prompt
```
🌲 ~/0-core (main*)
→ 100% · 5 today
fsh ❯
```

- **Path** — current directory, shortened
- **Git branch** — with dirty indicator (*)
- **Health %** — live system health
- **Today** — commands run this session

---

## Core Commands

### Forest Commands
```
d / health      — run doctor, show system health
gc              — git commits as structured table
since yesterday — everything that changed since a time
usage           — % of commands handled natively
debug last      — what fsh did with the last command
debug reactions — reaction engine state
debug preexec   — active guards and hooks
```

### Navigation
```
cd <path>       — change directory (feeds zoxide)
z <keyword>     — jump to frecent directory (zoxide)
ls              — eza with icons
ll              — eza long format with icons
ya / yazi       — file manager with cd-on-quit
fm              — faelight-fm with cd-on-quit
```

### Forest Data
```
intents         — show intent ledger
events          — show event stream
health          — health score
forecast        — health forecast
commits         — recent commits
story           — today's forest narrative
since <time>    — forest timeline since a point
```

### Pipelines
```
gc | first 5                    — last 5 commits
ps | sort cpu desc | first 5    — top CPU processes
gc | where message contains "fix"  — filter commits
histogram command               — command frequency
```

### Prediction (Core v11)
```
core predict sessions   — when do you typically work?
core predict health     — health trajectory forecast
core predict next       — what intent ships next?
core predict coupling   — architectural risk domains
core predict churn      — highest churn files
core predict accuracy   — model confidence
```

### Reaction Engine (Core v10)
```
core react run          — evaluate all rules now
core react list         — show rules and cooldown state
core react story        — today's reaction narrative
core react discipline-show — decay and coalesce config
```

---

## Natural Language Queries

Prefix any query with `?`:
```
? what files changed today
? which intents are in progress
? how is the forest doing
```

---

## Pipelines

faelight-shell uses structured data pipelines:
```
command | filter | transform | display
```

Pipeline operators:
- `first N` — take first N rows
- `last N` — take last N rows
- `sort <field> asc|desc` — sort by field
- `where <field> contains|=|>|< <value>` — filter
- `count` — count rows
- `select <fields>` — pick columns

---

## Config File

`~/.config/faelight-shell/config.fsh`
```
alias d = "core doctor run"
alias gc = "git-commits"
alias v = "nvim"
set prompt_health = true
set history_size = 10000
```

---

## Flow Mode

Focus your shell on an active intent:
```
flow focus INT-146    — set active intent
flow                  — show current focus
flow clear            — clear focus
```

Prompt updates to show active intent when flow is set.

---

## Git Guardrail

When core is locked, git commits are blocked:
```
fsh ❯ git commit -m "test"
🔒 Core is LOCKED — editing blocked
✗ No commits, pushes or changes allowed while locked
→ Run: unlock-core — then make your changes
```

---

## POSIX Escape Hatch

For scripts that require POSIX compatibility:
```
sh {
  awk '{print $1}' /etc/passwd | sort
}
```

---

## Daily Driver Status
```
fsh ❯ usage
▶ Command Coverage
  · 92% handled by fsh natively
  · 8% forwarded to PATH
▶ Migration Confidence: 🟢 HIGH
```

---

## See Also

- `help` — full command list
- `debug last` — transparency into shell behavior
- `core predict next` — what to work on next
- `docs/FAELIGHT-SHELL.md` — technical reference
