use anyhow::Result;
use clap::Parser;

use envee::{cli::Cli, cli::Task};

fn main() -> Result<()> {
    Cli::parse().run()
}
