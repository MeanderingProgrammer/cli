use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::Task;
use crate::env::Resolver;

#[derive(Debug, Parser)]
/// show final env
pub struct Show {
    /// path(s) to your env file(s)
    #[arg(short, long)]
    files: Vec<PathBuf>,
}

impl Task for Show {
    fn run(&self) -> Result<()> {
        let env = Resolver::new(self.files.clone()).get()?;
        for (key, value) in env {
            println!("export {key}=\"{value}\"");
        }
        Ok(())
    }
}
