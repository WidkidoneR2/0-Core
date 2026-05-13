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


// — Phase 2 extractions —

pub fn find_heredoc_delimiter(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' && (i == 0 || bytes[i - 1] != b'<') {
            let mut j = i + 2;
            if j < bytes.len() && bytes[j] == b'-' {
                j += 1;
            }
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            let quote = if j < bytes.len() && (bytes[j] == b'\'' || bytes[j] == b'"') {
                let q = bytes[j];
                j += 1;
                Some(q)
            } else {
                None
            };
            let start = j;
            while j < bytes.len() {
                let b = bytes[j];
                if let Some(q) = quote {
                    if b == q {
                        break;
                    }
                } else if !b.is_ascii_alphanumeric() && b != b'_' {
                    break;
                }
                j += 1;
            }
            if j > start {
                let delim = std::str::from_utf8(&bytes[start..j]).ok()?.to_string();
                return Some(delim);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

pub fn strip_comments(input: &str) -> String {
    // INT-285 BUG 1 FIX: heredoc-aware comment stripping
    // Lines inside a heredoc body are raw data -- never strip them
    let mut result: Vec<String> = Vec::new();
    let mut in_heredoc = false;
    let mut heredoc_delim = String::new();
    for line in input.lines() {
        if in_heredoc {
            // Inside heredoc body -- preserve content exactly as written
            result.push(line.to_string());
            // Check for closing delimiter (must match exactly, trimmed)
            if line.trim() == heredoc_delim.as_str() {
                in_heredoc = false;
                heredoc_delim.clear();
            }
            continue;
        }
        // Check if this line opens a heredoc
        if let Some(delim) = find_heredoc_delimiter(line) {
            heredoc_delim = delim;
            in_heredoc = true;
            result.push(line.to_string());
            continue;
        }
        // Normal comment stripping -- only outside heredoc bodies
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let mut in_single = false;
        let mut in_double = false;
        let mut comment_pos = None;
        for (i, ch) in line.char_indices() {
            match ch {
                '\'' if !in_double => in_single = !in_single,
                '"' if !in_single => in_double = !in_double,
                '#' if !in_single && !in_double => {
                    if i == 0 || line[..i].ends_with(|c: char| c.is_whitespace()) {
                        comment_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let stripped = match comment_pos {
            Some(pos) => line[..pos].trim_end().to_string(),
            None => line.to_string(),
        };
        if !stripped.trim().is_empty() {
            result.push(stripped);
        }
    }
    result.join("\n")
}

pub fn is_complete_command(buf: &str) -> (bool, &'static str) {
    let cleaned: String = buf
        .lines()
        .map(|l| {
            let mut in_s = false;
            let mut in_d = false;
            let mut in_b = false;
            let mut prev = '\0';
            let mut idx = None;
            for (i, ch) in l.char_indices() {
                if prev == '\\' {
                    prev = ch;
                    continue;
                }
                match ch {
                    '\'' if !in_d && !in_b => in_s = !in_s,
                    '"' if !in_s && !in_b => in_d = !in_d,
                    '`' if !in_s && !in_d => in_b = !in_b,
                    '#' if !in_s && !in_d && !in_b => {
                        idx = Some(i);
                        break;
                    }
                    _ => {}
                }
                prev = ch;
            }
            if let Some(i) = idx {
                l[..i].to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Heredoc check FIRST. If unclosed, return incomplete. If closed, strip body so
    // other checkers do not parse heredoc content as shell syntax.
    let cleaned = if let Some(delim) = find_heredoc_delimiter(&cleaned) {
        let mut out = String::new();
        let mut in_heredoc = false;
        let mut found_close = false;
        for line in cleaned.lines() {
            if !in_heredoc {
                out.push_str(line);
                out.push('\n');
                if line.contains("<<") {
                    in_heredoc = true;
                }
            } else {
                if line.trim() == delim {
                    in_heredoc = false;
                    found_close = true;
                    out.push_str(line);
                    out.push('\n');
                }
                // skip body lines entirely
            }
        }
        if !found_close {
            return (false, "unclosed heredoc");
        }
        out
    } else {
        cleaned
    };

    let last_meaningful = cleaned.lines().rev().find(|l| !l.trim().is_empty());
    if let Some(l) = last_meaningful {
        if l.trim_end().ends_with('\\') {
            return (false, "trailing backslash continuation");
        }
    }

    let mut in_s = false;
    let mut in_d = false;
    let mut prev = '\0';
    for ch in cleaned.chars() {
        if prev == '\\' {
            prev = ch;
            continue;
        }
        match ch {
            '\'' if !in_d => in_s = !in_s,
            '"' if !in_s => in_d = !in_d,
            _ => {}
        }
        prev = ch;
    }
    if in_s {
        return (false, "unclosed single quote");
    }
    if in_d {
        return (false, "unclosed double quote");
    }

    let mut depth_paren: i32 = 0;
    let mut depth_brace: i32 = 0;
    let mut depth_brack: i32 = 0;
    let mut in_s2 = false;
    let mut in_d2 = false;
    let mut prev2 = '\0';
    for ch in cleaned.chars() {
        if prev2 == '\\' {
            prev2 = ch;
            continue;
        }
        match ch {
            '\'' if !in_d2 => in_s2 = !in_s2,
            '"' if !in_s2 => in_d2 = !in_d2,
            '(' if !in_s2 && !in_d2 => depth_paren += 1,
            ')' if !in_s2 && !in_d2 => depth_paren -= 1,
            '{' if !in_s2 && !in_d2 => depth_brace += 1,
            '}' if !in_s2 && !in_d2 => depth_brace -= 1,
            '[' if !in_s2 && !in_d2 => depth_brack += 1,
            ']' if !in_s2 && !in_d2 => depth_brack -= 1,
            _ => {}
        }
        prev2 = ch;
    }
    if depth_paren > 0 {
        return (false, "unclosed paren");
    }
    if depth_brace > 0 {
        return (false, "unclosed brace");
    }
    if depth_brack > 0 {
        return (false, "unclosed bracket");
    }

    let closer_map: &[(&str, &str)] = &[
        ("for", "done"),
        ("while", "done"),
        ("until", "done"),
        ("if", "fi"),
        ("case", "esac"),
    ];
    // INT-245 #13: only count control-structure keywords OUTSIDE of quoted strings.
    // Otherwise messages like "files for deploy" trip the for/done balance check
    // and fsh waits forever for a non-existent `done` to close the loop.
    let unquoted_for_keywords = strip_quoted_regions(&cleaned);
    for (open_kw, close_kw) in closer_map {
        let opens = count_keyword_starts(&unquoted_for_keywords, open_kw);
        let closes = count_keyword_starts(&unquoted_for_keywords, close_kw);
        if opens > closes {
            return (false, "unclosed control structure");
        }
    }

    (true, "")
}

pub fn detect_redirect(line: &str) -> (String, Option<(String, bool)>) {
    // INT-245 #10: detect malformed redirects BEFORE permissive pattern matching.
    // A bare `>` or `>>` with no target file is a parse error, not a literal `>`.
    // We signal the error via a sentinel target name; the caller emits the error
    // to stderr without touching the filesystem.
    let trimmed = line.trim_end();
    if trimmed.ends_with(">") || trimmed.ends_with(">>") {
        return (
            line.to_string(),
            Some(("__redirect_error_no_target__".to_string(), false)),
        );
    }

    // Match 2>/dev/null and 2>file FIRST
    if line.contains(" 2>/dev/null")
        || line.contains(" 2>&1")
        || (line.contains(" 2>") && !line.contains(" 2>="))
    {
        // Return the line as-is but signal that it needs special handling
        // The caller will handle 2> patterns natively
        return (line.to_string(), Some(("__stderr__".to_string(), false)));
    }
    // Match >> before > (order matters)
    if let Some(idx) = line.rfind(" >> ") {
        let path = line[idx + 4..].trim().to_string();
        // Only treat as redirect if path looks like a file (not a number/comparison)
        if !path.is_empty()
            && !path
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            let cmd = line[..idx].trim().to_string();
            return (cmd, Some((path, true)));
        }
    }
    if let Some(idx) = line.rfind(" > ") {
        let path = line[idx + 3..].trim().to_string();
        // Only treat as redirect if:
        // - path is not empty
        // - path does not start with a digit (comparison like > 70)
        // - path does not start with = (>= comparison)
        // - it is not inside a pipe segment before a command
        let first_char = path.chars().next();
        let is_comparison = first_char
            .map(|c| c.is_ascii_digit() || c == '=')
            .unwrap_or(false);
        if !path.is_empty() && !is_comparison {
            let cmd = line[..idx].trim().to_string();
            return (cmd, Some((path, false)));
        }
    }
    (line.to_string(), None)
}

