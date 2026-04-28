use crate::error::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const HARDCODED_IGNORES: &[&str] = &["node_modules", "target", "dist", ".git"];

pub fn walk_justfiles(root: &Path) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .require_git(false);

    builder.filter_entry(|e| {
        let n = e.file_name().to_string_lossy();
        !HARDCODED_IGNORES.iter().any(|p| n == *p)
    });

    let mut out = Vec::new();
    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue, // log at debug later
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if is_justfile_name(&name) {
            out.push(absolutize(entry.into_path()));
        }
    }
    out.sort();
    Ok(out)
}

/// Lexically absolutize a path so downstream consumers (e.g. `just --justfile`
/// invoked from a PTY shell whose CWD differs from this process) resolve it
/// against a fixed root, not whatever the caller's CWD happens to be. Falls
/// back to the input path if absolutization fails.
pub(crate) fn absolutize(path: PathBuf) -> PathBuf {
    std::path::absolute(&path).unwrap_or(path)
}

fn is_justfile_name(name: &str) -> bool {
    matches!(name, "justfile" | "Justfile" | ".justfile") || name.ends_with(".just")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_matcher() {
        assert!(is_justfile_name("justfile"));
        assert!(is_justfile_name("Justfile"));
        assert!(is_justfile_name(".justfile"));
        assert!(is_justfile_name("common.just"));
        assert!(!is_justfile_name("foo.txt"));
    }
}
