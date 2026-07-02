// faelight-fm v3.1 -- active intent context

pub fn get_active_intent() -> String {
    let intents_dir = faelight_core::paths::intents_dir().join("in-progress");
    if let Ok(entries) = std::fs::read_dir(&intents_dir) {
        let mut files: Vec<_> = entries.flatten()
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
            .collect();
        files.sort_by_key(|e| e.file_name());
        if files.is_empty() { return "No active intents".to_string(); }
        for f in &files {
            if let Ok(content) = std::fs::read_to_string(f.path()) {
                for line in content.lines() {
                    if let Some(t) = line.strip_prefix("title:") {
                        let title = t.trim().trim_matches('"');
                        let short = if title.len() > 35 { &title[..35] } else { title };
                        let more = if files.len() > 1 {
                            format!(" (+{})", files.len()-1)
                        } else { String::new() };
                        return format!("▸ {}{}", short, more);
                    }
                }
            }
        }
    }
    "No active intents".to_string()
}
