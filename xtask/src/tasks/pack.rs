use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Ok, Result};
use cargo_packager::config::{Binary, HookCommand, NsisConfig, Resource};
use cargo_toml::Manifest;
use clap::Parser;
use serde::Deserialize;

use crate::workspace::load_workspace;

#[derive(Parser)]
pub struct PackArgs {
    /// Output directory
    #[arg(short, long)]
    out_dir: Option<String>
}

#[derive(Deserialize, Debug)]
struct PackageMetadata {
    identifier: String,
    #[serde(rename(deserialize = "product-name"))]
    product_name: String,
    resources: Option<Vec<Resource>>,
    icons: Option<Vec<String>>,
    nsis: Option<NsisConfig>
}

pub fn run_pack(args: PackArgs) -> Result<()> {
    let workspace = load_workspace()?;

    let app_package = match workspace.workspace_default_members.get(0) {
        Some(package) => &workspace[package],
        None => return Err(anyhow::anyhow!("app package not found"))
    };

    let dist_dir = workspace.target_directory.to_path_buf().join("release");

    let manifest = read_cargo_toml(app_package.manifest_path.clone())?;

    let binary_name = manifest.bin.get(0).unwrap().name.clone().unwrap();

    #[cfg(not(target_env = "msvc"))]
    let bin_path = dist_dir.join(binary_name);

    #[cfg(target_env = "msvc")]
    let bin_path = dist_dir.join(format!("{}.exe", binary_name));

    let package = manifest.package.unwrap();
    let version = package.version.unwrap().to_string();

    let metadata = package
        .metadata
        .unwrap()
        .try_into::<PackageMetadata>()
        .with_context(|| format!("failed to deserialize package metadata"))?;

    println!("metadata: {:?}", metadata);

    let out_dit = match &args.out_dir {
        Some(dir) => PathBuf::from(dir),
        None => workspace.workspace_root.to_path_buf().join("dist").into()
    };

    let resources = metadata.resources;

    let icons = metadata.icons;

    let build_command = HookCommand::Script("cargo build --release".into());

    let mut config_builder = cargo_packager::Config::builder()
        .before_packaging_command(build_command)
        .product_name(metadata.product_name)
        .identifier(metadata.identifier)
        .version(version)
        .binaries(vec![Binary::new(bin_path).main(true)])
        .out_dir(out_dit)
        .log_level(cargo_packager::config::LogLevel::Trace);

    if let Some(resources) = resources {
        config_builder = config_builder.resources(resources);
    }

    if let Some(icons) = icons {
        config_builder = config_builder.icons(icons);
    }

    if let Some(nsis) = metadata.nsis {
        config_builder = config_builder.nsis(nsis);
    }

    println!("building binary package...");
    cargo_packager::package(config_builder.config())
        .inspect_err(|err| eprintln!("failed to package:\n{err:?}"))?;
    Ok(())
}

/// Returns the contents of the `Cargo.toml` file at the given path.
fn read_cargo_toml(path: impl AsRef<Path>) -> Result<Manifest> {
    let path = path.as_ref();
    let cargo_toml_bytes = fs::read(path)?;
    Manifest::from_slice(&cargo_toml_bytes).with_context(|| format!("reading Cargo.toml at {path:?}"))
}
