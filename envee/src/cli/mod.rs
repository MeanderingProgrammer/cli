mod run;
mod show;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub trait Task {
    fn run(&self) -> Result<()>;
}

#[derive(Debug, Parser)]
#[command(version)]
/// manage environment variables for all your stages
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Task for Cli {
    fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(run::Run),
    Show(show::Show),
}

impl Task for Commands {
    fn run(&self) -> Result<()> {
        match self {
            Self::Run(task) => task.run(),
            Self::Show(task) => task.run(),
        }
    }
}
