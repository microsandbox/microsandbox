//! Toolchain management for Microsandbox.
//!
//! This module provides functionality for managing the Microsandbox toolchain,
//! including upgrades, and uninstallation. It handles the binaries and libraries
//! that make up the Microsandbox runtime.

use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{
    utils::path::{XDG_BIN_DIR, XDG_HOME_DIR, XDG_LIB_DIR},
    MicrosandboxResult,
};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Uninstall the Microsandbox toolchain.
///
/// This removes all installed binaries and libraries related to Microsandbox from
/// the user's system, including:
/// - Binaries in ~/.local/bin (msb, msbrun)
/// - Libraries in ~/.local/lib (libkrun, libkrunfw)
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::toolchain;
///
/// # async fn example() -> anyhow::Result<()> {
/// toolchain::uninstall().await?;
/// # Ok(())
/// # }
/// ```
pub async fn uninstall() -> MicrosandboxResult<()> {
    // Uninstall binaries
    uninstall_binaries().await?;

    // Uninstall libraries
    uninstall_libraries().await?;

    // Log success
    tracing::info!("Microsandbox toolchain has been successfully uninstalled");

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Uninstall Microsandbox binaries from the user's system.
async fn uninstall_binaries() -> MicrosandboxResult<()> {
    let bin_dir = XDG_HOME_DIR.join(XDG_BIN_DIR);

    // List of binary files to remove
    let binaries = ["msb", "msbrun"];

    for binary in binaries {
        let binary_path = bin_dir.join(binary);
        if binary_path.exists() {
            fs::remove_file(&binary_path).await?;
            tracing::info!("Removed binary: {}", binary_path.display());
        } else {
            tracing::info!("Binary not found: {}", binary_path.display());
        }
    }

    Ok(())
}

/// Uninstall Microsandbox libraries from the user's system.
async fn uninstall_libraries() -> MicrosandboxResult<()> {
    let lib_dir = XDG_HOME_DIR.join(XDG_LIB_DIR);

    // Remove base library symlinks first
    remove_if_exists(lib_dir.join("libkrun.dylib")).await?;
    remove_if_exists(lib_dir.join("libkrunfw.dylib")).await?;
    remove_if_exists(lib_dir.join("libkrun.so")).await?;
    remove_if_exists(lib_dir.join("libkrunfw.so")).await?;

    // Remove versioned libraries
    uninstall_versioned_libraries(&lib_dir, "libkrun").await?;
    uninstall_versioned_libraries(&lib_dir, "libkrunfw").await?;

    Ok(())
}

/// Remove a file if it exists, ignoring if it doesn't exist.
async fn remove_if_exists(path: PathBuf) -> MicrosandboxResult<()> {
    if path.exists() {
        fs::remove_file(&path).await?;
        tracing::info!("Removed library: {}", path.display());
    } else {
        tracing::debug!("Library not found: {}", path.display());
    }
    Ok(())
}

/// Uninstall versioned library files matching a prefix pattern.
async fn uninstall_versioned_libraries(lib_dir: &Path, lib_prefix: &str) -> MicrosandboxResult<()> {
    // Get directory entries
    let mut entries = fs::read_dir(lib_dir).await?;

    // Process each entry
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            // Check if it's one of our versioned libraries
            let is_dylib =
                filename.starts_with(&format!("{}.", lib_prefix)) && filename.ends_with(".dylib");
            let is_so = filename.starts_with(&format!("{}.", lib_prefix))
                || filename.starts_with(&format!("{}.so.", lib_prefix));

            if is_dylib || is_so {
                fs::remove_file(&path).await?;
                tracing::info!("Removed versioned library: {}", path.display());
            }
        }
    }

    Ok(())
}
