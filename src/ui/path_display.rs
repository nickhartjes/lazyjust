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
    let segments: Vec<&str> = s.split('/').collect();

    if segments.len() < 2 {
        return s.to_string();
    }

    let root = segments[0];
    let filename = segments[segments.len() - 1];
    let middle: &[&str] = &segments[1..segments.len() - 1];

    // No middle segments to drop — cannot shorten further.
    if middle.is_empty() {
        return s.to_string();
    }

    let minimum = format!("{root}/…/{filename}");

    let mut kept_from_right: usize = 0;
    let mut current = minimum.clone();
    while kept_from_right < middle.len() {
        let take = kept_from_right + 1;
        let tail_segments = &middle[middle.len() - take..];
        let candidate = format!("{root}/…/{}/{filename}", tail_segments.join("/"));
        if display_width(&candidate) > max_width {
            break;
        }
        current = candidate;
        kept_from_right = take;
    }

    if kept_from_right == 0 {
        return minimum;
    }

    current
}

#[cfg(test)]
mod tests {
    // These tests mutate the process-wide HOME env var. The whole suite runs
    // under `cargo test -- --test-threads=1`; if that ever changes, gate this
    // module with a Mutex<()> like tests/config_loader.rs.
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_unchanged_when_within_width() {
        let p = PathBuf::from("/tmp/justfile");
        assert_eq!(shorten(&p, 80), "/tmp/justfile");
    }

    #[test]
    fn replaces_home_with_tilde() {
        std::env::set_var("HOME", "/Users/nick");
        let p = PathBuf::from("/Users/nick/projects/foo/justfile");
        assert_eq!(shorten(&p, 80), "~/projects/foo/justfile");
    }

    #[test]
    fn home_only_path_renders_as_tilde() {
        std::env::set_var("HOME", "/Users/nick");
        let p = PathBuf::from("/Users/nick");
        assert_eq!(shorten(&p, 80), "~");
    }

    #[test]
    fn unrelated_path_is_unaffected_by_home() {
        std::env::set_var("HOME", "/Users/nick");
        let p = PathBuf::from("/var/log/justfile");
        assert_eq!(shorten(&p, 80), "/var/log/justfile");
    }

    #[test]
    fn home_prefix_is_not_a_path_substring_match() {
        std::env::set_var("HOME", "/Users/nick");
        let p = PathBuf::from("/Users/nicholas/justfile");
        assert_eq!(shorten(&p, 80), "/Users/nicholas/justfile");
    }

    #[test]
    fn middle_truncates_long_absolute_path() {
        std::env::set_var("HOME", "/Users/nick");
        let p =
            PathBuf::from("/Users/nick/projects/entrnce/trader/services/api/justfile");
        let out = shorten(&p, 28);
        assert!(out.starts_with("~/"), "expected leading ~/, got {out:?}");
        assert!(out.contains('…'), "expected ellipsis, got {out:?}");
        assert!(out.ends_with("/justfile"), "expected /justfile tail, got {out:?}");
        assert!(out.chars().count() <= 28, "expected ≤28 cols, got {} ({out:?})", out.chars().count());
    }

    #[test]
    fn middle_truncates_non_home_path() {
        std::env::remove_var("HOME");
        let p = PathBuf::from("/var/very/deeply/nested/repo/sub/dir/justfile");
        let out = shorten(&p, 24);
        assert!(out.starts_with("/…/"), "expected /…/ root anchor, got {out:?}");
        assert!(out.ends_with("/justfile"), "expected /justfile tail, got {out:?}");
        assert!(out.chars().count() <= 24, "got {} ({out:?})", out.chars().count());
    }

    #[test]
    fn very_tight_budget_returns_root_ellipsis_filename_even_if_over_budget() {
        std::env::remove_var("HOME");
        let p = PathBuf::from("/var/x/y/z/justfile");
        let out = shorten(&p, 5);
        assert_eq!(out, "/…/justfile");
    }

    #[test]
    fn root_only_path_unchanged() {
        std::env::remove_var("HOME");
        let p = PathBuf::from("/justfile");
        assert_eq!(shorten(&p, 5), "/justfile");
    }

    #[test]
    fn relative_path_with_no_root_segment() {
        std::env::remove_var("HOME");
        let p = PathBuf::from("a/b/c/d/e/justfile");
        let out = shorten(&p, 14);
        assert!(out.starts_with("a/"), "got {out:?}");
        assert!(out.contains('…'));
        assert!(out.ends_with("/justfile"));
        assert!(out.chars().count() <= 14, "got {} ({out:?})", out.chars().count());
    }
}
