# novashell-omarchy

Bash keeps the login seat. When a terminal opens, one marked line in
`~/.bashrc` sources the handoff, and nsh runs as a child of that bash. If nsh
crashes, the window stays open and you are still in bash.

What comes through is the environment, and the PATH entries named when the
handoff was proved:

- `OMARCHY_PATH`
- the mise shims directory
- `~/.local/bin`
- `~/.cargo/bin`

Nothing else from Omarchy's bash rc is copied. No alias is translated.

## What does not carry

nsh has no bash functions, and an alias copied across would hide nsh builtins.
These four would:

- `c` would hide clear
- `d` would hide the doctor. d is the doctor
- `h` would hide help
- `ls` would hide the directory listing

Omarchy's other aliases (`eza` for `ls`, `cd` via zoxide, the git shortcuts,
and the rest of `default/bash/aliases`) stay in bash for the same reason.

These stay in bash too:

- bash functions, including `try` and the mise functions
- starship. nsh draws its own prompt
- mise activation hooks. `mise activate bash` is a bash hook. The shims on
  PATH are the part nsh keeps
- fzf key bindings
- bash completions

`z` and `zi` are already nsh commands, so zoxide does not need a translation.

A line missing from this list is a failed test, not a quiet omission.

## The files

The package installs the handoff at
`/usr/share/novashell-omarchy/bashrc-handoff.sh` and the command at
`/usr/bin/novashell-omarchy`.

The package owns that file under `/usr`. The marked line is the only write in
`~/.bashrc`, and only when you run the command. `install` adds the line.
`revert` removes it and checks the sha256 taken before the edit. pacman does
not touch `~/.bashrc`.
