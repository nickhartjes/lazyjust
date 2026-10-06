#![cfg(not(windows))]

use lazyjust::session::osc::scan_done_marker;
use lazyjust::session::pty::spawn;
use lazyjust::session::shell::prime_line;
use lazyjust::session::wrapper::build_unix_command;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Once;
use std::time::{Duration, Instant};

// Force every PTY spawn in this test binary to run `/bin/sh` rather than
// the developer's login shell. `Once` guarantees a single mutation even
// under `cargo test`'s parallel runner. If a future test needs a non-POSIX
// shell, drop this helper and adopt a per-test RAII guard that saves and
// restores the prior value.
static INIT_SHELL: Once = Once::new();

fn force_posix_shell() {
    INIT_SHELL.call_once(|| {
        std::env::set_var("SHELL", "/bin/sh");
    });
}

fn make_justfile(tmp: &tempfile::TempDir) -> PathBuf {
    let path = tmp.path().join("justfile");
    std::fs::write(&path, "hi:\n\techo lazyjust-hello\n").unwrap();
    path
}

#[test]
fn spawn_echo_recipe_and_capture_done_marker() {
    force_posix_shell();

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

    force_posix_shell();

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
            panic!(
                "timeout waiting for done marker; got {:?}",
                String::from_utf8_lossy(&collected)
            );
        }
        match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Some(AppEvent::SessionBytes { id, bytes })) => {
                assert_eq!(id, 1);
                collected.extend_from_slice(&bytes);
                let (_, codes) = scan_done_marker(&collected);
                if !codes.is_empty() {
                    assert_eq!(codes[0], 0);
                    assert!(std::str::from_utf8(&collected)
                        .unwrap()
                        .contains("lazyjust-hello"));
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
                assert!(std::str::from_utf8(&collected)
                    .unwrap()
                    .contains("lazyjust-hello"));
                mgr.kill(1);
                return;
            }
            Ok(Some(_)) => continue,
            Ok(None) => panic!("channel closed before done marker"),
            Err(_) => panic!("timeout waiting for done marker"),
        }
    }
}

/// A stand-in for an rc file that probes the terminal before handing over
/// to the real shell — the shape of `fastfetch`, powerlevel10k's instant
/// prompt, and anything else that draws inline images. It emits
/// `ESC [ 6 n` and then blocks reading stdin one byte at a time until the
/// report's terminating `R` arrives.
///
/// Without an answer it never returns, and it eats the primed recipe line
/// while it waits — the user-visible symptom is a blank session pane.
const PROBE_SHELL: &str = r#"#!/bin/sh
stty raw -echo
printf '\033[6n'
while :; do
  c=$(dd bs=1 count=1 2>/dev/null)
  case "$c" in R) break ;; esac
done
stty sane
exec /bin/sh -i
"#;

fn write_probe_shell(tmp: &tempfile::TempDir) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = tmp.path().join("probe-shell.sh");
    std::fs::write(&path, PROBE_SHELL).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn answers_cursor_query_so_the_primed_recipe_still_runs() {
    use lazyjust::app::event_loop::feed_session_bytes;

    let tmp = tempfile::tempdir().unwrap();
    let justfile = make_justfile(&tmp);
    let probe = write_probe_shell(&tmp);

    let argv = vec![probe.display().to_string()];
    let mut spawned = spawn(&argv, tmp.path(), 24, 80).unwrap();

    // Drain the PTY on a thread so the main thread can answer queries and
    // prime the recipe on the same idle heuristic `SessionManager` uses.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let mut reader = spawned.reader;
    std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(chunk[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    let mut screen = vt100::Parser::new(24, 80, 0);
    let mut collected: Vec<u8> = Vec::new();
    let mut last_output: Option<Instant> = None;
    let mut primed = false;
    let start = Instant::now();
    let deadline = start + Duration::from_secs(20);

    loop {
        assert!(
            Instant::now() < deadline,
            "timed out: the probing shell never ran the recipe. \
             cursor query answered? captured {} bytes",
            collected.len()
        );

        if let Ok(bytes) = rx.recv_timeout(Duration::from_millis(50)) {
            collected.extend_from_slice(&bytes);
            last_output = Some(Instant::now());
            // This is exactly what the event loop does for SessionBytes.
            for report in feed_session_bytes(&mut screen, &bytes) {
                spawned.writer.write_all(&report).unwrap();
                spawned.writer.flush().unwrap();
            }
        }

        let (_, codes) = scan_done_marker(&collected);
        if !codes.is_empty() {
            assert_eq!(codes[0], 0, "recipe exited non-zero");
            assert!(
                String::from_utf8_lossy(&collected).contains("lazyjust-hello"),
                "done marker arrived without the recipe's output"
            );
            let _ = spawned.child.kill();
            return;
        }

        if !primed {
            let idle = last_output.is_some_and(|t| t.elapsed() >= Duration::from_millis(400));
            if idle || start.elapsed() >= Duration::from_secs(5) {
                let line = prime_line(&justfile, "hi", &[]);
                spawned.writer.write_all(line.as_bytes()).unwrap();
                spawned.writer.write_all(b"\r").unwrap();
                spawned.writer.flush().unwrap();
                primed = true;
            }
        }
    }
}
