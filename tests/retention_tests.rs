use lazyjust::session::retention::prune_sessions;
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

#[test]
fn prune_leaves_dirs_it_did_not_create() {
    // `sessions_log_dir` is user-configurable; pointing it at a populated
    // directory must not cost the user its old subdirectories.
    let tmp = tempfile::tempdir().unwrap();
    let foreign = tmp.path().join("photos");
    let near_miss = tmp.path().join("2020-01-01-backup");
    fs::create_dir(&foreign).unwrap();
    fs::create_dir(&near_miss).unwrap();

    let old_time = SystemTime::now() - Duration::from_secs(30 * 24 * 3600);
    for dir in [&foreign, &near_miss] {
        filetime::set_file_mtime(dir, filetime::FileTime::from_system_time(old_time)).unwrap();
    }

    prune_sessions(tmp.path(), Duration::from_secs(7 * 24 * 3600)).unwrap();
    assert!(foreign.exists());
    assert!(near_miss.exists());
}
