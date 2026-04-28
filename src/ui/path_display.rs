// Helper for rendering filesystem paths inside the TUI.
//
// See docs/superpowers/specs/2026-04-28-shorten-path-display-design.md.

use std::path::Path;

/// Render `path` for display in the TUI, fitting inside `max_width` columns
/// when possible.
///
/// Behaviour:
/// 1. The leading `$HOME` segment is replaced with `~`.
/// 2. If the resulting display width is `<= max_width`, the string is
///    returned unchanged.
/// 3. Otherwise a segment-aware middle truncation is applied, preserving
///    the root anchor and the filename, with `…` between them. See the
///    design doc for the full algorithm.
pub fn shorten(path: &Path, max_width: usize) -> String {
    let s = render_with_home_tilde(path);
    if display_width(&s) <= max_width {
        return s;
    }
    middle_truncate(&s, max_width)
}

fn render_with_home_tilde(path: &Path) -> String {
    let raw = path.display().to_string();
    let home = match std::env::var_os("HOME") {
        Some(h) if !h.is_empty() => h,
        _ => return raw,
    };
    let home = home.to_string_lossy();
    if raw == *home {
        return "~".to_string();
    }
    let prefix = format!("{home}/");
    if let Some(rest) = raw.strip_prefix(&prefix) {
        format!("~/{rest}")
    } else {
        raw
    }
}

fn display_width(s: &str) -> usize {
    // ASCII-dominant in practice; chars() is good enough today and avoids
    // pulling unicode-width as a direct dependency. See spec for the
    // rationale and upgrade path.
    s.chars().count()
}

fn middle_truncate(s: &str, max_width: usize) -> String {
    // Implementation arrives in a later step. Returning `s` here would make
    // the next test pass for the wrong reason, so return the minimal
    // safe placeholder: the input itself is never wider than the input.
    let _ = max_width;
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_unchanged_when_within_width() {
        let p = PathBuf::from("/tmp/justfile");
        assert_eq!(shorten(&p, 80), "/tmp/justfile");
    }
}
