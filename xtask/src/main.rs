mod tasks;
mod workspace;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo xtask")]
struct Args {
    #[command(subcommand)]
    command: CliCommand
}

#[derive(Subcommand)]
enum CliCommand {
    /// Build the main application package
    Build(tasks::build::BuildArgs),
    /// Package the application for distribution
    Pack(tasks::pack::PackArgs)
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        CliCommand::Build(args) => tasks::build::run_build(args),
        CliCommand::Pack(args) => tasks::pack::run_pack(args)
    }
}
