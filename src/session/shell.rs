//! Shell-string construction primitives used by the session layer.
//!
//! `shell_quote` returns POSIX single-quote-escaped input; `prime_line`
//! composes the full command line (plus OSC exit-code marker) fed into
//! an interactive shell's stdin. Both are pure and platform-agnostic.

/// POSIX single-quote escape. Returns a valid single-quoted shell word
/// whose expansion equals `s` byte-for-byte under any POSIX `sh`,
/// including `$`, backticks, `\`, newlines, and UTF-8.
///
/// Security: this is the only quoting layer between attacker-controllable
/// input (recipe name, justfile path, positional args) and a shell `eval`
/// context. Do not replace with `$'…'` (bash-only) or backslash-escaping
/// (content-dependent) — POSIX single-quote is the one form where no
/// interior character has meaning.
///
/// Caveat: a NUL byte in `s` is preserved in the returned `String` but
/// most shells truncate at NUL when the word crosses `execve` / PTY stdin.
pub fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Builds the line fed into an interactive shell's stdin to run a recipe.
/// Every user-controlled value (justfile path, recipe name, each arg) is
/// `shell_quote`'d. The trailing `printf` emits the `LazyjustDone=%d` OSC
/// marker parsed by `session::osc::scan_done_marker`.
pub fn prime_line(justfile: &std::path::Path, recipe: &str, args: &[String]) -> String {
    let mut line = format!(
        "just --justfile {} {}",
        shell_quote(&justfile.display().to_string()),
        shell_quote(recipe),
    );
    for a in args {
        line.push(' ');
        line.push_str(&shell_quote(a));
    }
    line.push_str(" ; printf '\\033]1337;LazyjustDone=%d\\007' $?");
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_plain() {
        assert_eq!(shell_quote("foo"), "'foo'");
    }

    #[test]
    fn shell_quote_with_space() {
        assert_eq!(shell_quote("foo bar"), "'foo bar'");
    }

    #[test]
    fn shell_quote_with_single_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_quote_with_dollar_and_paren() {
        assert_eq!(shell_quote("$(evil)"), "'$(evil)'");
    }

    #[test]
    fn shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn shell_quote_newline_preserved_literal() {
        assert_eq!(shell_quote("a\nb"), "'a\nb'");
    }

    #[test]
    fn prime_line_no_args() {
        let line = prime_line(std::path::Path::new("/p/Justfile"), "build", &[]);
        assert_eq!(
            line,
            "just --justfile '/p/Justfile' 'build' ; printf '\\033]1337;LazyjustDone=%d\\007' $?"
        );
    }

    #[test]
    fn prime_line_with_args_and_spaces() {
        let args = vec!["a b".to_string(), "x".to_string()];
        let line = prime_line(std::path::Path::new("/p/Justfile"), "build", &args);
        assert_eq!(
            line,
            "just --justfile '/p/Justfile' 'build' 'a b' 'x' ; printf '\\033]1337;LazyjustDone=%d\\007' $?"
        );
    }

    #[test]
    fn prime_line_escapes_dangerous_recipe_name() {
        let line = prime_line(std::path::Path::new("/p/Justfile"), "it's; rm -rf /", &[]);
        assert!(line.contains("'it'\\''s; rm -rf /'"));
        assert!(line.ends_with("$?"));
    }
}
