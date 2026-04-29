use super::ConfigError;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    // Unknown top-level keys are silently ignored (serde default) so
    // users can keep `[ui]` / `[keys]` blocks now; those become active
    // in later milestones without breaking this loader.
    pub ui: Option<UiSection>,
    pub paths: Option<PathsSection>,
    pub logging: Option<LoggingSection>,
    pub engine: Option<EngineSection>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PathsSection {
    pub state_dir: Option<String>,
    pub sessions_log_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LoggingSection {
    pub session_log_size_cap_mb: Option<u64>,
    pub session_log_retention_days: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EngineSection {
    pub render_throttle_ms: Option<u64>,
    pub tick_interval_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UiSection {
    pub theme: Option<String>,
    pub icon_style: Option<String>,
    pub list_mode: Option<String>,
}

impl ConfigFile {
    /// Reads and parses the file. Returns `Ok(None)` if the file does not exist.
    pub fn read(path: &Path) -> Result<Option<ConfigFile>, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(Some(toml::from_str(&s)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ConfigError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(contents: &str) -> tempfile::NamedTempFile {
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(f.path(), contents).unwrap();
        f
    }

    #[test]
    fn missing_file_returns_none() {
        let p = std::path::Path::new("/definitely/does/not/exist/lazyjust.toml");
        assert!(matches!(ConfigFile::read(p), Ok(None)));
    }

    #[test]
    fn empty_file_parses() {
        let f = write("");
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert!(cf.paths.is_none());
        assert!(cf.logging.is_none());
        assert!(cf.engine.is_none());
    }

    #[test]
    fn partial_file_parses() {
        let f = write(
            r#"
            [engine]
            render_throttle_ms = 8
            "#,
        );
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert_eq!(cf.engine.unwrap().render_throttle_ms, Some(8));
        assert!(cf.paths.is_none());
        assert!(cf.logging.is_none());
    }

    #[test]
    fn unknown_sections_ignored() {
        let f = write(
            r#"
            [ui]
            theme = "tokyo-night"

            [keys]
            quit = "q"
            "#,
        );
        // [keys] is not yet defined in ConfigFile; serde silently ignores unknown keys by default.
        let cf = ConfigFile::read(f.path()).unwrap().unwrap();
        assert!(cf.paths.is_none());
    }

    #[test]
    fn malformed_file_returns_parse_error() {
        let f = write("this is = = not valid toml [[");
        let err = ConfigFile::read(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
    }
}
