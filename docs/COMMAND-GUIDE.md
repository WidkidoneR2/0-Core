# 🌲 Faelight Forest — Command Guide
**Version:** v11.8.0 | **Updated:** 2026-04-13 | **Intents:** 176 complete
> Muscle memory reference. Only commands that work today.
> Health: 100% | Integrity: 100% | Jarvis: 105/100

---

## Every Session (non-negotiable)

| Command | What it does |
|---------|-------------|
| d | Full health + integrity check — run first, run last |
| lock-core | Lock core files — before shutdown |
| unlock-core | Unlock core files — before editing |
| fg commit | Forest git commit — after any changes |
| cistart NNN | Start an intent — before any intent work |
| cicomplete NNN | Complete an intent — after intent done |

---

## Daily Tools

| Command | What it does |
|---------|-------------|
| v | Neovim |
| g | Git |
| l | eza list (short) |
| ll | eza list (long, git column) |
| ya | Yazi file manager |
| b | Bat viewer |
| c | Clear terminal |
| lg | Lazygit |
| top | btm (better htop) |
| loc | Lines of code stats |
| fs | Launch faelight-shell |

---

## Navigation

| Command | What it does |
|---------|-------------|
| 0core | cd ~/0-core |
| src | cd ~/1-src |
| work | cd ~/2-work |
| keep | cd ~/3-keep |
| tmp | cd ~/9-temp |
| conf | cd ~/.config |
| cdp | cd - (previous dir) |
| .. ... .... | cd up 1 / 2 / 3 levels |

---

## Git

| Command | What it does |
|---------|-------------|
| fg commit | Forest commit — always use this |
| gst | git status |
| gaa | git add -A |
| gc | git commit -m |
| gp | git push |
| gl | git pull |
| gd | git diff |
| glog | git log --oneline -10 |
| gco | git checkout |
| gcb | git checkout -b |

---

## Deploy Pipeline

| Command | What it does |
|---------|-------------|
| deploy <tool> | Build + deploy a single tool |
| deploy all | Deploy all registry rust tools |
| deploy list | Show all deployable tools with type |
| deploy check | Detect missing deployable tools |
| deploy core | Deploy core engine |
| deploy faelight-shell | Deploy fsh |

---

| Command | What it does |
|---------|-------------|
| faelight-link status-v3 | Per-package health with intent tracing + CRITICAL markers |
| faelight-link audit-v3 | Intent traceability -- which intent owns each package |
| faelight-link verify | Deep validation -- valid/broken/unlinked counts per package |
| faelight-link why ~/.config/X | Which package owns this file and why |
| faelight-link stow <pkg> | Stow a package (create symlinks) |
| faelight-link unstow <pkg> | Unstow a package (remove symlinks) |

| Command | What it does |
|---------|-------------|
| fg commit | Smart commit -- auto-links intent, risk warnings, velocity check |
| fg rollback --list | Show last 10 commits with risk scores |
| fg rollback N | Interactive rollback to commit N |
| fg rollback --dry-run | Preview rollback without executing |

## Core Deploy Intelligence (INT-222)
| Command | What it does |
|---------|-------------|
| core deploy check <tool> | Pre-deploy health gate + dependency warning |
| core deploy record <tool> <ver> <outcome> | Log deploy pattern to state.db |
| core deploy log | Recent deploy history with health + timing |
| core deploy rollback <tool> | Rollback tool to previous version |
| core deploy rollback --dry-run | Preview rollback without executing |
| core deploy check-deps <tool> | Show full dependency graph |

| Command | What it does |
|---------|-------------|
| faelight-docs status | All docs, freshness, drift indicators |
| faelight-docs sync | Sync all auto-updatable fields |
| faelight-docs check | Identify stale documents (dry run) |
| faelight-docs diff | Show drift since last recorded state |
| faelight-docs record <doc> [note] | Record a manual update to state.db |
| faelight-docs log | Full document update history |
| faelight-docs why <doc> | Which intent owns this document |
| faelight-docs welcome | Regenerate zshrc welcome message |
| faelight-docs readme | Regenerate README static section |

| Command | What it does |
|---------|-------------|
| core events status-v2 | Show forest_events_v2 health, counts, signal types |
| core events emit-v2 <type> <payload> | Emit validated signal to canonical log |
| core events emit-v2 <type> <payload> --caused-by SEQ | Emit with causality link |
| core events replay <from> <to> | Replay signals from sequence range |
| core events chain <seq> | Show full causality chain for a signal |
**Signal types:** health, git_commit, intent_start, intent_complete, deploy, alignment, prediction, watchdog_alert

## Intent System

| Command | What it does |
|---------|-------------|
| cistart NNN | Start intent NNN |
| cicomplete NNN | Complete intent NNN |
| intent show NNN | Show intent detail |
| intent list | All intents |
| core intent new <type> <title> | Create new intent |

---

## fsh Native Builtins (INT-223)

| Command | What it does |
|---------|-------------|
| query file.rs 100:150 | Print lines 100-150 |
| query file.rs :50 | First 50 lines |
| query file.rs 900: | Line 900 to end |
| query file.rs 100:+30 | 30 lines from line 100 |
| query file.rs "pattern" | Lines containing pattern |
| fsearch "pattern" | Recursive search all text files |
| fsearch "pattern" --type rs | Search only .rs files |
| fsearch "pattern" --file main.rs | Search specific file |
| patch file.rs --old "x" --new "y" | In-place find-and-replace (unique match required) |
| edit file.rs | Open file in $EDITOR |
| edit file.rs:150 | Open file at line 150 |
| edit file.rs:fn_main | Open file at first pattern match |
| run file.py | Execute Python script natively |
| run file.sh | Execute shell script natively |
| run file.fsh | Execute fsh native script |

---

| Command | What it does |
|---------|-------------|
| gc | Abbreviation → fg commit |
| gp | Abbreviation → fg push |
| dep | Abbreviation → deploy |
| ds NNN | Abbreviation → cistart NNN |
| dc NNN | Abbreviation → cicomplete NNN |
| fsh diag | Shell health -- sessions, focus score, peak velocity, slow commands |
| fsh gaps | Missed builtin opportunities -- grep vs fsearch, head vs query, /tmp scripts |

## fsh v5 Builtins (INT-224)
| Command | What it does |
|---------|-------------|
| show file.rs 46:80 | Syntax-highlighted code view |
| show file.rs fn_main | Jump to function with color |
| goto file.rs:362 | Open editor at exact line |
| goto file.rs:362:5 | Open editor at line:col (from cargo errors) |
| goto "fn name" | Find function and open editor at it |
| rename old new | Rename across all files with confirmation |
| rename old new --dry-run | Preview rename without writing |
| rename old new --type rs | Rename only in .rs files |
| fdiff file.rs | git diff for specific file |
| fdiff file.rs HEAD~3 | Diff against older commit |
| fdiff file.rs --stat | Summary diff only |
| patch-multi file.rs old1 -- new1 old2 -- new2 | Atomic multi-replacement |
| ht today | Commands from today with timing |
| ht session | Commands from current session |
| ht intent | History grouped by active intent |
| ht slow | Commands that took >5s |

## Core Intelligence — Prediction (v11)

| Command | What it does |
|---------|-------------|
| core predict next | What intent ships next (weight-ranked) |
| core predict health | Health trajectory forecast |
| core predict session | Current session pattern analysis |
| core predict accuracy | How accurate have predictions been |
| core predict velocity | Intent completion velocity |
| core predict sessions | Work rhythm patterns |
| core predict cadence | Commit cadence + next week |
| core predict intents | Intent backlog forecast |
| core predict churn | Highest churn files |

---

## Core Intelligence — Reaction (v10)

| Command | What it does |
|---------|-------------|
| core react run | Evaluate all reaction rules |
| core react story | Today reaction narrative |
| core react rules | List all active reaction rules |
| core react status | Current rule firing status |

---

## Core Strategy (v12)

| Command | What it does |
|---------|-------------|
| core strategy next | Top recommended intent |
| core strategy next --list | All intents ranked by score |
| core strategy queue | 5-session work plan |
| core strategy blockers | What is blocking most progress |
| core strategy jarvis | Jarvis readiness score |
| core strategy now | What needs attention this session |
| core strategy week | Next 7 days focus |
| core strategy quarter | 90-day arc |
| core strategy gap | Capability gaps |
| core strategy trust | Evidence for expanded autonomy |
| core strategy history | Past strategies and outcomes |
| core strategy review | What worked, what did not |

---

## Core Autonomy (v13)

| Command | What it does |
|---------|-------------|
| core autonomy trust-score | Current trust score + gate status |
| core autonomy mandate-list | Active mandates |
| core autonomy mandate-set | Define a new mandate |
| core autonomy mandate-revoke <id> | Revoke mandate by ID |
| core autonomy mandate-revoke-all | Return to manual mode |
| core autonomy log | Autonomy action log |
| core autonomy pending | Pending autonomous actions |

---

## Core Partnership (v14)

| Command | What it does |
|---------|-------------|
| core partner status | Partner system status and readiness |
| core partner propose | Forest proposes a new intent |
| core partner discuss <id> | Forest opinion on an intent |
| core partner disagree <id> | Forest respectfully pushes back |
| core partner consult <q> | Consult before making a decision |
| core partner reflect | What has the forest learned about you |
| core partner pattern | Patterns that define how you work |
| core partner growth | How the system has grown over time |
| core partner pushback | Recent pushback moments |
| core partner roadmap | Forest view of optimal path forward |
| core partner roadmap-why | Why this roadmap |
| core partner roadmap-diff | How forest roadmap differs from current |

---

## Core Self-Transformation (v16)

| Command | What it does |
|---------|-------------|
| core self map | Architecture coupling analysis |
| core self evolve | Generate structural proposals |
| core self apply | Apply a proposal (use --dry-run first) |
| core self history | Evolution audit trail |
| core self learn | Record outcome of a proposal |
| core self accuracy | Proposal accuracy over time |
| core self calibrate | Adjust proposal thresholds |
| core self challenge | Stress test a plan |

---

## Core Pattern Weight Engine (v17)

| Command | What it does |
|---------|-------------|
| core weight list | All patterns ranked by weight |
| core weight top | Critical and Strong patterns only |
| core weight compute | Scan events and compute weights |
| core weight explain <id> | Full weight breakdown for a pattern |
| core weight calibrate <id> <outcome> | Record outcome for calibration |

---

## Core Delegation Engine

| Command | What it does |
|---------|-------------|
| core delegate simulate | Simulate delegation without executing |
| core delegate contracts | List trust contracts and status |
| core delegate history | Delegation simulation history |
| core delegate accuracy | Simulation accuracy over time |
| core delegate suspend | Suspend all delegation instantly |
| core delegate counterfactuals | Counterfactual comparison log |
| core delegate accuracy-report | Three-dimensional accuracy report |

---

## Core Engines

| Command | What it does |
|---------|-------------|
| core engines status | All engines and their sync state |
| core engines sync | Acknowledge engine upgrade |
| core engines signals | Recent cross-engine signals |
| core engines check | Verify all engines consistent |
| core engines upgrade-log | Engine upgrade history |

---

## Core Causality Engine

| Command | What it does |
|---------|-------------|
| core why summary | Why did the system do what it did today |
| core why health | Why is health at its current level |
| core why domain <d> | What has a domain been doing |
| core why chain | Full causal chain for last health drop |
| core why correlate | Correlate two domains |
| core why suggest | Proactive suggestions from patterns |
| core why focus | Focus quality over time |
| core why attention | Attention analysis |

---

## Core Decision System

| Command | What it does |
|---------|-------------|
| core decision record | Record a new decision with snapshot |
| core decision outcome | Record outcome of a decision |
| core decision list | List recorded decisions |
| core decision hindsight | Hindsight summary |
| core decision show <id> | Full detail for a decision |
| core decision stats | Correlation stats |
| core decision advise | Judgment advisory for current state |
| core decision heuristics | Auto-derived heuristics |
| core decision lessons | Human-readable lessons summary |
| core decision story | 30-day narrative |
| core decision patterns | Repeating decision patterns |
| core decision friction | Decisions requiring repeated corrections |

---

## Core Values and Alignment

| Command | What it does |
|---------|-------------|
| core values list | All declared values |
| core values define | Declare a new value |
| core values remove <id> | Deactivate a value |
| core values weight <id> | Update value weight |
| core align check | Check alignment against declared values |
| core align drift | Behavioral drift report (30 days) |
| core align report | Weekly alignment report |

---

## Core Journal

| Command | What it does |
|---------|-------------|
| core journal today | Today journal entries |
| core journal yesterday | Yesterday journal entries |
| core journal week | This week journal entries |
| core journal search <kw> | Search journal by keyword |
| core journal show <date> | Journal for a specific date |

---

## Core Integrity (INT-184)

| Command | What it does |
|---------|-------------|
| core integrity run | Full integrity scan with repair |
| core integrity status | Current integrity score + pending |
| core integrity log | History of all detected issues |
| core integrity fix | Show pending proposals |

---

## Core Anomaly Detection

| Command | What it does |
|---------|-------------|
| core anomaly scan | Detect unexpected system changes |
| core anomaly history | Anomaly detection history |
| core anomaly alert | Surface high-severity anomalies |

---

## Core Narrative

| Command | What it does |
|---------|-------------|
| core narrative | Full story of how the forest became what it is |
| core narrative --since <ver> | Narrative since a version |
| core narrative --intent <id> | Narrative for a specific intent |

---

## Core DB (INT-166)

| Command | What it does |
|---------|-------------|
| core db backup | Manual state.db backup |
| core db verify | Verify backup integrity |
| core db status | DB size + WAL status |
| core db compact | VACUUM database |

---

## Core Registry (INT-183)

| Command | What it does |
|---------|-------------|
| core registry list | All tools with type + status |
| core registry show <name> | Tool detail |
| core registry retire <name> | Mark tool as retired |
| core registry unretire <name> | Restore retired tool |

---

## Forest Tools

| Command | What it does |
|---------|-------------|
| fu | faelight-update |
| bump | faelight-release publish |
| sec | security-audit |
| fm | faelight-fm |
| vault | faelight-vault |
| fdocs | faelight-docs |

---

## fsh Shell Commands

| Command | What it does |
|---------|-------------|
| last_command | Show last failed command |
| last_command retry | Re-run last failed command |
| last_command explain | Explain the failure |
| last_command fix | Suggest corrected command |
| failures | Session failure log |
| last_error | Last structured error |
| errors | Error history |
| observe session | Current session summary |
| observe commands | Most used commands |
| observe diff | Changes vs last session |
| observe anomalies | Unusual patterns detected |
| observe patterns | Learned command patterns |

---

## fsh Builtins

| Command | What it does |
|---------|-------------|
| pwd | Current directory |
| which <cmd> | Where command resolves |
| type <cmd> | Full resolution detail |
| env | Shell environment table |
| theme <name> | Switch prompt theme |
| clear / c | Clear terminal |
| cat <file> | View file |

---

## Prompt Themes

| Theme | What it shows |
|-------|-------------|
| forest | Path, git, health, commits |
| minimal | Path only |
| classic | user@host path $ |
| jarvis | Forest + prediction inline |

---

## System

| Command | What it does |
|---------|-------------|
| sr | Reboot |
| ssn | Shutdown now |
| ports | Active network ports |
| myip | External IP |
| paci | Install package |
| pacr | Remove package |
| pacs | Search packages |

---

## Weekly

| Command | What it does |
|---------|-------------|
| fu | faelight-update |
| core security scan | Security check |
| core stress report | After major changes |
| alias-audit | Verify alias health |
| core integrity run | Full integrity scan |
| deploy check | Verify all tools deployed |

---

## Documentation

| File | Contents |
|------|---------|
| docs/COMMAND-GUIDE.md | This file |
| docs/KEYBINDINGS.md | All 114 keybindings |
| docs/ARCHITECTURE.md | System architecture |
| docs/PHILOSOPHY.md | Design principles |
| docs/AUTOSTART-MAP.md | Niri autostart chain |
| docs/core-commands.md | Core subcommand deep reference |
| Command | What it does |
|---------|-------------|
| core synthesize now | Generate synthesis snapshot -- unified brief from all intelligence signals |
| core synthesize brief | Show the current Friday brief (latest snapshot) |
| core synthesize history | Show past synthesis snapshots |
| Command | What it does |
|---------|-------------|
| core friday status | Friday's current state -- observations, patterns, facts |
| core friday ask "<question>" | Ask Friday about Linux, Rust, Wayland, or the forest |
| core friday suggest | Evidence-based recommendation from observed patterns |
| core friday observe | Manually trigger observation cycle |
| core friday extract-patterns | Extract patterns from shell history |
| core friday update-personality | Update personality from interaction data |
| core friday seed-knowledge | Seed Linux/Rust/Forest knowledge base |
| core synthesize now | Generate synthesis snapshot -- unified forest brief |
| core synthesize brief | Show current Friday brief |
| core synthesize history | Past synthesis snapshots |
| core version | Show core binary version + intelligence tier |
| Command | What it does |
|---------|-------------|
| rspatch file.rs --anchor "text" --new "content" | Anchor-based replacement (default: replace) |
| rspatch file.rs --anchor "text" --new "content" --mode after | Insert new content after anchor |
| rspatch file.rs --anchor "text" --new "content" --mode before | Insert new content before anchor |
**Rules:** anchor must be unique in file. Never use line numbers. For multiline content use quotes.
**Unicode:** write literal characters directly -- no Python escape sequences needed.
> ⚠️  **rspatch warning:** Never let --new content contain the --anchor text. If new content includes the anchor string, the next rspatch call will match inside the newly inserted content and double-replace. Always use a different string in --new than what you searched for in --anchor.
| Command | What it does |
|---------|-------------|
| core friday-arch run | Run full meta-interpretation cycle -- cross-layer patterns + contradictions + brief |
| core friday-arch models | Show Friday's models and trust scores |
| core friday-arch proposals | Show pending proposals awaiting human review |
| core friday-arch contradictions | Show active cross-layer contradictions |
**Architecture principle:** Friday produces insight, not authority.
Every proposal requires human approval before execution.
Friday sees all intelligence layers simultaneously -- health, prediction, alignment, intent.
Contradictions surface automatically in `d` (doctor) output.
| Command | What it does |
|---------|-------------|
| core friday name-abstraction "name" "description" | Name a Friday abstraction -- adds to vocabulary |
| core friday vocabulary | List all named abstractions |
| core friday propose-intent | Friday proposes a new intent based on observed patterns |
