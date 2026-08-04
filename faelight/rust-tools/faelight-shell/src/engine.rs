//! INT-201: the fsh execution engine -- the thing that runs a line, independent of who asked.
//!
//! ★ WHY THIS EXISTS. Until now nothing outside main.rs could execute a line at all. The spine
//! router had one caller, the background door one, the legacy dispatcher two, and every one of them
//! sat inside the REPL loop -- so `fsh -c` could not route through fsh and delegated the whole
//! string to `sh` instead. The pipeline was orchestrated in a loop rather than implemented in a
//! function, and the coupling ran through mutable locals rather than through arguments.
//!
//! ★ THE DIVISION OF LABOUR, and it is the reason this is not a fourth "Context":
//!   `ExecContext`   -- per-command PROVENANCE (raw, expanded, cwd, timestamp, execution id).
//!   `ShellContext`  -- the ephemeral READ-ONLY view variable resolution needs.
//!   `Engine`        -- the long-lived MUTABLE OWNER that produces the other two and runs commands.
//! An Engine owns; a ShellContext is a snapshot it lends out.

use crate::config::BeforeRunRule;
use crate::db::ForestDb;
use crate::exec::ShellContext;
use std::collections::HashMap;

/// Everything required to execute one line, and nothing required to draw a prompt.
///
/// ⚠️ EACH FIELD IS HERE FOR A MEASURED REASON (INT-201 gate 1). A census of the REPL loop found
/// eleven bindings; eight of them are session furniture the executor never reads.
#[allow(dead_code)]
pub struct Engine {
    /// Session variables. OWNED and MUTABLE: `FOO=1 cmd` inserts here, and the prefix form must
    /// restore the previous value afterwards -- a failed command has no business mutating durable
    /// state (INT-143 BUG B). `ShellContext` borrows this read-only for resolution.
    shell_vars: HashMap<String, String>,

    /// The last command's exit status. 77 reads and writes in the loop -- the dominant coupling,
    /// and the reason a shared owner was needed at all. `&&` and `||` decide the next segment from
    /// it, so a stale value silently changes control flow rather than merely misreporting.
    last_exit_code: Option<i32>,

    /// The forest database. A RESOURCE, not state: builtins read it, history and telemetry write
    /// through it, and the alias table lives in it.
    db: ForestDb,

    /// The forest root. A String rather than a PathBuf because every consumer here takes `&str`.
    core_root: String,

    /// The before-run rules from config.fsh. All five `cfg` uses in the loop were `before_rules`,
    /// so the engine takes the rules and the loop stops needing the config at all.
    before_rules: Vec<BeforeRunRule>,
}

#[allow(dead_code)]
impl Engine {
    pub fn new(db: ForestDb, core_root: String, before_rules: Vec<BeforeRunRule>) -> Self {
        Self {
            shell_vars: HashMap::new(),
            last_exit_code: None,
            db,
            core_root,
            before_rules,
        }
    }

    /// The forest database. A SHARED borrow is enough for every caller: not one db method
    /// main.rs calls takes `&mut self` (all nine are `&self`, even the four that write), and
    /// `conn` is public, so `engine.db().conn.execute(..)` needs no extra surface here.
    pub fn db(&self) -> &ForestDb {
        &self.db
    }

    /// The forest root, as `&str` because every consumer in the loop takes one.
    pub fn core_root(&self) -> &str {
        &self.core_root
    }

    /// The before-run rules. Every `cfg` use inside the loop was this field, so the
    /// engine takes them by partial move and the loop stops holding a config at all.
    pub fn before_rules(&self) -> &[BeforeRunRule] {
        &self.before_rules
    }

    /// Lend the read-only view that variable resolution and the spine router need.
    ///
    /// ⚠️ BUILT PER CALL, DELIBERATELY. `ShellContext` is a snapshot: it borrows the variables and
    /// COPIES the exit code, so holding one across an execution would hand the spine a value the
    /// command it is about to run is going to change.
    pub fn shell_context(&self) -> ShellContext<'_> {
        ShellContext {
            shell_vars: &self.shell_vars,
            last_exit_code: self.last_exit_code,
        }
    }
}
