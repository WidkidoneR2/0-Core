use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: get-version <package-name>");
        process::exit(1);
    }
    
    let package = &args[1];
    let home = env::var("HOME").expect("HOME not set");
    let dotmeta_path = PathBuf::from(&home)
        .join("0-core")
        .join(package)
        .join(".dotmeta");
    
    if !dotmeta_path.exists() {
        println!("unknown");
        process::exit(0);
    }
    
    let content = fs::read_to_string(&dotmeta_path)
        .unwrap_or_else(|_| String::new());
    
    // Find version = "x.y.z" line
    for line in content.lines() {
        if line.starts_with("version = ") {
            // Extract value between quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        println!("{}", &line[start + 1..end]);
                        return;
                    }
                }
            }
        }
    }
    
    println!("unknown");
}
