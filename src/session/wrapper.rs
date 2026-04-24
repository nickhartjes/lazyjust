pub const UNIX_WRAPPER: &str = r#"
justfile="$1"; shift
just --justfile "$justfile" "$@"
CODE=$?
printf '\033]1337;LazyjustDone=%d\007' "$CODE"
exec "${SHELL:-/bin/sh}" -i
"#;

pub const WINDOWS_WRAPPER: &str = r#"
@echo off
set JUSTFILE=%~1
shift
just --justfile "%JUSTFILE%" %*
echo 1337;LazyjustDone=%ERRORLEVEL%
%ComSpec%
"#;

pub fn build_unix_command(
    justfile: &std::path::Path,
    recipe: &str,
    args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let mut argv = vec![
        "sh".to_string(),
        "-c".to_string(),
        UNIX_WRAPPER.to_string(),
        "lazyjust-wrapper".to_string(),
        justfile.display().to_string(),
        recipe.to_string(),
    ];
    argv.extend(args.iter().cloned());
    (argv, Vec::new())
}

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

pub fn prime_line(
    justfile: &std::path::Path,
    recipe: &str,
    args: &[String],
) -> String {
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
