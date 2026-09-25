//! Run the real check set end to end. `cargo run -p zero-doctor --example run`

fn main() {
    let path = std::path::Path::new("/home/christian/0-core/zero/registry/doctor/checks.toml");
    let reg = match zero_doctor::Registry::load(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("LOAD FAILED: {}", e);
            std::process::exit(1);
        }
    };
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "LOADED: {} checks, {} refused",
        reg.checks.len(),
        reg.refused.len()
    );
    for (id, e) in &reg.refused {
        println!("  ✗ REFUSED {}: {}", id, e);
    }
    println!();

    let out = zero_doctor::run_all(&reg);
    for o in &out {
        let mark = match o.status {
            zero_doctor::Status::Pass => "✅",
            zero_doctor::Status::Warn => "⚠️",
            zero_doctor::Status::Fail => "❌",
            _ => "─",
        };
        println!("  {} {:-20}  {}", mark, o.id, o.message);
    }
    println!();

    let labels = reg.labels().count();
    let s = zero_doctor::Summary::of(&out, labels);
    println!(
        "{} measured, {} declared | pass {}  warn {}  fail {}  unknown {}",
        s.measured, s.declared, s.passing, s.warning, s.failing, s.unknown
    );
    println!("VERDICT: {:?}", zero_doctor::verdict(&out));

    // INT-199 shape, on every red outcome.
    for o in &out {
        if zero_doctor::is_red(o.status) {
            println!();
            print!("{}", zero_doctor::render(o));
        }
    }
}
