//! Web toolchain management.

use color_eyre::eyre;

use crate::{
    toolchain::{Installation, Toolchain, ToolchainError},
    utils::which,
};

/// Web toolchain (Rust wasm target + wasm-bindgen tooling).
#[derive(Debug, Clone, Copy)]
pub struct WebToolchain;

/// Installation plan for missing web toolchain components.
#[derive(Debug, Clone)]
pub struct WebToolchainInstall;

impl Installation for WebToolchainInstall {
    type Error = eyre::Report;

    async fn install(&self) -> Result<(), Self::Error> {
        Err(eyre::eyre!(
            "Auto-fix is not implemented for web toolchain. Install `wasm-bindgen-cli` and add Rust target `wasm32-unknown-unknown`."
        ))
    }
}

impl Toolchain for WebToolchain {
    type Installation = WebToolchainInstall;

    async fn check(&self) -> Result<(), ToolchainError<Self::Installation>> {
        // Check wasm-bindgen CLI.
        if which("wasm-bindgen").await.is_err() {
            return Err(ToolchainError::fixable(WebToolchainInstall));
        }

        // Check Rust target is installed by probing rustup target list.
        let output = crate::utils::run_command_output("rustup", ["target", "list", "--installed"])
            .await
            .map_err(|_| {
                ToolchainError::unfixable(
                    "Failed to run rustup",
                    "Install rustup and run: rustup target add wasm32-unknown-unknown",
                )
            })?;

        if !output.status.success() {
            return Err(ToolchainError::unfixable(
                "Failed to query installed Rust targets",
                "Run: rustup target add wasm32-unknown-unknown",
            ));
        }

        let installed = String::from_utf8_lossy(&output.stdout);
        if !installed.lines().any(|line| line.trim() == "wasm32-unknown-unknown") {
            return Err(ToolchainError::fixable(WebToolchainInstall));
        }

        Ok(())
    }
}

