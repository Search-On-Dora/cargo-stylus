// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

#![allow(clippy::println_empty_string)]

use std::path::PathBuf;

use eyre::{bail, eyre};

use ethers::middleware::Middleware;
use ethers::types::H256;

use serde::{Deserialize, Serialize};

use crate::{
    check,
    deploy::{self, extract_compressed_wasm, extract_program_evm_deployment_prelude},
    project, CheckConfig, VerifyConfig,
};
use cargo_stylus_util::{color::Color, sys};

#[derive(Debug, Deserialize, Serialize)]
struct RpcResult {
    input: String,
}

pub async fn verify(cfg: VerifyConfig) -> eyre::Result<()> {
    let provider = sys::new_provider(&cfg.common_cfg.endpoint)?;
    let hash = cargo_stylus_util::text::decode0x(cfg.deployment_tx)?;
    if hash.len() != 32 {
        bail!("Invalid hash");
    }
    let Some(result) = provider
        .get_transaction(H256::from_slice(&hash))
        .await
        .map_err(|e| eyre!("RPC failed: {e}"))?
    else {
        bail!("No code at address");
    };

    let output = sys::new_command("cargo")
        .arg("clean")
        .output()
        .map_err(|e| eyre!("failed to execute cargo clean: {e}"))?;
    if !output.status.success() {
        bail!("cargo clean command failed");
    }
    let check_cfg = CheckConfig {
        common_cfg: cfg.common_cfg.clone(),
        wasm_file: None,
        program_address: None,
    };
    let _ = check::check(&check_cfg)
        .await
        .map_err(|e| eyre!("Stylus checks failed: {e}"))?;
    let build_cfg = project::BuildConfig {
        opt_level: project::OptLevel::default(),
        stable: cfg.common_cfg.rust_stable,
    };
    let wasm_file: PathBuf = project::build_dylib(build_cfg.clone())
        .map_err(|e| eyre!("could not build project to WASM: {e}"))?;
    let project_hash =
        project::hash_files(cfg.common_cfg.source_files_for_project_hash, build_cfg)?;
    let (_, init_code) = project::compress_wasm(&wasm_file, project_hash)?;
    let deployment_data = deploy::program_deployment_calldata(&init_code);
    if deployment_data == *result.input {
        println!("Verified - program matches local project's file hashes");
    } else {
        let tx_prelude = extract_program_evm_deployment_prelude(&*result.input);
        let reconstructed_prelude = extract_program_evm_deployment_prelude(&deployment_data);
        println!(
            "{} - program deployment did not verify against local project's file hashes",
            "FAILED".red()
        );
        if tx_prelude != reconstructed_prelude {
            println!("Prelude mismatch");
            println!("Deployment tx prelude {}", hex::encode(tx_prelude));
            println!(
                "Reconstructed prelude {}",
                hex::encode(reconstructed_prelude)
            );
        } else {
            println!("Compressed WASM bytecode mismatch");
        }
        println!(
            "Compressed code length of locally reconstructed {}",
            init_code.len()
        );
        println!(
            "Compressed code length of deployment tx {}",
            extract_compressed_wasm(&*result.input).len()
        );
    }
    Ok(())
}
