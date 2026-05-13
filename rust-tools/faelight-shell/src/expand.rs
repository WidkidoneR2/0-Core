// expand.rs — pure string utility functions
// INT-299: structural decomposition Phase 1
// Extracted from main.rs — zero state dependencies

pub fn normalize_input(s: &str) -> String {
    s.replace('‘', "'")
        .replace('’', "'")
        .replace('“', "\"")
        .replace('”', "\"")
        .replace('–', "-")
        .replace('—', "--")
}

pub fn glob_match(pattern: &str, name: &str) -> bool {
    // Simple glob: * matches anything, ? matches one char
    let mut pi = 0;
    let mut ni = 0;
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let mut star_pi = usize::MAX;
    let mut star_ni = 0;
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star_pi = pi;
            star_ni = ni;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ni += 1;
            ni = star_ni;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

pub fn strip_quoted_regions(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_s = false;
    let mut in_d = false;
    let mut prev = '\0';
    for ch in s.chars() {
        if prev == '\\' {
            if in_s || in_d {
                out.push(' ');
            } else {
                out.push(ch);
            }
            prev = ch;
            continue;
        }
        match ch {
            '\'' if !in_d => {
                in_s = !in_s;
                out.push(' ');
            }
            '"' if !in_s => {
                in_d = !in_d;
                out.push(' ');
            }
            _ => {
                if in_s || in_d {
                    out.push(' ');
                } else {
                    out.push(ch);
                }
            }
        }
        prev = ch;
    }
    out
}

pub fn count_keyword_starts(s: &str, kw: &str) -> usize {
    let mut count = 0;
    for line in s.lines() {
        let trimmed = line.trim();
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        for w in words.iter() {
            if *w == kw {
                count += 1;
            }
        }
    }
    count
}

