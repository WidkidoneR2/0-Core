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
use colored::Colorize;
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

    /// `let NAME = VALUE` -- set a session variable, with the value expanded first.
    pub fn try_let(&mut self, line: &str) -> Option<SegmentOutcome> {
        let rest = line.trim().strip_prefix("let ")?;
        if let Some(eq) = rest.find(" = ") {
            let name = rest[..eq].trim().to_string();
            let val = rest[eq + 3..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            let expanded = crate::expand_vars(&val, self.vars(), self.last_exit());
            println!(
                "  {} {} = {}",
                colored::Colorize::bright_cyan("→"),
                colored::Colorize::bright_white(name.as_str()),
                colored::Colorize::dimmed(expanded.as_str())
            );
            self.set_var(name, expanded);
        } else {
            eprintln!(
                "  {} usage: let <name> = <value>",
                colored::Colorize::bright_red("✗")
            );
        }
        Some(SegmentOutcome::Next)
    }

    /// `export NAME=VALUE` -- set it in BOTH the process environment and the session.
    ///
    /// ⚠️ Both, deliberately: the environment is what child processes inherit, the session map is
    /// what `$NAME` expansion reads. Setting only one makes them disagree.
    pub fn try_export(&mut self, line: &str) -> Option<SegmentOutcome> {
        let rest = line.trim().strip_prefix("export ")?;
        let (name, val) = if let Some(eq) = rest.find('=') {
            (
                rest[..eq].trim(),
                rest[eq + 1..].trim().trim_matches('"').trim_matches('\''),
            )
        } else {
            (rest.trim(), "")
        };
        let expanded = crate::expand_vars(val, self.vars(), self.last_exit());
        std::env::set_var(name, &expanded);
        self.set_var(name.to_string(), expanded.clone());
        println!(
            "  {} export {} = {}",
            colored::Colorize::bright_cyan("→"),
            colored::Colorize::bright_white(name),
            colored::Colorize::dimmed(expanded.as_str())
        );
        Some(SegmentOutcome::Next)
    }

    /// `unset NAME` -- remove it from both the session and the environment.
    pub fn try_unset(&mut self, line: &str) -> Option<SegmentOutcome> {
        let name = line.trim().strip_prefix("unset ")?.trim().to_string();
        let _ = self.remove_var(&name);
        std::env::remove_var(&name);
        println!(
            "  {} unset {}",
            colored::Colorize::bright_cyan("→"),
            colored::Colorize::bright_white(name.as_str())
        );
        Some(SegmentOutcome::Next)
    }

    /// `persist NAME` -- write a variable to shell_persist so it survives the session.
    pub fn try_persist(&mut self, line: &str) -> Option<SegmentOutcome> {
        let name = line.trim().strip_prefix("persist ")?.trim().to_string();
        let env_val = std::env::var(&name).ok();
        let found = self
            .var(&name)
            .or_else(|| env_val.as_deref().and_then(|v| self.var(v)))
            .or(env_val.as_ref())
            .cloned();
        if let Some(val) = found {
            let _ = self.db().conn.execute(
                "INSERT OR REPLACE INTO shell_persist (key, value) VALUES (?1, ?2)",
                rusqlite::params![&name, &val],
            );
            println!(
                "  {} {} persisted across sessions",
                colored::Colorize::bright_cyan("→"),
                colored::Colorize::bright_white(name.as_str())
            );
        } else {
            println!(
                "  {} variable '{}' not set — use: export {}=value first",
                colored::Colorize::yellow("⚠️ "),
                name,
                name
            );
        }
        Some(SegmentOutcome::Next)
    }

    /// `friday chat [text]` -- interactive session, or one-shot when text follows.
    ///
    /// ⚠️ Two child shapes: bare `friday chat` inherits the terminal via .status(); with text it
    /// captures via .output(). Both BLOCK, so the child's status IS the command's status.
    pub fn try_friday_chat(&mut self, line: &str) -> Option<SegmentOutcome> {
        if !line.starts_with("friday chat") {
            return None;
        }
        let rest = line[11..].trim().to_string();
        // INT-189: foreground execution the user invoked -- .status()/.output() both BLOCK, so
        // this child's result IS the command result. Discarding it left the prompt
        // reporting the previous command.
        self.set_last_exit(if rest.is_empty() {
            match std::process::Command::new("friday-chat").status() {
                Ok(status) => Some(status.code().unwrap_or(1)),
                Err(_) => Some(1),
            }
        } else {
            match std::process::Command::new("friday-chat")
                .args(["chat", &rest])
                .output()
            {
                Ok(out) => {
                    print!("{}", String::from_utf8_lossy(&out.stdout));
                    Some(out.status.code().unwrap_or(1))
                }
                Err(_) => Some(1),
            }
        });
        Some(SegmentOutcome::Next)
    }

    /// Bare `friday` or `friday <question>` -- ask the daemon over its unix socket.
    ///
    /// ⚠️ Excludes lines containing a pipe: `friday ... | ...` is a PIPELINE, not a question,
    /// and must fall through to the pipeline path rather than being swallowed here.
    pub fn try_friday_ask(&mut self, line: &str) -> Option<SegmentOutcome> {
        if !(line.starts_with("friday")
            && (line == "friday" || line.starts_with("friday "))
            && !line.contains(" | "))
        {
            return None;
        }
        let question = if line == "friday" {
            "what should I work on next?".to_string()
        } else {
            line[7..].trim().to_string()
        };
        println!("  \u{1f332} Friday: {}", "thinking...".dimmed());
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let sock_path = format!("{}/.local/state/0-core/daemon.sock", home_dir);
        let q_escaped = question.replace('"', "'");
        let query_json = format!(
            r#"{{"id":2,"payload":{{"FridayQuery":{{"question":"{}","context":null}}}}}}"#,
            q_escaped
        );
        if std::path::Path::new(&sock_path).exists() {
            use std::io::{BufRead, BufReader, Write};
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&sock_path) {
                stream
                    .set_write_timeout(Some(std::time::Duration::from_millis(500)))
                    .ok();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                    .ok();
                let _ = stream.write_all(query_json.as_bytes());
                let _ = stream.write_all(b"\n");
                let mut reader = BufReader::new(&stream);
                let mut resp = String::new();
                if reader.read_line(&mut resp).is_ok() && !resp.is_empty() {
                    if resp.contains("FridayAnswer") {
                        if let Some(ans) = resp.split(r#""answer":""#).nth(1) {
                            let ans = ans.split('"').next().unwrap_or("").to_string();
                            println!();
                            println!("  \u{1f332} Friday: {}", ans.bright_white());
                            println!();
                        }
                    }
                }
            }
        } else {
            println!("  \u{26a0}  Friday daemon not running -- start with: faelight-daemon &");
        }
        // INT-169: record the status rather than leaving the PREVIOUS command's.
        // the friday query completed. A stale code here is invisible today, but `&&`
        // is about to read this value to decide whether the next part runs.
        self.set_last_exit(Some(0));
        Some(SegmentOutcome::Next)
    }

    /// A line containing `<<` -- delegated whole to sh.
    ///
    /// ⚠️ fsh does not parse heredocs; sh does. Recorded rather than hidden: this is one of the
    /// remaining places the shell hands a construct to sh instead of owning it.
    pub fn try_heredoc(&mut self, line: &str) -> Option<SegmentOutcome> {
        if !line.contains("<<") {
            return None;
        }
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(line)
            .status();
        self.set_last_exit(match status {
            Ok(s) => s.code(),
            Err(_) => Some(1),
        });
        Some(SegmentOutcome::Next)
    }
    /// INT-254 `?<query>` -- natural-language translation, with confirmation before running.
    ///
    /// ⚠️ TWO EXITS, BOTH `Next`: a diagnostic query runs its own steps and returns early,
    /// otherwise the query is translated and offered for confirmation. `None` only means the
    /// line did not start with `?`.
    ///
    /// ⚠️ READS STDIN for the y/n confirmation, so this is INTERACTIVE by nature -- a
    /// non-interactive caller must not reach it. Recorded here rather than discovered later.
    pub fn try_nl_query(&mut self, line: &str) -> Option<SegmentOutcome> {
        if !(line.starts_with('?') && line.len() > 1) {
            return None;
        }
        let query = line[1..].trim();
        // Phase 25 — auto-diagnose for complex queries
        if crate::nl::is_diagnostic(query) {
            println!();
            println!(
                "  {} Auto-diagnosing: {}",
                "🔍".normal(),
                query.bright_white()
            );
            println!();
            let steps = crate::nl::auto_diagnose(query);
            for step in &steps {
                println!("  {} {}", "→".bright_cyan(), step.dimmed());
                // Parse pipeline ops from step
                let pipe_parts: Vec<&str> = step.splitn(2, " | ").collect();
                let base = pipe_parts[0].trim();
                let pipeline_ops = if pipe_parts.len() > 1 {
                    crate::value::parse_pipeline(&format!("x | {}", pipe_parts[1..].join(" | ")))
                } else {
                    vec![]
                };
                // Resolve joins
                let pipeline_ops: Vec<crate::value::PipeOp> = pipeline_ops
                    .into_iter()
                    .map(|op| {
                        if let crate::value::PipeOp::Join { table, on } = op {
                            let right_result =
                                crate::commands::execute(&table, self.db(), self.core_root());
                            if let crate::commands::CommandResult::Value(
                                crate::value::Value::Table(rows),
                            ) = right_result
                            {
                                crate::value::PipeOp::JoinData { rows, on }
                            } else {
                                crate::value::PipeOp::JoinData { rows: vec![], on }
                            }
                        } else {
                            op
                        }
                    })
                    .collect();
                match crate::commands::execute(base, self.db(), self.core_root()) {
                    crate::commands::CommandResult::Value(v) if !pipeline_ops.is_empty() => {
                        println!(
                            "{}",
                            crate::value::apply_pipeline(v, &pipeline_ops).render()
                        );
                    }
                    crate::commands::CommandResult::Value(v) => println!("{}", v.render()),
                    crate::commands::CommandResult::Output(o) => println!("{}", o),
                    _ => {}
                }
                println!();
            }
            return Some(SegmentOutcome::Next);
        }
        let custom_patterns = crate::nl::load_toml_patterns(self.core_root());
        match crate::nl::translate_with_custom(query, &custom_patterns) {
            Some(t) => {
                print!("{}", crate::nl::render_translation(&t));
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let answer = stdin
                    .lock()
                    .lines()
                    .next()
                    .and_then(|l| l.ok())
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase();
                if answer == "y" || answer.is_empty() {
                    println!();
                    match crate::commands::execute(&t.pipeline, self.db(), self.core_root()) {
                        crate::commands::CommandResult::Value(v) => {
                            println!("{}", v.render())
                        }
                        crate::commands::CommandResult::Output(o) => println!("{}", o),
                        crate::commands::CommandResult::Error(e, _) => {
                            eprintln!("  ✗ {}", e)
                        }
                        _ => {}
                    }
                } else {
                    println!("  ○ cancelled");
                }
            }
            None => {
                eprintln!("  ✗ no pattern matched — try: ?memory hogs, ?biggest files");
            }
        }
        Some(SegmentOutcome::Next)
    }
    /// Background job -- the legacy trailing `&` path.
    ///
    /// MOVED VERBATIM from main.rs (INT-201), then repaired one defect per commit: the spawn
    /// Result is no longer discarded (76305252), and argv is no longer re-derived from text
    /// (this commit -- it goes through commands::tokenize, the shell's one quote-aware
    /// tokenizer). ONE REMAINS: trim_end_matches(" &") strips REPETITIONS, so `cmd & &`
    /// loses both ampersands. Its own commit when it comes.
    ///
    /// WITHOUT A JOB TABLE this declines and the line falls through to normal execution --
    /// correct for a non-interactive caller, which has no jobs to control.
    pub fn try_background(
        &mut self,
        line: &str,
        jobs: Option<&mut crate::jobs::JobTable>,
    ) -> Option<SegmentOutcome> {
        let segment_trimmed = line.trim_end();
        if segment_trimmed.ends_with(" &") || segment_trimmed == "&" {
            let jobs = jobs?;
            let cmd_part = segment_trimmed.trim_end_matches(" &").trim();
            if !cmd_part.is_empty() {
                // INT-195: ONE TOKENIZER, not a second derivation. splitn/split_whitespace re-derived
                // argv from raw text with no idea what a quote was, so `bash -c "echo one two" &`
                // reached the child as -c, "echo, one, two" -- quote characters included. bash took
                // `"echo` as the script and `one` as $0 and died on an unterminated quote. This is
                // INT-171 gate 1's tokenizer: the one the dispatcher and ExecContext already use.
                let tokens = crate::commands::tokenize(cmd_part);
                let cmd = tokens.first().cloned().unwrap_or_default();
                let args: Vec<String> = tokens.into_iter().skip(1).collect();
                // tokenize CAN return empty where splitn could not -- `"" &` is all quotes and no
                // token. Decline quietly rather than asking spawn to launch the empty string.
                if cmd.is_empty() {
                    return Some(SegmentOutcome::Next);
                }
                match jobs.spawn(&cmd, &args) {
                    // Exit 0 means the CHILD STARTED, not that the job succeeded -- a background
                    // job is never waited on, so its own status is not knowable here. Same
                    // meaning as a successful register() on the spine path.
                    Ok(_) => self.set_last_exit(Some(0)),
                    Err(e) => {
                        eprintln!("{} {}", "x".bright_red(), e);
                        self.set_last_exit(Some(1));
                    }
                }
            }
            return Some(SegmentOutcome::Next);
        }
        None
    }

    /// Does the QUERY LANGUAGE own this line?
    ///
    /// ★ NAMED FOR THE QUESTION, NOT FOR TODAY'S ANSWER. The body happens to recognise forest
    /// pipelines by their source word; what it is really deciding is which of fsh's two languages
    /// owns the input. The name should survive if the routing rule changes.
    ///
    /// ★★ WHY THIS IS A KEEP WHILE THE REDIRECT AND PIPELINE EXECUTORS ARE NOT (INT-201, 2026-08-05):
    /// those two exist because the shell parser could not yet absorb their constructs -- a
    /// HISTORICAL boundary. This one exists because it runs a DIFFERENT LANGUAGE the spine cannot
    /// parse at all: roughly four hundred history rows of `tt | where deployed == true`,
    /// `ps | where cpu > 0.5 | sort cpu desc`, `select * from ps where cpu > 1`. A real boundary.
    ///
    /// MOVED VERBATIM from main.rs. Two defects are PRESERVED so this is a move and nothing else:
    ///   1. `"deploys"` appears TWICE in the source list -- harmless to `contains`, a copy-paste
    ///      smell in the list that decides language routing.
    ///   2. ⚠️ `has_pipe` here is `line.contains(" | ")` and is NOT quote-aware, unlike the
    ///      `!in_quotes && ...` form used later in the loop. A forest-source command with a quoted
    ///      pipe in an argument routes here wrongly. That is a BOUNDARY CORRECTNESS issue rather
    ///      than parser polish, because under the two-languages design this predicate IS the
    ///      language router. Its own fix, its own evidence.
    pub fn try_query_executor(&mut self, line: &str) -> Option<SegmentOutcome> {
        // INT-171 gate 2: quote-aware command word for forest-pipeline detection.
        let first = crate::commands::command_word(line);
        let first = first.as_str();
        let forest_sources = [
            "from",
            "list",
            "find",
            "db",
            "intents",
            "deploys",
            "friday",
            "ps",
            "processes",
            "files",
            "tools",
            "events",
            "deploys",
        ];
        let has_pipe = line.contains(" | ");
        if forest_sources.contains(&first) && has_pipe {
            let explain = line.contains("--explain");
            let clean_line = line.replace(" --explain", "").replace("--explain", "");
            let clean_line = clean_line.as_str();
            let parts: Vec<&str> = clean_line.splitn(2, " | ").collect();
            let source_cmd = parts[0].trim();
            let stage_text = parts.get(1).copied().unwrap_or("").to_string();
            let pipe_rest = if parts.len() > 1 {
                format!("_source | {}", parts[1])
            } else {
                "_source".to_string()
            };
            let source_result = crate::commands::execute(source_cmd, self.db(), self.core_root());
            // INT-169: default to success, then let the Error arm below override with the
            // REAL code. Without this the whole branch left `$?` reporting the previous
            // command -- invisible today, load-bearing once `&&` reads it.
            self.set_last_exit(Some(0));
            match source_result {
                crate::commands::CommandResult::Value(v) => {
                    let source_count = match &v {
                        crate::value::Value::Table(rows) => rows.len(),
                        _ => 1,
                    };
                    let ops = crate::value::parse_pipeline(&pipe_rest);
                    if explain {
                        use colored::Colorize;
                        let stage_labels: Vec<String> = stage_text
                            .split(" | ")
                            .map(|s| s.trim().to_string())
                            .collect();
                        let (result, stats) =
                            crate::value::apply_pipeline_with_stats(v, &ops, &stage_labels);
                        println!("{}", result.render());
                        println!();
                        println!("  {} pipeline explain", "─".repeat(10).dimmed());
                        println!("  {:<28} {} rows", "source".bright_cyan(), source_count);
                        for stat in &stats {
                            let slow = if stat.duration_ms > 100 {
                                "  ⚠ slow"
                            } else {
                                ""
                            };
                            let zero = if stat.row_count == 0 {
                                " ← zero rows!"
                            } else {
                                ""
                            };
                            println!(
                                "  {:<28} {} rows  {}ms{}{}",
                                stat.label.bright_cyan(),
                                stat.row_count,
                                stat.duration_ms,
                                slow,
                                zero
                            );
                        }
                    } else {
                        let result = crate::value::apply_pipeline(v, &ops);
                        println!("{}", result.render());
                    }
                }
                crate::commands::CommandResult::Output(out) => println!("{}", out),
                crate::commands::CommandResult::Error(e, code) => {
                    eprintln!("  x {}", e);
                    self.set_last_exit(Some(code));
                }
                _ => {}
            }
            return Some(SegmentOutcome::Next);
        }
        None
    }

    /// `jobs` -- list background jobs.
    ///
    /// ⚠️ WITHOUT A JOB TABLE this declines entirely and the line falls through to normal
    /// execution -- correct for a non-interactive caller, which has no jobs to control.
    pub fn try_jobs(
        &mut self,
        line: &str,
        jobs: Option<&mut crate::jobs::JobTable>,
    ) -> Option<SegmentOutcome> {
        if crate::commands::command_word(line) != "jobs" {
            return None;
        }
        let jobs = jobs?;
        jobs.list();
        Some(SegmentOutcome::Next)
    }

    /// `fg [n]` -- resume a background job.
    ///
    /// ⚠️ FALLS THROUGH when the line is not job control: `fg commit`, `fg push` are aliases.
    /// INT-095: that rule lives in `is_repl_state_command`, which the spine router's exclusion
    /// consults too -- asked, never duplicated.
    ///
    /// ⚠️ WITHOUT A JOB TABLE this declines entirely and the line falls through to normal
    /// execution -- correct for a non-interactive caller, which has no jobs to control.
    pub fn try_fg(
        &mut self,
        line: &str,
        jobs: Option<&mut crate::jobs::JobTable>,
    ) -> Option<SegmentOutcome> {
        if crate::commands::command_word(line) != "fg" {
            return None;
        }
        let jobs = jobs?;
        let second = line.split_whitespace().nth(1).unwrap_or("");
        // Only intercept as job control if second token is a number
        // fg commit, fg push, etc. → fall through to execute_with_context
        // ASKED, NOT REPEATED: the router's exclusion consults the same predicate, so the
        // rule that `fg commit` is NOT job control lives in exactly one place.
        if crate::is_repl_state_command(line) {
            let id = second.parse::<usize>().unwrap_or(1);
            jobs.fg(id);
            return Some(SegmentOutcome::Next);
        }
        // Otherwise fall through — fg commit etc. handled by alias
        None
    }

    /// `kill` -- the job-spec form `kill %N`, and the real kill for everything else.
    ///
    /// ⚠️ INT-095: ONLY `kill %N` is a job spec. A PID is not a job id -- parsing any number as
    /// one made `kill <PID>` a silent no-op, which is how `vm down` left two VMs running.
    ///
    /// ⚠️ WITHOUT A JOB TABLE this declines entirely and the line falls through to normal
    /// execution -- correct for a non-interactive caller, which has no jobs to control.
    pub fn try_kill(
        &mut self,
        line: &str,
        jobs: Option<&mut crate::jobs::JobTable>,
    ) -> Option<SegmentOutcome> {
        if crate::commands::command_word(line) != "kill" {
            return None;
        }
        let jobs = jobs?;
        // INT-095: only `kill %N` is a job-spec. Everything else (PIDs, signals
        // like -9/-TERM, multiple PIDs) goes to the REAL kill -- a PID is NOT a
        // job id. The old code parsed any number as a job id, so `kill <PID>`
        // silently did nothing (corruption risk: vm down -> no-op -> two VMs).
        let arg = line.split_whitespace().nth(1).unwrap_or("");
        // Same predicate as the router's exclusion. INT-095: only `kill %N` is a job-spec,
        // and a PID parsed as a job id made `vm down` a silent no-op.
        if crate::is_repl_state_command(line) {
            // job-spec: kill %N -> the in-shell job table
            let id = arg.trim_start_matches('%').parse::<usize>().unwrap_or(0);
            if id > 0 {
                jobs.kill_job(id);
            } else {
                println!("  usage: kill %<job_id>");
            }
            return Some(SegmentOutcome::Next);
        }
        // PID / signal form: pass ALL args through to the real kill.
        let kill_args: Vec<&str> = line.split_whitespace().skip(1).collect();
        if kill_args.is_empty() {
            println!("  usage: kill <pid> | kill -SIG <pid> | kill %<job_id>");
            return Some(SegmentOutcome::Next);
        }
        match std::process::Command::new("kill").args(&kill_args).status() {
            Ok(s) if s.success() => {}
            Ok(_) => eprintln!("  kill: failed for {}", kill_args.join(" ")),
            Err(e) => eprintln!("  kill: {}", e),
        }
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

/// INT-201: execute one already-prepared segment and record its lifecycle.
///
/// ⚠️ EXTRACTION ONLY -- the advisory calls deliberately stay in the loop, in their existing
/// positions, so output order is unchanged. Moving them is a separate, named commit.
///
/// ★ Five inputs and one returned value, both MEASURED by the compiler rather than designed:
/// lifting this region reported exactly these names, inputs failing inside the function and the
/// output failing below it.
#[allow(clippy::too_many_arguments)]
pub fn execute_and_record(
    engine: &mut Engine,
    raw_line: &str,
    base_cmd: &str,
    original_line: &str,
    pipeline_ops: &[crate::value::PipeOp],
    has_external_op: bool,
    redirect: Option<(String, bool)>,
    is_fm_cmd: bool,
    fm_cwd_file: &std::path::Path,
) -> SegmentOutcome {
    let _cmd_timer_start = std::time::Instant::now();
    let execution = crate::exec::execute_with_context(
        &raw_line,
        &base_cmd,
        engine.db(),
        engine.core_root(),
        engine.before_rules(),
    );
    let execution_id = execution.execution_id;
    // INT-191: the state is derived from a BORROW, because the match below MOVES
    // `execution.result` and the outcome would be unavailable afterwards.
    let exec_state = crate::exec::execution_state(&execution.result);
    let cmd_output: Option<String> = match execution.result {
        crate::commands::CommandResult::Exit => {
            // INT-191: `break` escapes before the completion below, so this arm
            // closes its own lifecycle. exit_code is None DELIBERATELY -- this arm
            // never sets `last_exit_code`, so passing it would record the PREVIOUS
            // command's result, which is the stale-value bug INT-189 removed.
            // EXEC_EXIT already carries the meaning; no process exited.
            if let Err(e) =
                engine
                    .db()
                    .complete_command_execution(&crate::db::ExecutionCompletion {
                        session_id: crate::exec::session_id(),
                        execution_id,
                        executed_text: Some(&base_cmd),
                        state: crate::db::EXEC_EXIT,
                        exit_code: None,
                        duration_ms: Some(_cmd_timer_start.elapsed().as_millis() as u64),
                        finished_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    })
            {
                eprintln!("warning: failed to close exit command_execution record: {e}");
            }
            return SegmentOutcome::ExitShell;
        }
        crate::commands::CommandResult::Value(v)
            if !pipeline_ops.is_empty() && !has_external_op =>
        {
            // INT-189: `apply_pipeline` returns `Value`, not `Result`, so an
            // in-process value pipeline cannot report failure. 0 is not a chosen
            // policy here, it is the only coherent answer the type permits.
            // ⚠️ If that signature ever becomes fallible, this arm must change with
            // it -- a silent 0 over a real error would be the INT-189 bug returning.
            let result = crate::value::apply_pipeline(v, &pipeline_ops);
            engine.set_last_exit(Some(0));
            Some(result.render())
        }
        crate::commands::CommandResult::Value(_) if has_external_op => {
            // Pipeline contains external commands — pass full line to sh
            let sh_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(original_line)
                .output();
            match sh_output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    if !stderr.is_empty() {
                        eprint!("{}", stderr);
                    }
                    // INT-189: sh ALREADY computed this. The status was sitting
                    // in `Output` beside stdout and stderr the whole time and was
                    // simply never read, so `last_exit_code` carried over stale from
                    // the previous command. Nothing is being decided here: the
                    // semantics are whatever /bin/sh reported. `.code()` is None
                    // only if sh ITSELF was signalled -- a signalled child already
                    // arrives as 128+N through sh's own status.
                    engine.set_last_exit(Some(o.status.code().unwrap_or(1)));
                    Some(stdout)
                }
                Err(_) => {
                    // sh could not be launched at all. Leaving the code untouched
                    // would recreate the stale-state bug by another route.
                    engine.set_last_exit(Some(1));
                    None
                }
            }
        }
        crate::commands::CommandResult::Value(v) => {
            // INT-189: rendering a value is success; this arm previously left
            // `last_exit_code` carrying the previous command's result.
            engine.set_last_exit(Some(0));
            Some(v.render())
        }
        crate::commands::CommandResult::Output(out) if !pipeline_ops.is_empty() => {
            // External command with pipe — reconstruct full pipeline and run via sh
            let sh_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(original_line)
                .output();
            match sh_output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    if !stderr.is_empty() {
                        eprint!("{}", stderr);
                    }
                    // INT-189: inherit sh's status -- see the note on the Value
                    // arm above. Same omission, same repair.
                    engine.set_last_exit(Some(o.status.code().unwrap_or(1)));
                    Some(stdout)
                }
                Err(_) => {
                    engine.set_last_exit(Some(1));
                    Some(out)
                }
            }
        }
        crate::commands::CommandResult::Output(out) => {
            engine.set_last_exit(Some(0));
            Some(out)
        }
        crate::commands::CommandResult::Empty => {
            engine.set_last_exit(Some(0));
            None
        }
        crate::commands::CommandResult::Error(e, code) => {
            eprintln!("{} {}", colored::Colorize::bright_red("✗"), e);
            // INT-169: the REAL status, not an assumed 1. `ls /nonexistent` exits 2 and
            // printed "exited 2" while `$?` reported 1 -- the code was formatted into
            // the message and thrown away. It travels on the variant now.
            engine.set_last_exit(Some(code));
            None
        }
        // INT-143: UNREACHABLE BY CONSTRUCTION, not by luck. This match is fed by
        // crate::exec::execute_with_context, which dispatches through crate::commands::execute
        // (exec.rs:554), and execute() always passes allow_external: true -- so the
        // NotBuiltin arm in execute_impl cannot fire on this path. Only
        // try_builtin() can produce this variant.
        // Handled as Empty rather than todo!() or unreachable!(): BOTH PANIC, and a
        // panic here closes the shell. The codebase already knows this -- see
        // truncate_safe in commands/mod.rs, written so a multibyte anchor "never
        // panics the shell via an out-of-bounds byte slice (a panic here closes
        // fsh)". If a future refactor ever routes try_builtin through here, the
        // honest failure is a silent no-op, not a dead terminal.
        crate::commands::CommandResult::NotBuiltin => {
            engine.set_last_exit(Some(0));
            None
        }
    };
    // INT-191: close the lifecycle HERE, where the exit code finally exists.
    // postexec could not do it: the pipeline arms above decide the code after
    // `execute_with_context` has already returned.
    if let Err(e) = engine
        .db()
        .complete_command_execution(&crate::db::ExecutionCompletion {
            session_id: crate::exec::session_id(),
            execution_id,
            executed_text: Some(&base_cmd),
            state: exec_state,
            exit_code: engine.last_exit(),
            duration_ms: Some(_cmd_timer_start.elapsed().as_millis() as u64),
            finished_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        })
    {
        eprintln!("warning: failed to close command_execution record: {e}");
    }
    // Command timing intelligence — warn if command is unusually slow (INT-194)
    {
        let elapsed_ms = _cmd_timer_start.elapsed().as_millis() as i64;
        let cmd_key_owned = crate::commands::command_word(&base_cmd);
        let cmd_key = cmd_key_owned.as_str();
        if elapsed_ms > 500 {
            let _ = engine.db().conn.execute(
                "INSERT INTO shell_history (command, timestamp) VALUES (?1, ?2)",
                rusqlite::params![
                    format!("TIMING:{}:{}", cmd_key, elapsed_ms),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0)
                ],
            );
            let avg_ms: Option<f64> = engine.db().conn.query_row(
                "SELECT AVG(CAST(SUBSTR(command, INSTR(command, ':', INSTR(command, ':')+1)+1) AS REAL))
                 FROM shell_history WHERE command LIKE ?1 ORDER BY id DESC LIMIT 20",
                rusqlite::params![format!("TIMING:{}:%", cmd_key)],
                |r| r.get(0)
            ).ok().flatten();
            if let Some(avg) = avg_ms {
                if avg > 100.0 && elapsed_ms as f64 > avg * 2.0 {
                    println!(
                        "  {} {} took {}ms — {:.0}x slower than usual ({:.0}ms avg)",
                        "⚠️ ".normal(),
                        cmd_key.bright_yellow(),
                        elapsed_ms,
                        elapsed_ms as f64 / avg,
                        avg
                    );
                }
            }
            // Long command notification -- >30s fires faelight-notify
            if elapsed_ms > 30_000 {
                let secs = elapsed_ms / 1000;
                let msg = format!("{} finished in {}s", cmd_key, secs);
                // INT-299: reap child in thread to prevent zombie process
                if let Ok(mut child) = std::process::Command::new("faelight-notify")
                    .arg("--title")
                    .arg("Long command finished")
                    .arg("--body")
                    .arg(&msg)
                    .spawn()
                {
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                }
            }
        }
    }
    // Store last output for `last` command (INT-194)
    if let Some(ref out) = cmd_output {
        if !out.is_empty() {
            let _ = engine.db().conn.execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('last_output', ?1)",
                rusqlite::params![out],
            );
        }
    }
    // INT-201 — Track last command exit status for faelight-term indicator
    {
        // FIXED: this block used to RE-DERIVE success by scanning the output
        // text for the cross-mark prefix / "error" / "not found", and then
        // OVERWROTE last_exit_code with that guess. The guess was a SECOND
        // SOURCE OF TRUTH and was wrong in BOTH directions: a successful
        // command whose legitimate output mentions the word "error" (e.g. a
        // report COUNTING parse errors) was recorded as a failure, and a
        // genuinely failed builtin whose message lacks those words was recorded
        // as a success. That corrupted term_commands.exit_code, which Friday's
        // three-failures-in-a-row detector reads -- the shell was learning from
        // fabricated observations.
        //
        // The verdict is ALREADY correct: the CommandResult match above sets
        // last_exit_code (Output/Empty/NotBuiltin -> 0, Error -> 1). This block
        // now only CONSUMES it. The faelight-term cache write is kept; only the
        // re-derivation is gone.
        //
        // KNOWN GAP, recorded not hidden: four arms of that match never set
        // last_exit_code at all (both Value arms, and the two arms that spawn
        // `sh` for pipelines and discard its status), so on those paths the
        // value carries over from the previous command. The string scan was
        // crudely papering over that; removing it makes the staleness VISIBLE
        // rather than guessed. Fixing it touches pipeline execution semantics
        // (is pipeline status the last command? the first failure?) and belongs
        // in its own intent with its own verification -- deliberately NOT
        // bundled with a telemetry-corruption fix.
        let exit_ok = engine.last_exit().map(|c| c == 0).unwrap_or(true);
        let status_val = if exit_ok { "success" } else { "failure" };
        let cache_dir = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".cache/faelight");
        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::write(cache_dir.join("last-exit-status"), status_val);
    }
    // Write to file if redirect was detected, otherwise print
    if let Some(output) = cmd_output {
        if let Some((ref path, append)) = redirect {
            use std::io::Write;
            let home = std::env::var("HOME").unwrap_or_default();
            let full_path = if path.starts_with("~/") {
                format!("{}/{}", home, &path[2..])
            } else {
                path.clone()
            };
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(append)
                .truncate(!append)
                .open(&full_path);
            match file {
                Ok(mut f) => {
                    let _ = f.write_all(output.as_bytes());
                    let _ = f.write_all(
                        b"
    ",
                    );
                    let mode = if append { ">>" } else { ">" };
                    println!(
                        "  {} {} {}",
                        "○".bright_cyan(),
                        mode.dimmed(),
                        full_path.bright_white()
                    );
                }
                Err(e) => eprintln!("  ✗ redirect failed: {}", e),
            }
        } else {
            println!("{}", output);
        }
    }
    // Phase 20b — apply cwd after yazi/fm exits
    if is_fm_cmd {
        if let Ok(cwd) = std::fs::read_to_string(&fm_cwd_file) {
            let cwd = cwd.trim();
            if !cwd.is_empty() {
                let _ = std::env::set_current_dir(cwd);
            }
        }
        let _ = std::fs::remove_file(&fm_cwd_file);
    }
    SegmentOutcome::Next
}
