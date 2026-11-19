//! A refusal says, on its first line, whether anything changed.
//!
//! Green when the stop was safe, red when it was partway. Color is raw ANSI and
//! only when the caller says the stream is a tty and `NO_COLOR` is unset. No
//! `colored` crate: zero-core stays free of one.

/// What a stop has to say. `safe` is nothing changed. A partway stop names
/// `changed`. `finish` is the one command that continues the work.
pub struct Stop<'a> {
    pub safe: bool,
    pub reason: &'a str,
    pub changed: &'a [&'a str],
    pub finish: &'a str,
}

/// The text of a stop. `color` wraps the first line in raw ANSI; the rest stays plain.
pub fn format_stop(stop: &Stop<'_>, color: bool) -> String {
    let head = if stop.safe {
        "Stopped safely — nothing changed"
    } else {
        "Stopped partway — something changed"
    };
    let head = if color {
        let code = if stop.safe { "32" } else { "31" };
        format!("\u{1b}[{code}m{head}\u{1b}[0m")
    } else {
        head.to_string()
    };
    let mut out = String::new();
    out.push_str(&head);
    out.push('\n');
    if !stop.reason.is_empty() {
        out.push_str(stop.reason);
        out.push('\n');
    }
    if !stop.safe {
        for item in stop.changed {
            if !item.is_empty() {
                out.push_str(item);
                out.push('\n');
            }
        }
    }
    if !stop.finish.is_empty() {
        out.push_str(stop.finish);
        out.push('\n');
    }
    out
}

/// Whether a stop may be colored: the stream is a tty, and `NO_COLOR` is unset.
pub fn color_enabled(is_tty: bool) -> bool {
    is_tty && std::env::var_os("NO_COLOR").is_none()
}

/// Print a stop on stderr. Color follows the real stderr and `NO_COLOR`.
pub fn print_stop(stop: &Stop<'_>) {
    use std::io::IsTerminal;
    let color = color_enabled(std::io::stderr().is_terminal());
    eprint!("{}", format_stop(stop, color));
}

#[cfg(test)]
mod tests {
    use super::{color_enabled, format_stop, Stop};

    fn first_line(text: &str) -> &str {
        text.lines().next().unwrap_or("")
    }

    #[test]
    fn a_safe_stop_leads_with_the_safe_line() {
        let text = format_stop(
            &Stop {
                safe: true,
                reason: "the intent has no file",
                changed: &[],
                finish: "cicomplete 328",
            },
            false,
        );
        assert_eq!(first_line(&text), "Stopped safely — nothing changed");
        assert!(text.contains("the intent has no file"), "{text}");
        assert!(text.contains("cicomplete 328"), "{text}");
        assert!(
            !text.contains('\u{1b}'),
            "plain stop carried ANSI: {text:?}"
        );
    }

    #[test]
    fn a_partway_stop_names_what_changed() {
        let text = format_stop(
            &Stop {
                safe: false,
                reason: "bump failed for zero-core",
                changed: &["zero-core 1.3.0"],
                finish: "bump-versions zero-core minor",
            },
            false,
        );
        assert_eq!(first_line(&text), "Stopped partway — something changed");
        assert!(text.contains("zero-core 1.3.0"), "{text}");
        assert!(text.contains("bump failed for zero-core"), "{text}");
        assert!(text.contains("bump-versions zero-core minor"), "{text}");
        assert!(!text.contains('\u{1b}'), "{text:?}");
    }

    #[test]
    fn color_is_raw_ansi_only_on_a_tty_without_no_color() {
        let safe = format_stop(
            &Stop {
                safe: true,
                reason: "r",
                changed: &[],
                finish: "cicomplete 328",
            },
            true,
        );
        assert!(
            safe.contains("\u{1b}[32m"),
            "safe tty stop was not green: {safe:?}"
        );
        assert!(safe.contains("Stopped safely — nothing changed"), "{safe:?}");
        let part = format_stop(
            &Stop {
                safe: false,
                reason: "r",
                changed: &["zero-core 1.3.0"],
                finish: "bump-versions zero-core minor",
            },
            true,
        );
        assert!(
            part.contains("\u{1b}[31m"),
            "partway tty stop was not red: {part:?}"
        );
        let prev = std::env::var_os("NO_COLOR");
        std::env::set_var("NO_COLOR", "1");
        let enabled = color_enabled(true);
        let not_tty = color_enabled(false);
        match prev {
            Some(v) => std::env::set_var("NO_COLOR", v),
            None => std::env::remove_var("NO_COLOR"),
        }
        assert!(!enabled, "NO_COLOR still colored the stop");
        assert!(!not_tty, "a non-tty still colored the stop");
    }
}
