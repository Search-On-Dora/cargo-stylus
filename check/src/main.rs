// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

use clap::{ArgGroup, Args, Parser};
use ethers::types::{H160, U256};
use eyre::{eyre, Context, Result};
use std::path::PathBuf;
use tokio::runtime::Builder;

use cargo_stylus_check::{self, cache, check, deploy, docker, export_abi, new, verify, CacheConfig, CheckConfig, DeployConfig, VerifyConfig};

#[derive(Parser, Debug)]
#[command(name = "check")]
#[command(bin_name = "cargo stylus")]
#[command(author = "Offchain Labs, Inc.")]
#[command(propagate_version = true)]
#[command(version)]
struct Opts {
    #[command(subcommand)]
    command: Apis,
}

#[derive(Parser, Debug, Clone)]
enum Apis {
    /// Create a new Rust project.
    New {
        /// Project name.
        name: PathBuf,
        /// Create a minimal program.
        #[arg(long)]
        minimal: bool,
    },
    /// Export a Solidity ABI.
    ExportAbi {
        /// The output file (defaults to stdout).
        #[arg(long)]
        output: Option<PathBuf>,
        /// Write a JSON ABI instead using solc. Requires solc.
        #[arg(long)]
        json: bool,
    },
    /// Cache a contract using the Stylus CacheManager for Arbitrum chains.
    Cache(CacheConfig),
    /// Check a contract.
    #[command(alias = "c")]
    Check(CheckConfig),
    /// Deploy a contract.
    #[command(alias = "d")]
    Deploy(DeployConfig),
    /// Build in a Docker container to ensure reproducibility.
    ///
    /// Specify the Rust version to use, followed by the cargo stylus subcommand.
    /// Example: `cargo stylus reproducible 1.77 check`
    Reproducible {
        /// Rust version to use.
        #[arg()]
        rust_version: String,

        /// Stylus subcommand.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        stylus: Vec<String>,
    },
    /// Verify the deployment of a Stylus program.
    #[command(alias = "v")]
    Verify(VerifyConfig),
}

fn main() -> Result<()> {
    let args = Opts::parse();
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(main_impl(args))
}

async fn main_impl(args: Opts) -> Result<()> {
    macro_rules! run {
        ($expr:expr, $($msg:expr),+) => {
            $expr.wrap_err_with(|| eyre!($($msg),+))?
        };
    }

    match args.command {
        Apis::New { name, minimal } => {
            run!(new::new(&name, minimal), "failed to open new project");
        }
        Apis::ExportAbi { json, output } => {
            run!(export_abi::export_abi(output, json), "failed to export abi");
        }
        Apis::Cache(config) => {
            run!(cache::cache_program(&config).await, "stylus cache failed");
        }
        Apis::Check(config) => {
            run!(check::check(&config).await, "stylus checks failed");
        }
        Apis::Deploy(config) => {
            run!(deploy::deploy(config).await, "failed to deploy");
        }
        Apis::Reproducible {
            rust_version,
            stylus,
        } => {
            run!(
                docker::run_reproducible(&rust_version, &stylus),
                "failed reproducible run"
            );
        }
        Apis::Verify(config) => {
            run!(verify::verify(config).await, "failed to verify");
        }
    }
    Ok(())
}
