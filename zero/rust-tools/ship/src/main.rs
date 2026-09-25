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
    /// Remove an installed tool: back it up, then take it off PATH.
    ///
    /// The inverse of shipping, and deliberately NOT a second binary. `unship` is not a word,
    /// and a separate crate would duplicate the backup naming, the record call and the prune --
    /// three things that would then drift apart, which is the two-owners defect this tree keeps
    /// finding. `retire` borrows its name from the registry, which already carries a `retired`
    /// field that deadwood reads.
    #[arg(long, value_name = "TOOL")]
    retire: Option<String>,
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
use faelight_core::differs;

/// Keep the newest three backups of one tool, delete the rest.
///
/// bin/ reached 573MB in a single day -- thirteen copies of core alone, at roughly
/// 4.8MB each -- because ship wrote a backup on every install and never removed one.
/// It is gitignored, so nothing surfaced it.
///
/// KEEP-COUNT, NOT AGE. Time-based retention behaves badly under this workload: ten
/// deploys in an afternoon then nothing for a month leaves either everything or
/// nothing, depending which side of the window you land on. A count is deterministic.
///
/// Backups are named tool@<mtime>. Older ones from before that change are named
/// tool@<version>, and a plain string sort would rank core@3.2.10 beside a ten-digit
/// epoch and keep it forever. Anything whose suffix is not all digits is treated as
/// oldest, so the legacy names age out first.
///
/// Only files ship itself wrote are ever considered. Errors are ignored deliberately:
/// a backup that cannot be deleted is disk to reclaim later, not a failed deploy.
fn prune_backups(backup_dir: &Path, tool: &str) {
    const KEEP: usize = 3;
    let rd = match std::fs::read_dir(backup_dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    let prefix = format!("{}@", tool);
    let mut found: Vec<(u64, std::path::PathBuf)> = Vec::new();
    for e in rd.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        let Some(suffix) = name.strip_prefix(&prefix) else {
            continue;
        };
        // A non-numeric suffix is a pre-mtime backup: rank it oldest.
        let stamp = suffix.parse::<u64>().unwrap_or(0);
        found.push((stamp, e.path()));
    }
    if found.len() <= KEEP {
        return;
    }
    found.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, path) in found.iter().skip(KEEP) {
        let _ = std::fs::remove_file(path);
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
/// Take an installed tool off PATH, keeping the binary where rollback can find it.
///
/// ⚠ NOT A DELETE. It moves to bin/{tool}@{stamp}, the same place and the same naming ship
/// uses for an outgoing binary, so `core deploy rollback` can restore a retired tool exactly as
/// it restores a replaced one. Retiring is a decision, not an accident, and a decision you
/// cannot undo is a worse tool than one you can.
///
/// ⚠ THE STAMP IS MTIME, NOT THE VERSION, and ship's own comment says why: several of these
/// binaries are session daemons -- lock, idle, compositor, wallpaper -- and running one to ask
/// its version is how a deploy takes down a session. This does not execute what it retires.
///
/// THE REGISTRY IS CHECKED BUT NOT EDITED. A tool marked deployable with no binary on PATH is
/// the orphan deadwood flags -- it caught novashell doing exactly that on 2026-09-01. Retiring
/// warns rather than refuses: refusing would mean editing the registry BEFORE you can retire,
/// which makes the first step impossible.
fn retire(tool: &str, bin: &Path, backup_dir: &Path, dry_run: bool) -> i32 {
    let dest = bin.join(tool);
    println!();
    println!("  📦 retire  {}", tool);
    if !dest.exists() {
        println!("     {} is not installed at {}", tool, bin.display());
        leftovers(tool);
        return 1;
    }
    let stamp = std::fs::metadata(&dest)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let keep = backup_dir.join(format!("{}@{}", tool, stamp));
    if dry_run {
        println!("     would move  {}", dest.display());
        println!("             to  {}", keep.display());
        leftovers(tool);
        return 0;
    }
    let _ = std::fs::create_dir_all(backup_dir);
    if let Err(e) = std::fs::copy(&dest, &keep) {
        eprintln!("     backup FAILED -- {}", e);
        eprintln!("     nothing removed: a retire you cannot undo is a delete");
        return 1;
    }
    if let Err(e) = std::fs::remove_file(&dest) {
        eprintln!("     could not remove {} -- {}", dest.display(), e);
        return 1;
    }
    println!("     kept  {}", keep.display());
    println!("     gone  {}", dest.display());
    let mut errors: Vec<String> = Vec::new();
    record(tool, &format!("@{}", stamp), "retired", 0, &mut errors);
    for e in &errors {
        println!("     deploy record: {}", e);
    }
    leftovers(tool);
    0
}

/// INT-260: what still names a retired tool -- the registry entry and every alias -- reported on
/// EVERY way out of `retire`, not only after a real removal.
///
/// Measured 2026-09-23: both checks sat after the removal, so `--dry-run` skipped them (the one
/// run meant to show you what you are about to do), and a tool already gone -- faelight-clipboard,
/// retired that morning -- exited "not installed" before either ran. The tool whose aliases were
/// dangling was exactly the one ship could no longer say anything about.
fn leftovers(tool: &str) {
    let registry = faelight_core::paths::core_dir().join("zero/registry/tools.toml");
    let needle = format!("name = \"{}\"", tool);
    // INT-260: an unreadable registry SAYS SO. This was `if let Ok(text)`, which printed nothing
    // when the file could not be read -- and silence there reads as "the registry is fine".
    let claimed = match std::fs::read_to_string(&registry) {
        Ok(text) => text.split("[[tool]]").any(|b| {
            b.contains(&needle) && b.contains("deployable = true") && !b.contains("retired = true")
        }),
        Err(e) => {
            println!();
            println!("     could not read {} -- {}", registry.display(), e);
            println!(
                "     so whether the registry still claims {} is UNKNOWN, not clean.",
                tool
            );
            false
        }
    };
    if claimed {
        println!();
        println!("     the registry still lists {} as deployable.", tool);
        println!("     deadwood flags it as an orphan until that entry says retired = true.");
    }
    alias_report(tool);
}

/// INT-260: every alias still naming the retired tool, in BOTH files, at the moment you can act.
///
/// Measured 2026-09-23 over two retirement rounds in one session: six dead aliases, then four, and
/// nsh-test went red on deadwood ONE RUN LATER both times. The second round missed `guard` because
/// the search was for a guessed alias name rather than the TOOL name.
///
/// IT REPORTS; IT DOES NOT REMOVE. ship tells you when you can act, deadwood proves afterwards
/// that you did. A ship that removed silently would make deadwood's check go quiet and the proof
/// would vanish with the problem.
///
/// One of these files is NOT in the repository: config.nsh is the live shell, and no commit
/// records a change to it. That is why removal would need a flag, and why this only reports.
///
/// Matching is by the tool name in the line, deliberately loose: over-reporting costs a glance,
/// under-reporting costs a red suite one run later.
fn alias_report(tool: &str) {
    let repo = faelight_core::paths::core_dir().join("zero/registry/aliases.toml");
    let live = faelight_core::paths::shell_config();
    let mut found: Vec<String> = Vec::new();
    for path in [repo, live] {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                // INT-260: the toml is scanned by BLOCK, not by neighbour. The old check looked
                // only at the line after `command =`, so faelight-gen (primary before aliases)
                // lost both of its alias lines. Field order inside a block is not a contract.
                fn key(l: &str) -> &str {
                    l.split('=').next().unwrap_or("").trim()
                }
                let lines: Vec<&str> = text.lines().collect();
                let mut start = 0;
                while start < lines.len() {
                    let mut end = start + 1;
                    while end < lines.len() && !lines[end].trim_start().starts_with("[[") {
                        end += 1;
                    }
                    let block = &lines[start..end];
                    let owns = block
                        .iter()
                        .any(|l| key(l) == "command" && l.contains(tool));
                    for (k, line) in block.iter().enumerate() {
                        let named = owns && matches!(key(line), "command" | "primary" | "aliases");
                        let live_alias =
                            line.trim_start().starts_with("alias ") && line.contains(tool);
                        if named || live_alias {
                            found.push(format!(
                                "       {}:{}  {}",
                                path.display(),
                                start + k + 1,
                                line.trim()
                            ));
                        }
                    }
                    start = end;
                }
            }
            Err(e) => found.push(format!(
                "       {} COULD NOT BE READ -- {}",
                path.display(),
                e
            )),
        }
    }
    if found.is_empty() {
        return;
    }
    println!();
    println!("     {} line(s) still name {}:", found.len(), tool);
    for f in &found {
        println!("{}", f);
    }
    println!("     the registry file is in the repository and belongs in this commit;");
    println!("     config.nsh is your live shell, and no commit will record a change to it.");
}

fn main() {
    faelight_core::restore_sigpipe();
    let args = Args::parse();
    let root = faelight_core::paths::core_dir();
    let bin = faelight_core::paths::bin_dir();
    let release = root.join("target/release");
    let backup_dir = root.join("bin");

    if let Some(tool) = args.retire.clone() {
        // INT-260: SHIP_BACKUP_DIR, so a TEST retirement does not write into the real tree.
        //
        // Measured 2026-09-23: XDG_BIN_HOME moved which binary was retired, but the backup still
        // landed in 0-core/bin, because that path came straight from core_dir(). Exercising --retire
        // therefore left stray copies behind every time -- and cleaning them up by wildcard deleted
        // two legitimate backups that were not part of the test.
        //
        // Read HERE and nowhere else. The first version read it at the top of main(), where the
        // deploy path's backups and prune_backups share the same variable -- so a stray
        // SHIP_BACKUP_DIR would have moved every deploy backup and pruned the wrong directory.
        // Unset -- which is every real retirement -- the backup goes where it always went.
        let retire_dir = match std::env::var("SHIP_BACKUP_DIR") {
            Ok(v) if !v.is_empty() => std::path::PathBuf::from(v),
            _ => backup_dir.clone(),
        };
        std::process::exit(retire(&tool, &bin, &retire_dir, args.dry_run));
    }

    println!();
    println!("  \u{1F6A2} ship");
    println!("     from  {}", release.display());
    println!("     to    {}", bin.display());

    if !args.no_build {
        // SAY WHICH THING IS MISSING.
        //
        // `Command::status()` returns ENOENT for TWO different facts: cargo is not
        // installed, and the current_dir does not exist. The arm below reported the
        // first for both, so a missing REPO read as a missing TOOLCHAIN.
        //
        // Measured 2026-09-15: under the devbox policy HOME is /tmp/devbox-*, so
        // core_dir() points somewhere that does not exist, and `ship` said
        // "cargo did not run" while cargo sat at /usr/bin/cargo. Ten minutes went
        // into looking for a toolchain problem that did not exist.
        if !root.is_dir() {
            eprintln!();
            eprintln!("  x nothing to build: {} does not exist", root.display());
            eprintln!(
                "    the repo root comes from HOME, which is {}",
                std::env::var("HOME").unwrap_or_else(|_| "<unset>".to_string())
            );
            std::process::exit(1);
        }
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
                // The directory was checked above, so this is now genuinely about cargo.
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
            prune_backups(&backup_dir, &t.name);
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
