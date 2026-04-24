use super::osc::scan_done_marker;
use crate::app::action::AppEvent;
use crate::app::types::SessionId;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

/// Shared timestamp of the most recent PTY read. `None` until the first byte
/// arrives. The prime-coordinator watches this to decide when the shell has
/// finished rc-file output and gone idle at a prompt.
pub type LastOutput = Arc<Mutex<Option<Instant>>>;

pub fn spawn_reader<R>(mut reader: R, id: SessionId, tx: Sender<AppEvent>, last_output: LastOutput)
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let exit_code: i32 = loop {
            match reader.read(&mut buf) {
                Ok(0) => break 0,
                Ok(n) => {
                    if let Ok(mut t) = last_output.lock() {
                        *t = Some(Instant::now());
                    }
                    let (stripped, codes) = scan_done_marker(&buf[..n]);
                    if !stripped.is_empty() {
                        let _ = tx.blocking_send(AppEvent::SessionBytes {
                            id,
                            bytes: stripped,
                        });
                    }
                    for code in codes {
                        let _ = tx.blocking_send(AppEvent::RecipeExited { id, code });
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break -1,
            }
        };
        let _ = tx.blocking_send(AppEvent::SessionExited {
            id,
            code: exit_code,
        });
    });
}
