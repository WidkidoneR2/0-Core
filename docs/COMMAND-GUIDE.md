# 🌲 Faelight Forest — Command Guide
**Version:** v11.4.0 | **Updated:** 2026-03-30 | **Aliases:** 368

> Muscle memory reference. Only commands that work today.
> Replaces: ALIASES.md, QUICK_REFERENCE.md

---

## Every Session (non-negotiable)
| Command | What it does |
|---------|-------------|
| `d` | Full health check — run first, run last |
| `lock-core` | Lock core files — before shutdown |
| `unlock-core` | Unlock core files — before editing |
| `fg commit` | Forest git commit — after any changes |
| `cistart NNN` | Start an intent — before any intent work |
| `cicomplete NNN` | Complete an intent — after intent done |

---

## Daily Tools
| Command | What it does |
|---------|-------------|
| `v` | Neovim |
| `g` | Git |
| `l` | eza list (short) |
| `ll` | eza list (long, git column) |
| `ya` | Yazi file manager |
| `b` | Bat viewer |
| `c` | Clear terminal |
| `lg` | Lazygit |
| `top` | btm (better htop) |
| `loc` | Lines of code stats |

---

## Navigation
| Command | What it does |
|---------|-------------|
| `0core` | cd ~/0-core |
| `src` | cd ~/1-src |
| `work` | cd ~/2-work |
| `keep` | cd ~/3-keep |
| `tmp` | cd ~/9-temp |
| `conf` | cd ~/.config |
| `cdp` | cd - (previous dir) |
| `..` `...` `....` | cd up 1 / 2 / 3 levels |

---

## Git
| Command | What it does |
|---------|-------------|
| `fg commit` | Forest commit (always use this) |
| `gst` | git status |
| `gaa` | git add -A |
| `gc` | git commit -m |
| `gp` | git push |
| `gl` | git pull |
| `gd` | git diff |
| `glog` | git log --oneline -10 |
| `gco` | git checkout |
| `gcb` | git checkout -b |

---

## Forest Tools
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `bump` | faelight-release publish |
| `sec` | security-audit |
| `fm` | faelight-fm |
| `vault` | faelight-vault |
| `clip` | faelight-clipboard |
| `fdocs` | faelight-docs |

---

## Intent System
| Command | What it does |
|---------|-------------|
| `cistart NNN` | Start intent NNN |
| `cicomplete NNN` | Complete intent NNN |
| `intent show NNN` | Show intent detail |
| `intent list` | All intents |

---

## Core Intelligence
| Command | What it does |
|---------|-------------|
| `predict next` | Next intent prediction |
| `predict sessions` | Work rhythm patterns |
| `predict health` | Health trajectory |
| `react run` | Evaluate all rules now |
| `react story` | Today's reaction narrative |
| `doctor forecast` | Health forecast |
| `core stress report` | Full system verification |

---

## Weekly
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `core security scan` | Security check |
| `core stress report` | After major changes |
| `alias-audit` | Verify alias health |

---

## System
| Command | What it does |
|---------|-------------|
| `sr` | Reboot |
| `ssn` | Shutdown now |
| `ports` | Active network ports |
| `myip` | External IP |
| `paci` | Install package |
| `pacr` | Remove package |
| `pacs` | Search packages |

---

## fsh Builtins (native shell commands)
| Command | What it does |
|---------|-------------|
| `pwd` | Current directory |
| `which <cmd>` | Where command resolves |
| `type <cmd>` | Full resolution detail |
| `env` | Shell environment table |
| `theme <name>` | Switch prompt theme |
| `clear` / `c` | Clear terminal |
| `cat <file>` | View file |

---

## Prompt Themes
| Theme | What it shows |
|-------|-------------|
| `forest` | Path, git, health, commits |
| `minimal` | Path only |
| `classic` | user@host path $ |
| `jarvis` | Forest + prediction inline |

---

## Documentation
| File | Contents |
|------|---------|
| `docs/COMMAND-GUIDE.md` | This file |
| `docs/KEYBINDINGS.md` | All 116 keybindings |
| `docs/ARCHITECTURE.md` | System architecture |
| `docs/PHILOSOPHY.md` | Design principles |
