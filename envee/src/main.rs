mod cli;
mod env;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Task};

fn main() -> Result<()> {
    Cli::parse().run()
}
