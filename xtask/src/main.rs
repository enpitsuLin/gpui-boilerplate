mod tasks;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo xtask")]
struct Args {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    Pack(tasks::pack::PackArgs),
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        CliCommand::Pack(args) => tasks::pack::run_pack(args),
    }
}
