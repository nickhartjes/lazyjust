use lazyrust::session::retention::prune_sessions;
use std::fs;
use std::time::{Duration, SystemTime};

#[test]
fn prune_removes_old_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let old = tmp.path().join("2020-01-01");
    let new = tmp.path().join("2099-01-01");
    fs::create_dir(&old).unwrap();
    fs::create_dir(&new).unwrap();

    // Set old dir mtime to 30 days ago
    let old_time = SystemTime::now() - Duration::from_secs(30 * 24 * 3600);
    filetime::set_file_mtime(&old, filetime::FileTime::from_system_time(old_time)).unwrap();

    prune_sessions(tmp.path(), Duration::from_secs(7 * 24 * 3600)).unwrap();
    assert!(!old.exists());
    assert!(new.exists());
}
