use super::pty::{spawn, SpawnedPty};
use super::reader::spawn_reader;
use super::wrapper::build_unix_command;
use crate::app::action::AppEvent;
use crate::app::types::{SessionId, SessionMeta, Status};
use crate::error::Result;
use portable_pty::MasterPty;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct SessionHandle {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub writer: Box<dyn Write + Send>,
    pub log_writer: Option<std::fs::File>,
    pub log_written: u64,
    pub log_cap: u64,
}

#[derive(Default)]
pub struct SessionManager {
    handles: HashMap<SessionId, SessionHandle>,
}

impl SessionManager {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_recipe(
        &mut self,
        id: SessionId,
        justfile: &Path,
        recipe: &str,
        args: &[String],
        cwd: &Path,
        rows: u16,
        cols: u16,
        log_path: PathBuf,
        tx: Sender<AppEvent>,
        log_cap: u64,
    ) -> Result<SessionMeta> {
        let (argv, _) = build_unix_command(justfile, recipe, args);

        let SpawnedPty {
            master,
            child,
            writer,
            reader,
        } = spawn(&argv, cwd, rows, cols)?;

        let command_line = format!(
            "just --justfile {} {} {}",
            justfile.display(),
            recipe,
            args.join(" ")
        );

        spawn_reader(reader, id, tx);

        let log_writer = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        self.handles.insert(
            id,
            SessionHandle {
                master,
                child,
                writer,
                log_writer,
                log_written: 0,
                log_cap,
            },
        );

        Ok(SessionMeta {
            id,
            recipe_name: recipe.to_string(),
            command_line,
            status: Status::Running,
            unread: true,
            started_at: Instant::now(),
            log_path,
        })
    }

    pub fn write_log(&mut self, id: SessionId, bytes: &[u8]) {
        if let Some(h) = self.handles.get_mut(&id) {
            if let Some(f) = h.log_writer.as_mut() {
                if h.log_written.saturating_add(bytes.len() as u64) > h.log_cap {
                    return;
                }
                let _ = f.write_all(bytes);
                h.log_written += bytes.len() as u64;
            }
        }
    }

    pub fn write(&mut self, id: SessionId, bytes: &[u8]) -> std::io::Result<()> {
        if let Some(h) = self.handles.get_mut(&id) {
            h.writer.write_all(bytes)?;
            h.writer.flush()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, id: SessionId, rows: u16, cols: u16) -> Result<()> {
        if let Some(h) = self.handles.get_mut(&id) {
            let _ = h.master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        Ok(())
    }

    pub fn kill(&mut self, id: SessionId) {
        if let Some(mut h) = self.handles.remove(&id) {
            let _ = h.child.kill();
        }
    }

    pub fn try_wait(&mut self, id: SessionId) -> Option<i32> {
        if let Some(h) = self.handles.get_mut(&id) {
            match h.child.try_wait() {
                Ok(Some(status)) => Some(status.exit_code() as i32),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn running_ids(&self) -> Vec<SessionId> {
        self.handles.keys().copied().collect()
    }
}
