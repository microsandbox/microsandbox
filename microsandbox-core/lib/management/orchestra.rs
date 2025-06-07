//! Orchestra management functionality for Microsandbox.
//!
//! This module provides functionality for managing collections of sandboxes in a coordinated way,
//! similar to how container orchestration tools manage multiple containers. It handles the lifecycle
//! of multiple sandboxes defined in configuration, including starting them up, shutting them down,
//! and applying configuration changes.
//!
//! The main operations provided by this module are:
//! - `up`: Start up all sandboxes defined in configuration
//! - `down`: Gracefully shut down all running sandboxes
//! - `apply`: Reconcile running sandboxes with configuration

use crate::{
    config::{Microsandbox, START_SCRIPT_NAME},
    MicrosandboxError, MicrosandboxResult,
};

#[cfg(feature = "cli")]
use console::style;
#[cfg(feature = "cli")]
use microsandbox_utils::term;
use microsandbox_utils::{MICROSANDBOX_ENV_DIR, SANDBOX_DB_FILENAME};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
#[cfg(feature = "cli")]
use std::io::{self, IsTerminal};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
    time::{Duration, Instant},
};

use super::{config, db, menv, sandbox};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// TTL for cached directory sizes.
const DISK_SIZE_TTL: Duration = Duration::from_secs(30);

#[cfg(feature = "cli")]
const APPLY_CONFIG_MSG: &str = "Applying sandbox configuration";

#[cfg(feature = "cli")]
const START_SANDBOXES_MSG: &str = "Starting sandboxes";

#[cfg(feature = "cli")]
const STOP_SANDBOXES_MSG: &str = "Stopping sandboxes";

/// Global cache path -> (size, last_updated)
static DISK_SIZE_CACHE: Lazy<RwLock<HashMap<String, (u64, Instant)>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Information about a sandbox's resource usage
#[derive(Debug, Clone)]
pub struct SandboxStatus {
    /// The name of the sandbox
    pub name: String,

    /// Whether the sandbox is running
    pub running: bool,

    /// The PID of the supervisor process
    pub supervisor_pid: Option<u32>,

    /// The PID of the microVM process
    pub microvm_pid: Option<u32>,

    /// CPU usage percentage
    pub cpu_usage: Option<f32>,

    /// Memory usage in MiB
    pub memory_usage: Option<u64>,

    /// Disk usage of the RW layer in bytes
    pub disk_usage: Option<u64>,

    /// Rootfs paths
    pub rootfs_paths: Option<String>,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Reconciles the running sandboxes with the configuration.
///
/// This function ensures that the set of running sandboxes matches what is defined in the
/// configuration by:
/// - Starting any sandboxes that are in the config but not running
/// - Stopping any sandboxes that are running but not in the config
///
/// The function uses a file-based lock to prevent concurrent apply operations.
/// If another apply operation is in progress, this function will fail immediately.
/// The lock is automatically released when the function completes or if it fails.
///
/// ## Arguments
///
/// * `project_dir` - Optional path to the project directory. If None, defaults to current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
/// * `detach` - Whether to run sandboxes in detached mode (true) or with prefixed output (false)
///
/// ## Returns
///
/// Returns `MicrosandboxResult<()>` indicating success or failure. Possible failures include:
/// - Config file not found or invalid
/// - Database errors
/// - Sandbox start/stop failures
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Apply configuration changes from the default microsandbox.yaml
///     orchestra::apply(None, None, true).await?;
///
///     // Or specify a custom project directory and config file, in non-detached mode
///     orchestra::apply(
///         Some(&PathBuf::from("/path/to/project")),
///         Some("custom-config.yaml"),
///         false,
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn apply(
    project_dir: Option<&Path>,
    config_file: Option<&str>,
    detach: bool,
) -> MicrosandboxResult<()> {
    // Create spinner for CLI feedback
    #[cfg(feature = "cli")]
    let apply_config_sp = term::create_spinner(APPLY_CONFIG_MSG.to_string(), None, None);

    // Load the configuration first to validate it exists before acquiring lock
    let (config, canonical_project_dir, config_file) =
        match config::load_config(project_dir, config_file).await {
            Ok(result) => result,
            Err(e) => {
                #[cfg(feature = "cli")]
                term::finish_with_error(&apply_config_sp);
                return Err(e);
            }
        };

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    if let Err(e) = menv::ensure_menv_files(&menv_path).await {
        #[cfg(feature = "cli")]
        term::finish_with_error(&apply_config_sp);
        return Err(e);
    }

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = match db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await {
        Ok(pool) => pool,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&apply_config_sp);
            return Err(e);
        }
    };

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Get all running sandboxes from database
    let running_sandboxes = match db::get_running_config_sandboxes(&pool, &config_file).await {
        Ok(sandboxes) => sandboxes,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&apply_config_sp);
            return Err(e);
        }
    };
    let running_sandbox_names: Vec<String> =
        running_sandboxes.iter().map(|s| s.name.clone()).collect();

    // Collect sandboxes that need to be started
    let sandboxes_to_start: Vec<&String> = config_sandboxes
        .keys()
        .filter(|name| !running_sandbox_names.contains(*name))
        .collect();

    if sandboxes_to_start.is_empty() {
        tracing::info!("No new sandboxes to start");
    } else if detach {
        // Start sandboxes in detached mode
        for name in sandboxes_to_start {
            tracing::info!("starting sandbox: {}", name);
            if let Err(e) = sandbox::run(
                name,
                Some(START_SCRIPT_NAME),
                Some(&canonical_project_dir),
                Some(&config_file),
                vec![],
                true, // detached mode
                None,
                true,
            )
            .await
            {
                #[cfg(feature = "cli")]
                term::finish_with_error(&apply_config_sp);
                return Err(e);
            }
        }
    } else {
        // Start sandboxes in non-detached mode with multiplexed output
        let sandbox_commands = match prepare_sandbox_commands(
            &sandboxes_to_start,
            Some(START_SCRIPT_NAME),
            &canonical_project_dir,
            &config_file,
        )
        .await
        {
            Ok(commands) => commands,
            Err(e) => {
                #[cfg(feature = "cli")]
                term::finish_with_error(&apply_config_sp);
                return Err(e);
            }
        };

        if !sandbox_commands.is_empty() {
            // Finish the spinner before running commands with output
            #[cfg(feature = "cli")]
            apply_config_sp.finish();

            if let Err(e) = run_commands_with_prefixed_output(sandbox_commands).await {
                return Err(e);
            }

            // Return early as we've already finished the spinner
            return Ok(());
        }
    }

    // Stop sandboxes that are active but not in config
    for sandbox in running_sandboxes {
        if !config_sandboxes.contains_key(&sandbox.name) {
            tracing::info!("stopping sandbox: {}", sandbox.name);
            if let Err(e) = signal::kill(
                Pid::from_raw(sandbox.supervisor_pid as i32),
                Signal::SIGTERM,
            ) {
                #[cfg(feature = "cli")]
                term::finish_with_error(&apply_config_sp);
                return Err(e.into());
            }
        }
    }

    #[cfg(feature = "cli")]
    apply_config_sp.finish();

    Ok(())
}

/// Starts specified sandboxes from the configuration if they are not already running.
///
/// This function ensures that the specified sandboxes are running by:
/// - Starting any specified sandboxes that are in the config but not running
/// - Ignoring sandboxes that are not specified or already running
///
/// ## Arguments
///
/// * `sandbox_names` - List of sandbox names to start
/// * `project_dir` - Optional path to the project directory. If None, defaults to current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
/// * `detach` - Whether to run sandboxes in detached mode (true) or with prefixed output (false)
///
/// ## Returns
///
/// Returns `MicrosandboxResult<()>` indicating success or failure. Possible failures include:
/// - Config file not found or invalid
/// - Database errors
/// - Sandbox start failures
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Start specific sandboxes from the default microsandbox.yaml in detached mode
///     orchestra::up(vec!["sandbox1".to_string(), "sandbox2".to_string()], None, None, true).await?;
///
///     // Or specify a custom project directory and config file, in non-detached mode
///     orchestra::up(
///         vec!["sandbox1".to_string()],
///         Some(&PathBuf::from("/path/to/project")),
///         Some("custom-config.yaml"),
///         false,
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn up(
    sandbox_names: Vec<String>,
    project_dir: Option<&Path>,
    config_file: Option<&str>,
    detach: bool,
) -> MicrosandboxResult<()> {
    // Create spinner for CLI feedback
    #[cfg(feature = "cli")]
    let start_sandboxes_sp = term::create_spinner(START_SANDBOXES_MSG.to_string(), None, None);

    // Load the configuration first to validate it exists
    let (config, canonical_project_dir, config_file) =
        match config::load_config(project_dir, config_file).await {
            Ok(result) => result,
            Err(e) => {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_sandboxes_sp);
                return Err(e);
            }
        };

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Use all sandbox names from config if no names were specified
    let sandbox_names_to_start = if sandbox_names.is_empty() {
        // Use all sandbox names from config
        config_sandboxes.keys().cloned().collect()
    } else {
        // Validate all sandbox names exist in config before proceeding
        if let Err(e) = validate_sandbox_names(
            &sandbox_names,
            &config,
            &canonical_project_dir,
            &config_file,
        ) {
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_sandboxes_sp);
            return Err(e);
        }

        sandbox_names
    };

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    if let Err(e) = menv::ensure_menv_files(&menv_path).await {
        #[cfg(feature = "cli")]
        term::finish_with_error(&start_sandboxes_sp);
        return Err(e);
    }

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = match db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await {
        Ok(pool) => pool,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_sandboxes_sp);
            return Err(e);
        }
    };

    // Get all running sandboxes from database
    let running_sandboxes = match db::get_running_config_sandboxes(&pool, &config_file).await {
        Ok(sandboxes) => sandboxes,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_sandboxes_sp);
            return Err(e);
        }
    };
    let running_sandbox_names: Vec<String> =
        running_sandboxes.iter().map(|s| s.name.clone()).collect();

    // Collect sandboxes that need to be started
    let sandboxes_to_start: Vec<&String> = config_sandboxes
        .keys()
        .filter(|name| {
            sandbox_names_to_start.contains(*name) && !running_sandbox_names.contains(*name)
        })
        .collect();

    if sandboxes_to_start.is_empty() {
        tracing::info!("No new sandboxes to start");
        #[cfg(feature = "cli")]
        start_sandboxes_sp.finish();
        return Ok(());
    }

    if detach {
        // Start specified sandboxes in detached mode
        for name in sandboxes_to_start {
            tracing::info!("starting sandbox: {}", name);
            if let Err(e) = sandbox::run(
                name,
                None,
                Some(&canonical_project_dir),
                Some(&config_file),
                vec![],
                true, // detached mode
                None,
                true,
            )
            .await
            {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_sandboxes_sp);
                return Err(e);
            }
        }
    } else {
        // Start sandboxes in non-detached mode with multiplexed output
        let sandbox_commands = match prepare_sandbox_commands(
            &sandboxes_to_start,
            None, // Start script is None for normal up
            &canonical_project_dir,
            &config_file,
        )
        .await
        {
            Ok(commands) => commands,
            Err(e) => {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_sandboxes_sp);
                return Err(e);
            }
        };

        if !sandbox_commands.is_empty() {
            // Finish the spinner before running commands with output
            #[cfg(feature = "cli")]
            start_sandboxes_sp.finish();

            if let Err(e) = run_commands_with_prefixed_output(sandbox_commands).await {
                return Err(e);
            }

            // Return early as we've already finished the spinner
            return Ok(());
        }
    }

    #[cfg(feature = "cli")]
    start_sandboxes_sp.finish();

    Ok(())
}

/// Stops specified sandboxes that are both in the configuration and currently running.
///
/// This function ensures that the specified sandboxes are stopped by:
/// - Stopping any specified sandboxes that are both in the config and currently running
/// - Ignoring sandboxes that are not specified, not in config, or not running
///
/// ## Arguments
///
/// * `sandbox_names` - List of sandbox names to stop
/// * `project_dir` - Optional path to the project directory. If None, defaults to current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
///
/// ## Returns
///
/// Returns `MicrosandboxResult<()>` indicating success or failure. Possible failures include:
/// - Config file not found or invalid
/// - Database errors
/// - Sandbox stop failures
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Stop specific sandboxes from the default microsandbox.yaml
///     orchestra::down(vec!["sandbox1".to_string(), "sandbox2".to_string()], None, None).await?;
///
///     // Or specify a custom project directory and config file
///     orchestra::down(
///         vec!["sandbox1".to_string()],
///         Some(&PathBuf::from("/path/to/project")),
///         Some("custom-config.yaml"),
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn down(
    sandbox_names: Vec<String>,
    project_dir: Option<&Path>,
    config_file: Option<&str>,
) -> MicrosandboxResult<()> {
    // Create spinner for CLI feedback
    #[cfg(feature = "cli")]
    let stop_sandboxes_sp = term::create_spinner(STOP_SANDBOXES_MSG.to_string(), None, None);

    // Load the configuration first to validate it exists
    let (config, canonical_project_dir, config_file) =
        match config::load_config(project_dir, config_file).await {
            Ok(result) => result,
            Err(e) => {
                #[cfg(feature = "cli")]
                term::finish_with_error(&stop_sandboxes_sp);
                return Err(e);
            }
        };

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Use all sandbox names from config if no names were specified
    let sandbox_names_to_stop = if sandbox_names.is_empty() {
        // Use all sandbox names from config
        config_sandboxes.keys().cloned().collect()
    } else {
        // Validate all sandbox names exist in config before proceeding
        if let Err(e) = validate_sandbox_names(
            &sandbox_names,
            &config,
            &canonical_project_dir,
            &config_file,
        ) {
            #[cfg(feature = "cli")]
            term::finish_with_error(&stop_sandboxes_sp);
            return Err(e);
        }

        sandbox_names
    };

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    if let Err(e) = menv::ensure_menv_files(&menv_path).await {
        #[cfg(feature = "cli")]
        term::finish_with_error(&stop_sandboxes_sp);
        return Err(e);
    }

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = match db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await {
        Ok(pool) => pool,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&stop_sandboxes_sp);
            return Err(e);
        }
    };

    // Get all running sandboxes from database
    let running_sandboxes = match db::get_running_config_sandboxes(&pool, &config_file).await {
        Ok(sandboxes) => sandboxes,
        Err(e) => {
            #[cfg(feature = "cli")]
            term::finish_with_error(&stop_sandboxes_sp);
            return Err(e);
        }
    };

    // Stop specified sandboxes that are both in config and running
    for sandbox in running_sandboxes {
        if sandbox_names_to_stop.contains(&sandbox.name)
            && config_sandboxes.contains_key(&sandbox.name)
        {
            tracing::info!("stopping sandbox: {}", sandbox.name);
            if let Err(e) = signal::kill(
                Pid::from_raw(sandbox.supervisor_pid as i32),
                Signal::SIGTERM,
            ) {
                #[cfg(feature = "cli")]
                term::finish_with_error(&stop_sandboxes_sp);
                return Err(e.into());
            }
        }
    }

    #[cfg(feature = "cli")]
    stop_sandboxes_sp.finish();

    Ok(())
}

/// Gets status information about specified sandboxes.
///
/// This function retrieves the current status and resource usage of the specified sandboxes:
/// - Only reports on sandboxes that exist in the configuration
/// - For each sandbox, reports whether it's running and resource usage if it is
/// - If no sandbox names are specified (empty list), returns status for all sandboxes in the configuration
///
/// ## Arguments
///
/// * `sandbox_names` - List of sandbox names to get status for. If empty, all sandboxes in config are included.
/// * `project_dir` - Optional path to the project directory. If None, defaults to current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
///
/// ## Returns
///
/// Returns `MicrosandboxResult<Vec<SandboxStatus>>` containing status information for each sandbox.
/// Possible failures include:
/// - Config file not found or invalid
/// - Database errors
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Get status of specific sandboxes from the default microsandbox.yaml
///     let statuses = orchestra::status(
///         vec!["sandbox1".to_string(), "sandbox2".to_string()],
///         None,
///         None
///     ).await?;
///
///     // Or get status of all sandboxes from the default microsandbox.yaml
///     let all_statuses = orchestra::status(
///         vec![], // empty list means get all sandboxes
///         None,
///         None
///     ).await?;
///
///     for status in statuses {
///         println!("Sandbox: {}, Running: {}", status.name, status.running);
///         if status.running {
///             println!("  CPU: {:?}%, Memory: {:?}MiB, Disk: {:?}B",
///                 status.cpu_usage, status.memory_usage, status.disk_usage);
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn status(
    sandbox_names: Vec<String>,
    project_dir: Option<&Path>,
    config_file: Option<&str>,
) -> MicrosandboxResult<Vec<SandboxStatus>> {
    // Load the configuration first to validate it exists
    let (config, canonical_project_dir, config_file) =
        config::load_config(project_dir, config_file).await?;

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Use all sandbox names from config if no names were specified
    let sandbox_names_to_check = if sandbox_names.is_empty() {
        // Use all sandbox names from config
        config_sandboxes.keys().cloned().collect()
    } else {
        // Validate all sandbox names exist in config before proceeding
        validate_sandbox_names(
            &sandbox_names,
            &config,
            &canonical_project_dir,
            &config_file,
        )?;

        sandbox_names
    };

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    menv::ensure_menv_files(&menv_path).await?;

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;

    // Get all running sandboxes from database
    let running_sandboxes = db::get_running_config_sandboxes(&pool, &config_file).await?;

    // Create a HashMap for quick lookup of running sandboxes
    let running_sandbox_map: std::collections::HashMap<String, crate::models::Sandbox> =
        running_sandboxes
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();

    // Get status for each sandbox name to check
    let mut statuses = Vec::new();
    for sandbox_name in &sandbox_names_to_check {
        // Only process sandboxes that exist in config
        if config_sandboxes.contains_key(sandbox_name) {
            // Create a basic status with name and running status
            let mut sandbox_status = SandboxStatus {
                name: sandbox_name.clone(),
                running: running_sandbox_map.contains_key(sandbox_name),
                supervisor_pid: None,
                microvm_pid: None,
                cpu_usage: None,
                memory_usage: None,
                disk_usage: None,
                rootfs_paths: None,
            };

            // If the sandbox is running, get additional stats
            if sandbox_status.running {
                if let Some(sandbox) = running_sandbox_map.get(sandbox_name) {
                    sandbox_status.supervisor_pid = Some(sandbox.supervisor_pid);
                    sandbox_status.microvm_pid = Some(sandbox.microvm_pid);
                    sandbox_status.rootfs_paths = Some(sandbox.rootfs_paths.clone());

                    // Get CPU and memory usage for the microVM process
                    if let Ok(mut process) = psutil::process::Process::new(sandbox.microvm_pid) {
                        // CPU usage
                        if let Ok(cpu_percent) = process.cpu_percent() {
                            sandbox_status.cpu_usage = Some(cpu_percent);
                        }

                        // Memory usage
                        if let Ok(memory_info) = process.memory_info() {
                            // Convert bytes to MiB
                            sandbox_status.memory_usage = Some(memory_info.rss() / (1024 * 1024));
                        }
                    }

                    // Get disk usage of the RW layer if it's an overlayfs
                    if sandbox.rootfs_paths.starts_with("overlayfs:") {
                        let paths: Vec<&str> = sandbox.rootfs_paths.split(':').collect();
                        if paths.len() > 1 {
                            // The last path should be the RW layer
                            let rw_path = paths.last().unwrap();
                            if let Ok(metadata) = tokio::fs::metadata(rw_path).await {
                                // For a directory, we need to calculate the total size
                                if metadata.is_dir() {
                                    if let Ok(size) = get_directory_size(rw_path).await {
                                        sandbox_status.disk_usage = Some(size);
                                    }
                                } else {
                                    sandbox_status.disk_usage = Some(metadata.len());
                                }
                            }
                        }
                    } else if sandbox.rootfs_paths.starts_with("native:") {
                        // For native rootfs, get the size of the rootfs
                        let path = sandbox.rootfs_paths.strip_prefix("native:").unwrap();
                        if let Ok(metadata) = tokio::fs::metadata(path).await {
                            if metadata.is_dir() {
                                if let Ok(size) = get_directory_size(path).await {
                                    sandbox_status.disk_usage = Some(size);
                                }
                            } else {
                                sandbox_status.disk_usage = Some(metadata.len());
                            }
                        }
                    }
                }
            }

            statuses.push(sandbox_status);
        }
    }

    Ok(statuses)
}

/// Show the status of the sandboxes
///
/// ## Arguments
///
/// * `names` - The names of the sandboxes to show the status of
/// * `path` - The path to the microsandbox config file
/// * `config` - The config file to use
///
/// ## Returns
///
/// Returns `MicrosandboxResult<()>` indicating success or failure. Possible failures include:
/// - Config file not found or invalid
/// - Database errors
/// - Sandbox status retrieval failures
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     orchestra::show_status(
///         &["sandbox1".to_string(), "sandbox2".to_string()],
///         None,
///         None
///     ).await?;
///     Ok(())
/// }
/// ```
#[cfg(feature = "cli")]
pub async fn show_status(
    names: &[String],
    path: Option<&Path>,
    config: Option<&str>,
) -> MicrosandboxResult<()> {
    // Check if we're in a TTY to determine if we should do live updates
    let is_tty = io::stdin().is_terminal();
    let live_view = is_tty;
    let update_interval = std::time::Duration::from_secs(2);

    if live_view {
        println!("{}", style("Press Ctrl+C to exit live view").dim());
        // Use a loop with tokio sleep for live updates
        loop {
            // Clear the screen by printing ANSI escape code
            print!("\x1B[2J\x1B[1;1H");

            display_status(&names, path.as_deref(), config.as_deref()).await?;

            // Show update message
            println!(
                "\n{}",
                style("Updating every 2 seconds. Press Ctrl+C to exit.").dim()
            );

            // Wait for the update interval
            tokio::time::sleep(update_interval).await;
        }
    } else {
        // Just display once for non-TTY
        display_status(&names, path.as_deref(), config.as_deref()).await?;
    }

    Ok(())
}

/// Show status of sandboxes across multiple namespaces
///
/// This function displays the status of sandboxes from multiple namespaces in a consolidated view.
/// It's useful for server mode when you want to see all sandboxes across all namespaces.
///
/// ## Arguments
///
/// * `names` - List of sandbox names to show status for. If empty, shows all sandboxes.
/// * `namespaces_parent_dir` - The parent directory containing namespace directories
///
/// ## Returns
///
/// Returns `MicrosandboxResult<()>` indicating success or failure. Possible failures include:
/// - Config file not found or invalid
/// - Database errors
/// - Sandbox status retrieval failures
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use microsandbox_core::management::orchestra;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Get all namespace directories
///     let namespaces = vec![
///         PathBuf::from("/path/to/namespaces/ns1"),
///         PathBuf::from("/path/to/namespaces/ns2"),
///     ];
///
///     // Show status for all sandboxes in all namespaces
///     orchestra::show_status_namespaces(&[], namespaces.iter().map(|p| p.as_path()).collect()).await?;
///
///     // Or show status for specific sandboxes
///     orchestra::show_status_namespaces(
///         &["sandbox1".to_string(), "sandbox2".to_string()],
///         namespaces.iter().map(|p| p.as_path()).collect()
///     ).await?;
///
///     Ok(())
/// }
/// ```
#[cfg(feature = "cli")]
pub async fn show_status_namespaces(
    names: &[String],
    namespaces_parent_dir: &Path,
) -> MicrosandboxResult<()> {
    // Check if we're in a TTY to determine if we should do live updates
    let is_tty = io::stdin().is_terminal();
    let live_view = is_tty;
    let update_interval = std::time::Duration::from_secs(2);

    if live_view {
        println!("{}", style("Press Ctrl+C to exit live view").dim());
        // Use a loop with tokio sleep for live updates
        loop {
            // Clear the screen by printing ANSI escape code
            print!("\x1B[2J\x1B[1;1H");

            display_status_namespaces(names, namespaces_parent_dir).await?;

            // Show update message
            println!(
                "\n{}",
                style("Updating every 2 seconds. Press Ctrl+C to exit.").dim()
            );

            // Wait for the update interval
            tokio::time::sleep(update_interval).await;
        }
    } else {
        // Just display once for non-TTY
        display_status_namespaces(names, namespaces_parent_dir).await?;
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

// Helper function to prepare commands for multiple sandboxes
async fn prepare_sandbox_commands(
    sandbox_names: &[&String],
    script_name: Option<&str>,
    project_dir: &Path,
    config_file: &str,
) -> MicrosandboxResult<Vec<(String, tokio::process::Command)>> {
    let mut commands = Vec::new();

    for &name in sandbox_names {
        // Don't print any individual sandbox preparation logs

        let (command, _) = sandbox::prepare_run(
            name,
            script_name,
            Some(project_dir),
            Some(config_file),
            vec![],
            false, // non-detached
            None,
            true,
        )
        .await?;

        commands.push((name.clone(), command));
    }

    Ok(commands)
}

// Helper function to run multiple commands with prefixed output
async fn run_commands_with_prefixed_output(
    commands: Vec<(String, tokio::process::Command)>,
) -> MicrosandboxResult<()> {
    use console::style;
    use futures::future::join_all;
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Exit early if no commands to run
    if commands.is_empty() {
        return Ok(());
    }

    // This will hold our child process handles and associated tasks
    let mut children = Vec::new();
    let mut output_tasks = Vec::new();

    // Spawn all child processes
    for (i, (sandbox_name, mut command)) in commands.into_iter().enumerate() {
        // Configure command to pipe stdout and stderr
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Spawn the child process
        let mut child = command.spawn()?;
        let sandbox_name_clone = sandbox_name.clone();

        // Style the sandbox name based on index
        let styled_name = match i % 7 {
            0 => style(&sandbox_name).green().bold(),
            1 => style(&sandbox_name).blue().bold(),
            2 => style(&sandbox_name).red().bold(),
            3 => style(&sandbox_name).yellow().bold(),
            4 => style(&sandbox_name).magenta().bold(),
            5 => style(&sandbox_name).cyan().bold(),
            _ => style(&sandbox_name).white().bold(),
        };

        // Apply the same color to the separator bar
        let styled_separator = match i % 7 {
            0 => style("|").green(),
            1 => style("|").blue(),
            2 => style("|").red(),
            3 => style("|").yellow(),
            4 => style("|").magenta(),
            5 => style("|").cyan(),
            _ => style("|").white(),
        };

        tracing::info!(
            "{} {} started supervisor process with PID: {}",
            styled_name,
            styled_separator,
            child.id().unwrap_or(0)
        );

        // Create task to handle stdout
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let name_stdout = sandbox_name.clone();
        let color_index = i;
        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                // Style the sandbox name and separator with color, but leave the message as plain text
                let styled_name = match color_index % 7 {
                    0 => style(&name_stdout).green().bold(),
                    1 => style(&name_stdout).blue().bold(),
                    2 => style(&name_stdout).red().bold(),
                    3 => style(&name_stdout).yellow().bold(),
                    4 => style(&name_stdout).magenta().bold(),
                    5 => style(&name_stdout).cyan().bold(),
                    _ => style(&name_stdout).white().bold(),
                };

                // Apply the same color to the separator bar
                let styled_separator = match color_index % 7 {
                    0 => style("|").green(),
                    1 => style("|").blue(),
                    2 => style("|").red(),
                    3 => style("|").yellow(),
                    4 => style("|").magenta(),
                    5 => style("|").cyan(),
                    _ => style("|").white(),
                };

                println!("{} {} {}", styled_name, styled_separator, line);
            }
        });

        // Create task to handle stderr
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        let color_index = i;
        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                // Style the sandbox name and separator with color, but leave the message as plain text
                let styled_name = match color_index % 7 {
                    0 => style(&sandbox_name_clone).green().bold(),
                    1 => style(&sandbox_name_clone).blue().bold(),
                    2 => style(&sandbox_name_clone).red().bold(),
                    3 => style(&sandbox_name_clone).yellow().bold(),
                    4 => style(&sandbox_name_clone).magenta().bold(),
                    5 => style(&sandbox_name_clone).cyan().bold(),
                    _ => style(&sandbox_name_clone).white().bold(),
                };

                // Apply the same color to the separator bar
                let styled_separator = match color_index % 7 {
                    0 => style("|").green(),
                    1 => style("|").blue(),
                    2 => style("|").red(),
                    3 => style("|").yellow(),
                    4 => style("|").magenta(),
                    5 => style("|").cyan(),
                    _ => style("|").white(),
                };

                eprintln!("{} {} {}", styled_name, styled_separator, line);
            }
        });

        // Add to our collections
        children.push((sandbox_name, child));
        output_tasks.push(stdout_task);
        output_tasks.push(stderr_task);
    }

    // Create task to monitor child processes
    let monitor_task = tokio::spawn(async move {
        let mut statuses = Vec::new();

        for (name, mut child) in children {
            match child.wait().await {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(-1);
                    let success = status.success();
                    statuses.push((name, exit_code, success));
                }
                Err(_e) => {
                    #[cfg(feature = "cli")]
                    eprintln!("Error waiting for sandbox {}: {}", name, _e);
                    statuses.push((name, -1, false));
                }
            }
        }

        statuses
    });

    // Wait for all processes to complete and output tasks to finish
    let statuses = monitor_task.await?;
    join_all(output_tasks).await;

    // Check results and return error if any sandbox failed
    let failed_sandboxes: Vec<(String, i32)> = statuses
        .into_iter()
        .filter(|(_, _, success)| !success)
        .map(|(name, code, _)| (name, code))
        .collect();

    if !failed_sandboxes.is_empty() {
        // Format failure message with colored sandbox names
        let error_msg = failed_sandboxes
            .iter()
            .enumerate()
            .map(|(i, (name, code))| {
                // Apply colors directly based on index
                let styled_name = match i % 7 {
                    0 => style(name).green().bold(),
                    1 => style(name).blue().bold(),
                    2 => style(name).red().bold(),
                    3 => style(name).yellow().bold(),
                    4 => style(name).magenta().bold(),
                    5 => style(name).cyan().bold(),
                    _ => style(name).white().bold(),
                };

                // Apply the same color to the separator bar
                let styled_separator = match i % 7 {
                    0 => style("|").green(),
                    1 => style("|").blue(),
                    2 => style("|").red(),
                    3 => style("|").yellow(),
                    4 => style("|").magenta(),
                    5 => style("|").cyan(),
                    _ => style("|").white(),
                };

                format!("{} {} exit code: {}", styled_name, styled_separator, code)
            })
            .collect::<Vec<_>>()
            .join(", ");

        return Err(MicrosandboxError::SupervisorError(format!(
            "The following sandboxes failed: {}",
            error_msg
        )));
    }

    Ok(())
}

// Extracted the status display logic to a separate function
#[cfg(feature = "cli")]
async fn display_status(
    names: &[String],
    path: Option<&Path>,
    config: Option<&str>,
) -> MicrosandboxResult<()> {
    let mut statuses = status(names.to_vec(), path, config).await?;

    // Sort the statuses in a stable order to prevent entries from moving around between updates
    // Order by: running status (running first), CPU usage (highest first),
    // memory usage (highest first), disk usage (highest first), and finally name (alphabetical)
    statuses.sort_by(|a, b| {
        // First compare by running status (running sandboxes first)
        let running_order = b.running.cmp(&a.running);
        if running_order != std::cmp::Ordering::Equal {
            return running_order;
        }

        // Then compare by CPU usage (highest first)
        let cpu_order = b
            .cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal);
        if cpu_order != std::cmp::Ordering::Equal {
            return cpu_order;
        }

        // Then compare by memory usage (highest first)
        let memory_order = b
            .memory_usage
            .partial_cmp(&a.memory_usage)
            .unwrap_or(std::cmp::Ordering::Equal);
        if memory_order != std::cmp::Ordering::Equal {
            return memory_order;
        }

        // Then compare by disk usage (highest first)
        let disk_order = b
            .disk_usage
            .partial_cmp(&a.disk_usage)
            .unwrap_or(std::cmp::Ordering::Equal);
        if disk_order != std::cmp::Ordering::Equal {
            return disk_order;
        }

        // Finally sort by name (alphabetical)
        a.name.cmp(&b.name)
    });

    // Get current timestamp
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");

    // Display timestamp
    println!("{}", style(format!("Last updated: {}", timestamp)).dim());

    // Print a table-like output with status information
    println!(
        "\n{:<15} {:<10} {:<15} {:<12} {:<12} {:<12}",
        style("SANDBOX").bold(),
        style("STATUS").bold(),
        style("PIDS").bold(),
        style("CPU").bold(),
        style("MEMORY").bold(),
        style("DISK").bold()
    );

    println!("{}", style("─".repeat(80)).dim());

    for status in statuses {
        let (status_text, pids, cpu, memory, disk) = format_status_columns(&status);

        println!(
            "{:<15} {:<10} {:<15} {:<12} {:<12} {:<12}",
            style(&status.name).bold(),
            status_text,
            pids,
            cpu,
            memory,
            disk
        );
    }

    Ok(())
}

// Update display_status_namespaces to discover namespaces dynamically
#[cfg(feature = "cli")]
async fn display_status_namespaces(
    names: &[String],
    namespaces_parent_dir: &Path,
) -> MicrosandboxResult<()> {
    // Create a struct to hold status with namespace info
    #[derive(Clone)]
    struct NamespacedStatus {
        namespace: String,
        status: SandboxStatus,
    }

    // Collect statuses from all namespaces
    let mut all_statuses = Vec::new();
    let mut namespace_count = 0;

    // Check if the parent directory exists
    if !namespaces_parent_dir.exists() {
        return Err(MicrosandboxError::PathNotFound(format!(
            "Namespaces directory not found at {}",
            namespaces_parent_dir.display()
        )));
    }

    // Scan the parent directory for namespaces
    let mut entries = tokio::fs::read_dir(namespaces_parent_dir).await?;
    let mut namespace_dirs = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            namespace_dirs.push(path);
        }
    }

    // Sort namespace dirs alphabetically (initial sort to ensure deterministic behavior)
    namespace_dirs.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        a_name.cmp(b_name)
    });

    // Process each namespace directory
    for namespace_dir in &namespace_dirs {
        // Extract namespace name from path
        let namespace = namespace_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        namespace_count += 1;

        // Get statuses for this namespace
        match status(names.to_vec(), Some(namespace_dir), None).await {
            Ok(statuses) => {
                // Add namespace info to each status
                for status in statuses {
                    all_statuses.push(NamespacedStatus {
                        namespace: namespace.clone(),
                        status,
                    });
                }
            }
            Err(e) => {
                // Log error but continue with other namespaces
                tracing::warn!("Error getting status for namespace {}: {}", namespace, e);
            }
        }
    }

    // Group the statuses by namespace
    let mut statuses_by_namespace: std::collections::HashMap<String, Vec<SandboxStatus>> =
        std::collections::HashMap::new();

    for namespaced_status in all_statuses {
        statuses_by_namespace
            .entry(namespaced_status.namespace)
            .or_default()
            .push(namespaced_status.status);
    }

    // Get current timestamp
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S");

    // Display timestamp
    println!("{}", style(format!("Last updated: {}", timestamp)).dim());

    // Prepare namespaces with their activity metrics for sorting
    #[derive(Clone)]
    struct NamespaceActivity {
        name: String,
        running_count: usize,
        total_cpu: f32,
        total_memory: u64,
        statuses: Vec<SandboxStatus>,
    }

    let mut namespace_activities = Vec::new();

    // Calculate activity metrics for each namespace
    for (namespace, statuses) in statuses_by_namespace {
        if statuses.is_empty() {
            continue;
        }

        let running_count = statuses.iter().filter(|s| s.running).count();
        let total_cpu: f32 = statuses.iter().filter_map(|s| s.cpu_usage).sum();
        let total_memory: u64 = statuses.iter().filter_map(|s| s.memory_usage).sum();

        namespace_activities.push(NamespaceActivity {
            name: namespace,
            running_count,
            total_cpu,
            total_memory,
            statuses,
        });
    }

    // Sort namespaces by activity level (running count first, then resource usage)
    namespace_activities.sort_by(|a, b| {
        // First by number of running sandboxes (descending)
        let running_order = b.running_count.cmp(&a.running_count);
        if running_order != std::cmp::Ordering::Equal {
            return running_order;
        }

        // Then by total CPU usage (descending)
        let cpu_order = b
            .total_cpu
            .partial_cmp(&a.total_cpu)
            .unwrap_or(std::cmp::Ordering::Equal);
        if cpu_order != std::cmp::Ordering::Equal {
            return cpu_order;
        }

        // Then by total memory usage (descending)
        let memory_order = b.total_memory.cmp(&a.total_memory);
        if memory_order != std::cmp::Ordering::Equal {
            return memory_order;
        }

        // Finally by name (alphabetical) as a stable tiebreaker
        a.name.cmp(&b.name)
    });

    // Capture sandboxes count
    let mut total_sandboxes = 0;
    let mut is_first = true;

    // Display namespaces and their statuses with headers
    for activity in namespace_activities {
        // Add spacing between namespaces
        if !is_first {
            println!();
        }
        is_first = false;

        // Print namespace header
        print_namespace_header(&activity.name);

        // Sort the statuses in a stable order
        let mut statuses = activity.statuses;
        statuses.sort_by(|a, b| {
            // First compare by running status (running sandboxes first)
            let running_order = b.running.cmp(&a.running);
            if running_order != std::cmp::Ordering::Equal {
                return running_order;
            }

            // Then compare by CPU usage (highest first)
            let cpu_order = b
                .cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal);
            if cpu_order != std::cmp::Ordering::Equal {
                return cpu_order;
            }

            // Then compare by memory usage (highest first)
            let memory_order = b
                .memory_usage
                .partial_cmp(&a.memory_usage)
                .unwrap_or(std::cmp::Ordering::Equal);
            if memory_order != std::cmp::Ordering::Equal {
                return memory_order;
            }

            // Then compare by disk usage (highest first)
            let disk_order = b
                .disk_usage
                .partial_cmp(&a.disk_usage)
                .unwrap_or(std::cmp::Ordering::Equal);
            if disk_order != std::cmp::Ordering::Equal {
                return disk_order;
            }

            // Finally sort by name (alphabetical)
            a.name.cmp(&b.name)
        });

        total_sandboxes += statuses.len();

        // Print a table header for this namespace's sandboxes
        println!(
            "{:<15} {:<10} {:<15} {:<12} {:<12} {:<12}",
            style("SANDBOX").bold(),
            style("STATUS").bold(),
            style("PIDS").bold(),
            style("CPU").bold(),
            style("MEMORY").bold(),
            style("DISK").bold()
        );

        println!("{}", style("─".repeat(80)).dim());

        // Display the statuses for this namespace
        for status in statuses {
            let (status_text, pids, cpu, memory, disk) = format_status_columns(&status);

            println!(
                "{:<15} {:<10} {:<15} {:<12} {:<12} {:<12}",
                style(&status.name).bold(),
                status_text,
                pids,
                cpu,
                memory,
                disk
            );
        }
    }

    // Show summary with the captured counts
    println!(
        "\n{}: {}, {}: {}",
        style("Total Namespaces").dim(),
        namespace_count,
        style("Total Sandboxes").dim(),
        total_sandboxes
    );

    Ok(())
}

/// Prints a stylized header for namespace display
#[cfg(feature = "cli")]
fn print_namespace_header(namespace: &str) {
    // Create the simple title text without padding
    let title = format!("NAMESPACE: {}", namespace);

    // Print the title with white color and underline styling
    println!("\n{}", style(title).white().bold());

    // Print a separator line
    println!("{}", style("─".repeat(80)).dim());
}

/// Formats the status columns for display
#[cfg(feature = "cli")]
fn format_status_columns(
    status: &SandboxStatus,
) -> (
    console::StyledObject<String>,
    String,
    String,
    String,
    String,
) {
    let status_text = if status.running {
        style("RUNNING".to_string()).green()
    } else {
        style("STOPPED".to_string()).red()
    };

    let pids = if status.running {
        format!(
            "{}/{}",
            status.supervisor_pid.unwrap_or(0),
            status.microvm_pid.unwrap_or(0)
        )
    } else {
        "-".to_string()
    };

    let cpu = if let Some(cpu_usage) = status.cpu_usage {
        format!("{:.1}%", cpu_usage)
    } else {
        "-".to_string()
    };

    let memory = if let Some(memory_usage) = status.memory_usage {
        format!("{} MiB", memory_usage)
    } else {
        "-".to_string()
    };

    let disk = if let Some(disk_usage) = status.disk_usage {
        if disk_usage > 1024 * 1024 * 1024 {
            format!("{:.2} GB", disk_usage as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if disk_usage > 1024 * 1024 {
            format!("{:.2} MB", disk_usage as f64 / (1024.0 * 1024.0))
        } else if disk_usage > 1024 {
            format!("{:.2} KB", disk_usage as f64 / 1024.0)
        } else {
            format!("{} B", disk_usage)
        }
    } else {
        "-".to_string()
    };

    (status_text, pids, cpu, memory, disk)
}

/// Validate that all requested sandbox names exist in the configuration
fn validate_sandbox_names(
    sandbox_names: &[String],
    config: &Microsandbox,
    project_dir: &Path,
    config_file: &str,
) -> MicrosandboxResult<()> {
    let config_sandboxes = config.get_sandboxes();

    let missing_sandboxes: Vec<String> = sandbox_names
        .iter()
        .filter(|name| !config_sandboxes.contains_key(*name))
        .cloned()
        .collect();

    if !missing_sandboxes.is_empty() {
        return Err(MicrosandboxError::SandboxNotFoundInConfig(
            missing_sandboxes.join(", "),
            project_dir.join(config_file),
        ));
    }

    Ok(())
}

/// Recursively calculate the size of a directory, but cache the result for a short period so that
/// callers (status refresh every ~2 s) don't hammer the filesystem.
async fn get_directory_size(path: &str) -> MicrosandboxResult<u64> {
    // First attempt to serve from cache
    {
        let cache = DISK_SIZE_CACHE.read().unwrap();
        if let Some((size, ts)) = cache.get(path) {
            if ts.elapsed() < DISK_SIZE_TTL {
                return Ok(*size);
            }
        }
    }

    // Need to (re)compute – perform blocking walk in a separate thread so we don't block Tokio
    let path_buf = PathBuf::from(path);
    let size = tokio::task::spawn_blocking(move || -> MicrosandboxResult<u64> {
        use walkdir::WalkDir;

        let mut total: u64 = 0;
        for entry in WalkDir::new(&path_buf).follow_links(false) {
            let entry = entry?; // propagates walkdir::Error (already covered in MicrosandboxError)
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    })
    .await??; // first ? = JoinError, second ? = inner MicrosandboxError

    // Update cache
    {
        let mut cache = DISK_SIZE_CACHE.write().unwrap();
        cache.insert(path.to_string(), (size, Instant::now()));
    }

    Ok(size)
}

/// Checks if specified sandboxes from the configuration are running.
async fn _check_running(
    sandbox_names: Vec<String>,
    config: &Microsandbox,
    project_dir: &Path,
    config_file: &str,
) -> MicrosandboxResult<Vec<(String, bool)>> {
    // Ensure menv files exist
    let canonical_project_dir = project_dir.canonicalize().map_err(|e| {
        MicrosandboxError::InvalidArgument(format!(
            "Failed to canonicalize project directory: {}",
            e
        ))
    })?;
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    menv::ensure_menv_files(&menv_path).await?;

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Get all running sandboxes from database
    let running_sandboxes = db::get_running_config_sandboxes(&pool, config_file).await?;
    let running_sandbox_names: Vec<String> =
        running_sandboxes.iter().map(|s| s.name.clone()).collect();

    // Check status of specified sandboxes
    let mut statuses = Vec::new();
    for sandbox_name in sandbox_names {
        // Only check if sandbox exists in config
        if config_sandboxes.contains_key(&sandbox_name) {
            let is_running = running_sandbox_names.contains(&sandbox_name);
            statuses.push((sandbox_name, is_running));
        }
    }

    Ok(statuses)
}
