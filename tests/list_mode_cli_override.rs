//! End-to-end smoke that the CLI flag overrides the config-file value.
//! We run `Config::load()` against a tempdir that contains a config
//! setting `list_mode = "active"`, then simulate the CLI override path.

use lazyjust::app::types::ListMode;
use lazyjust::cli::ListModeArg;
use lazyjust::config::Config;
use std::sync::{Mutex, OnceLock};

fn guard() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

#[test]
fn cli_flag_overrides_config_file() {
    let _lock = guard().lock().unwrap_or_else(|e| e.into_inner());

    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("config.toml"),
        r#"
[ui]
list_mode = "active"
"#,
    )
    .unwrap();
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());

    let mut cfg = Config::load();
    assert_eq!(cfg.list_mode, ListMode::Active);

    cfg.list_mode = ListModeArg::All.into();
    assert_eq!(cfg.list_mode, ListMode::All);

    std::env::remove_var("LAZYJUST_CONFIG_DIR");
}
