//! Home directory management for Microsandbox.
//!
//! This module provides functionality for managing the global microsandbox home directory,
//! which contains cached images, layers, and databases. It also includes functions for
//! cleaning up the home directory and checking its existence.

use crate::{utils::env, MicrosandboxResult};

#[cfg(feature = "cli-viz")]
use crate::utils::viz;
use tokio::fs;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const REMOVE_HOME_DIR_MSG: &str = "Remove microsandbox home";

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Clean up the global microsandbox home directory
///
/// This removes the entire microsandbox home directory and all its contents, effectively
/// cleaning up all global microsandbox data including cached images, layers, and databases.
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::home;
///
/// # async fn example() -> anyhow::Result<()> {
/// home::clean().await?;
/// # Ok(())
/// # }
/// ```
pub async fn clean() -> MicrosandboxResult<()> {
    // Get the microsandbox home path from environment or default
    let home_path = env::get_microsandbox_home_path();

    #[cfg(feature = "cli-viz")]
    let remove_home_dir_sp = viz::create_spinner(REMOVE_HOME_DIR_MSG.to_string(), None, None);

    // Check if home directory exists
    if home_path.exists() {
        // Remove the home directory and all its contents
        fs::remove_dir_all(&home_path).await?;
        tracing::info!(
            "Removed microsandbox home directory at {}",
            home_path.display()
        );
    } else {
        tracing::info!(
            "No microsandbox home directory found at {}",
            home_path.display()
        );
    }

    #[cfg(feature = "cli-viz")]
    remove_home_dir_sp.finish_with_message(REMOVE_HOME_DIR_MSG);

    Ok(())
}
