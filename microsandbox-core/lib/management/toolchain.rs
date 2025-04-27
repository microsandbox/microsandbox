//! Toolchain management for Microsandbox.
//!
//! This module provides functionality for managing the Microsandbox toolchain,
//! including upgrades, and uninstallation. It handles the binaries and libraries
//! that make up the Microsandbox runtime.

use microsandbox_utils::{XDG_BIN_DIR, XDG_HOME_DIR, XDG_LIB_DIR};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::MicrosandboxResult;

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Clean up user-installed microsandbox scripts
///
/// This removes all scripts in ~/.local/bin that contain the MSB-ALIAS marker,
/// except for the core toolchain scripts (msi, msx, msr).
///
/// ## Arguments
/// * `force` - Whether to force cleaning even if script files exist
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::toolchain;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Clean all user-installed scripts
/// toolchain::clean(true).await?;
/// # Ok(())
/// # }
/// ```
pub async fn clean() -> MicrosandboxResult<()> {
    let bin_dir = XDG_HOME_DIR.join(XDG_BIN_DIR);

    // Clean all user scripts with MSB-ALIAS markers
    clean_user_scripts(&bin_dir).await?;

    Ok(())
}

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
/// // Remove core toolchain binaries and libraries
/// toolchain::uninstall().await?;
/// # Ok(())
/// # }
/// ```
pub async fn uninstall() -> MicrosandboxResult<()> {
    let bin_dir = XDG_HOME_DIR.join(XDG_BIN_DIR);

    // Uninstall executables
    uninstall_executables(&bin_dir).await?;

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
async fn uninstall_executables(bin_dir: &Path) -> MicrosandboxResult<()> {
    // List of executable files to remove
    let executables = ["msb", "msbrun", "msr", "msx", "msi", "msbserver"];

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

/// Clean all user scripts with MSB-ALIAS markers from the specified bin directory
async fn clean_user_scripts(bin_dir: &Path) -> MicrosandboxResult<()> {
    // Exit early if bin directory doesn't exist
    if !bin_dir.exists() {
        tracing::info!("bin directory not found: {}", bin_dir.display());
        return Ok(());
    }

    // Core executables that should not be removed by clean
    let protected_executables = ["msi", "msx", "msr"];

    // Get all files in the bin directory
    let mut entries = fs::read_dir(bin_dir).await?;
    let mut removed_count = 0;

    // Check each file for MSB-ALIAS marker
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and non-files
        if !path.is_file() {
            continue;
        }

        // Skip protected executables
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if protected_executables.contains(&filename) {
                tracing::debug!("skipping protected executable: {}", filename);
                continue;
            }
        }

        // Read file content and check for MSB-ALIAS marker
        if let Ok(content) = fs::read_to_string(&path).await {
            if content.contains("# MSB-ALIAS:") {
                // This is a microsandbox alias script, remove it
                fs::remove_file(&path).await?;
                tracing::info!("removed user script: {}", path.display());
                removed_count += 1;
            }
        }
    }

    tracing::info!(
        "removed {} user scripts with MSB-ALIAS markers",
        removed_count
    );

    Ok(())
}
