use std::path::PathBuf;

use clap::{arg, command, Args, ArgGroup};
use ethers::types::{H160, U256};

pub mod cache;
pub mod check;
pub mod constants;
pub mod deploy;
pub mod docker;
pub mod export_abi;
pub mod macros;
pub mod new;
pub mod project;
pub mod verify;
pub mod wallet;

#[derive(Args, Clone, Debug)]
pub struct CommonConfig {
    /// Arbitrum RPC endpoint.
    #[arg(short, long, default_value = "https://sepolia-rollup.arbitrum.io/rpc")]
    endpoint: String,
    /// Whether to use stable Rust.
    #[arg(long)]
    rust_stable: bool,
    /// Whether to print debug info.
    #[arg(long)]
    verbose: bool,
    /// The path to source files to include in the project hash, which
    /// is included in the contract deployment init code transaction
    /// to be used for verification of deployment integrity.
    /// If not provided, all .rs files and Cargo.toml and Cargo.lock files
    /// in project's directory tree are included.
    #[arg(long)]
    source_files_for_project_hash: Vec<String>,
    #[arg(long)]
    /// Optional max fee per gas in gwei units.
    max_fee_per_gas_gwei: Option<U256>,
}


#[derive(Clone, Debug, Args)]
#[clap(group(ArgGroup::new("key").required(true).args(&["private_key_path", "private_key", "keystore_path"])))]
pub struct AuthOpts {
    /// File path to a text file containing a hex-encoded private key.
    #[arg(long)]
    private_key_path: Option<PathBuf>,
    /// Private key as a hex string. Warning: this exposes your key to shell history.
    #[arg(long)]
    private_key: Option<String>,
    /// Path to an Ethereum wallet keystore file (e.g. clef).
    #[arg(long)]
    keystore_path: Option<String>,
    /// Keystore password file.
    #[arg(long)]
    keystore_password_path: Option<PathBuf>,
}

#[derive(Args, Clone, Debug)]
pub struct CacheConfig {
    #[command(flatten)]
    common_cfg: CommonConfig,
    /// Wallet source to use.
    #[command(flatten)]
    auth: AuthOpts,
    /// Deployed and activated program address to cache.
    #[arg(long)]
    program_address: H160,
    /// Bid, in wei, to place on the desired program to cache
    #[arg(short, long, hide(true))]
    bid: Option<u64>,
}

#[derive(Args, Clone, Debug)]
pub struct CheckConfig {
    #[command(flatten)]
    common_cfg: CommonConfig,
    /// The WASM to check (defaults to any found in the current directory).
    #[arg(long)]
    wasm_file: Option<PathBuf>,
    /// Where to deploy and activate the program (defaults to a random address).
    #[arg(long)]
    program_address: Option<H160>,
}

#[derive(Args, Clone, Debug)]
pub struct DeployConfig {
    #[command(flatten)]
    check_config: CheckConfig,
    /// Wallet source to use.
    #[command(flatten)]
    auth: AuthOpts,
    /// Only perform gas estimation.
    #[arg(long)]
    estimate_gas: bool,
}

#[derive(Args, Clone, Debug)]
pub struct VerifyConfig {
    #[command(flatten)]
    common_cfg: CommonConfig,

    /// Hash of the deployment transaction.
    #[arg(long)]
    deployment_tx: String,
}