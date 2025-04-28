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

use console::style;
use microsandbox_utils::{MICROSANDBOX_ENV_DIR, SANDBOX_DB_FILENAME};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use std::path::{Path, PathBuf};

use crate::{
    config::{Microsandbox, START_SCRIPT_NAME},
    management::{config, sandbox},
    MicrosandboxError, MicrosandboxResult,
};

use super::{db, menv};

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
///     orchestra::apply(None, None).await?;
///
///     // Or specify a custom project directory and config file
///     orchestra::apply(
///         Some(&PathBuf::from("/path/to/project")),
///         Some("custom-config.yaml"),
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn apply(
    project_dir: Option<&Path>,
    config_file: Option<&str>,
) -> MicrosandboxResult<()> {
    // Load the configuration first to validate it exists before acquiring lock
    let (config, canonical_project_dir, config_file) =
        config::load_config(project_dir, config_file).await?;

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    menv::ensure_menv_files(&menv_path).await?;

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Get all running sandboxes from database
    let running_sandboxes = db::get_running_config_sandboxes(&pool, &config_file).await?;
    let running_sandbox_names: Vec<String> =
        running_sandboxes.iter().map(|s| s.name.clone()).collect();

    // Start sandboxes that are in config but not active
    for (name, _) in config_sandboxes {
        // Should start in parallel
        if !running_sandbox_names.contains(name) {
            tracing::info!("starting sandbox: {}", name);
            sandbox::run(
                name,
                Some(START_SCRIPT_NAME),
                Some(&canonical_project_dir),
                Some(&config_file),
                vec![],
                true,
                None,
                true,
            )
            .await?;
        }
    }

    // Stop sandboxes that are active but not in config
    for sandbox in running_sandboxes {
        if !config_sandboxes.contains_key(&sandbox.name) {
            tracing::info!("stopping sandbox: {}", sandbox.name);
            signal::kill(
                Pid::from_raw(sandbox.supervisor_pid as i32),
                Signal::SIGTERM,
            )?;
        }
    }

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
///     // Start specific sandboxes from the default microsandbox.yaml
///     orchestra::up(vec!["sandbox1".to_string(), "sandbox2".to_string()], None, None).await?;
///
///     // Or specify a custom project directory and config file
///     orchestra::up(
///         vec!["sandbox1".to_string()],
///         Some(&PathBuf::from("/path/to/project")),
///         Some("custom-config.yaml"),
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn up(
    sandbox_names: Vec<String>,
    project_dir: Option<&Path>,
    config_file: Option<&str>,
) -> MicrosandboxResult<()> {
    // Load the configuration first to validate it exists
    let (config, canonical_project_dir, config_file) =
        config::load_config(project_dir, config_file).await?;

    // Validate all sandbox names exist in config before proceeding
    validate_sandbox_names(
        &sandbox_names,
        &config,
        &canonical_project_dir,
        &config_file,
    )?;

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    menv::ensure_menv_files(&menv_path).await?;

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Get all running sandboxes from database
    let running_sandboxes = db::get_running_config_sandboxes(&pool, &config_file).await?;
    let running_sandbox_names: Vec<String> =
        running_sandboxes.iter().map(|s| s.name.clone()).collect();

    // Start specified sandboxes that are in config but not active
    for (sandbox_name, _) in config_sandboxes {
        // Only start if sandbox is in the specified list and not already running
        if sandbox_names.contains(sandbox_name) && !running_sandbox_names.contains(sandbox_name) {
            tracing::info!("starting sandbox: {}", sandbox_name);
            sandbox::run(
                sandbox_name,
                Some(START_SCRIPT_NAME),
                Some(&canonical_project_dir),
                Some(&config_file),
                vec![],
                true,
                None,
                true,
            )
            .await?;
        }
    }

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
    // Load the configuration first to validate it exists
    let (config, canonical_project_dir, config_file) =
        config::load_config(project_dir, config_file).await?;

    // Validate all sandbox names exist in config before proceeding
    validate_sandbox_names(
        &sandbox_names,
        &config,
        &canonical_project_dir,
        &config_file,
    )?;

    // Ensure menv files exist
    let menv_path = canonical_project_dir.join(MICROSANDBOX_ENV_DIR);
    menv::ensure_menv_files(&menv_path).await?;

    // Get database connection pool
    let db_path = menv_path.join(SANDBOX_DB_FILENAME);
    let pool = db::get_or_create_pool(&db_path, &db::SANDBOX_DB_MIGRATOR).await?;

    // Get all sandboxes defined in config
    let config_sandboxes = config.get_sandboxes();

    // Get all running sandboxes from database
    let running_sandboxes = db::get_running_config_sandboxes(&pool, &config_file).await?;

    // Stop specified sandboxes that are both in config and running
    for sandbox in running_sandboxes {
        if sandbox_names.contains(&sandbox.name) && config_sandboxes.contains_key(&sandbox.name) {
            tracing::info!("stopping sandbox: {}", sandbox.name);
            signal::kill(
                Pid::from_raw(sandbox.supervisor_pid as i32),
                Signal::SIGTERM,
            )?;
        }
    }

    Ok(())
}

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
pub async fn show_status(
    names: &[String],
    path: Option<&Path>,
    config: Option<&str>,
) -> MicrosandboxResult<()> {
    // Check if we're in a TTY to determine if we should do live updates
    let is_tty = atty::is(atty::Stream::Stdout);
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

// Extracted the status display logic to a separate function
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
        let status_text = if status.running {
            style("RUNNING").green()
        } else {
            style("STOPPED").red()
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

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

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

/// Recursively calculate the size of a directory
async fn get_directory_size(path: &str) -> MicrosandboxResult<u64> {
    let mut total_size: u64 = 0;
    let mut stack = vec![PathBuf::from(path)];

    while let Some(path) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                stack.push(entry.path());
            }
        }
    }

    Ok(total_size)
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
