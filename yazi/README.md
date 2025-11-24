# Yazi Vim-Style Keymap

This folder contains the configuration for **Yazi**, a terminal-based file manager, with a **Vim-inspired keybinding system** for efficient navigation, file operations, and macros.

---

## Features

### Vim-Like Navigation
- `h` → move left
- `j` → move down
- `k` → move up
- `l` → move right

### Operators
- `d + motion` → delete range (supports multi-line)
- `c + motion` → rename range
- `y + motion` → yank (copy) range
- `.` → repeat last operator action

### Macros
- `q<register>` → start recording macro
- `@<register>` → replay macro

### Ex-Commands
- `:rename <newname>` → rename a file
- `:mkdir <foldername>` → create a new folder
- `:trash` → delete selected file
- `:preview!` → preview file contents

---

## Installation

1. **Clone your dotfiles repository** (if not already):

```bash
git clone https://github.com/<your-username>/dotfiles.git ~/dotfiles

