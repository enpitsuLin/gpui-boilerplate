use std::process::Command;

use anyhow::Result;
use clap::Parser;

use crate::workspace::load_workspace;

#[derive(Parser)]
pub struct BuildArgs {
    /// Build in release mode (optimized build)
    #[arg(long)]
    release: bool,

    /// Additional cargo build arguments (use -- to separate from xtask args)
    #[arg(last = true)]
    additional_args: Vec<String>
}

pub fn run_build(args: BuildArgs) -> Result<()> {
    // Load workspace metadata to get default members
    let workspace = load_workspace()?;

    // Get the first default member package
    let app_package = match workspace.workspace_default_members.get(0) {
        Some(package) => &workspace[package],
        None => return Err(anyhow::anyhow!("No default members found in workspace"))
    };

    // Get package name from the package metadata
    let package_name = &app_package.name;

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--package");
    cmd.arg(package_name);

    if args.release {
        cmd.arg("--release");
    }

    // Add any additional arguments passed by the user
    for arg in args.additional_args {
        cmd.arg(arg);
    }

    println!("Running: {:?}", cmd);

    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Build failed with exit code: {:?}",
            status.code()
        ));
    }

    println!("Build completed successfully!");
    Ok(())
}
