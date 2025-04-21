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
#[cfg(feature = "cli-viz")]
use console::style;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

use crate::utils::path::{LOG_SUBDIR, MICROSANDBOX_ENV_DIR, PATCH_SUBDIR, SANDBOX_DB_FILENAME};

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
#[cfg(feature = "cli-viz")]
const CLEAN_SANDBOX_MSG: &str = "Clean sandbox";

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
        sp.finish();
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
    update_gitignore_sp.finish();

    Ok(())
}

/// Clean up the microsandbox environment for a project or a specific sandbox
///
/// This function can either:
/// 1. Remove the entire .menv directory and all its contents (when sandbox_name is None)
/// 2. Remove just a specific sandbox's data (when sandbox_name is provided)
///
/// ## Arguments
/// * `project_dir` - Optional path where the microsandbox environment should be cleaned.
///                   If None, uses current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
/// * `sandbox_name` - Optional name of the sandbox to clean. If None, cleans entire project
/// * `force` - Whether to force cleaning even if the sandbox exists in config or config file exists
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::menv;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Clean entire project in current directory
/// menv::clean(None, None, None, false).await?;
///
/// // Clean specific sandbox in current directory
/// menv::clean(None, None, Some("dev"), false).await?;
///
/// // Clean specific sandbox with custom config file, forcing cleanup
/// menv::clean(None, Some("custom.yaml"), Some("dev"), true).await?;
/// # Ok(())
/// # }
/// ```
pub async fn clean(
    project_dir: Option<PathBuf>,
    config_file: Option<&str>,
    sandbox_name: Option<&str>,
    force: bool,
) -> MicrosandboxResult<()> {
    // Get the target path, defaulting to current directory if none specified
    let project_dir = project_dir.unwrap_or_else(|| PathBuf::from("."));
    let menv_path = project_dir.join(MICROSANDBOX_ENV_DIR);

    // Try to load the configuration if the file exists
    let config_result =
        crate::management::config::load_config(Some(&project_dir), config_file).await;

    // If no sandbox name is provided, clean the entire project
    if sandbox_name.is_none() {
        #[cfg(feature = "cli-viz")]
        let remove_menv_dir_sp = viz::create_spinner(REMOVE_MENV_DIR_MSG.to_string(), None, None);

        // If the config file exists and force is false, don't clean
        if config_result.is_ok() && !force {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&remove_menv_dir_sp);

            #[cfg(feature = "cli-viz")]
            println!(
                "Configuration file exists. Use {} to clean the entire environment",
                style("--force").yellow()
            );

            tracing::info!(
                "Configuration file exists. Use --force to clean the entire environment"
            );
            return Ok(());
        }

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
        remove_menv_dir_sp.finish();

        return Ok(());
    }

    // At this point we know we're cleaning a specific sandbox
    let sandbox_name = sandbox_name.unwrap();
    let config_file = config_file.unwrap_or(MICROSANDBOX_CONFIG_FILENAME);

    #[cfg(feature = "cli-viz")]
    let clean_sandbox_sp = viz::create_spinner(
        format!("{} '{}'", CLEAN_SANDBOX_MSG, sandbox_name),
        None,
        None,
    );

    // If the sandbox exists in the config and force is false, don't clean
    if let Ok((config, _, _)) = config_result {
        if config.get_sandbox(sandbox_name).is_some() && !force {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&clean_sandbox_sp);

            #[cfg(feature = "cli-viz")]
            println!(
                "Sandbox '{}' exists in configuration. Use {} to clean it",
                sandbox_name,
                style("--force").yellow()
            );

            tracing::info!(
                "Sandbox '{}' exists in configuration. Use --force to clean it",
                sandbox_name
            );
            return Ok(());
        }
    }

    // Get sandbox namespace
    let namespaced_name = PathBuf::from(config_file).join(sandbox_name);

    // Clean up sandbox-specific directories
    let rw_path = menv_path.join(RW_SUBDIR).join(&namespaced_name);
    let patch_path = menv_path.join(PATCH_SUBDIR).join(&namespaced_name);

    // Remove sandbox directories if they exist
    if rw_path.exists() {
        fs::remove_dir_all(&rw_path).await?;
        tracing::info!("Removed sandbox RW directory at {}", rw_path.display());
    }

    if patch_path.exists() {
        fs::remove_dir_all(&patch_path).await?;
        tracing::info!(
            "Removed sandbox patch directory at {}",
            patch_path.display()
        );
    }

    // Remove log file if it exists
    let log_file = menv_path
        .join(LOG_SUBDIR)
        .join(config_file)
        .join(format!("{}.log", sandbox_name));

    if log_file.exists() {
        fs::remove_file(&log_file).await?;
        tracing::info!("Removed sandbox log file at {}", log_file.display());
    }

    // Remove sandbox from database
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    if db_path.exists() {
        let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;
        db::delete_sandbox(&pool, sandbox_name, config_file).await?;
        tracing::info!("Removed sandbox {} from database", sandbox_name);
    }

    #[cfg(feature = "cli-viz")]
    clean_sandbox_sp.finish();

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
        create_default_config_sp.finish();
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
