# Project 0 — Agent Instructions

This file is the operating contract for this repository.

docs/CONVENTIONS.md holds the reasoning behind several of these rules. Read it; do not copy it here. docs/NSH-COMPATIBILITY.md owns version tiers. One owner.

A sentence in this file is either enforceable now, marked UNVERIFIED / CLOSED / BOARD, or it does not belong here.

## 0. Stop

Read this before editing. If a step can lock the machine, stop and ask.

Stop and ask before:

- boot chain, disk layout, LUKS, or the greeter
- sudo, sudoers, or any privilege escalation
- broad process kills
- repository-wide renames
- changing persistent data locations or formats
- starting substantive work without an open intent

If something breaks: stop, assess, roll back, record it in the ledger, then update the rule that failed to prevent it.

### Sudo

Never change, touch, or work around sudo.

- Do not edit /etc/sudoers or /etc/sudoers.d.
- Do not add NOPASSWD entries.
- Do not add sudo to a command that was not already privileged.
- Do not cache, script around, or skip a password prompt.
- Never sudo rm. nsh refuses it on every door, without asking, and refuses any sudo through nsh -c, so scripts and agents get no sudo through nsh. An agent that runs bash directly never meets that guard; the password prompt is the wall there.
- No automation runs privileged: no boot timers, no scheduled updates, no cron with sudo. Automation is opt-in and hand-triggered.

Why (2025-12-14): a systemd user timer at boot attempted sudo with no credentials, tripped faillock, locked the account. Hand the exact elevated command to a human and say why it needs privilege.

### Recovery

- TTY2: Fn+Ctrl+Alt+F2 — kernel VT. Works when the compositor is hung. Depends on nothing this project ships.
- ship has no rollback. It overwrites the previous binary. Way back: git checkout <sha> then ship. Push before deploying something you cannot rebuild.
- snapper and limine are installed. Whether a snapshot rollback is configured and bootable is UNVERIFIED. Do not plan around it until someone has restored from one and written the steps.
- There is no Omarchy recovery runbook. The NixOS runbook was deleted 2026-09-29 because its steps are wrong here. That runbook is INT-225. The gap is stated, not assumed.

### Risk

Each directory may carry a RISK.toml. Read it before editing there.

- critical — boot, login, disk. Failure means the machine does not come back.
- system — shared across hosts. Failure breaks builds, loudly, not boots.
- user — recoverable from a running session.

Promote a directory to critical the moment it carries boot, login, or disk settings.

### Security

- Secure Boot is not enforcing. MEASURED 2026-09-05: sbctl is not installed; /var/lib/sbctl does not exist. Do not describe the boot chain as signed until that has been demonstrated.
- Secrets are never committed. Configs reference secrets; they do not embed them. Every commit is scanned by zero-gate's pre-commit hook: ripsecrets, then gitleaks with the repo's .gitleaks.toml, redacted. A scanner that is missing fails the commit.
- Before rebooting after a boot-chain change, verify signature state first.

### Fingerprint

A fingerprint is the declared identity of this machine and this tree. It is computed once, in one place -- zero_core::fingerprint, at zero/tools/zero-core/src/fingerprint.rs -- and asked about, not re-derived in every crate. Only `core fingerprint record` writes the expected record; the doctor never does.

* One collector. Do not add a second hash, probe set, or "good enough" identity check. Every consumer asks zero_core::fingerprint: `core fingerprint` and the doctor do today; integrity, DevBox verify and `devshell` Law 0 probes will when each is built. Same class of rule as INT-230: one place the system asks about 0-Core.
* Inputs are declared. Machine facts the kernel already publishes (identity / machine probes) and tree facts the repo already owns (process / filesystem probes). No silent extras. A field that is not on the declared list is not part of the fingerprint.
* Must not include: secrets, home contents outside the declared paths, network reachability, timestamps, or anything that changes because a session started, or because the machine booted or a filesystem was mounted (a device number, a boot id). fingerprint.rs refuses them in a test. A fingerprint that moves when you blink is not a fingerprint.
* Rename is not a new machine. INT-247 path and display-name changes must not flip the fingerprint by themselves. If a rename moves a probe input, the adapter is updated in the same change or the gate stays red.
* Outcomes are tri-state. A mismatch is FAIL. A missing capability is UNDETERMINED, not a guessed match. Clean is only clean when every declared input was actually read. Doctor already distinguishes these; do not collapse them. A record written under another schema is UNDETERMINED, never FAIL: an old question is not a changed machine.
* Law 0 consumes it, it does not invent it. Built consumers today: `core fingerprint show` and the doctor's Fingerprint check. `devshell` launch probes (INT-257), DevBox verify and integrity are planned, each its own intent; until then they do not read the fingerprint, and they must not grow a private copy when they do.

The full flow is in `docs/FINGERPRINT.md` — invariants stay here; the walkthrough does not.

### Devshell

devshell is where a dangerous operation is done for real. Credentials are not inside. Packages may install as root. Leaving undoes what was not promoted.

Declared paths are visible. A write is not a keep. The repository may be seen from inside; a write reaches the host only through devshell-promote. nsh-test keeps two laws honest every run: a write inside never reaches the host, and every mount is read-only or disposable while every socket refuses a connection.

TEN LAWS. Each was proven by breaking it (INT-257). Law 0 is ten launch checks; any failure refuses the session. Do not add an eleventh law until it has its own probe that can fail the start.

```text
0  A sandbox that cannot keep one of these laws refuses to start.
   It never starts anyway and hopes.
1  Everything written inside is thrown away when you leave,
   unless you promote it by name. Visible is not kept.
2  Nothing inside can see or signal a process outside.
3  Nothing inside reaches the network.
4  You are root inside, and root inside has no authority outside.
5  Nothing from your environment crosses unless it is declared.
6  The sandbox tells the truth about what it is. It never claims
   an isolation it did not get.
7  Leaving puts the machine back exactly as it was.
8  Everything that changed can be listed before you decide to keep any of it.
9  sudo inside is not a way out.
```

```text
devshell                          start a session (~0.3s)
devshell --resume last            the same session again
devshell-diff --last              list what changed
devshell-promote --last <path>    keep one named file; type PROMOTE to confirm
```

Home inside starts empty. Five paths are declared: the repository, ~/.local/bin, the shell config directory, and the cargo cache are visible; the shell state directory is a snapshot. Anything else is absent — not hidden, absent. SSH keys, git credentials, and AI tool directories do not exist inside.

The declared list lives in devshell-lib. That file is the authority; this is a summary.

Scripts: zero/scripts/devshell and zero/scripts/devshell-lib. devshell-lib is the authority for the exact names, and two nsh-test cases reach these paths. devshell, devshell-diff, devshell-promote and devshell-checkpoint are on PATH as links in ~/.local/bin to these scripts; ship does not deploy scripts, and nsh-test checks that each link resolves.

## 1. This host / this tree

Project 0 is the public name. Not eventual. Decided 2026-09-14:

| Register | Form |
|---|---|
| Spoken | Project 0 |
| Root and repo | 0-core / 0-Core — unchanged |
| CLI | 0 (decided; not built yet) |
| Crates | zero-* |
| Env vars | ZERO_* |

Cargo refuses a package name starting with a digit. Bash refuses an env var that does. See INT-247.

One line, current: a shell being made good, a sandbox that tests it, and the tools that survived “what breaks tomorrow if I delete this?” Omarchy (Arch, systemd, Hyprland) is the substrate, not the product.

Historical names: Faelight Forest, Faelight Shell, fsh. Prefix the future; do not rename the past.

### Layer 0 — the freeze

No new crate, binary, path, doc heading, or config key begins with faelight. New work is nsh, core, friday, devbox, or zero-*.

This is a rule, not a plan. It cannot break anything and it stops the old name from growing.

Do not perform repository-wide search-and-replace. Classify first:

| Category | Strategy |
|---|---|
| User-facing name, documentation | Rename |
| New APIs, new files | zero-* |
| Package / module names | Deliberate migration |
| Internal identifiers | Gradual |
| Environment variables, config dirs | Compatibility, then migration |
| Persistent data | Preserve; needs an explicit plan |
| Existing scripts | Test before changing |
| Git history, old commit subjects, old intent titles | Leave alone |

Moving a data directory is a data migration, not a rename.

Moving a directory that holds data: alias, flip, swap — never rm, mv, ln. New name is a relative link to the old one, proven by inode. Code flips in one commit, red first. Then renameat2(RENAME_EXCHANGE) swaps the names on the same filesystem; the old name is repointed at the new one. The name the code uses exists at every instant. rm/mv/ln leaves a window where a tool creates an empty directory — a silent empty ledger. (INT-247 Layer 3b.)

A /etc/ path is an assumption from an OS that no longer exists. Omarchy has no environment.etc reconciler. Those files vanished 2026-08-26. Absence must say so. (INT-250)

Generated files are regenerated, never hand-edited or hand-renamed. If one is wrong, fix the generator.

Every migration step leaves the tree buildable. Never mix an unrelated architectural change into a naming commit.

### Paths that are real (measured 2026-09-30, INT-247)

Real directories: ~/.local/state/zero, ~/.config/zero, ~/.cache/zero, ~/.local/share/zero, and ~/.config/nsh for NovaShell. The compatibility links that carried the old names were removed 2026-09-30 (INT-247 pass 7); d's Zero Paths check goes red if one comes back.

The intent ledger lives at zero/intents/; INT-252 moved it there. Do not add files under a faelight/ name.

Where a file goes: docs/TREE.md. It names every entry at the repo root and in zero/, each with one line of purpose. A new file goes inside an entry the map already names. A new entry gets its own line in the map in the same commit, or nsh-test goes red (the_tree_map_names_every_entry). Do not restructure the tree to make a file fit: moves are INT-267's.

Hardcoded readers still to classify, not blindly replace: zero-core/src/paths.rs, zero-deadwood/src/main.rs, integrity/mod.rs, cheatsheet_tui.rs. UNVERIFIED since the crate renames: whether each still needs classifying has not been measured.

### This machine

- Repo on disk: /home/christian/0-core
- Deployed binaries: /home/christian/.local/bin
- cat is bat. ls is eza — ls -lt fails; eza wants --sort=modified.
- d is the health check. gc is git commit. gp is git push.
- Retired: dep, rebuild, rebuild-safe, rebuild-dry, nix develop, generations, rollback-by-generation, sbctl, grub. Installed and present: snapper, limine. Installed is not working.

## 2. How a change is allowed

### Intent lifecycle

Every change of consequence runs through the ledger. Order is not optional.

- Recon. Read the code, config, and running state. Run the lookup. Do not narrate a hypothesis in place of evidence.
- Plan. One direction. Scope it, name the gates, say what proof each gate needs. Never defer a gate to be resolved later — build it in, or discuss it before starting.
- cistart <id>. Open the intent before writing code.
- Apply only the change that was agreed.
- Test the debug binary.
- ship. The only step that produces what actually runs.
- Reload and verify the deployed artifact.
- cicomplete <id>. Only once the evidence exists. Never to tidy the end of a session.

If a step cannot be completed, say so and stop. Do not proceed and log the gap as follow-up.

DevBox (INT-167) is not a required step until it is the verifier. Until then it is deferred.

Working shorthand: recon, solve, test, then rebuild.

### Evidence

A ticked box is a promise. Evidence is the receipt. Format — an HTML comment on the line after the gate:

```text
- [x] Secure Boot enforcing on metal with custom keys
<!-- evidence: commit f0d0a08e, 2026-07-16. bootctl status -> Secure Boot: enabled (user). -->
```

A commit hash, a file:line, a log path, or demonstrated: <what and how>. The claim must be checkable.

Hard: recon and cistart before code. Evidence on any gate that could pass by doing nothing. A gate you have only watched pass might be doing nothing — prove it by watching it fail first, then pass.

Light: “file created” needs no artifact. “The VM boots” does.

An exit code is evidence only from a command that sets one on purpose. diff on this machine is a structural viewer that exits 0 with or without changes; compare files with cmp.

Forward-only: never retrofit old intents.

A gate can be closed by declining the thing, with numbered reasons. That is still proof.

Findings and commits point at each other by name. A problem found and not fixed is filed with core intent find, never left as prose in an intent. A commit carries Intent:, and Fixes: F-x when it closes a finding; a commit made from a reviewed plan also carries Seal:, with the plan text kept as a note under refs/notes/seals. core intent trace answers what is open, what fixed it, and which plan produced it. Reasoning: docs/CONVENTIONS.md (INT-266).

### Edit

Edits go through fpatch (zero/scripts/dev/fpatch.py). INT-258: this rule existed for months without saying how. There is no CLI — shell arguments would put the anchors back into shell syntax.

```text
import sys
sys.path.insert(0, "/home/christian/0-core/zero/scripts/dev")
from fpatch import patch, patch_between
patch("path/to.rs", old, new)
```

Absolute path always. old and new stay Python string literals. The payload is one argv word.

- refuse exits 1 and promises nothing was written. internal exits 2 and promises nothing.
- Anchors match the file byte for byte. An anchor that matches three times is refused; widen it.
- Any edit invalidates line numbers below it. Re-read before the next patch.
- Em dash and double dash are not interchangeable.
- One concern per patch.
- Non-ASCII anchors: patch_between(path, start_marker, end_marker, new_lines) — two short ASCII markers, replace by index.
- A payload defines, then acts on its last line. A cut paste must fail to parse or never call — it must not run half an edit.
- dry prints the plan and writes nothing. Writing modes refuse unless preconditions hold.
- Rehearse unread behaviour on a mktemp copy first.
- An intent section that claims a commit is pushed or a path is real checks that against git and disk before it is written.
- Never make unrequested changes. Surface them.

Paste blocks for a human contain no apostrophes, no heredocs, and no bare --help.

### Transport

Generated text crosses the shell as data. Do not let it become shell syntax because the shell is carrying it.

Live form — one argv word, never a fixed /tmp file:

```text
python3 -c 'import base64,sys; exec(base64.b64decode(sys.argv[1]).decode("utf-8"))' PAYLOAD
```

Small enough to stay inline: single quotes. Double quotes expand. Do not add escaping; remove the shell from the path.

Heredocs work in nsh (measured 2026-09-13, both doors, both executors, against bash). They remain banned in paste blocks because a quoted delimiter contains apostrophes, and apostrophes do not survive the paste path.

Base64 is the AI-to-shell transport. It is not part of the shell’s user-facing architecture.

This rule was wrong twice by generalising from one observation. The live form above is the rule. Dead forms do not belong in this file.

### Build and test

Measured 2026-09-05:

```text
edit → cargo build -p <crate> → test DEBUG → ship → exec /home/christian/.local/bin/nsh → verify DEPLOYED
```

- ship builds release into ~/0-core/target/release and copies changed tools into ~/.local/bin. A green cargo build is not deployed.
- Debug binary: ~/0-core/target/debug/<bin>. That is what you test before ship. It is never what runs at the prompt.
- Suite, to a file, never through a filter:

```text
env NSH_BIN=/home/christian/0-core/target/debug/nsh nsh-test > /tmp/suite.txt 2>&1
```

- Use env, not a bare VAR=value prefix — nsh drops the prefix for some children. Absolute path — INT-241, $PWD is not updated.
- After ship, exec /home/christian/.local/bin/nsh — absolute. exec does not search PATH. Bare exec nsh fails. Without the exec you are testing the old binary.
- Deployed binary can trail HEAD by many commits with no warning. Version string does not change per commit. Compare mtime (/usr/bin/ls -la --time-style=full-iso ~/.local/bin/nsh) to git log --format="%h %ci %s" before treating a red as a source bug.
- After any rename: cargo check --workspace.
- ship has no health gate and no rollback.

Recon uses fsearch, not grep or awk:

```text
fsearch <pattern> [--type ext] [--file name] [--live] [--intent] [--all|--scripts]
```

Its table truncates. When the exact text matters, read numbered lines. Disk state is read the same way before anything touches it.

### Version control

Version numbers follow contract impact, not diff size. Tiers live in docs/NSH-COMPATIBILITY.md: CONTRACTUAL (breaking it is major), INCIDENTAL (never offered), ERRONEOUS (fixing a broken promise is a patch).

The commit is the receipt. CHANGELOG.md lags it. The ledger cites SHAs. The body says what was false, what is true now, and how that was watched.

Subject names the subsystem and the fact: nsh: the suite can tell a stale binary from a current one, not fix tests.

One boundary per commit. A wrong pushed commit gets a FOLLOW-UP that cites it. No rebase to tidy a month. No amend of anything pushed. No squash, no force-push, no Conventional Commits-as-policy — they flatten the log the ledger uses.

Prefix the future. July commits say fsh because that is what it was called in July.

A skipped hook is stated in the next commit body.

git add scope is a bug class. If the change touches a workspace dependency, add Cargo.lock deliberately.

### Definition of done

All of:

- Demonstrated, not declared.
- Evidence recorded — what was run, what came back, when.
- Verified against the deployed artifact, not only the build output.
- Nothing else changed along the way.

Never mark something done to be resolved later.

## 3. When something is wrong

### Which layer

Name the layer before touching code. First ask whether the behaviour belongs to nsh at all.

```text
0. not nsh (kernel, alias table, another binary)
1. message transport / quoting
2. shell parsing
3. file creation / filesystem
4. program logic
5. dependency or environment
6. architecture
```

Do not rewrite application logic when the fault is payload corruption or shell interpretation. Measured 2026-09-12: four sites patched for an exit status that stayed 0 because the command never reached them.

Three 2026-09-14 reports that named nsh and were wrong: an orphaned stopped child (kernel SIGHUP/SIGCONT), a stale Ctrl+C handler (kernel delivers to the foreground pgid), a dropped redirect exit (alias diff → difft, which exits 0 either way). The cost of asking is one command. The cost of not asking is a permanent wrong belief.

### Measure first

When something is slow, measure where the time goes. Census: 158s at a 30s default timeout, 58s at 10s, 8s of actual work. One constant, after measuring, removed 100 seconds. Parallelism would have hidden the waste.

Put the exception on the case that needs it. Keep the default at what the ordinary case requires. Measure, change, measure again.

### Testing

A green build is not the claim. The claim is the thing running.

- After deploying a service, ps for the process.
- Exercise shell behaviour through the real REPL, not only -c. nsh -c executes nsh, not sh (INT-201; the -c branch in novashell main.rs). The safety guard is on that door.
- Test with NSH_SPINE on and off. The executors do not always agree; disagreements are not all defects. nsh -c "echo test > 0.5" writes a file; the same line under NSH_SPINE=0 is refused — spine narrowed the digit guard to the query language so where cpu > 0.5 still works.
- spine migrate cannot tell a ruled improvement from a defect. It diffs rendered IoPlans. Read the parser before treating a row as a bug.
- VM first for compositor, greeter, or login. Never on bare metal blind.
- Test the class, not the example. Quoting, $, &&, Unicode, multiline, pipes, redirection, substitution — cover the family.
- Red first. A test that has only ever passed has not been shown to test anything.
- Test an owner against the real tree, not only a stand-in. Beside the fixture tests, one test asks the owner and checks the disk itself -- the crate it names exists, the directories it scans hold what it returns -- so the owner and the tree cannot disagree without a red test.
- After a visual change, take a screenshot. Do not call a visual change done from the config diff.
- Check ps before any broad process kill. Never pkill -f on a loose pattern.

### Messages and tool output

A message says what happened, what the shell could not do, and what the reader can act on.

- A category where a fact belongs (exited 1 -- general error for grep finding nothing) is a guess sounding certain.
- If the shell does not know why something failed, it says so.
- Leave a next move: a pid, a path, a signal, a command.

Same rule as Unknown, Skipped, and the degradation list: a tool that cannot answer must say so. An exit code is not a diagnosis.

A safe abort and a crash must not look the same. Lead with what did not happen. Result first. Recovery steps numbered and runnable. Tracebacks behind a debug flag. Assertions are for bugs, not refusals. Keep the non-zero exit either way. fpatch _refuse is the reference. Reasoning: docs/CONVENTIONS.md (INT-199).

## 4. Board — not law

Dated work list. If the date is stale, distrust this section before the rest of the file. Standing rules are §0–§3.

October 2026. The Faelight → Project 0 rename is complete: INT-247 and INT-252 closed 2026-10-01, and nsh-test keeps the old names out of every live file.

Open, in an order that is an order because the safety guard is not yet on every door:

- Digit guard disagrees across doors. MEASURED 2026-09-05: nsh -c "echo test > 0.5" creates 0.5; the interactive shell refuses the same line. First move is recon: where the REPL applies the guard and the -c branch does not. Delete the stale delegates-to-sh comment in the same change.
- INT-197 remainder — verify before implementing. Claim: check(cmd, first_word) gets the alias-expanded first word but the typed line as cmd, so alias zap='rm -rf /tmp/x' then zap never sees -rf. INT-196 and INT-197 are marked complete. One reproduction settles it. Do that first.
- Honesty pass, deletions only. Help text vs behaviour; .fsh vs .nsh in plugin help; cmdguard vs guard; zero-gate --help still offering risk tiers after risk-gate.sh died. Fix or delete. Do not add documentation.

CLOSED by measurement, do not reopen: the -c boot tax (2026-09-02). 13ms vs bash 1ms. The 305ms figure predates work already in tree. Suite time is pty sessions, not boot.

CLOSED, do not reopen: the crate renames faelight-* → zero-*. All fifteen crates are zero-*; the last, zero-core, landed in c4634250 (2026-09-26).

CLOSED, do not reopen: one git policy. The git reset --hard rule was deleted, not disabled; safety_guard.rs records why.

Deferred, because of ordering rather than objection:

- Hook or plugin surface — would sit on a guard that does not cover every door.
- INT-169 spine as the only executor.
- Splitting commands/mod.rs (17k lines; wait for a guard that makes the split provable).
- New heuristics, more deny words, chmod rules.
- nsh as login shell. INT-190 records what that cost.
- DevBox as the required verifier (INT-167).
- Omarchy recovery runbook (INT-225).

## 5. Do not use this section to place files

Architecture is boundaries. Directories are for maintaining code. Do not restructure the repository to match a layer diagram. Do not use this section to decide PR scope or crate placement.

Conceptual order, for orientation only:

```text
Foundation → System → Runtime → Objects → Storage → Security → Network
          → Shell → Extensions → Experience → Intelligence
```

One hard dependency that is operational: Core must never depend on Intelligence. Remove every AI component and the system still boots, runs, and is usable. Never fix a problem by importing a higher layer into a lower one.

Shell pipeline, keep separate:

```text
input → lexer/parser → command representation → expansion → execution
      → process management → streams → UI and events
```

When adding a subsystem, state its boundary and its API. Prefer a reusable primitive to a one-off. Ask what happens when it fails.

AI will use this shell. Predictable semantics, structured output where it helps, machine-readable errors, clear exits. Not at the expense of being a good shell for humans.

Destructive actions are explicit in the shell itself. That is what nsh owes a user. What an agent must stop and ask about in this repository is §0.

Manual control over automation. Understanding over convenience. A known way back — and where there isn’t one, the gap is written down. Everything has an expiration date, including Rust as the implementation language; that is a measurement, not an identity. “No bloat” does not mean write everything ourselves. If a proven Linux component does exactly what is needed, use it.

Lessons that recur become invariants in §0–§3. Do not rediscover them in this section.
