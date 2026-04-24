#![cfg(not(windows))]

use lazyjust::session::osc::scan_done_marker;
use lazyjust::session::pty::spawn;
use lazyjust::session::wrapper::{build_unix_command, prime_line};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn make_justfile(tmp: &tempfile::TempDir) -> PathBuf {
    let path = tmp.path().join("justfile");
    std::fs::write(&path, "hi:\n\techo lazyjust-hello\n").unwrap();
    path
}

#[test]
fn spawn_echo_recipe_and_capture_done_marker() {
    // Force a minimal shell so rc files cannot eat stdin or reorder output.
    std::env::set_var("SHELL", "/bin/sh");

    let tmp = tempfile::tempdir().unwrap();
    let justfile = make_justfile(&tmp);

    let (argv, _) = build_unix_command(&justfile, "hi", &[]);
    let mut spawned = spawn(&argv, tmp.path(), 24, 80).unwrap();

    let line = prime_line(&justfile, "hi", &[]);
    spawned.writer.write_all(line.as_bytes()).unwrap();
    spawned.writer.write_all(b"\n").unwrap();
    spawned.writer.flush().unwrap();

    let mut buf = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut chunk = [0u8; 4096];
    loop {
        if Instant::now() > deadline {
            panic!("timeout waiting for done marker");
        }
        match spawned.reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                let (_, codes) = scan_done_marker(&buf);
                if !codes.is_empty() {
                    assert_eq!(codes[0], 0);
                    assert!(std::str::from_utf8(&buf)
                        .unwrap()
                        .contains("lazyjust-hello"));
                    let _ = spawned.child.kill();
                    return;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => panic!("read err: {e}"),
        }
    }
    panic!("EOF before done marker");
}

#[tokio::test]
async fn session_manager_spawn_recipe_primes_shell_and_emits_done() {
    use lazyjust::app::action::AppEvent;
    use lazyjust::session::manager::SessionManager;

    std::env::set_var("SHELL", "/bin/sh");

    let tmp = tempfile::tempdir().unwrap();
    let justfile = make_justfile(&tmp);
    let log_path = tmp.path().join("session.log");

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(256);
    let mut mgr = SessionManager::default();

    let _meta = mgr
        .spawn_recipe(
            1,
            &justfile,
            "hi",
            &[],
            tmp.path(),
            24,
            80,
            log_path.clone(),
            tx,
            1024 * 1024,
        )
        .unwrap();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    let mut collected: Vec<u8> = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            panic!("timeout waiting for done marker; got {:?}", String::from_utf8_lossy(&collected));
        }
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(AppEvent::SessionBytes { id, bytes })) => {
                assert_eq!(id, 1);
                collected.extend_from_slice(&bytes);
                let (_, codes) = scan_done_marker(&collected);
                if !codes.is_empty() {
                    assert_eq!(codes[0], 0);
                    assert!(std::str::from_utf8(&collected).unwrap().contains("lazyjust-hello"));
                    mgr.kill(1);
                    return;
                }
            }
            Ok(Some(AppEvent::RecipeExited { id, code })) => {
                // `spawn_reader` strips the OSC done marker from `SessionBytes` and
                // surfaces it as a separate `RecipeExited` event; treat it as the
                // channel-level equivalent of the marker.
                assert_eq!(id, 1);
                assert_eq!(code, 0);
                assert!(std::str::from_utf8(&collected).unwrap().contains("lazyjust-hello"));
                mgr.kill(1);
                return;
            }
            Ok(Some(_)) => continue,
            Ok(None) => panic!("channel closed before done marker"),
            Err(_) => panic!("timeout waiting for done marker"),
        }
    }
}
