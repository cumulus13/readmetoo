mod cli;
mod config;
mod error;
mod pager;
mod renderer;
mod theme;

use anyhow::Result;
use clap::Parser;
use clap_version_flag::colorful_version;
use cli::Cli;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && (args[1] == "-V" || args[1] == "--version") {
        let version = colorful_version!();
        version.print_and_exit();
    }
    let cli = Cli::parse();
    cli.run()
}
