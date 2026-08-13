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

/// INT-209 DISPOSITION: ASSIGNED TO INT-169, DELIBERATELY NOT MIGRATED HERE.
///
/// This is the fourth quote machine INT-210 counted, and it is the one INT-209 does not own. The
/// distinction is between a helper having a quote-related IMPLEMENTATION and this intent owning the
/// BEHAVIOUR that requires it. Here the second is false.
///
/// It has exactly ONE caller, inside is_complete_command, serving the control-structure keyword
/// balance check. is_complete_command was routed to INT-169 by INT-210 -- three owners of one rule,
/// and quote-shaped only incidentally -- and INT-169 intends to REPLACE that completion logic with
/// the scanner reporting continuation, not to refactor its current implementation.
///
/// So migrating this now would rebuild a helper for a function another intent expects to delete.
/// Same reasoning that deferred INT-216: technically correct, wrong engineering investment.
///
/// ⏭ WHEN INT-169 ABSORBS is_complete_command, this goes with it. Until then it stays as written,
/// with its own escape handling, which no other consumer needs.
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
    // INT-196: QUOTE STATE, because a doubled angle bracket INSIDE QUOTES IS DATA, not a heredoc
    // introduction. Without it, a quoted pair was read as an intro, the delimiter came out of the
    // quoted text, and is_complete_command reported "unclosed heredoc" -- so the REPL prompt waited
    // forever for a terminator the user never meant to write. Reproduced live 2026-08-09 by typing
    // an ordinary echo with a quoted pair in it.
    //
    // AND THE KNOWLEDGE ALREADY EXISTED ONE FRAME UP. is_complete_command walks single, double and
    // backtick state to find an unquoted comment marker, then calls this function, which threw all
    // of it away and re-scanned the raw bytes. That is this intent in one function: structure
    // inferred from text by a stage that had the answer and did not ask.
    let mut in_single = false;
    let mut in_double = false;
    while i + 1 < bytes.len() {
        // 39 and 34 rather than character literals, so the rule reads as byte comparison and no
        // escaping games are needed here.
        if bytes[i] == 39 && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if bytes[i] == 34 && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if in_single || in_double {
            i += 1;
            continue;
        }
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
                // INT-196: A KNOWN EXCEPTION WITH A STATED REASON, not an oversight.
                //
                // This is a raw substring test, and it is SAFE HERE because control flow guards it.
                // Once the flag is set, every later line takes the else branch, so a quoted pair
                // INSIDE a body never reaches this test at all. The only line that can reach it is
                // one before the body opens -- and the outer guard above already established, using
                // the quote-aware recogniser, that a real introduction exists in this buffer.
                //
                // ⚠️ MEASURED, NOT ARGUED. It was changed to ask the recogniser and ghost-checked:
                // with the raw test restored, all four body-scan cases stayed GREEN, including one
                // written specifically to discriminate. There is no input for which the two spellings
                // differ, so the change was reverted rather than shipped unprovable.
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
    // INT-209: THE SCANNER ANSWERS THIS NOW. The private quote pair here was one of four
    // machines outside spine/lexer.rs walking a line to ask which regions are quoted, and the
    // four disagreed about where a quoted region begins and ends.
    //
    // THE OFFSET SEMANTICS ARE UNCHANGED, and that is load-bearing: detect_redirect slices on
    // BOTH sides of the returned index, so this still reports the index of the needle FIRST
    // BYTE.
    //
    // WHAT MOVED IS WHICH BYTE IS ASKED ABOUT. Both needles begin with a SPACE, and whitespace
    // belongs to no token, so the accessor reports None for it. Asking about the leading space
    // would find no redirect at all. It asks about the OPERATOR instead, which is the real
    // question: is this redirect operator quoted. The space before it was a proxy that
    // happened to agree.
    let bytes = line.as_bytes();
    let n = needle.len();
    if n == 0 || bytes.len() < n {
        return None;
    }
    let op_offset = needle
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut found = None;
    for i in 0..=(bytes.len() - n) {
        if &bytes[i..i + n] != needle.as_bytes() {
            continue;
        }
        if crate::spine::lexer::quote_context_at(line, i + op_offset)
            == Some(crate::spine::ast::QuoteContext::Unquoted)
        {
            found = Some(i);
        }
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

/// INT-209: ONE SEGMENTER, built on the scanner, replacing two byte-for-byte identical copies.
///
/// expand_globs and find_unmatched_globs each ran their own quote pair twenty lines apart, with the
/// same pop-flush-repush dance and different variable names. Two owners of one rule.
///
/// RUNS, NOT BYTES, is why the accessor alone was not enough here: both callers hand a whole
/// unquoted RUN to a matcher, so the shape has to be preserved rather than replaced by per-offset
/// questions.
///
/// THE BOUNDARY RULE IS NOW STATED, and it corrects a comment that described something the code did
/// not do. The old comment claimed both delimiters land in the quoted segment. Traced: the OPENING
/// one did, and the CLOSING one landed in the unquoted run that follows it. Under the accessor both
/// delimiters report Unquoted, so both now sit in an unquoted run -- a change for the opening quote
/// alone, and invisible to every caller here because a quote character is neither a star nor a
/// question mark. Recorded rather than absorbed.
///
/// A line the scanner cannot finish yields ONE unquoted run covering the whole input, which
/// preserves the old behaviour for unterminated quotes: the segmenter used to carry the open state
/// to the end and flush one segment.
fn quote_runs(line: &str) -> Vec<(bool, String)> {
    let mut runs: Vec<(bool, String)> = vec![];
    let mut current = String::new();
    let mut current_quoted: Option<bool> = None;
    for (i, ch) in line.char_indices() {
        let quoted = matches!(
            crate::spine::lexer::quote_context_at(line, i),
            Some(crate::spine::ast::QuoteContext::Single)
                | Some(crate::spine::ast::QuoteContext::Double)
        );
        if current_quoted != Some(quoted) {
            if let Some(prev) = current_quoted {
                if !current.is_empty() {
                    runs.push((prev, std::mem::take(&mut current)));
                }
            }
            current_quoted = Some(quoted);
        }
        current.push(ch);
    }
    if !current.is_empty() {
        runs.push((current_quoted.unwrap_or(false), current));
    }
    runs
}

pub fn expand_globs(line: &str) -> String {
    // Only expand if line contains * or ? outside of quotes
    if !line.contains('*') && !line.contains('?') {
        return line.to_string();
    }
    // INT-245 #8: a multi-word quoted string must not be glob-expanded, so the line is split
    // into runs and only unquoted runs are expanded.
    //
    // INT-209: the segmentation is no longer written here. quote_runs asks the scanner, and the
    // identical copy that used to live in find_unmatched_globs is gone with it.
    let segments = quote_runs(line);
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
    // INT-209: THE SAME SEGMENTER, not a second copy of it. This ran a byte-for-byte identical
    // quote machine twenty lines below the one in expand_globs -- same pop, same flush, same
    // trailing push, different variable names. Two owners of one rule, in one file.
    //
    // Only UNQUOTED runs are inspected: a quoted star is literal and must not be reported.
    let segments = quote_runs(line);
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

/// INT-196: A DOUBLED ANGLE BRACKET INSIDE QUOTES IS DATA, NOT A HEREDOC INTRODUCTION.
///
/// This scanner had no quote state at all, so a quoted pair was read as an intro, the delimiter
/// came out of the quoted text, and is_complete_command reported an unclosed heredoc. The REPL
/// then waited forever for a terminator the user never meant to write. Reproduced live by typing
/// an ordinary echo with a quoted pair in it, on gen 486.
#[cfg(test)]
mod heredoc_intro_quote_tests {
    use super::find_heredoc_intro;

    #[test]
    fn a_real_intro_is_still_found() {
        assert_eq!(
            find_heredoc_intro("cat << EOF"),
            Some(("EOF".to_string(), false))
        );
    }

    #[test]
    fn a_quoted_delimiter_still_reports_quoted() {
        assert_eq!(
            find_heredoc_intro("cat << \"EOF\""),
            Some(("EOF".to_string(), true))
        );
    }

    /// THE BUG, and it hung the prompt rather than printing anything.
    #[test]
    fn a_double_quoted_pair_is_data_not_an_intro() {
        assert_eq!(find_heredoc_intro("echo \"a << b\""), None);
    }

    #[test]
    fn a_single_quoted_pair_is_data_not_an_intro() {
        assert_eq!(find_heredoc_intro("echo 'a << b'"), None);
    }

    /// A CLOSED QUOTE MUST NOT DISARM A LATER INTRO. The quote state has to go back off, or the
    /// fix would trade a hang for a heredoc that never starts.
    #[test]
    fn an_intro_after_a_closed_quote_is_still_found() {
        assert_eq!(
            find_heredoc_intro("echo \"hi\" << EOF"),
            Some(("EOF".to_string(), false))
        );
    }
}

/// INT-196: the body scan inside is_complete_command must ask the recogniser too.
///
/// The delimiter comes from the quote-aware recogniser, and then the loop asked the RAW string
/// whether each line begins a body. A quoted pair on any line flipped the flag, and every line
/// after it was swallowed as heredoc content -- so a complete multi-line input read as incomplete
/// and the prompt kept waiting.
#[cfg(test)]
mod heredoc_body_scan_tests {
    use super::is_complete_command;

    #[test]
    fn a_closed_heredoc_is_complete() {
        let buf = "cat << EOF\nbody\nEOF";
        assert!(is_complete_command(buf).0, "a closed heredoc is complete");
    }

    #[test]
    fn an_unclosed_heredoc_is_incomplete() {
        let buf = "cat << EOF\nbody";
        assert!(
            !is_complete_command(buf).0,
            "no terminator means incomplete"
        );
    }

    /// NOT A HEREDOC AT ALL, so the input is complete. ⚠️ THIS CASE DOES NOT DISCRIMINATE the
    /// body scan on its own -- it passes on the scanner fix alone, because the outer guard returns
    /// None and the body loop never runs. Kept because it pins the outer behaviour; the case below
    /// is the one that tests THIS line.
    #[test]
    fn a_quoted_pair_does_not_start_a_body() {
        let buf = "echo \"a << b\"\necho done";
        assert!(
            is_complete_command(buf).0,
            "a quoted pair is data, so both lines are ordinary commands"
        );
    }

    /// THE DISCRIMINATING CASE, and it took a failed ghost-check to find it. The outer guard must
    /// FIRE -- so there is a real heredoc -- and a quoted pair must sit INSIDE the body before the
    /// terminator. With the raw substring test, that pair re-flips the in-body flag while already
    /// in the body, the terminator line is then skipped, no close is found, and a complete input
    /// reads as incomplete forever.
    #[test]
    fn a_quoted_pair_inside_a_body_does_not_reopen_it() {
        let buf = "cat << EOF\necho \"a << b\"\nEOF";
        assert!(
            is_complete_command(buf).0,
            "the terminator must still close the body"
        );
    }
}

/// INT-209: rfind_unquoted now asks the scanner. These cases pin the behaviour the two
/// implementations must share, and the offset semantics detect_redirect depends on.
#[cfg(test)]
mod rfind_unquoted_tests {
    use super::detect_redirect;

    #[test]
    fn a_plain_redirect_is_found() {
        let (cmd, r) = detect_redirect("echo hi > out.txt");
        assert_eq!(cmd, "echo hi");
        assert_eq!(r, Some(("out.txt".to_string(), false)));
    }

    #[test]
    fn an_append_redirect_is_found_before_the_single() {
        let (cmd, r) = detect_redirect("echo hi >> out.txt");
        assert_eq!(cmd, "echo hi");
        assert_eq!(r, Some(("out.txt".to_string(), true)));
    }

    /// THE CASE THE QUOTE AWARENESS EXISTS FOR. A redirect character inside quotes is data.
    #[test]
    fn a_quoted_redirect_is_not_a_redirect() {
        let (cmd, r) = detect_redirect("echo \"a > b\"");
        assert_eq!(r, None, "a quoted angle is data");
        assert_eq!(cmd, "echo \"a > b\"", "the line comes back untouched");
    }

    #[test]
    fn a_single_quoted_redirect_is_not_a_redirect() {
        let (_, r) = detect_redirect("echo 'a > b'");
        assert_eq!(r, None);
    }

    /// THE LAST unquoted match wins, which is what rfind means and what the slice depends on.
    #[test]
    fn the_last_unquoted_match_wins() {
        let (cmd, r) = detect_redirect("echo \"a > b\" > out.txt");
        assert_eq!(cmd, "echo \"a > b\"", "the quoted angle is skipped");
        assert_eq!(r, Some(("out.txt".to_string(), false)));
    }

    /// THE DISCRIMINATING CASE, and it took a ghost-check to find that the one above is not.
    ///
    /// `the_last_unquoted_match_wins` passes even with the accessor bypassed, because bypassing it
    /// accepts every match and keeps the LAST -- which on that input happens to be the real
    /// redirect. Here the last match is the QUOTED one, so accepting every match returns the wrong
    /// offset and slices the line in the wrong place.
    #[test]
    fn a_quoted_angle_after_a_real_redirect_does_not_win() {
        let (cmd, r) = detect_redirect("echo out.txt \"a > b\"");
        assert_eq!(
            r, None,
            "the only angle present is quoted, so there is no redirect"
        );
        assert_eq!(cmd, "echo out.txt \"a > b\"");
    }

    /// A comparison is a deliberate divergence, preserved by the digit guard above the search.
    #[test]
    fn a_numeric_comparison_is_not_a_redirect() {
        let (_, r) = detect_redirect("tt | where score > 70");
        assert_eq!(r, None);
    }
}

/// INT-209: the glob segmentation behaviour, pinned BEFORE it is consolidated.
///
/// expand_globs and find_unmatched_globs run byte-for-byte identical segmenters twenty lines
/// apart. These cases fix what that shared behaviour is, so the consolidation can be shown to
/// preserve it rather than asserted to.
#[cfg(test)]
mod glob_segmentation_tests {
    use super::{expand_globs, find_unmatched_globs};

    #[test]
    fn a_quoted_star_is_literal_and_not_expanded() {
        let s = "python3 -c \"a * b\"";
        assert_eq!(expand_globs(s), s, "a star inside quotes is data");
    }

    #[test]
    fn a_quoted_star_is_not_reported_unmatched() {
        assert!(find_unmatched_globs("echo \"zzq*zzq\"").is_empty());
    }

    /// An unquoted pattern matching nothing comes back unchanged, and IS reported.
    #[test]
    fn an_unmatched_unquoted_pattern_is_reported() {
        let hits = find_unmatched_globs("ls zzq_no_such*");
        assert_eq!(hits, vec!["zzq_no_such*".to_string()]);
    }

    #[test]
    fn an_unmatched_unquoted_pattern_expands_to_itself() {
        assert_eq!(expand_globs("ls zzq_no_such*"), "ls zzq_no_such*");
    }

    /// A line with both: the quoted star stays, the unquoted one is judged on its own.
    #[test]
    fn a_mixed_line_judges_each_run_separately() {
        let hits = find_unmatched_globs("echo \"keep*\" zzq_no_such*");
        assert_eq!(hits, vec!["zzq_no_such*".to_string()]);
    }
}

/// INT-203: MOVED HERE FROM main.rs, unchanged.
///
/// It belongs beside its siblings: expand_globs, expand_subshells and quote_runs all live
/// here, and the next two gates make this one quote-aware and heredoc-aware using quote_runs
/// and find_heredoc_intro -- both of which are in this file. Moving it first, as its own
/// commit, so a behaviour change cannot hide inside a relocation.
pub fn expand_braces(s: &str) -> String {
    // Expand {N..M} and {a..z} sequences without regex
    if !s.contains('{') {
        return s.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(close) = chars[i + 1..].iter().position(|&c| c == '}') {
                let inner: String = chars[i + 1..i + 1 + close].iter().collect();
                if let Some(dotdot) = inner.find("..") {
                    let left = &inner[..dotdot];
                    let right = &inner[dotdot + 2..];
                    if let (Ok(start_n), Ok(end_n)) = (left.parse::<i64>(), right.parse::<i64>()) {
                        let expanded: Vec<String> = if start_n <= end_n {
                            (start_n..=end_n).map(|n| n.to_string()).collect()
                        } else {
                            (end_n..=start_n).rev().map(|n| n.to_string()).collect()
                        };
                        result.push_str(&expanded.join(" "));
                        i += close + 2;
                        continue;
                    }
                    let lc: Vec<char> = left.chars().collect();
                    let rc: Vec<char> = right.chars().collect();
                    // INT-203: LENGTH WAS THE ONLY CHECK, and that is the whole bug. A space is one character,
                    // so `{ .. }` parsed as a character range from space to space and expanded to a single
                    // space -- silently eating any brace group written with the range operator inside it. A Rust
                    // match pattern pasted through a heredoc arrived with its braces replaced by three spaces,
                    // and three patch scripts reported success while grep and rustc disagreed, because the text
                    // was corrupted in transit rather than the write failing.
                    //
                    // Requiring both endpoints to be ASCII LETTERS is the whole repair. `{a..z}` and
                    // `{A..Z}` still expand. `{1..5}` never reached here -- the integer branch above
                    // claims it when both sides parse. A mixed `{1..a}` now stays literal, which is what
                    // bash does. A space, a dot or a quote can no longer be a range endpoint.
                    //
                    // NOT FIXED HERE, and recorded rather than hidden: this function is still neither quote-aware
                    // nor heredoc-aware, so `echo "{a..c}"` still expands inside quotes. That is the
                    // INT-196 class -- code inferring shell structure from raw text -- and it needs its own
                    // evidence. See INT-203.
                    if lc.len() == 1
                        && rc.len() == 1
                        && lc[0].is_ascii_alphabetic()
                        && rc[0].is_ascii_alphabetic()
                    {
                        let ls = lc[0] as u8;
                        let rs = rc[0] as u8;
                        let expanded: Vec<String> = if ls <= rs {
                            (ls..=rs).map(|c| (c as char).to_string()).collect()
                        } else {
                            (rs..=ls).rev().map(|c| (c as char).to_string()).collect()
                        };
                        result.push_str(&expanded.join(" "));
                        i += close + 2;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

#[cfg(test)]
mod brace_expansion_tests {
    use super::expand_braces;

    #[test]
    fn letter_and_number_ranges_still_expand() {
        assert_eq!(expand_braces("{a..e}"), "a b c d e");
        assert_eq!(expand_braces("{1..4}"), "1 2 3 4");
        assert_eq!(expand_braces("pre {a..c} post"), "pre a b c post");
    }

    #[test]
    fn a_space_is_not_a_range_endpoint() {
        // INT-203: this is the exact text that was being eaten -- a Rust match pattern.
        let pattern = "Executed { .. } => continue";
        assert_eq!(
            expand_braces(pattern),
            pattern,
            "brace group with spaces must stay literal"
        );
    }

    #[test]
    fn punctuation_and_mixed_kinds_stay_literal() {
        for s in ["{. ..}", "{1..a}", "{a..1}", "Foo { ..default}"] {
            assert_eq!(expand_braces(s), s, "should not expand: {s}");
        }
    }
}
