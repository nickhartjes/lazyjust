use std::process::Command;

fn cargo_bin() -> String {
    env!("CARGO_BIN_EXE_lazyjust").to_string()
}

#[test]
fn init_writes_template_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let path = tmp.path().join("config.toml");
    assert!(path.exists());
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("[ui]"));
    assert!(contents.contains("theme = \"tokyo-night\""));
}

#[test]
fn init_refuses_to_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("config.toml"), "existing = true\n").unwrap();

    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("refusing to overwrite"));

    // Existing contents preserved.
    let preserved = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
    assert_eq!(preserved, "existing = true\n");
}

#[test]
fn init_then_load_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(out.status.success());

    // Load from inside the test process using the same env override.
    std::env::set_var("LAZYJUST_CONFIG_DIR", tmp.path());
    let cfg = lazyjust::config::Config::load();
    std::env::remove_var("LAZYJUST_CONFIG_DIR");

    // Template values match defaults, so load succeeds and key values equal defaults.
    assert_eq!(cfg.render_throttle, std::time::Duration::from_millis(16));
    assert_eq!(cfg.tick_interval, std::time::Duration::from_millis(250));
    assert_eq!(cfg.session_log_size_cap, 10 * 1024 * 1024);
}
