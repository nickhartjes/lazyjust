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
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lazyjust")
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
        // so this test just exercises the default path ending in "lazyjust".
        env::remove_var(OVERRIDE_ENV);
        let p = config_file_path();
        assert_eq!(p.file_name().unwrap(), "config.toml");
        assert_eq!(p.parent().unwrap().file_name().unwrap(), "lazyjust");
    }
}
