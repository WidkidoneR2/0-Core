//! INT-207: fsh's structured observability, owned rather than depended on.
//!
//! WHY THIS EXISTS. Five hand-rolled instruments were counted on 2026-08-23: FSH_SPINE_TRACE
//! (stderr), FSH_TRACE (stderr), FSH_BOOT_PROFILE (stderr), legacy-exec.log (file) and
//! sh-fallback.log (file). Each was decisive when it was needed -- FSH_TRACE found a lifecycle
//! recorder that had been dead three days, FSH_BOOT_PROFILE found a 210ms alias transaction -- and
//! each was written from scratch, learning the same lessons separately.
//!
//! ★ THE PROOF THAT A SHARED SCHEMA IS NEEDED IS IN THEIR OWN COMMENTS. Two file instruments
//! independently invented the SAME TWO FIELDS, each explaining why it had been added after the
//! fact: `door` ("what makes a row mean anything") and `build` ("a row that cannot be dated to a
//! binary cannot be read as evidence"). They are fields of every event, not per-caller afterthoughts.
//!
//! WHY NOT THE `tracing` CRATE. INT-198 ruled tracing as the mechanism, and that ruling STANDS --
//! but it names the CAPABILITY AND CONTRACT, not a mandatory dependency. INT-198 owns what must
//! exist; portability owns how fsh implements it. A shell that starts in 8ms and may run on musl
//! has reason to stay small, and the event SCHEMA below is the stable contract, so a `tracing`
//! backend can sit behind `emit` later without touching a single caller.
//!
//! ⚠️ AND THE CONSTRAINT THAT KEEPS THIS HONEST: do not accidentally recreate `tracing`. The API is
//! three items -- Event, emit, enabled. If it grows spans, subscribers or a registry, that is the
//! signal to adopt the real crate instead of reimplementing it badly.

use std::io::Write;
use std::sync::OnceLock;

/// THE process clock. One monotonic origin for the whole process, owned here so a caller cannot
/// create a fourth one.
///
/// ⚠️ INITIALIZED EXPLICITLY, NOT LAZILY, and the difference is the whole point. A OnceLock set on
/// first use is technically one clock, but if twelve milliseconds of work happen before the first
/// event, the clock starts late and that event reports 0ms. That is not a boot clock. If something
/// before `init()` needs measuring, move `init()` earlier rather than making the clock lazy.
static PROCESS_START: OnceLock<std::time::Instant> = OnceLock::new();

/// Where accumulating observations go, if anywhere. `None` unless FSH_OBSERVE_FILE is set.
static SINK: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();

/// Start the process clock and resolve the optional file sink.
///
/// ⚠️⚠️ THE DIRECTORY IS CREATED HERE AND ITS FAILURE IS LOUD, and that is not defensive
/// programming -- it is a repair. Two instruments wrote to `faelight/runtime/` with
/// `OpenOptions::create(true)`, which creates a FILE and never its PARENT. The directory was lost
/// in the Phase 1 tree move, so every open failed, every `if let Ok(f)` arm was skipped, and NOT
/// ONE ROW was ever written. Their emptiness was then read as "the code path is cold" -- an
/// unanswered question mistaken for an answered one.
///
/// ★ THE RULE THAT COMES OUT OF IT: an instrumentation path must FAIL VISIBLY when its destination
/// cannot be established. Otherwise "no observations" and "nothing happened" are the same output.
pub fn init() {
    let _ = PROCESS_START.set(std::time::Instant::now());
    let resolved = std::env::var_os("FSH_OBSERVE_FILE").and_then(|raw| {
        let path = std::path::PathBuf::from(raw);
        if let Some(dir) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                let _ = writeln!(
                    std::io::stderr(),
                    "fsh: observation sink unavailable: cannot create {} ({}). \
                     Events will go to stderr only -- an empty log would otherwise be \
                     indistinguishable from nothing happening.",
                    dir.display(),
                    e
                );
                return None;
            }
        }
        Some(path)
    });
    let _ = SINK.set(resolved);
}

/// The build that emitted a row. Two file instruments each added this after the fact, and both
/// left the same note: a row that cannot be dated to a binary cannot be read as evidence about the
/// current shell. So the emission path attaches it rather than each caller remembering.
fn build() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Milliseconds since this process's observability clock started.
///
/// NAMED FOR ITS CLOCK ON PURPOSE. `duration_ms`, `startup_elapsed_ms` and `command_elapsed_ms`
/// are all plausible future measurements, and a bare `elapsed_ms` would leave nobody able to say
/// which zero a number belongs to. This one means: since fsh began observing itself.
fn process_elapsed_ms() -> u128 {
    PROCESS_START
        .get()
        .map(|t| t.elapsed().as_millis())
        .unwrap_or(0)
}

/// Severity. Ordered: a target enabled at `Debug` also emits `Info` and above.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
        }
    }
}

/// What part of the shell an event came from. An enum rather than a string so a typo is a
/// compile error and the set stays enumerable -- the registry is closed by construction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Router,
    Lexer,
    Expansion,
    Executor,
    Jobs,
    Boot,
}

impl Target {
    fn as_str(self) -> &'static str {
        match self {
            Target::Router => "router",
            Target::Lexer => "lexer",
            Target::Expansion => "expansion",
            Target::Executor => "executor",
            Target::Jobs => "jobs",
            Target::Boot => "boot",
        }
    }

    fn from_str(s: &str) -> Option<Target> {
        Some(match s {
            "router" => Target::Router,
            "lexer" => Target::Lexer,
            "expansion" => Target::Expansion,
            "executor" => Target::Executor,
            "jobs" => Target::Jobs,
            "boot" => Target::Boot,
            _ => return None,
        })
    }
}

/// One observation. THIS IS THE STABLE CONTRACT -- the renderer is not.
pub struct Event<'a> {
    pub level: Level,
    pub target: Target,
    pub message: &'a str,
    /// Structured pairs. The emission path adds `door`, `build` and `correlation_id`; a caller
    /// supplies only what is specific to its own event.
    pub fields: &'a [(&'a str, String)],
}

/// Is this target enabled at this level?
///
/// FSH_OBSERVE selects targets: `FSH_OBSERVE=router,jobs` or `FSH_OBSERVE=all`. FSH_OBSERVE_LEVEL
/// sets the floor, defaulting to `debug`. Nothing is enabled without the variable, which is the
/// gate that keeps an ordinary session exactly as quiet as it is today.
pub fn enabled(target: Target, level: Level) -> bool {
    // FSH_BOOT_PROFILE IS A RENDERING MODE, NOT AN INSTRUMENT. It selects the boot target and asks
    // for the human boot format; it does not measure anything itself. That is what collapsed three
    // separate clocks into one.
    if target == Target::Boot && std::env::var("FSH_BOOT_PROFILE").is_ok() {
        return true;
    }
    let Ok(spec) = std::env::var("FSH_OBSERVE") else {
        return false;
    };
    let floor = std::env::var("FSH_OBSERVE_LEVEL")
        .ok()
        .and_then(|s| match s.as_str() {
            "trace" => Some(Level::Trace),
            "debug" => Some(Level::Debug),
            "info" => Some(Level::Info),
            "warn" => Some(Level::Warn),
            _ => None,
        })
        .unwrap_or(Level::Debug);
    if level < floor {
        return false;
    }
    spec == "all"
        || spec
            .split(',')
            .filter_map(|t| Target::from_str(t.trim()))
            .any(|t| t == target)
}

/// THE ONE EMISSION PATH. Filtering, field attachment and rendering happen here and nowhere else,
/// so a new instrument cannot invent its own destination or forget a field the way the five
/// hand-rolled ones did.
pub fn emit(ev: Event<'_>) {
    if !enabled(ev.target, ev.level) {
        return;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let elapsed = process_elapsed_ms();

    // ONE EVENT, SEVERAL RENDERERS. The boot profile is a VIEW of the same event every other sink
    // sees, which is why its numbers can no longer disagree with a trace line's.
    if ev.target == Target::Boot && std::env::var("FSH_BOOT_PROFILE").is_ok() {
        let mut boot = format!("[boot] {:>6}ms {}", elapsed, ev.message);
        for (k, v) in ev.fields {
            boot.push_str(&format!(" {}={}", k, v));
        }
        let _ = writeln!(std::io::stderr(), "{}", boot);
        return;
    }

    let mut line = format!(
        "[fsh {} {}] {}",
        ev.level.as_str(),
        ev.target.as_str(),
        ev.message
    );
    for (k, v) in ev.fields {
        line.push_str(&format!(" {}={}", k, v));
    }
    // The two fields five instruments each discovered they needed, attached once.
    line.push_str(&format!(" door={}", door()));
    if let Some(id) = correlation() {
        line.push_str(&format!(" correlation_id={}", id));
    }
    // A FIRST-CLASS PROPERTY, not an arbitrary field: every event carries it, derived from the one
    // clock, so no caller can supply a number measured from somewhere else.
    line.push_str(&format!(" process_elapsed_ms={}", elapsed));
    line.push_str(&format!(" ts={}", ts));
    line.push_str(&format!(" build={}", build()));

    let _ = writeln!(std::io::stderr(), "{}", line);

    // THE ACCUMULATING SINK, opt-in. Some questions -- "does any -c line still reach sh over a
    // whole deploy cycle?" -- cannot be answered by a stderr line nobody was watching.
    if let Some(Some(path)) = SINK.get() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{}", line);
        }
    }
}

/// Which entry point the shell was invoked through. Interactive lines and `-c` lines reach the
/// same code, and a row that does not say which is evidence about neither.
fn door() -> &'static str {
    if crate::IS_DASH_C.load(std::sync::atomic::Ordering::SeqCst) {
        "dash-c"
    } else {
        "interactive"
    }
}

/// The session:execution pair, so a trace line and a command_execution row describe the same
/// event. Read from the environment rather than minted -- one owner per typed line.
fn correlation() -> Option<String> {
    let s = std::env::var("FSH_SESSION_ID").ok()?;
    let e = std::env::var("FSH_EXECUTION_ID").ok()?;
    Some(format!("{}:{}", s, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gate that matters most: nothing is chattier by default.
    #[test]
    fn silent_without_the_variable() {
        std::env::remove_var("FSH_OBSERVE");
        assert!(!enabled(Target::Router, Level::Warn));
        assert!(!enabled(Target::Jobs, Level::Trace));
    }

    #[test]
    fn targets_select_independently() {
        std::env::set_var("FSH_OBSERVE", "router,jobs");
        assert!(enabled(Target::Router, Level::Info));
        assert!(enabled(Target::Jobs, Level::Info));
        assert!(!enabled(Target::Lexer, Level::Info));
        std::env::remove_var("FSH_OBSERVE");
    }

    #[test]
    fn level_is_a_floor() {
        std::env::set_var("FSH_OBSERVE", "all");
        std::env::set_var("FSH_OBSERVE_LEVEL", "warn");
        assert!(enabled(Target::Router, Level::Warn));
        assert!(!enabled(Target::Router, Level::Debug));
        std::env::remove_var("FSH_OBSERVE");
        std::env::remove_var("FSH_OBSERVE_LEVEL");
    }

    #[test]
    fn unknown_target_names_are_ignored_not_matched() {
        std::env::set_var("FSH_OBSERVE", "nonsense");
        assert!(!enabled(Target::Router, Level::Warn));
        std::env::remove_var("FSH_OBSERVE");
    }
}
