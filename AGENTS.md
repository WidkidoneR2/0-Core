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

Project 0 is a NixOS-based system. NixOS is the substrate, not the product.
The architecture defines the abstraction; Linux and NixOS provide the initial implementation.

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
6. **`dep`.** Rebuild. This is the only step that produces what actually runs.
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

The rebuild cycle:

```
edit  ->  debug shell  ->  dep  ->  reload  ->  test in the new build
```

- **`dep` is the only thing that compiles what actually runs.** A green `cargo build` is not
  proof of anything deployed.
- **`cargo build -p <crate>` is a silent no-op for deployment.** It compiles; it does not ship.
- **After an fsh rebuild, `exec faelight-shell`** to pick up the new binary. Without this you
  are testing the old one and will read a working fix as a failure.
- Nix cannot see a file that git has not been told about. `git add` before building.
- After any rename, `cargo check --workspace`. A per-crate check misses the breakage.
- Wayland crates require `nix develop`.
- `rebuild-safe`, not plain `rebuild`, for any risky change — it runs a dry-run first, gates on
  health, and rolls back if health drops. `rebuild-dry` to catch evaluation errors alone.

<!-- VERIFY: confirm rebuild-safe / rebuild-dry still exist and how they relate to dep -->

---

## Sudo

**Never change, touch, or work around sudo.**

- Do not edit `/etc/sudoers`, `/etc/sudoers.d`, or the Nix options that generate them.
- Do not add `NOPASSWD` entries for any user, command, or tool.
- Do not add `sudo` to a command that was not already privileged.
- Do not cache, script around, or otherwise avoid a password prompt.
- Never `sudo rm`. fsh blocks it deliberately.
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
- Shell behaviour must be exercised through the real REPL, not only `-c`. **`fsh -c` delegates
  to `sh`** and does not exercise the same path.
- **VM first for anything touching the compositor, the greeter, or login.** Never on bare metal
  blind.

---

## Recovery

Know the way back before you need it.

- **SafeShell:** at the greeter, F3 gives a working `fsh` with no compositor (INT-056). A broken
  compositor cannot lock you out.
- **TTY2:** `Fn+Ctrl+Alt+F2`, the kernel-level backup.
- **`rollback`** restores the previous NixOS generation.
- `docs/recovery-runbook.md` is the written procedure. Anything that invalidates a path in it
  must update it in the same commit.

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
- transactional change — a change produces a new generation, verified before it is trusted
- generation-based recovery — failure rolls back, it does not require repair
- security by default
- explicit architectural boundaries
- coherent, designed experience — designed, not decorated

Do not add functionality merely because conventional Linux distributions include it.

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

- Secure Boot is enforcing. Any change to the kernel or boot chain re-signs the UKI.
- Secure Boot keys live at `/var/lib/sbctl`, deliberately outside this repository. Never commit
  them. A rebuild from this repo alone does not reproduce them.
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
- **user** — home-manager scope. Failure is recoverable from a running session.

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

## Definition of Done

An item is done when **all** of the following are true:

1. It was demonstrated, not declared.
2. The evidence is recorded in the format above — what was run, what came back, and when.
3. The verification ran against the deployed artifact, not the build output.
4. Nothing else was changed along the way.

Never mark something done to be resolved later. Build it into the work, or discuss it first.
