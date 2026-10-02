//! INT-269 step 2 -- `core fingerprint show` and `core fingerprint record`.
//!
//! show never writes. record is the ONE writer of paths::fingerprint_file(), and it refuses a
//! record with a hole: a missing input written down would make every later compare
//! UNDETERMINED. If a fingerprint is not written, nothing can move forward on it.
//!
//! Exit codes, so scripts can ask: show 0 PASS, 1 FAIL, 2 UNDETERMINED; record 0 written,
//! 1 refused.

use crate::cli::commands::FingerprintCommand;
use crate::errors::CoreResult;
use colored::Colorize;
use zero_core::fingerprint::{collect, compare, Outcome, Record, DECLARED};

pub fn run(cmd: FingerprintCommand) -> CoreResult<()> {
    let code = match cmd {
        FingerprintCommand::Show => show(),
        FingerprintCommand::Record => record(),
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn print_record(r: &Record) {
    for k in DECLARED {
        match r.facts.get(k).cloned().flatten() {
            Some(v) => println!("  {k:<20} {v}"),
            None => println!("  {k:<20} MISSING"),
        }
    }
}

fn show() -> i32 {
    let live = collect();
    let path = zero_core::paths::fingerprint_file();
    println!("fingerprint -- the nine declared inputs, read now");
    print_record(&live);
    match live.digest() {
        Some(d) => println!("  digest               {d}"),
        None => println!("  digest               none -- an input could not be read"),
    }
    let expected = match std::fs::read_to_string(&path) {
        Ok(t) => Record::from_text(&t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "{}",
                "UNDETERMINED -- not recorded yet. Run: core fingerprint record".red()
            );
            return 2;
        }
        Err(e) => {
            println!(
                "{}",
                format!(
                    "UNDETERMINED -- could not read the record at {}: {}",
                    path.display(),
                    e
                )
                .red()
            );
            return 2;
        }
    };
    match compare(&live, &expected) {
        Outcome::Pass => {
            println!(
                "{}",
                "PASS -- this is the recorded machine and tree".green()
            );
            0
        }
        Outcome::Fail(axes) => {
            println!(
                "{}",
                format!("FAIL -- these differ from the record: {}", axes.join(", ")).red()
            );
            1
        }
        Outcome::Undetermined(missing) => {
            println!(
                "{}",
                format!("UNDETERMINED -- not read: {}", missing.join(", ")).red()
            );
            2
        }
    }
}

fn record() -> i32 {
    let live = collect();
    let missing = live.missing();
    if !missing.is_empty() {
        eprintln!("REFUSED -- nothing was written. A record with a hole would make every later compare UNDETERMINED.");
        eprintln!("  could not read: {}", missing.join(", "));
        return 1;
    }
    let path = zero_core::paths::fingerprint_file();
    let previous = std::fs::read_to_string(&path)
        .ok()
        .map(|t| Record::from_text(&t))
        .and_then(|r| r.digest());
    let tmp = path.with_extension("tmp");
    let parent = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let written = std::fs::create_dir_all(&parent)
        .and_then(|_| std::fs::write(&tmp, live.to_text()))
        .and_then(|_| std::fs::rename(&tmp, &path));
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        eprintln!("REFUSED -- could not write {}: {}", path.display(), e);
        return 1;
    }
    let digest = live.digest().unwrap_or_default();
    match previous {
        Some(p) if p == digest => {
            println!("recorded {digest} -- unchanged from the previous record")
        }
        Some(p) => println!("recorded {digest} -- REPLACED the previous record {p}"),
        None => println!("recorded {digest} to {}", path.display()),
    }
    0
}
