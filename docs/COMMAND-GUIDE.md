# Project 0 -- Command Reference

**System:** Omarchy (Arch) + Hyprland + Project 0

> The commands you reach for daily. Every line was checked against `core --help` and
> `~/.config/nsh/config.nsh`. For the full list of a domain, run `core <domain> --help`.

---

## Daily Commands

### Health & Status
```bash
d                          # core doctor run -- health, integrity, and a verdict
core status                # state in one coherent narrative
core doctor trend          # also: quick, forecast, history, entropy, bins, aliases
```

### Intent Ledger
```bash
intl                       # core intent list
ints <id>                  # core intent show -- one intent in full
inta                       # core intent add -- file a new intent
cistart <id>               # core intent start
cicomplete <id>            # core intent complete -- refuses while a gate is open (intc does the same)
core intent next           # recommendation on what to work on
core intent stats          # velocity and completion metrics
core intent brief          # session brief
core intent defer <id> "reason"     # defer a gate with a reason
core intent override <id> "reason"  # override a gate, with an audit log entry
```

### Friday
```bash
friday-chat                # interactive Friday session
core friday status         # Friday system status
core knowledge show <key>  # one knowledge entry; also: search, patterns, add
```

### Tools
```bash
yazi                       # file manager (the retired fm's replacement)
zdocs                      # zero-docs -- the documentation engine
zg                         # zero-git -- git workflow governance
core security scan         # security audit; also: report, debt, trend, history
cheat                      # cheatsheet TUI -- every alias, read live from config.nsh
reload                     # restart into the newly deployed nsh
```

---

## Git Commands

```bash
g                          # git
gs                         # git status
gc "message"               # git commit -m
gp                         # git push
lg                         # lazygit -- interactive git TUI
core git status            # core's git view; also: risk, log, verify
```

---

## Core Engine Commands

### Sandbox
```bash
core sandbox status        # sandbox state; also: run, diff, snapshot, snapshots, restore, clear
```

### Profile
```bash
core profile status        # current profile
core profile list          # all profiles
core profile switch <name> # switch profile
```

### Events & History
```bash
core events list           # the event ledger
core journal today         # the system's own journal; also: yesterday, week, search
core why summary           # causality -- why the system is in this state
core trace last            # trace event history; also: domain
```

### Prediction & Intelligence
```bash
core predict next          # prediction engine
core react list            # reaction engine
core goals list            # goal engine
```

---

## Key Bindings

Omarchy owns the Hyprland key bindings and lists them in its own menu. This guide does not
copy them: a copy goes stale the day Omarchy changes a bind.

---

## Related Documentation

- [PHILOSOPHY.md](PHILOSOPHY.md) -- Core principles
- [ALIASES.md](ALIASES.md) -- where the aliases live and how to read them
- [NOVASHELL.md](NOVASHELL.md) -- NovaShell documentation
- [ARCHITECTURE.md](ARCHITECTURE.md) -- how the pieces fit
