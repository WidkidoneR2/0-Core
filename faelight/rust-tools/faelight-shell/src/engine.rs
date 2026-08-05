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
use std::rc::Rc;

/// What executing ONE SEGMENT tells the caller to do next.
///
/// ★ NAMED FOR THE CONTRACT, NOT THE BOUNDARY. Today the unit is a segment: the REPL's
/// `'segments` loop has already split the line. If this later owns the split too, renaming to
/// `ExecutionOutcome` would state the wider contract -- better than overstating scope now.
///
/// ⚠️ TWO VARIANTS BECAUSE THERE ARE EXACTLY TWO, measured: 25 `continue 'segments` and 4
/// `break 'repl`. The 12 `continue 'repl` sites all sit ABOVE the segments loop and stay with
/// the REPL. There is no `break 'segments` anywhere -- `&&`/`||` short-circuiting rides on the
/// per-line `prev_op`, so an "abandon this line" variant would invent a capability fsh lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentOutcome {
    /// Carry on with the next segment of this command line.
    Next,
    /// The shell should exit.
    ExitShell,
}

/// Everything required to execute one line, and nothing required to draw a prompt.
///
/// ⚠️ EACH FIELD IS HERE FOR A MEASURED REASON (INT-201 gate 1). A census of the REPL loop found
/// eleven bindings; eight of them are session furniture the executor never reads.
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
    /// ⚠️ Rc, not a plain owner: the completion helper holds `&ForestDb` for the WHOLE
    /// session (rustyline stores it), which pinned the engine as immutably borrowed and made
    /// every `&mut self` method uncallable inside the loop. Rc states what was already true --
    /// the database is a SHARED resource, not exclusively-owned execution state. Rc rather
    /// than Arc because no thread in this crate ever takes the db.
    db: Rc<ForestDb>,

    /// The forest root. A String rather than a PathBuf because every consumer here takes `&str`.
    core_root: String,

    /// The before-run rules from config.fsh. All five `cfg` uses in the loop were `before_rules`,
    /// so the engine takes the rules and the loop stops needing the config at all.
    before_rules: Vec<BeforeRunRule>,
}

impl Engine {
    pub fn new(db: ForestDb, core_root: String, before_rules: Vec<BeforeRunRule>) -> Self {
        Self {
            shell_vars: HashMap::new(),
            last_exit_code: None,
            db: Rc::new(db),
            core_root,
            before_rules,
        }
    }

    /// A second handle to the database, for session-lived holders like the completion helper.
    pub fn db_handle(&self) -> Rc<ForestDb> {
        Rc::clone(&self.db)
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

    /// INT-342 `db-browse` -- launch the state.db TUI browser.
    ///
    /// ★ THE FIRST HANDLER MOVED OUT OF THE REPL LOOP. `None` means "not mine, keep looking",
    /// so the REPL stays a chain of guards; `Some(..)` is the outcome it must honour.
    pub fn try_db_browse(&mut self, line: &str) -> Option<SegmentOutcome> {
        if line != "db-browse" && !line.starts_with("db-browse ") {
            return None;
        }
        let table_arg = if line.len() > 10 {
            line[10..].trim().to_string()
        } else {
            String::new()
        };
        let mut cmd = std::process::Command::new("db-browse");
        if !table_arg.is_empty() {
            cmd.arg(&table_arg);
        }
        // INT-189: foreground execution the user invoked -- .status() BLOCKS, so this child's
        // result IS the command result. Discarding it left the prompt reporting the previous one.
        self.set_last_exit(match cmd.status() {
            Ok(status) => Some(status.code().unwrap_or(1)),
            Err(_) => Some(1),
        });
        Some(SegmentOutcome::Next)
    }

    /// Absorb a `CommandResult`: print it, record its exit status, and say what happens next.
    ///
    /// ★ THIS WAS DUPLICATED VERBATIM at the spine-exec door and the spine router, differing only
    /// in one diagnostic string -- hence `source_label`, so the two messages stay distinct rather
    /// than being quietly unified.
    ///
    /// ⚠️ ENGINE DEPENDS ON THE COMMANDS LAYER HERE, deliberately: the engine sits ABOVE commands
    /// and drives it. That is the opposite of the spine's `CommandRunner`, which refuses
    /// `CommandResult` on purpose because plan.rs sits BELOW and must not learn shell types.
    ///
    /// ⚠️ `Exit` returns rather than setting a code: the shell never sets `last_exit` on exit, and
    /// recording one would report the PREVIOUS command's status (the INT-189 stale-value bug).
    pub fn absorb_result(
        &mut self,
        result: crate::commands::CommandResult,
        source_label: &str,
    ) -> SegmentOutcome {
        use crate::commands::CommandResult;
        match result {
            // INT-169: a command that PRINTED succeeded. Without this the previous failure's code
            // survives, and since the chain decision reads it, `false || echo one && echo two`
            // skipped the last part.
            CommandResult::Output(out) => {
                println!("{}", out);
                self.set_last_exit(Some(0));
            }
            CommandResult::Value(v) => {
                println!("{}", v.render());
                self.set_last_exit(Some(0));
            }
            // INT-169: the REAL status, not an assumed 1. `ls /nonexistent` printed "exited 2"
            // while `$?` reported 1 -- the code was formatted into the message and thrown away.
            CommandResult::Error(e, code) => {
                eprintln!("{} {}", colored::Colorize::bright_red("x"), e);
                self.set_last_exit(Some(code));
            }
            CommandResult::Empty => self.set_last_exit(Some(0)),
            // Named rather than a catch-all: a `_` arm would silently swallow Exit, so
            // `spine-exec exit` would print instead of leaving the shell.
            CommandResult::Exit => return SegmentOutcome::ExitShell,
            CommandResult::NotBuiltin => {
                // Unreachable in practice -- execute_plan_dispatch converts NotBuiltin into a
                // direct spawn or an alias diagnostic. Handled honestly rather than panicking.
                eprintln!("  {source_label}: no arm matched and no spawn attempted");
                self.set_last_exit(Some(1));
            }
        }
        SegmentOutcome::Next
    }

    /// INT-169 `friday dismiss [trigger]` -- tell the daemon to drop a pattern.
    ///
    /// ⚠️ Best-effort over the unix socket: if the daemon is absent or silent the command still
    /// succeeds, because dismissing a suggestion nobody is making is not an error.
    pub fn try_friday_dismiss(&mut self, line: &str) -> Option<SegmentOutcome> {
        if line != "friday dismiss" && !line.starts_with("friday dismiss ") {
            return None;
        }
        let trigger = if line == "friday dismiss" {
            "null".to_string()
        } else {
            format!("\"{}\"", line[15..].trim().replace('"', "'"))
        };
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let sock_path = format!("{}/.local/state/0-core/daemon.sock", home_dir);
        let dismiss_json = format!(
            r#"{{"id":3,"payload":{{"FridayDismiss":{{"pattern_trigger":{}}}}}}}"#,
            trigger
        );
        if std::path::Path::new(&sock_path).exists() {
            use std::io::{BufRead, BufReader, Write};
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&sock_path) {
                stream
                    .set_write_timeout(Some(std::time::Duration::from_millis(200)))
                    .ok();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .ok();
                let _ = stream.write_all(dismiss_json.as_bytes());
                let _ = stream.write_all(b"\n");
                let mut reader = BufReader::new(&stream);
                let mut resp = String::new();
                if reader.read_line(&mut resp).is_ok() && resp.contains("FridaySpeak") {
                    if let Some(msg) = resp.split("\"message\":\"").nth(1) {
                        if let Some(msg) = msg.split('"').next() {
                            if !msg.is_empty() && msg != "null" {
                                println!("  \u{1f332} Friday: {}", msg);
                            }
                        }
                    }
                }
            }
        }
        // INT-169: record the status rather than leaving the PREVIOUS command's. A stale code here
        // is invisible today, but `&&` is about to read this value to decide whether the next runs.
        self.set_last_exit(Some(0));
        Some(SegmentOutcome::Next)
    }

    /// `friday <subcommand>` -- delegate to `core friday ...`.
    ///
    /// ⚠️ `None` MEANS "KEEP LOOKING", NOT "THE PREFIX DID NOT MATCH". `friday ` matching but
    /// naming no known subcommand deliberately falls through to the FQL guard below it, which is
    /// why this cannot be written as an early-return-on-prefix.
    pub fn try_friday_subcommand(&mut self, line: &str) -> Option<SegmentOutcome> {
        let rest = line.strip_prefix("friday ")?.trim();
        const SUBCMDS: [&str; 19] = [
            "status",
            "suggest",
            "observe",
            "extract-patterns",
            "update-personality",
            "seed-knowledge",
            "learning-loop",
            "vocabulary",
            "propose-intent",
            "phase2-init",
            "phase2-status",
            "plan",
            "temporal-models",
            "detect-temporal-patterns",
            "resolve-contradictions",
            "health-forecast",
            "interrupt-level",
            "cross-intent-patterns",
            "phase2-status-full",
        ];
        let is_sub = SUBCMDS.contains(&rest)
            || rest.starts_with("name-abstraction ")
            || rest.starts_with("ask ");
        if !is_sub {
            return None;
        }
        let mut cmd = std::process::Command::new("core");
        cmd.arg("friday");
        if let Some(q) = rest.strip_prefix("ask ") {
            cmd.arg("ask");
            cmd.arg(q.trim());
        } else {
            for a in rest.split_whitespace() {
                cmd.arg(a);
            }
        }
        // INT-189: foreground execution the user invoked; its status is the command's status.
        self.set_last_exit(match cmd.status() {
            Ok(status) => Some(status.code().unwrap_or(1)),
            Err(_) => Some(1),
        });
        Some(SegmentOutcome::Next)
    }

    /// INT-279 FQL -- `friday where/show/explain/trace/recall <query>`.
    pub fn try_friday_query(&mut self, line: &str) -> Option<SegmentOutcome> {
        const VERBS: [&str; 5] = [
            "friday where ",
            "friday show ",
            "friday explain ",
            "friday trace ",
            "friday recall ",
        ];
        if !VERBS.iter().any(|v| line.starts_with(v)) {
            return None;
        }
        let query = line[7..].trim().to_string(); // strip "friday "
                                                  // INT-189: .output() BLOCKS, so this child's result IS the command result. Discarding it
                                                  // left the prompt reporting the previous command.
        self.set_last_exit(
            match std::process::Command::new("friday-chat")
                .args(["chat", &query])
                .output()
            {
                Ok(out) => {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    if !result.trim().is_empty() {
                        println!("{}", result.trim());
                    }
                    Some(out.status.code().unwrap_or(1))
                }
                Err(_) => Some(1),
            },
        );
        Some(SegmentOutcome::Next)
    }

    /// The last command's exit status.
    pub fn last_exit(&self) -> Option<i32> {
        self.last_exit_code
    }

    /// Record the exit status of the command that just ran.
    ///
    /// ⚠️ 52 of the loop's 69 references to this field are WRITES -- it is the most-assigned
    /// binding in `repl_main`, and `&&` / `||` decide the next segment from it, so a missed
    /// write changes control flow rather than merely misreporting a status.
    pub fn set_last_exit(&mut self, code: Option<i32>) {
        self.last_exit_code = code;
    }

    /// Read one session variable. `None` means UNSET, matching `expand_vars`.
    pub fn var(&self, name: &str) -> Option<&String> {
        self.shell_vars.get(name)
    }

    /// All session variables, for `expand_vars`, which takes the map itself.
    pub fn vars(&self) -> &HashMap<String, String> {
        &self.shell_vars
    }

    /// Set one session variable.
    pub fn set_var(&mut self, name: String, value: String) {
        self.shell_vars.insert(name, value);
    }

    /// Unset one session variable, returning its previous value if it had one.
    pub fn remove_var(&mut self, name: &str) -> Option<String> {
        self.shell_vars.remove(name)
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
