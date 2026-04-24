//! Platform-specific argv builders. On Unix the PTY spawns `$SHELL -i`
//! directly; the recipe itself is primed via `session::shell::prime_line`.
//! Windows still uses a wrapper batch script.

pub const WINDOWS_WRAPPER: &str = r#"
@echo off
set JUSTFILE=%~1
shift
just --justfile "%JUSTFILE%" %*
echo 1337;LazyjustDone=%ERRORLEVEL%
%ComSpec%
"#;

/// Returns the argv for the PTY spawn on Unix: `[$SHELL, "-i"]` (fallback
/// `/bin/sh`). The recipe itself is not in argv — it is delivered via
/// `crate::session::shell::prime_line` written to the shell's stdin after
/// rc-file init. Parameters are kept for signature stability with the
/// (future) Windows builder.
pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
