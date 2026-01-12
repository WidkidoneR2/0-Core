# 🌲 Faelight Forest Login Screen

## Configuration
Location: `/etc/greetd/config.toml`

## Current Settings
```toml
[terminal]
vt = 1

[default_session]
command = "tuigreet --remember --remember-session --time --greeting '=== Welcome to Faelight Forest ===' --asterisks --asterisks-char '●' --cmd sway --window-padding 2 --container-padding 2 --theme 'border=green;text=white;prompt=green;time=green;action=green'"
user = "greeter"
```

## Theme Colors
- `border=green` - Box border
- `text=white` - Main text
- `prompt=green` - Input prompts
- `time=green` - Date/time display
- `action=green` - Action buttons

## Features
- Remember last username
- Remember last session
- Display time
- Password shown as ●●●●●●
- Centered greeting
- Padding for clean look

## To Edit
```bash
sudo nvim /etc/greetd/config.toml
```

---
_The forest welcomes you._ 🌲
