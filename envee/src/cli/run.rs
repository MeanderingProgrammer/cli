use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, bail};
use clap::Parser;

use crate::Task;
use crate::env::Resolver;

#[derive(Debug, Parser)]
/// inject env at runtime
pub struct Run {
    /// path(s) to your env file(s)
    #[arg(short, long)]
    files: Vec<PathBuf>,

    /// command to run in environment
    #[arg(required = true, last = true)]
    args: Vec<String>,
}

impl Task for Run {
    fn run(&self) -> Result<()> {
        let program = &self.args[0];
        let args = &self.args[1..];
        let env = Resolver::new(self.files.clone()).get()?;
        let status = Command::new(program).args(args).envs(env).status()?;
        if !status.success() {
            bail!("command failed: {status}")
        }
        Ok(())
    }
}
