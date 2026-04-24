use super::file::ConfigFile;
use super::Config;
use std::path::PathBuf;
use std::time::Duration;

/// Overlay a parsed file onto a base `Config`, filling missing values
/// from the base. The returned `Config` is ready for use.
pub fn merge(file: ConfigFile, base: Config) -> Config {
    let mut out = base;

    if let Some(p) = file.paths {
        if let Some(d) = p.state_dir {
            out.state_dir = PathBuf::from(d);
        }
        if let Some(d) = p.sessions_log_dir {
            out.sessions_log_dir = PathBuf::from(d);
        }
    }

    if let Some(l) = file.logging {
        if let Some(mb) = l.session_log_size_cap_mb {
            out.session_log_size_cap = mb.saturating_mul(1024 * 1024);
        }
        if let Some(days) = l.session_log_retention_days {
            out.session_log_retention = Duration::from_secs(days.saturating_mul(24 * 3600));
        }
    }

    if let Some(e) = file.engine {
        if let Some(ms) = e.render_throttle_ms {
            out.render_throttle = Duration::from_millis(ms);
        }
        if let Some(ms) = e.tick_interval_ms {
            out.tick_interval = Duration::from_millis(ms);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::defaults::defaults;
    use crate::config::file::{ConfigFile, EngineSection, LoggingSection, PathsSection};

    #[test]
    fn empty_file_yields_defaults() {
        let d = defaults();
        let expected_throttle = d.render_throttle;
        let merged = merge(ConfigFile::default(), d);
        assert_eq!(merged.render_throttle, expected_throttle);
    }

    #[test]
    fn engine_overrides_apply() {
        let file = ConfigFile {
            engine: Some(EngineSection {
                render_throttle_ms: Some(8),
                tick_interval_ms: Some(500),
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.render_throttle, Duration::from_millis(8));
        assert_eq!(merged.tick_interval, Duration::from_millis(500));
    }

    #[test]
    fn logging_mb_converts_to_bytes() {
        let file = ConfigFile {
            logging: Some(LoggingSection {
                session_log_size_cap_mb: Some(5),
                session_log_retention_days: Some(2),
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.session_log_size_cap, 5 * 1024 * 1024);
        assert_eq!(
            merged.session_log_retention,
            Duration::from_secs(2 * 24 * 3600)
        );
    }

    #[test]
    fn paths_override_apply() {
        let file = ConfigFile {
            paths: Some(PathsSection {
                state_dir: Some("/tmp/lj-test".into()),
                sessions_log_dir: None,
            }),
            ..Default::default()
        };
        let merged = merge(file, defaults());
        assert_eq!(merged.state_dir, PathBuf::from("/tmp/lj-test"));
        // sessions_log_dir untouched when not specified
        assert_ne!(merged.sessions_log_dir, PathBuf::from("/tmp/lj-test"));
    }
}
