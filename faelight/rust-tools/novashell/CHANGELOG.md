# Changelog -- NovaShell (nsh)

## 🎉 3.0.0 -- The Nix Crossing
- 🌲 The shell crossed from Arch to NixOS -- fully native, no package-manager assumptions, no distro lock-in.
- 🗑️ Removed the Arch package commands (`pkg`, `pkgs`, `sys_packages`) -- a breaking change; the forest no longer speaks pacman.
- ✨ Candy-neon powerline prompt: a two-line prompt where the directory segment's color tells you what kind of place you're in.
- ✨ Multi-command paste blocks -- paste many lines, each runs as its own command, heredocs and quotes stay intact.
- ♻️ Command-snapshot schema split cleanly from health snapshots -- one purpose per table.
- ♻️ Relocated into the unified `faelight/` tree.

## 2.x -- The Shell Grows Up
- ✨ Tab completion, structured output, session save/load/replay.
- ✨ Natural-language `?` prefix -- ask the shell in plain words.
- ✨ Parallel execution (`|||`) -- true concurrent commands.
- ⚡ Startup and rendering improvements.

## Earlier
- 🌲 Born as the forest's own shell: speaks human first, UNIX as fallback. The daily driver.
