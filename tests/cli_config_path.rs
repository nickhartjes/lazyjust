use std::process::Command;

fn cargo_bin() -> String {
    env!("CARGO_BIN_EXE_lazyjust").to_string()
}

#[test]
fn prints_config_file_path_with_env_override() {
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(cargo_bin())
        .env("LAZYJUST_CONFIG_DIR", tmp.path())
        .args(["config", "path"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let expected = tmp.path().join("config.toml");
    assert_eq!(stdout.trim(), expected.to_string_lossy());
}
