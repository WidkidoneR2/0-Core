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

/// The delimiter AND whether it was written quoted.
///
/// INT-169 G1: the quoting was always computed here -- it decides where the delimiter token ends --
/// and then thrown away, because the only caller wanted a name. A quoted delimiter (`<<'EOF'`) means
/// the body does not expand, which the scanner has to report and a future executor has to honour.
/// Same shape as the audit's Difference: the answer was already known and the type would not say it.
pub fn find_heredoc_intro(s: &str) -> Option<(String, bool)> {
    find_heredoc_intro_inner(s)
}

/// Name only. Kept so its three existing callers are untouched by G1.
pub fn find_heredoc_delimiter(s: &str) -> Option<String> {
    find_heredoc_intro_inner(s).map(|(d, _)| d)
}

fn find_heredoc_intro_inner(s: &str) -> Option<(String, bool)> {
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
                return Some((delim, quote.is_some()));
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

/// INT-099: split a completed multi-line buffer into independent logical commands.
/// Boundary detection is delegated to is_complete_command, so heredocs, quotes,
/// line-continuations, and block constructs (for/while/if/case, paren/brace/bracket)
/// all stay glued as single commands. A single command in yields a 1-element vec
/// (identical behaviour, zero regression); a pasted block of N independent commands
/// yields N elements, each dispatched separately so abbreviations expand per-command.
pub fn split_into_commands(buf: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current = String::new();
    for line in buf.lines() {
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        let (complete, _) = is_complete_command(&current);
        if complete && !current.trim().is_empty() {
            commands.push(current.trim().to_string());
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        commands.push(current.trim().to_string());
    }
    commands
}

/// The last occurrence of `needle` that is NOT inside quotes. INT-169 follow-up: `rfind` scanned
/// the whole line, so `echo "a > b"` split at the QUOTED `>` -- the command became `echo "a`, the
/// target became `b"`, and a file named `b"` appeared in the working directory while the command
/// printed nothing. Same for `echo 'x >> y'`.
///
/// ⚠️ TRACKS BOTH QUOTE KINDS. The pipe scan in main.rs tracks only `"`, which is enough there but
/// would leave `'x >> y'` broken here.
///
/// ★ Found only after the spine router was disabled: `echo "a > b"` has no unquoted pipe and parses
/// cleanly, so the router CLAIMED it and it never reached this function. The flip was masking a
/// legacy defect rather than fixing it -- and every command the router DECLINES still comes here.
fn rfind_unquoted(line: &str, needle: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let n = needle.len();
    let mut in_single = false;
    let mut in_double = false;
    let mut found = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            _ => {}
        }
        if !in_single && !in_double && i + n <= bytes.len() && &bytes[i..i + n] == needle.as_bytes()
        {
            found = Some(i);
        }
        i += 1;
    }
    found
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
    if let Some(idx) = rfind_unquoted(line, " >> ") {
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
    if let Some(idx) = rfind_unquoted(line, " > ") {
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

// — Phase 3 extractions —

pub fn expand_subshells(line: &str) -> String {
    let trigger: &str = &('$'.to_string() + "(");
    if !line.contains(trigger) {
        return line.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    // INT-174: track quote state. Single quotes are LITERAL -- `$(...)` inside them
    // must NOT be executed (POSIX: single quotes suppress all expansion). Double
    // quotes still allow command substitution, so only in_single gates expansion.
    let mut in_single = false;
    let mut in_double = false;
    while i < chars.len() {
        // Update quote state before deciding whether to expand.
        if chars[i] == '\'' && !in_double {
            in_single = !in_single;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i] == '"' && !in_single {
            in_double = !in_double;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if !in_single && chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
            i += 2;
            let mut depth = 1usize;
            let mut inner = String::new();
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                if depth > 0 {
                    inner.push(chars[i]);
                }
                i += 1;
            }
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&inner)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            result.push_str(&output);
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

pub fn split_logical(line: &str) -> Vec<(String, Option<bool>)> {
    let mut result = vec![];
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
                current.push(ch);
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
                current.push(ch);
            }
            '&' if !in_quote && i + 1 < chars.len() && chars[i + 1] == '&' => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    result.push((seg, Some(true)));
                }
                current.clear();
                i += 2; // skip &&
                continue;
            }
            '|' if !in_quote && i + 1 < chars.len() && chars[i + 1] == '|' => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    result.push((seg, Some(false)));
                }
                current.clear();
                i += 2; // skip ||
                continue;
            }
            _ => current.push(ch),
        }
        i += 1;
    }
    let seg = current.trim().to_string();
    if !seg.is_empty() {
        result.push((seg, None));
    }
    if result.is_empty() {
        result.push((line.trim().to_string(), None));
    }
    result
}

pub fn parse_parallel_block(input: &str) -> Option<Vec<String>> {
    let trimmed = input.trim();
    if !trimmed.starts_with("parallel") {
        return None;
    }
    let rest = trimmed["parallel".len()..].trim();
    if !rest.starts_with('{') {
        return None;
    }
    let inner = rest.trim_start_matches('{');
    let inner = if let Some(pos) = inner.rfind('}') {
        &inner[..pos]
    } else {
        return None;
    };
    // Split by newlines first, then by semicolons for single-line usage
    let cmds: Vec<String> = if inner.contains('\n') {
        inner
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        // Single line: parallel {cmd1; cmd2; cmd3}
        inner
            .split(';')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };
    if cmds.is_empty() {
        None
    } else {
        Some(cmds)
    }
}

pub fn expand_globs(line: &str) -> String {
    // Only expand if line contains * or ? outside of quotes
    if !line.contains('*') && !line.contains('?') {
        return line.to_string();
    }
    // INT-245 #8: track quote state across the whole line so multi-word quoted
    // strings (e.g. python3 -c "code with * inside") don't get glob-expanded.
    // We segment the line into runs of (in_quotes, text) and only expand globs
    // in unquoted runs.
    let mut segments: Vec<(bool, String)> = vec![];
    let mut current = String::new();
    let mut in_double = false;
    let mut in_single = false;
    for ch in line.chars() {
        let was_in_quote = in_double || in_single;
        match ch {
            '"' if !in_single => {
                in_double = !in_double;
                current.push(ch);
            }
            '\'' if !in_double => {
                in_single = !in_single;
                current.push(ch);
            }
            _ => current.push(ch),
        }
        let now_in_quote = in_double || in_single;
        // Quote state just changed -- flush the prior segment with its prior quote state
        if now_in_quote != was_in_quote {
            // The character we just pushed is the boundary marker. The push includes it
            // in the segment STARTED by this transition (the new state), so we need to
            // pop it back if it should belong to the prior segment.
            // Simpler: at a transition, split AT THIS CHAR. The boundary char (quote)
            // belongs to the segment with quotes around it. Convention: include the opening
            // quote in the quoted segment, the closing quote in the quoted segment too.
            //
            // Since we already pushed the boundary char to `current`, and it should belong
            // to the new state's segment, we pop it, push current as old-state, then push
            // the boundary char into a fresh current with new state.
            let boundary = current.pop();
            if !current.is_empty() {
                segments.push((was_in_quote, std::mem::take(&mut current)));
            }
            if let Some(c) = boundary {
                current.push(c);
            }
        }
    }
    let final_in_quote = in_double || in_single;
    if !current.is_empty() {
        segments.push((final_in_quote, current));
    }
    let mut out = String::new();
    for (quoted, segment) in &segments {
        if *quoted {
            out.push_str(segment);
            continue;
        }
        // Apply glob expansion only to this unquoted segment.
        let expanded = expand_globs_in_segment(segment);
        out.push_str(&expanded);
    }
    out
}

/// INT-097: failglob support. Return the list of unquoted glob patterns in `line`
/// that match NOTHING. Mirrors expand_globs' quote-awareness and tilde handling so
/// the report matches what expansion actually attempted. Empty vec = all good.
pub fn find_unmatched_globs(line: &str) -> Vec<String> {
    let mut unmatched: Vec<String> = vec![];
    // Reuse the same quote-aware segmentation as expand_globs: only inspect
    // UNQUOTED segments (quoted * is literal and must not be reported).
    let mut in_double = false;
    let mut in_single = false;
    let mut segment = String::new();
    let mut segments: Vec<(bool, String)> = vec![];
    for ch in line.chars() {
        let was = in_double || in_single;
        match ch {
            '"' if !in_single => {
                in_double = !in_double;
                segment.push(ch);
            }
            '\'' if !in_double => {
                in_single = !in_single;
                segment.push(ch);
            }
            _ => segment.push(ch),
        }
        let now = in_double || in_single;
        if now != was {
            let b = segment.pop();
            if !segment.is_empty() {
                segments.push((was, std::mem::take(&mut segment)));
            }
            if let Some(c) = b {
                segment.push(c);
            }
        }
    }
    if !segment.is_empty() {
        segments.push((in_double || in_single, segment));
    }

    for (quoted, seg) in &segments {
        if *quoted {
            continue;
        }
        for part in seg.split_whitespace() {
            if !part.contains('*') && !part.contains('?') {
                continue;
            }
            let expanded = if part.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                part.replacen("~", &home, 1)
            } else {
                part.to_string()
            };
            let pattern_path = std::path::Path::new(&expanded);
            let parent = {
                let p = pattern_path.parent().unwrap_or(std::path::Path::new("."));
                if p.as_os_str().is_empty() {
                    std::path::Path::new(".")
                } else {
                    p
                }
            };
            let file_pattern = pattern_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(part);
            let mut matched = false;
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if glob_match(file_pattern, &name.to_string_lossy()) {
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                unmatched.push(part.to_string());
            }
        }
    }
    unmatched
}

pub fn expand_globs_in_segment(line: &str) -> String {
    if !line.contains('*') && !line.contains('?') {
        return line.to_string();
    }
    // Preserve original whitespace by splitting on whitespace runs but tracking them.
    // Simpler: split_whitespace + rejoin with a single space. The quote-aware caller
    // has already preserved leading/trailing spacing in adjacent quoted segments.
    let mut result_parts: Vec<String> = vec![];
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if part.contains('*') || part.contains('?') {
            // Expand tilde
            let expanded = if part.starts_with("~/") {
                let home = std::env::var("HOME").unwrap_or_default();
                part.replacen("~", &home, 1)
            } else {
                part.to_string()
            };
            // Use glob crate pattern matching via std::fs
            let pattern_path = std::path::Path::new(&expanded);
            let parent = {
                let p = pattern_path.parent().unwrap_or(std::path::Path::new("."));
                if p.as_os_str().is_empty() {
                    std::path::Path::new(".")
                } else {
                    p
                }
            };
            let file_pattern = pattern_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(part);
            let mut matches: Vec<String> = vec![];
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if glob_match(file_pattern, &name_str) {
                        let p = entry.path().to_string_lossy().to_string();
                        let p = p.strip_prefix("./").unwrap_or(&p).to_string();
                        matches.push(p);
                    }
                }
            }
            matches.sort();
            if matches.is_empty() {
                result_parts.push(part.to_string());
            } else {
                result_parts.extend(matches);
            }
        } else {
            result_parts.push(part.to_string());
        }
    }
    result_parts.join(" ")
}
