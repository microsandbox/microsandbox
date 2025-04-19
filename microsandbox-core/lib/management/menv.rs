//! Microsandbox environment management.
//!
//! This module handles the initialization and management of Microsandbox environments.
//! A Microsandbox environment (menv) is a directory structure that contains all the
//! necessary components for running sandboxes, including configuration files,
//! databases, and log directories.

use crate::{
    config::DEFAULT_CONFIG,
    utils::{MICROSANDBOX_CONFIG_FILENAME, RW_SUBDIR},
    MicrosandboxResult,
};

#[cfg(feature = "cli-viz")]
use crate::utils::viz;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

use crate::utils::path::{LOG_SUBDIR, MICROSANDBOX_ENV_DIR, SANDBOX_DB_FILENAME};

use super::db;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "cli-viz")]
const REMOVE_MENV_DIR_MSG: &str = "Remove .menv directory";
#[cfg(feature = "cli-viz")]
const INITIALIZE_MENV_DIR_MSG: &str = "Initialize .menv directory";
#[cfg(feature = "cli-viz")]
const CREATE_DEFAULT_CONFIG_MSG: &str = "Create default config file";
#[cfg(feature = "cli-viz")]
const UPDATE_GITIGNORE_MSG: &str = "Update .gitignore";

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Initialize a new microsandbox environment at the specified path
///
/// ## Arguments
/// * `project_dir` - Optional path where the microsandbox environment will be initialized. If None, uses current directory
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::menv;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Initialize in current directory
/// menv::initialize(None).await?;
///
/// // Initialize in specific directory
/// menv::initialize(Some("my_project".into())).await?;
/// # Ok(())
/// # }
/// ```
pub async fn initialize(project_dir: Option<PathBuf>) -> MicrosandboxResult<()> {
    // Get the target path, defaulting to current directory if none specified
    let project_dir = project_dir.unwrap_or_else(|| PathBuf::from("."));
    let menv_path = project_dir.join(MICROSANDBOX_ENV_DIR);
    #[cfg(feature = "cli-viz")]
    let menv_exists = menv_path.exists();

    #[cfg(feature = "cli-viz")]
    let initialize_menv_dir_sp = if !menv_exists {
        Some(viz::create_spinner(
            INITIALIZE_MENV_DIR_MSG.to_string(),
            None,
            None,
        ))
    } else {
        None
    };

    fs::create_dir_all(&menv_path).await?;

    // Create the required files for the microsandbox environment
    ensure_menv_files(&menv_path).await?;

    #[cfg(feature = "cli-viz")]
    if let Some(sp) = initialize_menv_dir_sp {
        sp.finish_with_message(INITIALIZE_MENV_DIR_MSG);
    }

    // Create default config file if it doesn't exist
    create_default_config(&project_dir).await?;
    tracing::info!(
        "config file at {}",
        project_dir.join(MICROSANDBOX_CONFIG_FILENAME).display()
    );

    #[cfg(feature = "cli-viz")]
    let update_gitignore_sp = viz::create_spinner(UPDATE_GITIGNORE_MSG.to_string(), None, None);

    // Update .gitignore to include .menv directory
    update_gitignore(&project_dir).await?;

    #[cfg(feature = "cli-viz")]
    update_gitignore_sp.finish_with_message(UPDATE_GITIGNORE_MSG);

    Ok(())
}

/// Clean up the microsandbox environment for a project
///
/// This removes the .menv directory and all its contents, effectively
/// cleaning up all microsandbox data for the project.
///
/// ## Arguments
/// * `project_dir` - Optional path where the microsandbox environment should be cleaned.
///                   If None, uses current directory
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::menv;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Clean in current directory
/// menv::clean(None).await?;
///
/// // Clean in specific directory
/// menv::clean(Some("my_project".into())).await?;
/// # Ok(())
/// # }
/// ```
pub async fn clean(project_dir: Option<PathBuf>) -> MicrosandboxResult<()> {
    // Get the target path, defaulting to current directory if none specified
    let project_dir = project_dir.unwrap_or_else(|| PathBuf::from("."));
    let menv_path = project_dir.join(MICROSANDBOX_ENV_DIR);

    #[cfg(feature = "cli-viz")]
    let remove_menv_dir_sp = viz::create_spinner(REMOVE_MENV_DIR_MSG.to_string(), None, None);

    // Check if .menv directory exists
    if menv_path.exists() {
        // Remove the .menv directory and all its contents
        fs::remove_dir_all(&menv_path).await?;
        tracing::info!(
            "Removed microsandbox environment at {}",
            menv_path.display()
        );
    } else {
        tracing::info!(
            "No microsandbox environment found at {}",
            menv_path.display()
        );
    }

    #[cfg(feature = "cli-viz")]
    remove_menv_dir_sp.finish_with_message(REMOVE_MENV_DIR_MSG);

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Create the required directories and files for a microsandbox environment
pub(crate) async fn ensure_menv_files(menv_path: &PathBuf) -> MicrosandboxResult<()> {
    // Create log directory if it doesn't exist
    fs::create_dir_all(menv_path.join(LOG_SUBDIR)).await?;

    // We'll create rootfs directory later when monofs is ready
    fs::create_dir_all(menv_path.join(RW_SUBDIR)).await?;

    // Get the sandbox database path
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);

    // Initialize sandbox database
    let _ = db::initialize(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;
    tracing::info!("sandbox database at {}", db_path.display());

    Ok(())
}

/// Create a default microsandbox configuration file
pub(crate) async fn create_default_config(project_dir: &Path) -> MicrosandboxResult<()> {
    let config_path = project_dir.join(MICROSANDBOX_CONFIG_FILENAME);

    // Only create if it doesn't exist
    if !config_path.exists() {
        #[cfg(feature = "cli-viz")]
        let create_default_config_sp =
            viz::create_spinner(CREATE_DEFAULT_CONFIG_MSG.to_string(), None, None);

        let mut file = fs::File::create(&config_path).await?;
        file.write_all(DEFAULT_CONFIG.as_bytes()).await?;

        #[cfg(feature = "cli-viz")]
        create_default_config_sp.finish_with_message(CREATE_DEFAULT_CONFIG_MSG);
    }

    Ok(())
}

/// Updates or creates a .gitignore file to include the .menv directory
pub(crate) async fn update_gitignore(project_dir: &Path) -> MicrosandboxResult<()> {
    let gitignore_path = project_dir.join(".gitignore");
    let canonical_entry = format!("{}/", MICROSANDBOX_ENV_DIR);
    let acceptable_entries = [MICROSANDBOX_ENV_DIR, &canonical_entry[..]];

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).await?;
        let already_present = content.lines().any(|line| {
            let trimmed = line.trim();
            acceptable_entries.contains(&trimmed)
        });

        if !already_present {
            // Ensure we start on a new line
            let prefix = if content.ends_with('\n') { "" } else { "\n" };
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&gitignore_path)
                .await?;
            file.write_all(format!("{}{}\n", prefix, canonical_entry).as_bytes())
                .await?;
        }
    } else {
        // Create new .gitignore with canonical entry (.menv/)
        fs::write(&gitignore_path, format!("{}\n", canonical_entry)).await?;
    }

    Ok(())
}
