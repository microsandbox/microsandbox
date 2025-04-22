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
/// - Executables in ~/.local/bin (msb, msbrun, msr, msx, msi)
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
    // Uninstall executables
    uninstall_executables().await?;

    // Uninstall libraries
    uninstall_libraries().await?;

    // Log success
    tracing::info!("microsandbox toolchain has been successfully uninstalled");

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Uninstall Microsandbox executables from the user's system.
async fn uninstall_executables() -> MicrosandboxResult<()> {
    let bin_dir = XDG_HOME_DIR.join(XDG_BIN_DIR);

    // List of executable files to remove
    let executables = ["msb", "msbrun", "msr", "msx", "msi"];

    for executable in executables {
        let executable_path = bin_dir.join(executable);
        if executable_path.exists() {
            fs::remove_file(&executable_path).await?;
            tracing::info!("removed executable: {}", executable_path.display());
        } else {
            tracing::info!("executable not found: {}", executable_path.display());
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
        tracing::info!("removed library: {}", path.display());
    } else {
        tracing::debug!("library not found: {}", path.display());
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
                tracing::info!("removed versioned library: {}", path.display());
            }
        }
    }

    Ok(())
}
