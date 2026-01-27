use std::path::PathBuf;
use std::env;

fn main() {
    let home = env::var("HOME").unwrap();
    let intent_dir = PathBuf::from(format!("{}/0-core/INTENT", home));
    let test_path = PathBuf::from(format!("{}/0-core/rust-tools/faelight-fm", home));
    
    println!("Intent dir: {:?}", intent_dir);
    println!("Intent dir exists: {}", intent_dir.exists());
    println!("Test path: {:?}", test_path);
    println!("Test path exists: {}", test_path.exists());
    println!();
    
    // This will use the intent module from faelight-fm
    let intents = faelight_fm::intent::find_intents_for_path(&intent_dir, &test_path);
    
    println!("Found {} intents:", intents.len());
    for intent in &intents {
        println!("  - Intent {}: {}", intent.id, intent.title);
    }
    
    if intents.is_empty() {
        println!("\nNO INTENTS FOUND - debugging...");
        
        // Test if we can read the intent file directly
        let intent_file = PathBuf::from(format!("{}/0-core/INTENT/future/074-faelight-fm-semantic-file-manager.md", home));
        println!("Testing direct file read: {:?}", intent_file);
        
        if let Some(intent) = faelight_fm::intent::Intent::from_file(&intent_file) {
            println!("  ✅ File parsed: {}: {}", intent.id, intent.title);
            
            // Test matching
            println!("\nTesting matches_path:");
            println!("  Against {:?}: {}", test_path, intent.matches_path(&test_path));
        } else {
            println!("  ❌ Failed to parse file");
        }
    }
}
