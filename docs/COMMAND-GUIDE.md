# 🌲 Faelight Forest — Command Guide
**Version:** v11.5.0 | **Updated:** 2026-03-31 | **Intents:** 140 complete

> Muscle memory reference. Only commands that work today.
> Health: 100% | Integrity: 100% | Jarvis: 90/100

---

## Every Session (non-negotiable)
| Command | What it does |
|---------|-------------|
| `d` | Full health + integrity check — run first, run last |
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
| `fs` | Launch faelight-shell |

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

## Deploy Pipeline
| Command | What it does |
|---------|-------------|
| `deploy <tool>` | Build + deploy a single tool |
| `deploy all` | Deploy all registry rust tools |
| `deploy list` | Show all deployable tools with type |
| `deploy check` | Detect missing deployable tools |
| `deploy core` | Deploy core engine |
| `deploy faelight-shell` | Deploy fsh |

---

## Intent System
| Command | What it does |
|---------|-------------|
| `cistart NNN` | Start intent NNN |
| `cicomplete NNN` | Complete intent NNN |
| `intent show NNN` | Show intent detail |
| `intent list` | All intents |
| `core intent new <type> <title>` | Create new intent |

---

## Core Intelligence
| Command | What it does |
|---------|-------------|
| `core predict sessions` | Work rhythm patterns |
| `core predict cadence` | Commit cadence + next week |
| `core predict health` | Health trajectory |
| `core predict accuracy` | Prediction accuracy dashboard |
| `core predict intents` | Intent backlog forecast |
| `core predict churn` | Highest churn files |
| `core react run` | Evaluate all reaction rules |
| `core react story` | Today's reaction narrative |
| `core doctor forecast` | Health forecast |
| `core stress report` | Full system verification |

---

## Core Strategy (v12)
| Command | What it does |
|---------|-------------|
| `core strategy next` | Top recommended intent |
| `core strategy next --list` | All intents ranked by score |
| `core strategy queue` | 5-session work plan |
| `core strategy blockers` | What is blocking most progress |
| `core strategy jarvis` | Jarvis readiness score (90/100) |
| `core strategy now` | What needs attention this session |
| `core strategy week` | Next 7 days focus |
| `core strategy quarter` | 90-day arc toward Jarvis |
| `core strategy gap` | Capability gaps to full Jarvis |
| `core strategy trust` | Evidence for expanded autonomy |
| `core strategy history` | Past strategies and outcomes |
| `core strategy review` | What worked, what didn't |

---

## Core Integrity (INT-184)
| Command | What it does |
|---------|-------------|
| `core integrity run` | Full integrity scan with repair |
| `core integrity status` | Current integrity score + pending |
| `core integrity log` | History of all detected issues |
| `core integrity fix` | Show pending proposals |

---

## Core Registry (INT-183)
| Command | What it does |
|---------|-------------|
| `core registry list` | All tools with type + status |
| `core registry show <name>` | Tool detail |
| `core registry retire <name>` | Mark tool as retired |
| `core registry unretire <name>` | Restore retired tool |

---

## Core Autonomy (INT-156 — DORMANT until 95/100)
| Command | What it does |
|---------|-------------|
| `core autonomy trust-score` | Current trust score + gate status |
| `core autonomy mandate-list` | Active mandates |
| `core autonomy mandate-set "<rule>"` | Define a new mandate |
| `core autonomy mandate-revoke <id>` | Revoke mandate by ID |
| `core autonomy mandate-revoke-all` | Return to manual mode |
| `core autonomy log` | Autonomy action log |
| `core autonomy pending` | Pending autonomous actions |

---

## Core DB (INT-166)
| Command | What it does |
|---------|-------------|
| `core db backup` | Manual state.db backup |
| `core db verify` | Verify backup integrity |
| `core db status` | DB size + WAL status |
| `core db compact` | VACUUM database |

---

## Context & Memory (INT-159, INT-160)
| Command | What it does |
|---------|-------------|
| `faelight-context scan .` | Index codebase |
| `faelight-context map ~/0-core` | Architectural domain map |
| `faelight-context patterns ~/0-core` | Convention detection |
| `faelight-context summary ~/0-core` | Natural language overview |
| `faelight-memory show` | All stored knowledge |
| `faelight-memory add --category <cat> "<fact>"` | Teach the forest |
| `faelight-memory query <topic>` | Search knowledge |
| `faelight-memory extract` | Auto-extract from session history |
| `faelight-memory confidence` | Confidence distribution |

---

## fsh Shell Commands (INT-162, INT-174, INT-176, INT-177)
| Command | What it does |
|---------|-------------|
| `last_command` | Show last failed command |
| `last_command retry` | Re-run last failed command |
| `last_command explain` | Explain the failure |
| `last_command fix` | Suggest corrected command |
| `failures` | Session failure log |
| `last_error` | Last structured error |
| `errors` | Error history |
| `observe session` | Current session summary |
| `observe commands` | Most used commands |
| `observe diff` | Changes vs last session |
| `observe anomalies` | Unusual patterns detected |
| `observe patterns` | Learned command patterns |

---

## Forest Tools
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `bump` | faelight-release publish |
| `sec` | security-audit |
| `fm` | faelight-fm |
| `vault` | faelight-vault |
| `fdocs` | faelight-docs |
| `fdocs sync` | Sync README + welcome |
| `fdocs check` | Verify docs are current |
| `fdocs links` | Verify all README links |

---

## Weekly
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `core security scan` | Security check |
| `core stress report` | After major changes |
| `alias-audit` | Verify alias health |
| `core integrity run` | Full integrity scan |
| `faelight-memory extract` | Extract new session insights |
| `deploy check` | Verify all tools deployed |

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

## fsh Builtins
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
| `docs/AUTOSTART-MAP.md` | Niri autostart chain |
