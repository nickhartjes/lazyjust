use std::path::PathBuf;

/// Override for tests and advanced users.
pub const OVERRIDE_ENV: &str = "LAZYJUST_CONFIG_DIR";

pub fn config_file_path() -> PathBuf {
    config_root().join("config.toml")
}

pub fn user_themes_dir() -> PathBuf {
    config_root().join("themes")
}

fn config_root() -> PathBuf {
    if let Ok(v) = std::env::var(OVERRIDE_ENV) {
        return PathBuf::from(v);
    }
    // Honor XDG_CONFIG_HOME on every platform (including macOS, where
    // dirs::config_dir() ignores it and returns ~/Library/Application
    // Support). Users who prefer the XDG layout on macOS can export
    // XDG_CONFIG_HOME and get ~/.config/lazyjust/.
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        if !v.is_empty() {
            return PathBuf::from(v).join("lazyjust");
        }
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazyjust")
}

/// Process-global mutex for tests that mutate environment variables read
/// by config discovery (`LAZYJUST_CONFIG_DIR`, `XDG_CONFIG_HOME`). All such
/// tests across the crate must lock this same mutex; using per-module
/// mutexes lets parallel tests race on shared env state.
#[cfg(test)]
pub(crate) fn env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn env_override_wins() {
        let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        env::set_var(OVERRIDE_ENV, tmp.path());
        assert_eq!(config_file_path(), tmp.path().join("config.toml"));
        assert_eq!(user_themes_dir(), tmp.path().join("themes"));
        env::remove_var(OVERRIDE_ENV);
    }

    #[test]
    fn falls_back_when_dirs_unavailable() {
        let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        env::remove_var(OVERRIDE_ENV);
        let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
        env::remove_var("XDG_CONFIG_HOME");
        let p = config_file_path();
        assert_eq!(p.file_name().unwrap(), "config.toml");
        assert_eq!(p.parent().unwrap().file_name().unwrap(), "lazyjust");
        if let Some(v) = prev_xdg {
            env::set_var("XDG_CONFIG_HOME", v);
        }
    }

    #[test]
    fn xdg_config_home_wins_over_platform_default() {
        let _g = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        env::remove_var(OVERRIDE_ENV);
        let tmp = tempfile::tempdir().unwrap();
        let prev = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", tmp.path());
        assert_eq!(
            config_file_path(),
            tmp.path().join("lazyjust").join("config.toml")
        );
        assert_eq!(
            user_themes_dir(),
            tmp.path().join("lazyjust").join("themes")
        );
        match prev {
            Some(v) => env::set_var("XDG_CONFIG_HOME", v),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }
}
