use std::path::PathBuf;

/// Override for tests and advanced users.
pub const OVERRIDE_ENV: &str = "LAZYRUST_CONFIG_DIR";

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
    // XDG_CONFIG_HOME and get ~/.config/lazyrust/.
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        if !v.is_empty() {
            return PathBuf::from(v).join("lazyrust");
        }
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazyrust")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn env_override_wins() {
        let tmp = tempfile::tempdir().unwrap();
        env::set_var(OVERRIDE_ENV, tmp.path());
        assert_eq!(config_file_path(), tmp.path().join("config.toml"));
        assert_eq!(user_themes_dir(), tmp.path().join("themes"));
        env::remove_var(OVERRIDE_ENV);
    }

    #[test]
    fn falls_back_when_dirs_unavailable() {
        // dirs::config_dir() returns Some on every supported platform in CI,
        // so this test just exercises the default path ending in "lazyrust".
        env::remove_var(OVERRIDE_ENV);
        let p = config_file_path();
        assert_eq!(p.file_name().unwrap(), "config.toml");
        assert_eq!(p.parent().unwrap().file_name().unwrap(), "lazyrust");
    }

    #[test]
    fn xdg_config_home_wins_over_platform_default() {
        env::remove_var(OVERRIDE_ENV);
        let tmp = tempfile::tempdir().unwrap();
        let prev = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", tmp.path());
        assert_eq!(
            config_file_path(),
            tmp.path().join("lazyrust").join("config.toml")
        );
        assert_eq!(
            user_themes_dir(),
            tmp.path().join("lazyrust").join("themes")
        );
        match prev {
            Some(v) => env::set_var("XDG_CONFIG_HOME", v),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }
}
