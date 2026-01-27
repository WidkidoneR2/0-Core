use std::path::PathBuf;

fn main() {
    let home = PathBuf::from("/home/christian");
    let archive = PathBuf::from("/home/christian/3-archive");
    
    let (zone, display) = faelight_zone::current_zone(&archive, &home);
    println!("Zone: {:?}", zone);
    println!("Display: {}", display);
    println!("Short label: {}", zone.short_label());
}
