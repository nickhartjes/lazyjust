pub mod parse;
pub mod walk;

use crate::app::types::Justfile;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Default, Clone, Copy)]
pub struct DiscoverOptions<'a> {
    pub path: Option<&'a Path>,
    pub justfile: Option<&'a Path>,
}

#[derive(Debug)]
pub struct DiscoveryResult {
    pub justfiles: Vec<Justfile>,
    pub errors: Vec<(PathBuf, String)>,
    /// Index into `justfiles` of the entry to pre-select on launch.
    /// 0 when no pin is set or the pin is not found in the result.
    pub active_index: usize,
}

pub fn discover(opts: DiscoverOptions) -> Result<DiscoveryResult> {
    ensure_just_on_path()?;

    let roots = walk_roots(&opts);

    let mut paths: Vec<PathBuf> = Vec::new();
    for root in &roots {
        let walked = walk::walk_justfiles(root)?;
        for p in walked {
            if !paths.iter().any(|existing| existing == &p) {
                paths.push(p);
            }
        }
    }

    let pinned = opts.justfile.map(|p| walk::absolutize(p.to_path_buf()));
    if let Some(pin) = &pinned {
        if !paths.iter().any(|p| p == pin) {
            paths.push(pin.clone());
        }
    }

    paths.sort();

    let mut justfiles = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match dump_and_parse(&path) {
            Ok(jf) => justfiles.push(jf),
            Err(e) => errors.push((path, e.to_string())),
        }
    }

    let active_index = match &pinned {
        Some(pin) => justfiles.iter().position(|j| &j.path == pin).unwrap_or(0),
        None => 0,
    };

    Ok(DiscoveryResult {
        justfiles,
        errors,
        active_index,
    })
}

fn walk_roots(opts: &DiscoverOptions) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(p) = opts.path {
        roots.push(p.to_path_buf());
    }

    if let Some(jf) = opts.justfile {
        let abs = walk::absolutize(jf.to_path_buf());
        let parent = abs
            .parent()
            .map(Path::to_path_buf)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        roots.push(parent);
    }

    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }

    let mut seen: Vec<PathBuf> = Vec::new();
    let mut deduped: Vec<PathBuf> = Vec::new();
    for r in roots {
        let key = walk::absolutize(r.clone());
        if !seen.iter().any(|s| s == &key) {
            seen.push(key);
            deduped.push(r);
        }
    }
    deduped
}

fn ensure_just_on_path() -> Result<()> {
    let out = Command::new("just").arg("--version").output();
    match out {
        Ok(o) if o.status.success() => Ok(()),
        _ => Err(Error::JustNotFound),
    }
}

fn dump_and_parse(path: &Path) -> Result<Justfile> {
    let output = Command::new("just")
        .arg("--justfile")
        .arg(path)
        .arg("--dump")
        .arg("--dump-format=json")
        .output()
        .map_err(|e| Error::JustInvocation {
            path: path.to_path_buf(),
            source: e,
        })?;

    if !output.status.success() {
        return Err(Error::JustDump {
            path: path.to_path_buf(),
            code: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let recipes = parse::parse_dump_with_path(&json, &path.to_path_buf())?;

    let mut groups: Vec<String> = Vec::new();
    for r in &recipes {
        if let Some(g) = &r.group {
            if !groups.iter().any(|existing| existing == g) {
                groups.push(g.clone());
            }
        }
    }

    Ok(Justfile {
        path: path.to_path_buf(),
        recipes,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn walk_roots_default_is_cwd() {
        let roots = walk_roots(&DiscoverOptions::default());
        assert_eq!(roots, vec![PathBuf::from(".")]);
    }

    #[test]
    fn walk_roots_path_only() {
        let p = PathBuf::from("some/path");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: None,
        });
        assert_eq!(roots, vec![PathBuf::from("some/path")]);
    }

    #[test]
    fn walk_roots_justfile_only_uses_parent() {
        let jf = PathBuf::from("some/path/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: None,
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 1);
        assert!(
            roots[0].ends_with("some/path"),
            "expected parent of justfile, got {:?}",
            roots[0]
        );
    }

    #[test]
    fn walk_roots_path_plus_justfile_unions() {
        let p = PathBuf::from("a");
        let jf = PathBuf::from("b/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0], PathBuf::from("a"));
        assert!(roots[1].ends_with("b"));
    }

    #[test]
    fn walk_roots_path_equal_to_justfile_parent_dedups() {
        let p = PathBuf::from("a");
        let jf = PathBuf::from("a/justfile");
        let roots = walk_roots(&DiscoverOptions {
            path: Some(&p),
            justfile: Some(&jf),
        });
        assert_eq!(roots.len(), 1);
    }
}
