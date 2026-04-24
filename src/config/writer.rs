//! Comment-preserving writes to the user config file. Used by the
//! theme picker to persist `[ui].theme` without blowing away the rest
//! of the file.

use std::path::Path;
use toml_edit::{value, DocumentMut};

#[derive(Debug, thiserror::Error)]
pub enum WriterError {
    #[error("config file IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config file parse error: {0}")]
    Parse(#[from] toml_edit::TomlError),
}

pub fn set_theme(path: &Path, name: &str) -> Result<(), WriterError> {
    let mut doc: DocumentMut = match std::fs::read_to_string(path) {
        Ok(s) => s.parse()?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => DocumentMut::new(),
        Err(e) => return Err(WriterError::Io(e)),
    };

    if !doc.contains_key("ui") {
        let mut t = toml_edit::Table::new();
        t.set_implicit(false);
        doc["ui"] = toml_edit::Item::Table(t);
    }

    doc["ui"]["theme"] = value(name);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_fresh_file_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        set_theme(&p, "gruvbox-dark").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("theme = \"gruvbox-dark\""));
        assert!(s.contains("[ui]"));
    }

    #[test]
    fn preserves_comments_and_other_keys() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        let original = r#"# top-level comment
[ui]
# keep this comment
theme = "tokyo-night"
split_ratio = 0.30

[engine]
render_throttle_ms = 16
"#;
        std::fs::write(&p, original).unwrap();
        set_theme(&p, "dracula").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("# top-level comment"));
        assert!(s.contains("# keep this comment"));
        assert!(s.contains("theme = \"dracula\""));
        assert!(s.contains("split_ratio = 0.3"));
        assert!(s.contains("render_throttle_ms = 16"));
    }

    #[test]
    fn adds_ui_section_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("config.toml");
        std::fs::write(&p, "[engine]\nrender_throttle_ms = 8\n").unwrap();
        set_theme(&p, "nord").unwrap();
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.contains("[ui]"));
        assert!(s.contains("theme = \"nord\""));
        assert!(s.contains("render_throttle_ms = 8"));
    }
}
