use crate::error::{Error, Result};
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::Path;

pub struct SpawnedPty {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub writer: Box<dyn Write + Send>,
    pub reader: Box<dyn Read + Send>,
}

pub fn spawn(argv: &[String], cwd: &Path, rows: u16, cols: u16) -> Result<SpawnedPty> {
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| Error::PtySpawn(e.to_string()))?;

    let mut cmd = CommandBuilder::new(&argv[0]);
    for arg in &argv[1..] {
        cmd.arg(arg);
    }
    cmd.cwd(cwd);

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| Error::PtySpawn(e.to_string()))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| Error::PtySpawn(e.to_string()))?;
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| Error::PtySpawn(e.to_string()))?;

    drop(pair.slave);

    Ok(SpawnedPty {
        master: pair.master,
        child,
        writer,
        reader,
    })
}
