use core::panic;
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

    let dist_dir = workspace
        .target_directory
        .to_path_buf()
        .into_std_path_buf()
        .join("release");

    let app_dir = app_package
        .manifest_path
        .parent()
        .unwrap()
        .to_path_buf()
        .into_std_path_buf();

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
        let resources = resources.iter().map(|resource| {
            //
            match resource {
                &Resource::Single(ref path) => {
                    let path = PathBuf::from(path);
                    let path = if path.is_absolute() {
                        path
                    } else {
                        app_dir.join(path).into()
                    };
                    Resource::Single(path.to_string_lossy().to_string())
                },
                &Resource::Mapped { ref src, ref target } => {
                    let (src, target) = (PathBuf::from(src), PathBuf::from(target));
                    let src = if src.is_absolute() {
                        src
                    } else {
                        app_dir.join(src).into()
                    };
                    let target = if target.is_absolute() {
                        target.clone()
                    } else {
                        app_dir.join(target).into()
                    };
                    Resource::Mapped {
                        src: src.to_string_lossy().to_string(),
                        target: target
                    }
                },
                _ => panic!("Unsupported resource type")
            }
        });
        config_builder = config_builder.resources(resources);
    }

    if let Some(icons) = icons {
        config_builder = config_builder.icons(
            icons
                .iter()
                .map(|item| app_dir.join(item).to_string_lossy().to_string())
        );
    }

    if let Some(nsis) = metadata.nsis {
        config_builder = config_builder.nsis(nsis);
    }

    let config = config_builder.config();
    println!("building binary package... {:?}", config);
    cargo_packager::package(config).inspect_err(|err| eprintln!("failed to package:\n{err:?}"))?;
    Ok(())
}

/// Returns the contents of the `Cargo.toml` file at the given path.
fn read_cargo_toml(path: impl AsRef<Path>) -> Result<Manifest> {
    let path = path.as_ref();
    let cargo_toml_bytes = fs::read(path)?;
    Manifest::from_slice(&cargo_toml_bytes).with_context(|| format!("reading Cargo.toml at {path:?}"))
}
