pub mod parse;
pub mod walk;

use crate::app::types::Justfile;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct DiscoveryResult {
    pub justfiles: Vec<Justfile>,
    pub errors: Vec<(PathBuf, String)>,
}

pub fn discover(root: &Path) -> Result<DiscoveryResult> {
    discover_inner(root, None)
}

/// Like `discover`, but pin the discovery to a single explicit justfile,
/// bypassing the walk. The path is lexically absolutized so the spawned
/// `just --justfile` resolves it from any PTY CWD.
pub fn discover_explicit(justfile: &Path) -> Result<DiscoveryResult> {
    discover_inner(Path::new(""), Some(justfile))
}

fn discover_inner(root: &Path, explicit: Option<&Path>) -> Result<DiscoveryResult> {
    ensure_just_on_path()?;
    let paths = match explicit {
        Some(p) => vec![walk::absolutize(p.to_path_buf())],
        None => walk::walk_justfiles(root)?,
    };

    let mut justfiles = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match dump_and_parse(&path) {
            Ok(jf) => justfiles.push(jf),
            Err(e) => errors.push((path, e.to_string())),
        }
    }

    justfiles.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(DiscoveryResult { justfiles, errors })
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
