use lazyjust::config::Config;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// Serialize these tests: they all mutate the same env var.
// Default `cargo test` parallelism would race on LAZYJUST_CONFIG_DIR;
// a module-level mutex avoids that without pulling in serial_test.
fn guard() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

fn with_config_dir<T>(contents: Option<&str>, body: impl FnOnce() -> T) -> T {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    if let Some(c) = contents {
        std::fs::write(tmp.path().join("config.toml"), c).unwrap();
    }
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let out = body();
    std::env::remove_var("LAZYJUST_CONFIG_DIR");
    out
}

#[test]
fn no_file_returns_defaults() {
    let cfg = with_config_dir(None, Config::load);
    assert_eq!(cfg.render_throttle, Duration::from_millis(16));
    assert_eq!(cfg.tick_interval, Duration::from_millis(250));
}

#[test]
fn partial_file_overrides_only_specified_keys() {
    let cfg = with_config_dir(Some("[engine]\nrender_throttle_ms = 8\n"), Config::load);
    assert_eq!(cfg.render_throttle, Duration::from_millis(8));
    // tick_interval stayed at default
    assert_eq!(cfg.tick_interval, Duration::from_millis(250));
}
