
## Dependency Satisfaction Contract (G4 -- decided 2026-08-17)

**The rule, stated before the code changes, as this gate requires.**

| Dependency status | Satisfies? | Effect on the dependent |
| --- | --- | --- |
| `complete` | ✅ yes | unblocked, no flag |
| `cancelled` | ✅ yes | **unblocked, but FLAGGED as questionable** |
| `planned` | ❌ no | blocked |
| `in-progress` | ❌ no | blocked |
| `deferred` | ❌ no | blocked -- paused is not abandoned |
| `superseded` | -- | out of scope; the status does not exist yet |

### Why cancelled satisfies

⚠️ Today only `complete` clears, so **a cancelled dependency blocks its dependent forever**, and
eight cancelled intents exist. That is plainly wrong: when B is cancelled, A is no longer waiting
for anything.

★ But silently clearing it is also wrong. **Cancellation removes the blocking condition without
retroactively making the dependency assumption true.**

**Proven live, 2026-08-17.** INT-223 declared `depends_on: [175]`. INT-175 is cancelled -- and
cancelled *because its premise was false*: `faelight-daemon` is Arch-era prototype code and there
was never a NixOS event bus to finish. So 223 encoded the assumption *the event bus will be
finished*, and that assumption became **false, not satisfied**. What 223 actually needed was a
decision, and it got one (decisions/147).

Had the flag existed, it would have surfaced that faster than `core intent blocked` did.

### The flag

A dependent whose dependency was cancelled is reported as:

> depends on a cancelled intent -- the assumption behind this edge may no longer hold

⚠️ This is **INT-192 applied to edges**: a state that is neither clean nor blocked must be
expressible, or it gets reported as one of the two. Neither "blocked forever" nor "silently fine"
is the truth here.

### Superseded

Doc A proposed a `superseded` status; nothing implements it. **Deciding its semantics blind would be
guessing.** If it is introduced later, it satisfies **only through the completed replacement** --
a chain (`A depends_on B`, `B superseded_by C`, satisfied only when `C` is complete), not a flag.

### Implementation note

⚠️ **The rule has FIVE owners, not one.** `complete_ids` is rebuilt independently in `start` (802),
`blocked` (2702), `next_intent` (2766), `brief` (2859) and `graph` (2927) -- all user-facing
commands, all treating only `complete` as satisfying.

★ The fix is **one shared helper and five call sites**, following INT-070 (*"three copies of this
logic existed before"*) and INT-135 (*"now calls the ONE validator"*).

📍 `deps` (1696) and `deps_critical_path` (2553) do **not** build `complete_ids` -- they determine
satisfaction some other way. Check both before claiming the helper covers everything.
