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
