//! Unix argv builder. The PTY spawns `$SHELL -i` directly; the recipe
//! itself is primed into the shell's stdin via `session::shell::prime_line`.
//! Windows support is not yet implemented (see
//! `session::manager::SessionManager::spawn_recipe` for the runtime stub).

/// Returns the argv for the PTY spawn on Unix: `[$SHELL, "-i"]` (fallback
/// `/bin/sh`). The recipe itself is not in argv — it is delivered via
/// `crate::session::shell::prime_line` written to the shell's stdin after
/// rc-file init. Parameters are retained (and underscored) for call-site
/// stability; a future Windows builder is expected to consume them.
pub fn build_unix_command(
    _justfile: &std::path::Path,
    _recipe: &str,
    _args: &[String],
) -> (Vec<String>, Vec<(String, String)>) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    (vec![shell, "-i".to_string()], Vec::new())
}
