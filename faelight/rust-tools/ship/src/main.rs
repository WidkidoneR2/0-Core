// ship -- build the workspace and place binaries where PATH can see them.
//
// THE ACT THAT NIXOS USED TO PERFORM. `dep` was nixos-rebuild: one command
// reconciled the whole system from source, binaries landed in the store, and the
// PATH directory was regenerated. Nothing ever copied a file, which is why
// `scripts/` could be deleted in e733287d with thirty-six references still
// pointing at it and nobody noticed.
//
// On Arch there is no reconciler. This is that missing half.
//
// THREE RULES IT WILL NOT BREAK:
//   1. It only touches binaries cargo says it built. The sixteen third-party
//      tools in the bin directory (and both cargo symlinks) are safe BY
//      CONSTRUCTION, not by an exclusion list that could go stale.
//   2. It asks cargo for the target list rather than reading the directory.
//      target/release also holds libfaelight_core, libfaelight_git and
//      libfaelight_zone -- rlibs that must never be shipped, and no naming rule
//      could tell them apart reliably. The compiler knows; ask it.
//   3. It replaces a running binary by rename, never by overwrite. A copy onto
//      a live executable gives Text file busy; a rename leaves the running
//      process on its unlinked inode and installs the new one atomically.
//      Verified on the live `core` binary before this file was written.
use clap::Parser;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ship", about = "Build and install workspace binaries")]
struct Args {
    /// Ship a single tool by name. Omit to ship everything.
    tool: Option<String>,
    /// Report what would change and touch nothing.
    #[arg(long)]
    dry_run: bool,
    /// Skip the cargo build and install what is already in target/release.
    #[arg(long)]
    no_build: bool,
    /// Do not keep a versioned copy of the outgoing binary.
    #[arg(long)]
    no_backup: bool,
}

struct Target {
    name: String,
    version: String,
}

fn metadata_targets(root: &Path) -> Result<Vec<Target>, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo metadata failed to run: {}", e))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad metadata json: {}", e))?;
    let pkgs = v["packages"].as_array().ok_or("no packages in metadata")?;
    let mut found = Vec::new();
    for p in pkgs {
        let version = p["version"].as_str().unwrap_or("unknown").to_string();
        if let Some(targets) = p["targets"].as_array() {
            for t in targets {
                let is_bin = t["kind"]
                    .as_array()
                    .map(|k| k.iter().any(|x| x == "bin"))
                    .unwrap_or(false);
                if is_bin {
                    if let Some(n) = t["name"].as_str() {
                        found.push(Target {
                            name: n.to_string(),
                            version: version.clone(),
                        });
                    }
                }
            }
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

// Cheap first, exact second. Length differs -> certainly changed. Same length ->
// read both, because a rebuild that produces an identical binary is common and
// reinstalling it would make every run look like it did work it did not do.
fn differs(a: &Path, b: &Path) -> bool {
    let (ma, mb) = (std::fs::metadata(a), std::fs::metadata(b));
    match (ma, mb) {
        (Ok(x), Ok(y)) => {
            if x.len() != y.len() {
                return true;
            }
            match (std::fs::read(a), std::fs::read(b)) {
                (Ok(da), Ok(db)) => da != db,
                _ => true,
            }
        }
        _ => true,
    }
}

fn install(src: &Path, dest: &Path) -> Result<(), String> {
    let dir = dest.parent().ok_or("destination has no parent")?;
    let stem = dest.file_name().and_then(|s| s.to_str()).unwrap_or("tool");
    // Temp file must sit in the SAME directory: rename is only atomic within one
    // filesystem, and a cross-device rename would fall back to a copy -- which is
    // the exact failure this avoids.
    let tmp = dir.join(format!(".{}.ship-tmp", stem));
    std::fs::copy(src, &tmp).map_err(|e| format!("stage failed: {}", e))?;
    let mut perm = std::fs::metadata(&tmp)
        .map_err(|e| format!("stat failed: {}", e))?
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
    std::fs::set_permissions(&tmp, perm).map_err(|e| format!("chmod failed: {}", e))?;
    std::fs::rename(&tmp, dest).map_err(|e| format!("rename failed: {}", e))?;
    Ok(())
}

// The deploy log is only worth reading if a write that fails SAYS SO. The first
// version of this called core with `let _ = ... .output()` and threw the result
// away -- twenty-one ships recorded nothing and the log sat unchanged at a date
// three months old, silently. That is the same discarded-error shape this
// session found in ade and in the old update_readiness, written into the fix for
// it. Errors are collected and reported at the end.
//
// NOTE: duration is a FLAG (--duration-ms), not a positional. Passing it
// positionally is what the first version did, and clap rejected every call.
fn record(tool: &str, version: &str, outcome: &str, ms: i64, errors: &mut Vec<String>) {
    let out = Command::new("core")
        .args([
            "deploy",
            "record",
            tool,
            version,
            outcome,
            "--duration-ms",
            &ms.to_string(),
        ])
        .output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let why = String::from_utf8_lossy(&o.stderr).trim().to_string();
            errors.push(format!(
                "{}: {}",
                tool,
                if why.is_empty() {
                    "core exited nonzero".to_string()
                } else {
                    why
                }
            ));
        }
        Err(e) => errors.push(format!("{}: core did not run ({})", tool, e)),
    }
}

// THE LABEL MUST DESCRIBE THE BINARY BEING SAVED, NOT THE ONE REPLACING IT.
// cargo metadata reports the CURRENT source version, so using it here would name
// the outgoing binary after its successor: rollback would restore the old code
// under the new version string and report a version that never existed at that
// path. Ask the outgoing binary itself.
//
// Fall back to a timestamp rather than to a guess. A file named @unknown-<epoch>
// is honest about what it does not know; a wrong version number is not, and
fn main() {
    let args = Args::parse();
    let root = faelight_core::paths::core_dir();
    let bin = faelight_core::paths::bin_dir();
    let release = root.join("target/release");
    let backup_dir = root.join("bin");

    println!();
    println!("  \u{1F6A2} ship");
    println!("     from  {}", release.display());
    println!("     to    {}", bin.display());

    if !args.no_build {
        println!();
        println!("  building release profile...");
        let started = Instant::now();
        let st = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&root)
            .status();
        match st {
            Ok(s) if s.success() => {
                println!("  build ok in {:.1}s", started.elapsed().as_secs_f64());
            }
            Ok(_) => {
                eprintln!("  build FAILED -- nothing shipped");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("  cargo did not run: {}", e);
                std::process::exit(1);
            }
        }
    }

    let targets = match metadata_targets(&root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  could not read cargo metadata: {}", e);
            std::process::exit(1);
        }
    };

    let selected: Vec<&Target> = match &args.tool {
        Some(name) => targets.iter().filter(|t| &t.name == name).collect(),
        None => targets.iter().collect(),
    };
    if selected.is_empty() {
        eprintln!("  no binary target named {}", args.tool.unwrap_or_default());
        eprintln!(
            "  known: {}",
            targets
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(1);
    }

    if let Err(e) = std::fs::create_dir_all(&bin) {
        eprintln!("  cannot create {}: {}", bin.display(), e);
        std::process::exit(1);
    }

    let mut shipped: Vec<String> = Vec::new();
    let mut unchanged = 0usize;
    let mut missing: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    let mut record_errors: Vec<String> = Vec::new();
    println!();

    for t in &selected {
        let src = release.join(&t.name);
        let dest = bin.join(&t.name);
        if !src.exists() {
            missing.push(t.name.clone());
            continue;
        }
        if !differs(&src, &dest) {
            unchanged += 1;
            continue;
        }
        if args.dry_run {
            println!("  would ship  {}  {}", t.name, t.version);
            shipped.push(t.name.clone());
            continue;
        }
        // Versioned copy of the OUTGOING binary. core deploy rollback already
        // looks for bin/{name}@{version} and has never found one, because
        // nothing ever wrote there -- so rollback has been a command that could
        // not roll back. This is the half it was missing.
        if !args.no_backup && dest.exists() {
            let _ = std::fs::create_dir_all(&backup_dir);
            // NOT the version. ship does NOT execute the binary it is replacing:
            // several of these are session daemons (lock, idle, compositor,
            // wallpaper) and running one to ask its version is how a deploy
            // takes down a session. The version is already recorded per install
            // by core deploy record; this name only has to be unique and
            // ordered, and mtime is both without running anything.
            let stamp = std::fs::metadata(&dest)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let keep = backup_dir.join(format!("{}@{}", t.name, stamp));
            if let Err(e) = std::fs::copy(&dest, &keep) {
                println!("  backup of {} FAILED -- {}", t.name, e);
            }
        }
        let started = Instant::now();
        match install(&src, &dest) {
            Ok(()) => {
                let ms = started.elapsed().as_millis() as i64;
                println!("  shipped  {}  {}", t.name, t.version);
                shipped.push(t.name.clone());
                record(&t.name, &t.version, "success", ms, &mut record_errors);
            }
            Err(e) => {
                println!("  FAILED   {}  -- {}", t.name, e);
                failed.push(t.name.clone());
                record(&t.name, &t.version, "failed", 0, &mut record_errors);
            }
        }
    }

    println!();
    println!(
        "  {} shipped  {} unchanged  {} failed  {} not built",
        shipped.len(),
        unchanged,
        failed.len(),
        missing.len()
    );
    if !missing.is_empty() {
        println!("  not built: {}", missing.join(", "));
    }
    if !record_errors.is_empty() {
        println!();
        println!(
            "  {} deploy record write(s) FAILED -- the log is incomplete:",
            record_errors.len()
        );
        for e in &record_errors {
            println!("    {}", e);
        }
    }
    if shipped.iter().any(|n| n == "ship") && !args.dry_run {
        println!(
            "  note: ship replaced itself -- this process is still the old code until it exits"
        );
    }
    println!();
    if !failed.is_empty() {
        std::process::exit(1);
    }
}
