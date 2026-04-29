//! End-to-end smoke that the CLI flag overrides the config-file value.
//! We run `Config::load()` against a tempdir that contains a config
//! setting `list_mode = "active"`, then simulate the CLI override path.

use lazyjust::app::types::ListMode;
use lazyjust::cli::ListModeArg;
use lazyjust::config::Config;

#[test]
fn cli_flag_overrides_config_file() {
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
