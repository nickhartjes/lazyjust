pub mod error;

pub use error::{Error, Result};

pub fn run() -> anyhow::Result<()> {
    println!("lazyjust skeleton");
    Ok(())
}
