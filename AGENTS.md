# Project 0 — Agent Instructions

This file is the operating contract for working in this repository.

`docs/CONVENTIONS.md` is the reasoning behind two of these rules and is current — read it, do not
duplicate it here. Where a rule below has a "why", it points there.

---

## Project Identity

- **Codename:** Project 0
- **Public name (eventual):** Zero Core
- **One line:** Project 0 — a new computing system built around objects, state, and capabilities.
- **Repository:** `0-Core`. Historical identity: Faelight Forest / Faelight Shell.

Project 0 runs on Omarchy -- Arch-based, systemd, Hyprland. **The operating system is
the substrate, not the product.** The architecture defines the abstraction; the current
substrate is one implementation of it, and is expected to be replaceable.

---

## Architectural Model

Eleven conceptual layers, in dependency order:

```
Foundation → System → Runtime → Objects → Storage → Security → Network
          → Shell → Extensions → Experience → Intelligence
```

**These are conceptual boundaries, not directories.** Do not restructure the repository to
mirror them. Architecture is about boundaries; directories are about maintaining code.

---

## Dependency Rules

- Foundation MUST NOT depend on higher layers.
- System MAY depend on Foundation.
- Runtime MAY depend on Foundation and System.
- Objects MAY depend on Foundation and Runtime primitives, but MUST NOT depend on
  Experience or Intelligence.
- Shell MAY depend on Core services.
- Experience MAY depend on Shell.
- Intelligence MAY depend on public Core interfaces.
- **Core MUST NEVER depend on Intelligence.** Remove every AI component and the system
  must still boot, run, and be usable.

Never resolve a problem by importing from a higher layer into a lower one. That creates an
architectural cycle. Raise it instead.

---

## The Intent Lifecycle

Every change of consequence runs through the intent ledger, at `faelight/intents/`. The order is
not optional, and no step is skipped because a change looks small.

1. **Recon — look before touch.** Read the actual code, config, and running state before
   proposing anything. When a direct lookup can answer a question, run the lookup. Do not
   narrate hypotheses in place of evidence.
2. **Formulate the plan.** One direction, stated plainly. Scope it, name the gates, and say
   what proof each gate will require. **Never defer a gate to be resolved later** — build it
   into the intent, or discuss it before starting.
3. **`cistart <id>`.** Open the intent *before* writing code, not after the work is done.
4. **Apply the code.** Only the change that was agreed. No unrequested edits riding along.
5. **Test in the debug shell.** Exercise the behaviour before it reaches the running system.
6. **`ship`.** Build the release and deploy. This is the only step that produces
   what actually runs. `dep` was the NixOS verb and no longer exists -- see
   Build System below.
7. **Reload and test in the new build.** Re-verify against the deployed artifact.
8. **DevBox** (INT-167), as it comes online — instrumented verification in place of manual
   checking.
9. **`cicomplete <id>`.** Only once the evidence exists and is recorded. Never to tidy up the
   end of a session.

Working shorthand: **recon, solve, test, then rebuild.**

If a step cannot be completed, say so and stop. Do not proceed and log the gap as follow-up.

---

## Evidence

**A ticked box is a promise. Evidence is the receipt.** Full reasoning in
`docs/CONVENTIONS.md` (INT-158).

Format — an HTML comment on the line *after* the ticked gate:

```markdown
- [x] Secure Boot enforcing on metal with custom keys
<!-- evidence: commit f0d0a08e, 2026-07-16. bootctl status -> Secure Boot: enabled (user).
     db read from the efivar = exactly 2 certs, ZERO Microsoft. Reboot survived. -->
```

A commit hash, a `file:line`, a log or artifact path, or `demonstrated: <what and how>`.
Prose counts. The point is that the claim is checkable, not that it has a schema.

Three limits:

- **Forward-only.** Never retrofit old intents. That is busywork with no payoff.
- **Soft.** This is a discipline, not gate-police.
- **Light.** Trivial self-evident gates need no artifact. "File created" does not. "The VM
  boots" does.

**The tell: a gate you have only watched pass might be doing nothing.** Where you can, prove a
gate by watching it FAIL first — stage something broken, watch it be rejected, then fix it and
watch it pass. The rustfmt hook "passed" for six days by never running.

**A gate can be closed by declining the thing,** with numbered reasons. That is still proof.

---

## Build System
The rebuild cycle on Omarchy (measured 2026-09-05):
```
edit  ->  cargo build -p <crate>  ->  test the DEBUG binary  ->  ship  ->  exec /home/christian/.local/bin/nsh  ->  verify on the DEPLOYED binary
```
- **`ship` is the only thing that builds what actually runs.** It builds the release profile
  into ~/0-core/target/release and copies CHANGED tools into ~/.local/bin (it reports
  "N shipped  M unchanged"). A green `cargo build` is not proof of anything deployed.
- **`cargo build -p <crate>` produces the DEBUG binary** at ~/0-core/target/debug/<bin>. That is
  the binary to test before shipping. It is never what runs at the prompt.
- **Test the debug shell with the suite, output to a file, never through a filter:**
  `env NSH_BIN=/home/christian/0-core/target/debug/nsh nsh-test > /tmp/suite.txt 2>&1`
  then read the file. `env`, not a bare `VAR=value` prefix -- nsh drops the prefix for some
  children. Absolute path -- INT-241, `$PWD` is not updated.
- **After `ship`, `exec /home/christian/.local/bin/nsh` -- ABSOLUTE PATH.** The `exec` builtin does
  not search PATH (found 2026-09-05, recorded in INT-230's body): bare `exec nsh` fails with
  "No such file". Without the exec you are testing the old binary and will read a working fix
  as a failure.
- **The deployed binary can trail HEAD by many commits with no warning.** The version string does
  not change per commit. 2026-09-05: the shipped nsh was built at the `cistart 230` commit and
  carried none of the intent's sixteen commits. When a red does not match the source, compare
  the binary mtime (`/usr/bin/ls -la --time-style=full-iso ~/.local/bin/nsh`) against
  `git log --format="%h %ci %s"` before reading further.
- After any rename, `cargo check --workspace`. A per-crate check misses the breakage.
- **Gone with NixOS, do not look for them:** `dep`, `rebuild`, `rebuild-safe`, `rebuild-dry`,
  `nix develop`, generations, rollback-by-generation, and the rule that git must know a file
  before it builds. `ship` has no health gate and no rollback.
<!-- MEASURED 2026-09-05: snapper and limine ARE installed (/usr/bin/snapper,
     /usr/bin/limine); dep, rebuild, sbctl and grub are NOT. Whether a snapper
     rollback is configured AND bootable remains UNVERIFIED -- installed is not the
     same as working, and nobody has restored from one. Still open: whether Wayland
     crates need any special environment on Arch now that nix develop is gone -->

---

## Sudo

**Never change, touch, or work around sudo.**

- Do not edit `/etc/sudoers`, `/etc/sudoers.d`, or the Nix options that generate them.
- Do not add `NOPASSWD` entries for any user, command, or tool.
- Do not add `sudo` to a command that was not already privileged.
- Do not cache, script around, or otherwise avoid a password prompt.
- Never `sudo rm`. nsh blocks it deliberately.
- **No automation runs privileged.** No systemd timers at boot, no scheduled updates, no cron
  with sudo. Automation is opt-in and explicitly triggered.

**Why (2025-12-14, twelve hours):** a systemd user timer ran at boot, attempted sudo with no
credentials, tripped faillock after three attempts, locked the account, and broke sudo
authentication system-wide. *Automation at boot plus sudo is a debugging nightmare.*

When a step genuinely requires elevation, hand the exact command over to be run by hand and say
why it needs privilege. Privilege escalation is never a convenience.

---

## Testing

- A green build is not the claim. The claim is the thing running.
- After deploying a service or daemon, confirm with `ps` that the process exists.
- Run the health check — `d` — at session start and before closing. Fix warnings rather than
  noting them.
- Shell behaviour must be exercised through the real REPL, not only `-c` -- but not
  for the old reason. **`nsh -c` executes nsh, not `sh`** (INT-201 gate 4,
  main.rs:905), and the safety guard is on that door (main.rs:946). The stale
  comment at main.rs:860 still says it delegates to sh; it is wrong.
  ⚠️ The two doors still disagree -- see Current Work item 1. Test both.
- **VM first for anything touching the compositor, the greeter, or login.** Never on bare metal
  blind.

---

## Recovery

Know the way back before you need it.

- **TTY2:** `Fn+Ctrl+Alt+F2` -- a kernel VT switch, so it works even when the compositor
  is hung. The one recovery path that depends on nothing this project ships.
- **`ship` has no rollback.** It overwrites the previous binary. The way back from a bad
  deploy is `git checkout <sha>` then `ship` -- which makes the pushed history the
  recovery mechanism. Push before you deploy something you cannot rebuild.
- **snapper and limine are installed** -- but whether a snapshot rollback is configured
  and bootable is UNVERIFIED. Do not plan around it until someone has restored from one
  and written down what they did.
- ⚠️ **`docs/recovery-runbook.md` IS STALE AND DESCRIBES A MACHINE THAT NO LONGER
  EXISTS.** It assumes NixOS, a flake target, btrfs subvolumes including `@nix`, MangoWM,
  greetd, and the F3 SafeShell session. The only thing in it still true here is the VT
  key sequence. Treat it as history until it is rewritten. **The system currently has
  no trusted written recovery procedure.** That is a gap, stated rather than assumed.

---

## Visual Changes

- After any visual change, take a screenshot and analyse it. Do not report a visual change as
  done on the strength of the config diff.
- For interactive UI, drive it with simulated keyboard input, track the PID, and stop it after.
- **Check `ps` before any broad process kill.** Never `pkill -f` on a loose pattern.

---

## Naming and Identity

The project is migrating from the historical "Faelight" identity to Zero Core. New user-facing
functionality uses Zero Core terminology. Historical Faelight names remain where changing them
would create compatibility or migration risk.

**Do not perform broad mechanical renames of Faelight identifiers.**

Classify before migrating:

| Category | Strategy |
| --- | --- |
| User-facing name | Rename |
| New APIs, new files | Use Zero |
| Documentation | Rename |
| Internal identifiers | Migrate gradually |
| Package/module names | Deliberate migration |
| Environment variables | Compatibility period |
| Config directories | Compatibility, then migration |
| Persistent data | Preserve; requires an explicit migration plan |
| URLs / domains | Deliberate migration |
| Existing scripts | Test before changing |
| Git history | Leave alone |

`~/.faelight/` → `~/.zero/` is **not a rename. It is a data migration.**

Tool naming: `faelight-` prefixed today. New tools should carry a purpose in the prefix.

### Known hardcoded Faelight paths

A mechanical rename breaks three layers at once. Measured, not assumed:

- Rust: `faelight-core/src/paths.rs`, `faelight-deadwood/src/main.rs`, `integrity/mod.rs`,
  `doctor/checks.rs`, `cheatsheet_tui.rs`
- Nix: `environment.etc."faelight/VERSION"`, `xdg.configFile."faelight/profiles.toml"`
- Persistent data: `~/.config/faelight*`, `faelight/runtime/state.db`

---

## Migration Rules

1. Do not perform repository-wide search-and-replace renames.
2. Preserve compatibility where practical.
3. New APIs and components use Zero Core terminology.
4. Classify every legacy identifier before migrating it.
5. Persistent data formats require an explicit migration strategy.
6. Package and module renames must preserve dependency correctness.
7. Generated files are regenerated, never hand-renamed.
8. Every migration step leaves the system buildable and testable.
9. Never mix unrelated architectural changes into a naming migration.
10. Prefer small, independently testable migration commits.

---

## Design Philosophy

**Manual control over automation. Understanding over convenience.**

Project 0 prioritizes:

- minimalism — every component justifies its existence
- reproducibility — the system is described by configuration, not accumulated changes
- verified change — a change is trusted once it has been demonstrated, never once it has built
- a known way back — and where there isn't one, the gap is written down rather than assumed
  (`ship` has no rollback today)
- security by default
- explicit architectural boundaries
- coherent, designed experience — designed, not decorated

Do not add functionality merely because conventional Linux distributions include it.

**Everything has an expiration date, including the choices this project has already
made.** Rust is the right implementation language today. That is a measurement, not an
identity. If something serves the architecture better -- another language, another toolkit,
something not written yet -- the question is answered by building something small in it and
comparing, not by how much has already been written in the current one.

An idea earns its place by what it does, **never by how popular it is** -- neither by
popularity outside the project nor by how settled it feels inside it. The architecture
is the product; the implementation is this decade's best available answer to it. Design
for where the project is going, not only for what it is now.

**"No bloat" does not mean "we write everything ourselves."** If a proven Linux component does
exactly what is needed, use it. The differentiator is integration, not authorship.
Build to understand, replace when better exists, keep the intelligence in our own code.

---

## Edit Discipline

Edits go through **`fpatch`** (`faelight/scripts/dev/fpatch.py`), not ad-hoc rewriting.

- Anchors must match the file byte for byte, including whitespace. An anchor that matches three
  lines is refused; widen it until it is unique rather than guessing.
- Any edit invalidates every line number below it. Re-read before the next edit — never patch
  from a stale view.
- Watch for em dash versus double dash in anchors. They look alike, are not interchangeable, and
  are a recurring cause of failed patches.
- One concern per patch. A patch that fixes two things cannot be reverted for one of them.
- When an anchor cannot be made to match -- a line carrying an em dash, a box-drawing rule, or any
  non-ASCII character -- use fpatch.patch_between(path, start_marker, end_marker, new_lines). It
  locates a span by two SHORT ASCII markers and replaces it BY INDEX, so the body is never
  retyped. fpatch.patch refuses a non-ASCII anchor outright and names the offending characters.
- Never make unrequested changes. Surface improvements for discussion instead.

### Paste blocks

Command blocks written for a human to paste must contain no apostrophes, no heredocs (`<<`),
and no bare `--help` invocations.

### Known aliases that change command behaviour

- `cat` is `bat`
- `ls` is `eza` — **`ls -lt` fails**; eza spells it `--sort=modified`
- `d` health check · `gc` git commit · `gp` git push · `fm` / `fmd` file manager

---

## Tool Output

**A safe abort and a crash must not look the same. Lead with what did NOT happen.**
Full reasoning in `docs/CONVENTIONS.md` (INT-199); `fpatch`'s `_refuse` is the reference.

- Result first. Say what was not written before any internal detail.
- The message carries the diagnostic. No error codes to look up.
- Recovery steps are part of the interface — numbered and runnable.
- Tracebacks behind a debug flag; structured output by default.
- Assertions are for bugs, not for refusals. Keep the non-zero exit either way.

---

## Security

- ⚠️ **Secure Boot is NOT enforcing.** MEASURED 2026-09-05: `sbctl` is not installed and
  `/var/lib/sbctl` does not exist. The custom-key setup went with NixOS and has not been
  rebuilt. Do not describe the boot chain as signed until it is, and until that has been
  demonstrated rather than configured.
- Secrets are never committed. `gitleaks` scans before commits. Configs reference secrets, they
  do not embed them.
- Passwords are set at install time, not declared in the repo.
- Before rebooting after a boot-chain change, verify the signature state first.

---

## Risk Tiers

Each directory carries a `RISK.toml` declaring its tier. Read it before editing in that
directory.

- **critical** — boot, login, disk. Failure means the machine does not come back.
- **system** — shared across hosts. Failure breaks builds, loudly, but not boots.
- **user** — user-scope configuration and tools. Failure is recoverable from a running session.

Promote a directory to critical the moment it starts carrying boot, login, or disk settings.

---

## Dangerous Operations

Stop and ask before:

- anything touching the boot chain, disk layout, LUKS, or the greeter — lockout-class
- anything involving sudo, sudoers, or privilege escalation
- broad process kills
- repository-wide renames
- deleting or moving anything referenced by `docs/recovery-runbook.md`
- changing persistent data locations or formats
- starting substantive work without an open intent

If something breaks: stop immediately, assess, roll back, document it in the ledger, then update
the rule that failed to prevent it.

---

## Generated Files

Generated files are regenerated, never hand-edited and never hand-renamed. If a generated file
is wrong, fix the generator.

---

## Version Control

**Version numbers are decided by contract impact, never by size.** A one-line
change can be major and a fifty-thousand-line rewrite can be a patch. The
deciding question is what a caller is owed, not what the work cost.

Three tiers govern it: CONTRACTUAL behaviour is owed and breaking it is major;
INCIDENTAL behaviour was never offered; ERRONEOUS behaviour violates the shell
own stated semantics, and correcting it is a PATCH even when observable --
nobody reasonably depends on a guard failing to guard.

Read docs/NSH-COMPATIBILITY.md before choosing a level. It carries the
promise, the three tiers, and the question sequence. Do not restate its rules
here: one owner.

The commit is the receipt. CHANGELOG.md is a lagging copy of it, and the intent
ledger cites SHAs -- so a commit body is load-bearing evidence rather than a
courtesy.

**The subject names the subsystem and the fact.** `nsh: the suite can tell a
stale binary from a current one`, not `fix tests`. The body says what was false,
what is true now, and how that was watched.

**One boundary per commit.** The fsh -> nsh rename went bashrc, then the
`[[bin]]`, then the resolvers, then the tables -- each pushed separately. Mixing
a rename, a new guard door and a WAL change into one SHA makes it impossible to
say afterwards which one moved the number.

**History is not a whiteboard.** No rebase to tidy a month, no amend of anything
pushed. A wrong commit gets a FOLLOW-UP that cites it by hash. Rewriting deletes
the measurements the ledger depends on -- the evidence
lives in the bodies, not in a separate document.

**Prefix the future, never rename the past.** Commit messages from July say
`fsh` because that is what it was called in July. Leave them. New commits say
`nsh`. The same rule governs intent files and changelogs: they record what was
true when written.

**A skipped hook is stated, not hidden.** If `--no-verify` lands a WIP, the next
commit's body says the hook was skipped and why -- the same class of sentence as
"the eighth gate is deliberately left red".

**`git add` scope is a bug class, not a preference.** Commit 39031dc exists
because a previous commit scoped `git add` to `faelight/` and missed the root
`Cargo.lock` -- the second time that day. If the change touches a workspace
dependency, add the lock deliberately.

Conventional Commits, squash merges and force-pushes are all rejected for the
same reason: each one flattens or discards the part of the log that is actually
used.

---

## Current Work -- September 2026

⚠️ THIS SECTION IS DATED AND MEANT TO BE REPLACED. Everything above it is a
standing convention; this is what is being worked on right now and in what
order. If the date is stale, distrust the list before you distrust the rest of
the file.

### The ordering, and why it is an ordering

Coherence does not move until the safety guard is on every execution door.
Until then, more code means more comments to distrust. So:

1. **The digit guard is absent from `nsh -c`.** MEASURED 2026-09-05:
   `cd /tmp && nsh -c "echo test > 0.5"` creates a file named `0.5`, while the
   interactive shell refuses the same line and prints it as text. The two doors
   disagree about the LANGUAGE, in the one construct the history corpus proved is
   a deliberate divergence -- `where cpu > 0.5` depends on it.
   ⚠️ THIS REPLACES THE OLD ITEM 1, WHICH WAS FALSE IN BOTH HALVES. `-c` does
   not delegate to sh (main.rs:905) and does not skip the safety guard
   (main.rs:946; nsh-test `dashc_reaches_the_safety_guard` and
   `dashc_guard_refusal_stays_off_stdout` both pass). The real gap is narrower
   and worse: same shell, same guard, different redirect semantics.
   ⏭ First move is recon: find where the REPL path applies the guard and the
   `-c` branch does not. Delete the stale delegates-to-sh comment at
   main.rs:860 in the same change.
2. **One git policy.** `git` is in the `safe` set AND there is a
   `git reset --hard` arm in the heuristics. The arm is dead while git is
   safe. Remove git from safe, or delete the arm. Name the discarded half in
   the commit.
3. **INT-197 remainder -- VERIFY BEFORE IMPLEMENTING.** The claim is that
   `check(cmd, first_word)` gets the alias-EXPANDED first word but the TYPED
   line as cmd, so `alias zap='rm -rf /tmp/x'` then `zap` never sees `-rf`.
   INT-196 and INT-197 are both marked complete, so either this was fixed or a
   gate was ticked without demonstration. One reproduction settles it. Do that
   first.
4. **The `-c` boot tax -- CLOSED BY MEASUREMENT 2026-09-02, no change made.**
   Measured 13ms against bash's 1ms, ten runs timed externally; db open on the
   276MB file with its TRUNCATE checkpoint is 2ms and config apply is 5ms. The
   305ms figure predates the BEGIN/COMMIT already in tree. None of the three
   proposed fixes is warranted, and 160 processes x 13ms is two seconds, so the
   suite's 155s is pty sessions rather than boot.
   ~~`nsh -c true` costs ~305ms release against bash's ~3ms.~~ The hypothesis: `ForestDb::open` runs `wal_checkpoint(TRUNCATE)` on
   every process; `config::apply` re-seeds 270 aliases every time; prune and
   settings sit outside the alias transaction.
   ⚠️ THAT IS A HYPOTHESIS READ FROM THE CODE, NOT A MEASUREMENT. Profile
   first: `NSH_BOOT_PROFILE=1 NSH_OBSERVE=boot nsh -c true`, and
   `ls -l ~/.local/state/faelight/state.db*` before and after. The profile
   either confirms the chain or names something else.
5. **An honesty pass, deletions only.** `plugin-reload` help versus what it
   does; `.fsh` versus `.nsh` in the plugin help; `cmdguard` versus `guard`;
   `zero-gate --help` still offering risk tiers after risk-gate.sh died with
   nix/. Fix or delete -- do not add documentation.

### Deferred, and the reason is ordering rather than objection

Not this month, and not because they are wrong:

- A hook or plugin surface. It would be built on top of a guard that does not
  cover every door.
- INT-169 spine as the only executor.
- Splitting `commands/mod.rs`. It is 17k lines and it IS the coherence
  problem -- which is why it waits for a guard that makes the split provable
  rather than hopeful.
- New heuristics, more deny words, chmod rules.
- Making nsh the login shell. INT-190 records what that cost.

---

## Definition of Done

An item is done when **all** of the following are true:

1. It was demonstrated, not declared.
2. The evidence is recorded in the format above — what was run, what came back, and when.
3. The verification ran against the deployed artifact, not the build output.
4. Nothing else was changed along the way.

Never mark something done to be resolved later. Build it into the work, or discuss it first.
