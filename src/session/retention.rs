use std::path::Path;
use std::time::{Duration, SystemTime};

pub fn prune_sessions(root: &Path, max_age: Duration) -> std::io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let now = SystemTime::now();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta.modified().unwrap_or(now);
        if let Ok(age) = now.duration_since(modified) {
            if age > max_age {
                std::fs::remove_dir_all(&path)?;
            }
        }
    }
    Ok(())
}
