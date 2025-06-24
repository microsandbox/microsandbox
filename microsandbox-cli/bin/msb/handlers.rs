use clap::{error::ErrorKind, CommandFactory};
use microsandbox_cli::{
    AnsiStyles, MicrosandboxArgs, MicrosandboxCliError, MicrosandboxCliResult, SelfAction,
};
use microsandbox_core::{
    config::START_SCRIPT_NAME,
    management::{
        config::{self, Component, ComponentType},
        home, menv, orchestra, sandbox, toolchain,
    },
    oci::Reference,
};
use microsandbox_server::MicrosandboxServerResult;
use microsandbox_utils::{env, NAMESPACES_SUBDIR};
use std::{collections::HashMap, path::PathBuf};
use typed_path::Utf8UnixPathBuf;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const SANDBOX_SCRIPT_SEPARATOR: char = '~';

//--------------------------------------------------------------------------------------------------
// Functions: Handlers
//--------------------------------------------------------------------------------------------------

/// Set the log level based on the command line arguments
pub fn log_level(args: &MicrosandboxArgs) {
    let level = if args.trace {
        Some("trace")
    } else if args.debug {
        Some("debug")
    } else if args.info {
        Some("info")
    } else if args.warn {
        Some("warn")
    } else if args.error {
        Some("error")
    } else {
        None
    };

    // Set RUST_LOG environment variable only if a level is specified
    if let Some(level) = level {
        std::env::set_var("RUST_LOG", format!("microsandbox={},msb={}", level, level));
    }
}

pub async fn add_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    image: String,
    memory: Option<u32>,
    cpus: Option<u32>,
    volumes: Vec<String>,
    ports: Vec<String>,
    envs: Vec<String>,
    env_file: Option<Utf8UnixPathBuf>,
    depends_on: Vec<String>,
    workdir: Option<Utf8UnixPathBuf>,
    shell: Option<String>,
    scripts: Vec<(String, String)>,
    start: Option<String>,
    imports: Vec<(String, String)>,
    exports: Vec<(String, String)>,
    scope: Option<String>,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "add", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "add", Some("[NAMES]"));

    let mut scripts = scripts
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect::<HashMap<String, String>>();

    if let Some(start) = start {
        scripts.insert(START_SCRIPT_NAME.to_string(), start.into());
    }

    let component = Component::Sandbox {
        image,
        memory,
        cpus,
        volumes,
        ports,
        envs,
        env_file,
        depends_on,
        workdir,
        shell,
        scripts,
        imports: imports.into_iter().map(|(k, v)| (k, v.into())).collect(),
        exports: exports.into_iter().map(|(k, v)| (k, v.into())).collect(),
        scope,
    };

    config::add(&names, &component, path.as_deref(), config.as_deref()).await?;

    Ok(())
}

pub async fn remove_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    file: Option<PathBuf>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "remove", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "remove", Some("[NAMES]"));

    let (path, config) = parse_file_path(file);
    config::remove(
        ComponentType::Sandbox,
        &names,
        path.as_deref(),
        config.as_deref(),
    )
    .await?;

    Ok(())
}

pub async fn list_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    file: Option<PathBuf>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "list", None);
    unsupported_build_group_error(build, group, "list", None);

    let (path, config) = parse_file_path(file);
    let (config, _, _) = config::load_config(path.as_deref(), config.as_deref()).await?;

    // Use the new show_list function to display sandboxes
    menv::show_list(config.get_sandboxes());

    Ok(())
}

pub async fn init_subcommand(path: Option<PathBuf>) -> MicrosandboxCliResult<()> {
    menv::initialize(path).await?;
    Ok(())
}

pub async fn run_subcommand(
    sandbox: bool,
    build: bool,
    name: String,
    file: Option<PathBuf>,
    detach: bool,
    exec: Option<String>,
    args: Vec<String>,
) -> MicrosandboxCliResult<()> {
    if build && sandbox {
        MicrosandboxArgs::command()
            .override_usage(usage("run", Some("[NAME]"), Some("<ARGS>")))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot specify both `{}` and `{}` flags",
                    "--sandbox".literal(),
                    "--build".literal()
                ),
            )
            .exit();
    }

    unsupported_build_group_error(build, false, "run", Some("[NAME]"));

    let (sandbox, script) = parse_name_and_script(&name);
    if matches!((script, &exec), (Some(_), Some(_))) {
        MicrosandboxArgs::command()
            .override_usage(usage("run", Some("[NAME[~SCRIPT]]"), Some("<ARGS>")))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot specify both a script and an `{}` option.",
                    "--exec".placeholder()
                ),
            )
            .exit();
    }

    let (path, config) = parse_file_path(file);
    sandbox::run(
        &sandbox,
        script,
        path.as_deref(),
        config.as_deref(),
        args,
        detach,
        exec.as_deref(),
        true,
    )
    .await?;

    Ok(())
}

pub async fn script_run_subcommand(
    sandbox: bool,
    build: bool,
    name: String,
    script: String,
    file: Option<PathBuf>,
    detach: bool,
    args: Vec<String>,
) -> MicrosandboxCliResult<()> {
    if build && sandbox {
        MicrosandboxArgs::command()
            .override_usage(usage(&script, Some("[NAME]"), Some("<ARGS>")))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot specify both `{}` and `{}` flags",
                    "--sandbox".literal(),
                    "--build".literal()
                ),
            )
            .exit();
    }

    unsupported_build_group_error(build, false, &script, Some("[NAME]"));

    let (path, config) = parse_file_path(file);
    sandbox::run(
        &name,
        Some(&script),
        path.as_deref(),
        config.as_deref(),
        args,
        detach,
        None,
        true,
    )
    .await?;

    Ok(())
}

pub async fn exe_subcommand(
    name: String,
    cpus: Option<u8>,
    memory: Option<u32>,
    volumes: Vec<String>,
    ports: Vec<String>,
    envs: Vec<String>,
    workdir: Option<Utf8UnixPathBuf>,
    scope: Option<String>,
    exec: Option<String>,
    args: Vec<String>,
) -> MicrosandboxCliResult<()> {
    let (image, script) = parse_name_and_script(&name);
    let image = image.parse::<Reference>()?;

    if matches!((script, &exec), (Some(_), Some(_))) {
        MicrosandboxArgs::command()
            .override_usage(usage("exe", Some("[NAME[~SCRIPT]]"), Some("<ARGS>")))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot specify both a script and an `{}` option.",
                    "--exec".placeholder()
                ),
            )
            .exit();
    }

    sandbox::run_temp(
        &image,
        script,
        cpus,
        memory,
        volumes,
        ports,
        envs,
        workdir,
        scope,
        exec.as_deref(),
        args,
        true,
    )
    .await?;

    Ok(())
}

pub async fn up_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    file: Option<PathBuf>,
    detach: bool,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "up", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "up", Some("[NAMES]"));

    let (path, config) = parse_file_path(file);
    orchestra::up(names, path.as_deref(), config.as_deref(), detach).await?;

    Ok(())
}

pub async fn down_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    file: Option<PathBuf>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "down", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "down", Some("[NAMES]"));

    let (path, config) = parse_file_path(file);
    orchestra::down(names, path.as_deref(), config.as_deref()).await?;

    Ok(())
}

/// Handle the status subcommand to show resource usage stats for specified sandboxes
pub async fn status_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    file: Option<PathBuf>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "status", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "status", Some("[NAMES]"));

    let (path, config) = parse_file_path(file);
    orchestra::show_status(&names, path.as_deref(), config.as_deref()).await?;

    Ok(())
}

/// Handle the `log` subcommand to show logs for a specific sandbox
pub async fn log_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    name: String,
    file: Option<PathBuf>,
    follow: bool,
    tail: Option<usize>,
) -> MicrosandboxCliResult<()> {
    trio_conflict_error(build, sandbox, group, "log", Some("[NAME]"));
    unsupported_build_group_error(build, group, "log", Some("[NAME]"));

    // Check if tail command exists when follow mode is requested
    if follow {
        let tail_exists = which::which("tail").is_ok();
        if !tail_exists {
            MicrosandboxArgs::command()
                .override_usage(usage("log", Some("[NAME]"), None))
                .error(
                    ErrorKind::InvalidValue,
                    "'tail' command not found. Please install it to use the follow (-f) option.",
                )
                .exit();
        }
    }

    let (project_dir, config_file) = parse_file_path(file);
    menv::show_log(
        project_dir.as_ref(),
        config_file.as_deref(),
        &name,
        follow,
        tail,
    )
    .await?;

    Ok(())
}

/// Handles the clean subcommand, which removes the .menv directory from a project
pub async fn clean_subcommand(
    _sandbox: bool,
    name: Option<String>,
    user: bool,
    all: bool,
    file: Option<PathBuf>,
    force: bool,
) -> MicrosandboxCliResult<()> {
    if user || all {
        // User-level cleanup - clean the microsandbox home directory
        home::clean(force).await?;
        tracing::info!("user microsandbox home directory cleaned");

        // User-level cleanup - clean the user scripts (MSB-ALIAS)
        if force {
            toolchain::clean().await?;
        }

        tracing::info!("user microsandbox scripts cleaned");
    }

    if !user || all {
        // Local project cleanup
        if let Some(sandbox_name) = name {
            // Clean specific sandbox if sandbox name is provided
            tracing::info!("cleaning sandbox: {}", sandbox_name);
            let (path, config) = parse_file_path(file);
            menv::clean(path, config.as_deref(), Some(&sandbox_name), force).await?;
        } else {
            // Clean the entire .menv directory if no sandbox is specified
            tracing::info!("cleaning entire project environment");
            let (path, config) = parse_file_path(file);
            menv::clean(path, config.as_deref(), None, force).await?;
        }
    }

    Ok(())
}

pub async fn server_start_subcommand(
    host: Option<String>,
    port: Option<u16>,
    namespace_dir: Option<PathBuf>,
    dev_mode: bool,
    key: Option<String>,
    detach: bool,
    reset_key: bool,
) -> MicrosandboxCliResult<()> {
    microsandbox_server::start(key, host, port, namespace_dir, dev_mode, detach, reset_key).await?;
    Ok(())
}

pub async fn server_stop_subcommand() -> MicrosandboxServerResult<()> {
    microsandbox_server::stop().await?;
    Ok(())
}

pub async fn server_keygen_subcommand(
    expire: Option<String>,
    namespace: Option<String>,
) -> MicrosandboxCliResult<()> {
    // Convert the string duration to chrono::Duration
    let duration = if let Some(expire_str) = expire {
        Some(parse_duration_string(&expire_str)?)
    } else {
        None
    };

    // If namespace is None, use "*" to represent all namespaces
    let namespace_value = namespace.unwrap_or_else(|| "*".to_string());

    microsandbox_server::keygen(duration, namespace_value).await?;

    Ok(())
}

/// Handles the server ssh subcommand, which spawns a new SSH session into a sandbox
pub async fn server_ssh_subcommand(
    _namespace: String,
    _sandbox: bool,
    _name: String,
) -> MicrosandboxCliResult<()> {
    MicrosandboxArgs::command()
        .override_usage(usage("ssh", Some("[NAME]"), None))
        .error(
            ErrorKind::InvalidValue,
            "SSH functionality is not yet implemented",
        )
        .exit();
}

/// Handle the self subcommand, which manages microsandbox itself
pub async fn self_subcommand(action: SelfAction) -> MicrosandboxCliResult<()> {
    match action {
        SelfAction::Upgrade => {
            println!(
                "{} upgrade functionality is not yet implemented",
                "error:".error()
            );
            return Ok(());
        }
        SelfAction::Uninstall => {
            // Clean the home directory first
            home::clean(true).await?;

            // Clean user scripts
            toolchain::clean().await?;

            // Then uninstall the binaries and libraries
            toolchain::uninstall().await?;
        }
    }

    Ok(())
}

/// Handles the install subcommand for installing sandbox scripts from images
pub async fn install_subcommand(
    name: String,
    alias: Option<String>,
    cpus: Option<u8>,
    memory: Option<u32>,
    volumes: Vec<String>,
    ports: Vec<String>,
    envs: Vec<String>,
    workdir: Option<Utf8UnixPathBuf>,
    scope: Option<String>,
    exec: Option<String>,
    args: Vec<String>,
) -> MicrosandboxCliResult<()> {
    let (image, script) = parse_name_and_script(&name);
    let image = image.parse::<Reference>()?;

    if matches!((script, &exec), (Some(_), Some(_))) {
        MicrosandboxArgs::command()
            .override_usage(usage(
                "install",
                Some("[NAME[~SCRIPT]] [ALIAS]"),
                Some("<ARGS>"),
            ))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "cannot specify both a script and an `{}` option.",
                    "--exec".placeholder()
                ),
            )
            .exit();
    }

    // If extra args are provided, show a warning as they will be ignored during install
    if !args.is_empty() {
        tracing::warn!("Extra arguments will be ignored during install. They will be passed to the sandbox when the alias is used.");
    }

    home::install(
        &image,
        script,
        alias.as_deref(),
        cpus,
        memory,
        volumes,
        ports,
        envs,
        workdir,
        scope,
        exec.as_deref(),
        args,
        true,
    )
    .await?;

    Ok(())
}

/// Handles the uninstall subcommand for removing installed script aliases
pub async fn uninstall_subcommand(script: Option<String>) -> MicrosandboxCliResult<()> {
    match script {
        Some(script_name) => {
            // Uninstall the specified script
            home::uninstall(&script_name).await?;
            tracing::info!("Successfully uninstalled script: {}", script_name);
        }
        None => {
            // No script specified, print error message
            MicrosandboxArgs::command()
                .override_usage(usage("uninstall", Some("[SCRIPT]"), None))
                .error(
                    ErrorKind::InvalidValue,
                    "Please specify the name of the script to uninstall.",
                )
                .exit();
        }
    }

    Ok(())
}

pub async fn server_log_subcommand(
    _sandbox: bool,
    name: String,
    namespace: String,
    follow: bool,
    tail: Option<usize>,
) -> MicrosandboxCliResult<()> {
    // Ensure microsandbox home directory exists
    let namespace_path = env::get_microsandbox_home_path()
        .join(NAMESPACES_SUBDIR)
        .join(&namespace);

    if !namespace_path.exists() {
        return Err(MicrosandboxCliError::NotFound(format!(
            "Namespace '{}' not found",
            namespace
        )));
    }

    // Reuse the same log viewing functionality
    menv::show_log(Some(namespace_path), None, &name, follow, tail).await?;

    Ok(())
}

pub async fn server_list_subcommand(namespace: Option<String>) -> MicrosandboxCliResult<()> {
    // Get the microsandbox home path
    let microsandbox_home_path = env::get_microsandbox_home_path();
    let namespaces_path = microsandbox_home_path.join(NAMESPACES_SUBDIR);

    // Check if we need to show all namespaces or just one
    if let Some(namespace) = namespace {
        // Single namespace mode
        let namespace_path = namespaces_path.join(&namespace);

        if !namespace_path.exists() {
            return Err(MicrosandboxCliError::NotFound(format!(
                "Namespace '{}' not found",
                namespace
            )));
        }

        // Load configuration from the namespace directory
        let config_result = config::load_config(Some(namespace_path.as_path()), None).await;
        match config_result {
            Ok((config, _, _)) => {
                // Use the common show_list function to display sandboxes
                menv::show_list(config.get_sandboxes());
            }
            Err(err) => {
                return Err(MicrosandboxCliError::ConfigError(format!(
                    "Failed to load configuration from namespace '{}': {}",
                    namespace, err
                )));
            }
        }
    } else {
        // All namespaces mode - use the dedicated function
        match menv::show_list_namespaces(namespaces_path.as_path()).await {
            Ok(_) => (),
            Err(err) => {
                return Err(MicrosandboxCliError::NamespaceError(format!(
                    "Failed to list namespaces: {}",
                    err
                )));
            }
        }
    }

    Ok(())
}

pub async fn server_status_subcommand(
    _sandbox: bool,
    names: Vec<String>,
    namespace: Option<String>,
) -> MicrosandboxCliResult<()> {
    // Get the microsandbox home path
    let microsandbox_home_path = env::get_microsandbox_home_path();
    let namespaces_path = microsandbox_home_path.join(NAMESPACES_SUBDIR);

    // Check if we need to show all namespaces or just one
    if let Some(namespace) = namespace {
        // Single namespace mode
        let namespace_path = namespaces_path.join(&namespace);

        if !namespace_path.exists() {
            return Err(MicrosandboxCliError::NotFound(format!(
                "Namespace '{}' not found",
                namespace
            )));
        }

        orchestra::show_status(&names, Some(namespace_path.as_path()), None).await?;
    } else {
        // All namespaces mode
        // First check if namespaces directory exists
        if !namespaces_path.exists() {
            return Err(MicrosandboxCliError::NotFound(
                "No namespaces directory found".to_string(),
            ));
        }

        // Show status for all namespaces, passing the parent directory
        // instead of a pre-collected list of namespace directories
        match orchestra::show_status_namespaces(&names, namespaces_path.as_path()).await {
            Ok(_) => (),
            Err(err) => {
                return Err(MicrosandboxCliError::NamespaceError(format!(
                    "Failed to show namespace statuses: {}",
                    err
                )));
            }
        }
    }

    Ok(())
}

pub async fn login_subcommand() -> MicrosandboxCliResult<()> {
    println!(
        "{} login functionality is not yet implemented",
        "error:".error()
    );
    Ok(())
}

pub async fn push_subcommand(
    _image: bool,
    _image_group: bool,
    _name: String,
) -> MicrosandboxCliResult<()> {
    println!(
        "{} push functionality is not yet implemented",
        "error:".error()
    );
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Common Errors
//--------------------------------------------------------------------------------------------------

fn trio_conflict_error(
    build: bool,
    sandbox: bool,
    group: bool,
    command: &str,
    positional_placeholder: Option<&str>,
) {
    match (build, sandbox, group) {
        (true, true, _) => conflict_error("build", "sandbox", command, positional_placeholder),
        (true, _, true) => conflict_error("build", "group", command, positional_placeholder),
        (_, true, true) => conflict_error("sandbox", "group", command, positional_placeholder),
        _ => (),
    }
}

fn conflict_error(arg1: &str, arg2: &str, command: &str, positional_placeholder: Option<&str>) {
    MicrosandboxArgs::command()
        .override_usage(usage(command, positional_placeholder, None))
        .error(
            ErrorKind::ArgumentConflict,
            format!(
                "cannot specify both `{}` and `{}` flags",
                format!("--{}", arg1).literal(),
                format!("--{}", arg2).literal()
            ),
        )
        .exit();
}

fn unsupported_build_group_error(
    build: bool,
    group: bool,
    command: &str,
    positional_placeholder: Option<&str>,
) {
    if build || group {
        MicrosandboxArgs::command()
            .override_usage(usage(command, positional_placeholder, None))
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "`{}`, `{}`, `{}`, and `{}` flags are not yet supported.",
                    "--build".literal(),
                    "-b".literal(),
                    "--group".literal(),
                    "-g".literal()
                ),
            )
            .exit();
    }
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

fn usage(command: &str, positional_placeholder: Option<&str>, varargs: Option<&str>) -> String {
    let mut usage = format!(
        "{} {} {} {}",
        "msb".literal(),
        command.literal(),
        "[OPTIONS]".placeholder(),
        positional_placeholder.unwrap_or("").placeholder()
    );

    if let Some(varargs) = varargs {
        usage.push_str(&format!(
            " {} {} {}",
            "[--".literal(),
            format!("{}...", varargs).placeholder(),
            "]".literal()
        ));
    }

    usage
}

fn parse_name_and_script(name_and_script: &str) -> (&str, Option<&str>) {
    let (name, script) = match name_and_script.split_once(SANDBOX_SCRIPT_SEPARATOR) {
        Some((name, script)) => (name, Some(script)),
        None => (name_and_script, None),
    };

    (name, script)
}

/// Parse a file path into project path and config file name.
///
/// If the file path is a directory, it is treated as the project path.
/// If the file path is a file, its parent directory is treated as the project path
/// and its name is treated as the config file.
/// If the file has no parent directory (e.g., a simple filename like "config.yaml")
/// or its parent is an empty string, the current directory is used as the project path.
///
/// # Arguments
///
/// * `file` - Optional file path that could be either a directory or a file
///
/// # Returns
///
/// Tuple of (Option<PathBuf>, Option<String>) for project path and config file name
pub fn parse_file_path(file: Option<PathBuf>) -> (Option<PathBuf>, Option<String>) {
    let (project_path, config_name) = match file {
        Some(file_path) => {
            if file_path.is_dir() {
                tracing::debug!("File path is a directory: {:?}", file_path);
                // If it's a directory, it's the project path
                (Some(file_path), None)
            } else {
                // Get the config file name
                let config_name = file_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(String::from);

                // Get the parent directory
                let parent = file_path.parent();

                // Handle the cases:
                // 1. No parent (None)
                // 2. Empty parent (Some(""))
                // 3. Valid parent directory
                let project_path = match parent {
                    Some(p) if p.as_os_str().is_empty() => {
                        // Parent is empty string, use current directory
                        Some(PathBuf::from("."))
                    }
                    Some(p) => {
                        // Valid parent directory
                        Some(PathBuf::from(p))
                    }
                    None => {
                        // No parent, use current directory
                        Some(PathBuf::from("."))
                    }
                };

                (project_path, config_name)
            }
        }
        None => (None, None),
    };

    (project_path, config_name)
}

/// Parse a duration string like "1s", "1m", "3h", "2d" into a chrono::Duration
fn parse_duration_string(duration_str: &str) -> MicrosandboxCliResult<chrono::Duration> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return Err(MicrosandboxCliError::InvalidArgument(
            "Empty duration string".to_string(),
        ));
    }

    // Extract the numeric value and unit
    let (value_str, unit) = duration_str.split_at(
        duration_str
            .chars()
            .position(|c| !c.is_ascii_digit())
            .unwrap_or(duration_str.len()),
    );

    if value_str.is_empty() {
        return Err(MicrosandboxCliError::InvalidArgument(format!(
            "Invalid duration: {}. No numeric value found.",
            duration_str
        )));
    }

    let value: i64 = value_str.parse().map_err(|_| {
        MicrosandboxCliError::InvalidArgument(format!(
            "Invalid numeric value in duration: {}",
            value_str
        ))
    })?;

    match unit {
        "s" => Ok(chrono::Duration::seconds(value)),
        "m" => Ok(chrono::Duration::minutes(value)),
        "h" => Ok(chrono::Duration::hours(value)),
        "d" => Ok(chrono::Duration::days(value)),
        "w" => Ok(chrono::Duration::weeks(value)),
        "mo" => Ok(chrono::Duration::days(value * 30)), // Approximate
        "y" => Ok(chrono::Duration::days(value * 365)), // Approximate
        "" => Ok(chrono::Duration::hours(value)),       // Default to hours if no unit specified
        _ => Err(MicrosandboxCliError::InvalidArgument(format!(
            "Invalid duration unit: {}. Expected one of: s, m, h, d, w, mo, y",
            unit
        ))),
    }
}
