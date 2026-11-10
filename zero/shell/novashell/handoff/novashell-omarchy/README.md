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

## Install

Build in this directory with `makepkg -f`. Christian installs the package
himself with `sudo pacman -U novashell-omarchy-<ver>-any.pkg.tar.zst`. That
is a root step. An agent never runs it. There is no pacman post_install.

If `~/.bashrc` still holds the inline 27-line 277 block,
`novashell-omarchy install` refuses and writes nothing. Christian deletes that
block by hand. Then `novashell-omarchy install` saves the sha256 of
`~/.bashrc` to `~/.local/state/novashell-omarchy/bashrc.sha256` and appends
this one line:

`[ -r /usr/share/novashell-omarchy/bashrc-handoff.sh ] && source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy`

`novashell-omarchy status` shows line, handoff, off and nsh.

`~/.config/nsh/off` stays in bash. `NSH_ACTIVE=1` stays in bash.
`omarchy-reinstall-configs` can wipe the marked line. The doctor warns, and
`novashell-omarchy install` puts the line back.

## Revert

`novashell-omarchy revert` deletes the marked line and checks the sha256 of
`~/.bashrc` against the saved value. Then `sudo pacman -R novashell-omarchy`
if wanted. A removed package with the line left behind is quiet, and bash
continues.

## Ownership

The package owns `/usr/share/novashell-omarchy/bashrc-handoff.sh` and
`/usr/bin/novashell-omarchy`. Christian owns `~/.bashrc`. The one marked
line is the only write there, and only when he runs the command. pacman
never touches `~/.bashrc`.

The canonical handoff is `zero/shell/novashell/handoff/bashrc-handoff.sh`
in 0-core. Change it there first.

This README is person-owned.

HAND-MAINTAINED
