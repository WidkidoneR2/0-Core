# 🌲 Faelight Forest — Command Guide
**Version:** v11.7.0 | **Updated:** 2026-04-09 | **Intents:** 167 complete
> Muscle memory reference. Only commands that work today.
> Health: 100% | Integrity: 100% | Jarvis: 105/100
---
| Command | What it does |
|---------|-------------|
| `d` | Full health + integrity check — run first, run last |
| `lock-core` | Lock core files — before shutdown |
| `unlock-core` | Unlock core files — before editing |
| `fg commit` | Forest git commit — after any changes |
| `cistart NNN` | Start an intent — before any intent work |
| `cicomplete NNN` | Complete an intent — after intent done |
---
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
| Command | What it does |
|---------|-------------|
| `fg commit` | Forest commit — always use this |
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
| Command | What it does |
|---------|-------------|
| `deploy <tool>` | Build + deploy a single tool |
| `deploy all` | Deploy all registry rust tools |
| `deploy list` | Show all deployable tools with type |
| `deploy check` | Detect missing deployable tools |
| `deploy core` | Deploy core engine |
| `deploy faelight-shell` | Deploy fsh |
---
| Command | What it does |
|---------|-------------|
| `cistart NNN` | Start intent NNN |
| `cicomplete NNN` | Complete intent NNN |
| `intent show NNN` | Show intent detail |
| `intent list` | All intents |
| `core intent new <type> <title>` | Create new intent |
---
| Command | What it does |
|---------|-------------|
| `core predict next` | What intent ships next (weight-ranked) |
| `core predict health` | Health trajectory forecast |
| `core predict session` | Current session pattern analysis |
| `core predict accuracy` | How accurate have predictions been |
| `core predict velocity` | Intent completion velocity |
| `core predict sessions` | Work rhythm patterns |
| `core predict cadence` | Commit cadence + next week |
| `core predict intents` | Intent backlog forecast |
| `core predict churn` | Highest churn files |
---
| Command | What it does |
|---------|-------------|
| `core react run` | Evaluate all reaction rules |
| `core react story` | Today reaction narrative |
| `core react rules` | List all active reaction rules |
| `core react status` | Current rule firing status |
---
| Command | What it does |
|---------|-------------|
| `core strategy next` | Top recommended intent |
| `core strategy next --list` | All intents ranked by score |
| `core strategy queue` | 5-session work plan |
| `core strategy blockers` | What is blocking most progress |
| `core strategy jarvis` | Jarvis readiness score |
| `core strategy now` | What needs attention this session |
| `core strategy week` | Next 7 days focus |
| `core strategy quarter` | 90-day arc |
| `core strategy gap` | Capability gaps |
| `core strategy trust` | Evidence for expanded autonomy |
| `core strategy history` | Past strategies and outcomes |
| `core strategy review` | What worked, what did not |
---
| Command | What it does |
|---------|-------------|
| `core autonomy trust-score` | Current trust score + gate status |
| `core autonomy mandate-list` | Active mandates |
| `core autonomy mandate-set` | Define a new mandate |
| `core autonomy mandate-revoke <id>` | Revoke mandate by ID |
| `core autonomy mandate-revoke-all` | Return to manual mode |
| `core autonomy log` | Autonomy action log |
| `core autonomy pending` | Pending autonomous actions |
---
| Command | What it does |
|---------|-------------|
| `core partner status` | Partner system status and readiness |
| `core partner propose` | Forest proposes a new intent |
| `core partner discuss <id>` | Forest opinion on an intent |
| `core partner disagree <id>` | Forest respectfully pushes back |
| `core partner consult <q>` | Consult before making a decision |
| `core partner reflect` | What has the forest learned about you |
| `core partner pattern` | Patterns that define how you work |
| `core partner growth` | How the system has grown over time |
| `core partner pushback` | Recent pushback moments |
| `core partner roadmap` | Forest view of optimal path forward |
| `core partner roadmap-why` | Why this roadmap |
| `core partner roadmap-diff` | How forest roadmap differs from current |
---
| Command | What it does |
|---------|-------------|
| `core self map` | Architecture coupling analysis |
| `core self evolve` | Generate structural proposals |
| `core self apply` | Apply a proposal (use --dry-run first) |
| `core self history` | Evolution audit trail |
| `core self learn` | Record outcome of a proposal |
| `core self accuracy` | Proposal accuracy over time |
| `core self calibrate` | Adjust proposal thresholds |
| `core self challenge` | Stress test a plan |
---
| Command | What it does |
|---------|-------------|
| `core weight list` | All patterns ranked by weight |
| `core weight top` | Critical and Strong patterns only |
| `core weight compute` | Scan events and compute weights |
| `core weight explain <id>` | Full weight breakdown for a pattern |
| `core weight calibrate <id> <outcome>` | Record outcome for calibration |
---
| Command | What it does |
|---------|-------------|
| `core delegate simulate` | Simulate delegation without executing |
| `core delegate contracts` | List trust contracts and status |
| `core delegate history` | Delegation simulation history |
| `core delegate accuracy` | Simulation accuracy over time |
| `core delegate suspend` | Suspend all delegation instantly |
| `core delegate counterfactuals` | Counterfactual comparison log |
| `core delegate accuracy-report` | Three-dimensional accuracy report |
---
| Command | What it does |
|---------|-------------|
| `core engines status` | All engines and their sync state |
| `core engines sync` | Acknowledge engine upgrade |
| `core engines signals` | Recent cross-engine signals |
| `core engines check` | Verify all engines consistent |
| `core engines upgrade-log` | Engine upgrade history |
---
| Command | What it does |
|---------|-------------|
| `core why summary` | Why did the system do what it did today |
| `core why health` | Why is health at its current level |
| `core why domain <d>` | What has a domain been doing |
| `core why chain` | Full causal chain for last health drop |
| `core why correlate` | Correlate two domains |
| `core why suggest` | Proactive suggestions from patterns |
| `core why focus` | Focus quality over time |
| `core why attention` | Attention analysis |
---
| Command | What it does |
|---------|-------------|
| `core decision record` | Record a new decision with snapshot |
| `core decision outcome` | Record outcome of a decision |
| `core decision list` | List recorded decisions |
| `core decision hindsight` | Hindsight summary |
| `core decision show <id>` | Full detail for a decision |
| `core decision stats` | Correlation stats |
| `core decision advise` | Judgment advisory for current state |
| `core decision heuristics` | Auto-derived heuristics |
| `core decision lessons` | Human-readable lessons summary |
| `core decision story` | 30-day narrative |
| `core decision patterns` | Repeating decision patterns |
| `core decision friction` | Decisions requiring repeated corrections |
---
| Command | What it does |
|---------|-------------|
| `core values list` | All declared values |
| `core values define` | Declare a new value |
| `core values remove <id>` | Deactivate a value |
| `core values weight <id>` | Update value weight |
| `core align check` | Check alignment against declared values |
| `core align drift` | Behavioral drift report (30 days) |
| `core align report` | Weekly alignment report |
---
| Command | What it does |
|---------|-------------|
| `core journal today` | Today journal entries |
| `core journal yesterday` | Yesterday journal entries |
| `core journal week` | This week journal entries |
| `core journal search <kw>` | Search journal by keyword |
| `core journal show <date>` | Journal for a specific date |
---
| Command | What it does |
|---------|-------------|
| `core integrity run` | Full integrity scan with repair |
| `core integrity status` | Current integrity score + pending |
| `core integrity log` | History of all detected issues |
| `core integrity fix` | Show pending proposals |
---
| Command | What it does |
|---------|-------------|
| `core anomaly scan` | Detect unexpected system changes |
| `core anomaly history` | Anomaly detection history |
| `core anomaly alert` | Surface high-severity anomalies |
---
| Command | What it does |
|---------|-------------|
| `core narrative` | Full story of how the forest became what it is |
| `core narrative --since <ver>` | Narrative since a version |
| `core narrative --intent <id>` | Narrative for a specific intent |
---
| Command | What it does |
|---------|-------------|
| `core db backup` | Manual state.db backup |
| `core db verify` | Verify backup integrity |
| `core db status` | DB size + WAL status |
| `core db compact` | VACUUM database |
---
| Command | What it does |
|---------|-------------|
| `core registry list` | All tools with type + status |
| `core registry show <name>` | Tool detail |
| `core registry retire <name>` | Mark tool as retired |
| `core registry unretire <name>` | Restore retired tool |
---
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `bump` | faelight-release publish |
| `sec` | security-audit |
| `fm` | faelight-fm |
| `vault` | faelight-vault |
| `fdocs` | faelight-docs |
---
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
| Theme | What it shows |
|-------|-------------|
| `forest` | Path, git, health, commits |
| `minimal` | Path only |
| `classic` | user@host path $ |
| `jarvis` | Forest + prediction inline |
---
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
| Command | What it does |
|---------|-------------|
| `fu` | faelight-update |
| `core security scan` | Security check |
| `core stress report` | After major changes |
| `alias-audit` | Verify alias health |
| `core integrity run` | Full integrity scan |
| `deploy check` | Verify all tools deployed |
---
| File | Contents |
|------|---------|
| docs/COMMAND-GUIDE.md | This file |
| docs/KEYBINDINGS.md | All 114 keybindings |
| docs/ARCHITECTURE.md | System architecture |
| docs/PHILOSOPHY.md | Design principles |
| docs/AUTOSTART-MAP.md | Niri autostart chain |
| docs/core-commands.md | Core subcommand deep reference |
