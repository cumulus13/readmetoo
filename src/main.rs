mod cli;
mod config;
mod error;
mod pager;
mod renderer;
mod theme;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
