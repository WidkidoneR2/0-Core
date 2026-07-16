# Conventions

Small rules that keep the forest honest. Each one is here because something broke without it.

---

## Evidence-backed gates (INT-158)

**A ticked box is a promise. Evidence is the receipt. Make "completed" mean "proven".**

When you tick a gate, put the proof in an HTML comment on the next line:

```markdown
- [x] Secure Boot enforcing on metal with custom keys
<!-- evidence: commit f0d0a08e, 2026-07-16. bootctl status -> Secure Boot: enabled (user),
     Measured UKI: yes. db read from the efivar = exactly 2 certs (mine + Framework's),
     ZERO Microsoft. Reboot survived; dep signed gen 383 without complaint. -->
```

Anything that lets future-you check the claim: a **commit hash**, a **file:line**, a **log or
artifact path**, or **`demonstrated: <what and how>`**. Prose counts -- the point is that the
claim is checkable, not that it has a schema.

### The three limits

**Forward-only.** Never retrofit old intents. That is busywork with no payoff.

**Soft.** Nothing enforces this. It is a discipline, not gate-police. An intent that closes
without evidence is not rejected -- it is just less trustworthy, and you will find out later.

**Light.** Trivial self-evident gates need no artifact. "File created" does not need a receipt.
"The VM boots" does.

### Why this exists

This was not invented. INT-133 was already doing it, and the strongest intents in the ledger
all did some version of it. INT-158 wrote it down.

The cost of NOT doing it was measured on 2026-07-16. An audit of the 123 intents marked
complete found gates ticked green that were not true:

- **INT-119** said rustfmt was *"sandboxed, reproducible, unskippable"*. `.git/hooks/pre-commit`
  did not exist. Nothing was ever skipped because nothing ever ran -- ~30 commits landed that
  day alone with zero complaints. **INT-113 had been retired for the identical bug six days
  earlier.** The same defect, shipped twice, with "unskippable" in the comment both times.
- **INT-061** claimed the tree was *"still in the CURRENT layout"* long after it wasn't, and
  claimed Phase 1 was *"substantially complete"* while `nix/profiles/` had never existed. Wrong
  in both directions at once.
- Three separate comments said a file *"mirrors framework16"*. All three were false, and one had
  the VM testing a different greeter than the laptop actually runs.

Every one of those would have been caught by a gate that had to cite something.

### The tell

**A gate you have only watched pass might be doing nothing.** The rustfmt hook "passed" for six
days by never running. When you can, prove a gate by watching it FAIL first -- stage something
broken and watch it get rejected -- then fix it and watch it pass. That is the difference between
a gate and a green light.

### Exemplars

INT-133 (the original), INT-161 (Secure Boot, 9 gates), INT-112 (RISK.toml, 6 gates), INT-061
(the v2 restructure), INT-027:58 -- which discharges a `(consider)` gate by **declining** it, with
four numbered reasons. A gate can be closed by deciding NOT to do the thing. That is still proof.
