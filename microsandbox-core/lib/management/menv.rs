//! Microsandbox environment management.
//!
//! This module handles the initialization and management of Microsandbox environments.
//! A Microsandbox environment (menv) is a directory structure that contains all the
//! necessary components for running sandboxes, including configuration files,
//! databases, and log directories.

use crate::{MicrosandboxError, MicrosandboxResult};

#[cfg(feature = "cli")]
use microsandbox_utils::term;
use microsandbox_utils::{
    DEFAULT_CONFIG, LOG_SUBDIR, MICROSANDBOX_CONFIG_FILENAME, MICROSANDBOX_ENV_DIR, PATCH_SUBDIR,
    RW_SUBDIR, SANDBOX_DB_FILENAME,
};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

use super::{config, db};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "cli")]
const REMOVE_MENV_DIR_MSG: &str = "Remove .menv directory";
#[cfg(feature = "cli")]
const INITIALIZE_MENV_DIR_MSG: &str = "Initialize .menv directory";
#[cfg(feature = "cli")]
const CREATE_DEFAULT_CONFIG_MSG: &str = "Create default config file";
#[cfg(feature = "cli")]
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
    #[cfg(feature = "cli")]
    let initialize_menv_dir_sp = if !menv_path.exists() {
        Some(term::create_spinner(
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

    // Create default config file if it doesn't exist
    create_default_config(&project_dir).await?;
    tracing::info!(
        "config file at {}",
        project_dir.join(MICROSANDBOX_CONFIG_FILENAME).display()
    );

    // Update .gitignore to include .menv directory
    update_gitignore(&project_dir).await?;

    #[cfg(feature = "cli")]
    if let Some(sp) = initialize_menv_dir_sp {
        sp.finish();
    }

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
        #[cfg(feature = "cli")]
        let remove_menv_dir_sp = term::create_spinner(REMOVE_MENV_DIR_MSG.to_string(), None, None);

        // If the config file exists and force is false, don't clean
        if config_result.is_ok() && !force {
            #[cfg(feature = "cli")]
            term::finish_with_error(&remove_menv_dir_sp);

            #[cfg(feature = "cli")]
            println!(
                "Configuration file exists. Use {} to clean the entire environment",
                console::style("--force").yellow()
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

        #[cfg(feature = "cli")]
        remove_menv_dir_sp.finish();

        return Ok(());
    }

    // At this point we know we're cleaning a specific sandbox
    let sandbox_name = sandbox_name.unwrap();
    let config_file = config_file.unwrap_or(MICROSANDBOX_CONFIG_FILENAME);

    #[cfg(feature = "cli")]
    let clean_sandbox_sp = term::create_spinner(
        format!("{} '{}'", CLEAN_SANDBOX_MSG, sandbox_name),
        None,
        None,
    );

    // If the sandbox exists in the config and force is false, don't clean
    if let Ok((config, _, _)) = config_result {
        if config.get_sandbox(sandbox_name).is_some() && !force {
            #[cfg(feature = "cli")]
            term::finish_with_error(&clean_sandbox_sp);

            #[cfg(feature = "cli")]
            println!(
                "Sandbox '{}' exists in configuration. Use {} to clean it",
                sandbox_name,
                console::style("--force").yellow()
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

    #[cfg(feature = "cli")]
    clean_sandbox_sp.finish();

    Ok(())
}

/// Show logs for a sandbox
///
/// This function can show logs for a sandbox in either follow mode or regular mode.
/// In follow mode, it uses `tail -f` to continuously show new log entries.
/// In regular mode, it shows either all logs or the last N lines.
///
/// ## Arguments
/// * `project_dir` - Optional path where the microsandbox environment is located.
///                   If None, uses current directory
/// * `config_file` - Optional path to the Microsandbox config file. If None, uses default filename
/// * `sandbox_name` - Name of the sandbox to show logs for
/// * `follow` - Whether to follow the log file (tail -f mode)
/// * `tail` - Optional number of lines to show from the end
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::menv;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Show all logs for a sandbox
/// menv::show_log(None, None, "my-sandbox", false, None).await?;
///
/// // Show last 100 lines of logs
/// menv::show_log(None, None, "my-sandbox", false, Some(100)).await?;
///
/// // Follow logs in real-time
/// menv::show_log(None, None, "my-sandbox", true, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn show_log(
    project_dir: Option<impl AsRef<Path>>,
    config_file: Option<&str>,
    sandbox_name: &str,
    follow: bool,
    tail: Option<usize>,
) -> MicrosandboxResult<()> {
    // Check if tail command exists when follow mode is requested
    if follow {
        let tail_exists = which::which("tail").is_ok();
        if !tail_exists {
            return Err(MicrosandboxError::CommandNotFound(
                "tail command not found. Please install it to use the follow (-f) option."
                    .to_string(),
            ));
        }
    }

    // Load the configuration to get canonical paths
    let (_, canonical_project_dir, config_file) =
        config::load_config(project_dir.as_ref().map(|p| p.as_ref()), config_file).await?;

    // Construct log file path using the hierarchical structure: <project_dir>/.menv/log/<config>/<sandbox>.log
    let log_path = canonical_project_dir
        .join(MICROSANDBOX_ENV_DIR)
        .join(LOG_SUBDIR)
        .join(&config_file)
        .join(format!("{}.log", sandbox_name));

    // Check if log file exists
    if !log_path.exists() {
        return Err(MicrosandboxError::LogNotFound(format!(
            "Log file not found at {}",
            log_path.display()
        )));
    }

    if follow {
        // For follow mode, use tokio::process::Command to run `tail -f`
        let mut child = tokio::process::Command::new("tail")
            .arg("-f")
            .arg(&log_path)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;

        // Wait for the tail process
        let status = child.wait().await?;
        if !status.success() {
            return Err(MicrosandboxError::ProcessWaitError(format!(
                "tail process exited with status: {}",
                status
            )));
        }
    } else {
        // Read the file contents
        let contents = tokio::fs::read_to_string(&log_path).await?;

        // Split into lines
        let lines: Vec<&str> = contents.lines().collect();

        // If tail is specified, only show the last N lines
        let lines_to_print = if let Some(n) = tail {
            if n >= lines.len() {
                &lines[..]
            } else {
                &lines[lines.len() - n..]
            }
        } else {
            &lines[..]
        };

        // Print the lines
        for line in lines_to_print {
            println!("{}", line);
        }
    }

    Ok(())
}

/// Show a formatted list of sandboxes
///
/// This function can display sandbox information from any config in a standardized format.
///
/// ## Arguments
/// * `sandboxes` - A reference to a HashMap of sandbox configurations
///
/// ## Example
/// ```no_run
/// use microsandbox_core::management::menv;
/// use microsandbox_core::management::config;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Show all sandboxes for a local project
/// let (config, _, _) = config::load_config(None, None).await?;
/// menv::show_list(config.get_sandboxes());
///
/// // Show all sandboxes for a remote namespace
/// let (config, _, _) = config::load_config(Some(namespace_path), None).await?;
/// menv::show_list(config.get_sandboxes());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "cli")]
pub fn show_list<'a, I>(sandboxes: I)
where
    I: IntoIterator<Item = (&'a String, &'a crate::config::Sandbox)>,
{
    use console::style;
    use std::collections::HashMap;

    // Convert the iterator into a HashMap for easier processing
    let sandboxes: HashMap<&String, &crate::config::Sandbox> = sandboxes.into_iter().collect();

    if sandboxes.is_empty() {
        println!("No sandboxes found");
        return;
    }

    for (i, (name, sandbox)) in sandboxes.iter().enumerate() {
        // Number and name
        println!("\n{}. {}", style(i + 1).bold(), style(*name).bold());

        // Image
        println!(
            "   {}: {}",
            style("Image").dim(),
            sandbox.get_image().to_string()
        );

        // Resources
        let mut resources = Vec::new();
        if let Some(cpus) = sandbox.get_cpus() {
            resources.push(format!("{} CPUs", cpus));
        }
        if let Some(memory) = sandbox.get_memory() {
            resources.push(format!("{} MiB", memory));
        }
        if !resources.is_empty() {
            println!("   {}: {}", style("Resources").dim(), resources.join(", "));
        }

        // Network
        println!(
            "   {}: {}",
            style("Network").dim(),
            format!("{:?}", sandbox.get_scope())
        );

        // Ports
        if !sandbox.get_ports().is_empty() {
            let ports = sandbox
                .get_ports()
                .iter()
                .map(|p| format!("{}:{}", p.get_host(), p.get_guest()))
                .collect::<Vec<_>>()
                .join(", ");
            println!("   {}: {}", style("Ports").dim(), ports);
        }

        // Volumes
        if !sandbox.get_volumes().is_empty() {
            let volumes = sandbox
                .get_volumes()
                .iter()
                .map(|v| format!("{}:{}", v.get_host(), v.get_guest()))
                .collect::<Vec<_>>()
                .join(", ");
            println!("   {}: {}", style("Volumes").dim(), volumes);
        }

        // Scripts
        if !sandbox.get_scripts().is_empty() {
            let scripts = sandbox
                .get_scripts()
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            println!("   {}: {}", style("Scripts").dim(), scripts);
        }

        // Dependencies
        if !sandbox.get_depends_on().is_empty() {
            println!(
                "   {}: {}",
                style("Depends On").dim(),
                sandbox.get_depends_on().join(", ")
            );
        }
    }

    println!("\n{}: {}", style("Total").dim(), sandboxes.len());
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
        #[cfg(feature = "cli")]
        let create_default_config_sp =
            term::create_spinner(CREATE_DEFAULT_CONFIG_MSG.to_string(), None, None);

        let mut file = fs::File::create(&config_path).await?;
        file.write_all(DEFAULT_CONFIG.as_bytes()).await?;

        #[cfg(feature = "cli")]
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
