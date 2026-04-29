//! Render a path made relative to a root for use in `All`-mode list
//! section headers. Falls back to the absolute display when the path
//! does not live under the root.

use std::path::Path;

pub fn relativize_to_root(path: &Path, root: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(root) {
        let s = rel.display().to_string();
        if s.is_empty() {
            return "./".to_string();
        }
        if s == "." {
            return "./".to_string();
        }
        return s;
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn under_root_strips_prefix() {
        let p = PathBuf::from("/root/api/justfile");
        let r = PathBuf::from("/root");
        assert_eq!(relativize_to_root(&p, &r), "api/justfile");
    }

    #[test]
    fn equal_to_root_renders_dot_slash() {
        let p = PathBuf::from("/root");
        let r = PathBuf::from("/root");
        assert_eq!(relativize_to_root(&p, &r), "./");
    }

    #[test]
    fn outside_root_returns_absolute() {
        let p = PathBuf::from("/elsewhere/justfile");
        let r = PathBuf::from("/root");
        assert_eq!(relativize_to_root(&p, &r), "/elsewhere/justfile");
    }

    #[test]
    fn root_is_relative_dot() {
        let p = PathBuf::from("./api/justfile");
        let r = PathBuf::from(".");
        assert_eq!(relativize_to_root(&p, &r), "api/justfile");
    }

    #[test]
    fn justfile_at_root_renders_filename_only() {
        let p = PathBuf::from("/root/justfile");
        let r = PathBuf::from("/root");
        assert_eq!(relativize_to_root(&p, &r), "justfile");
    }
}
