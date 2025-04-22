use clap::{error::ErrorKind, CommandFactory};
use microsandbox_cli::{AnsiStyles, MicrosandboxArgs, SelfAction};
use microsandbox_core::{
    config::DEFAULT_SHELL,
    management::{
        config::{self, Component, ComponentType},
        home, menv, orchestra, sandbox, server, toolchain,
    },
    oci::Reference,
    MicrosandboxError, MicrosandboxResult,
};
use std::path::PathBuf;
use typed_path::Utf8UnixPathBuf;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

const SANDBOX_SCRIPT_SEPARATOR: char = '~';
const MICROSANDBOX_ENV_DIR: &str = ".menv";
const LOG_SUBDIR: &str = "log";

//--------------------------------------------------------------------------------------------------
// Functions: Handlers
//--------------------------------------------------------------------------------------------------

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
        std::env::set_var("RUST_LOG", format!("micro={},msb={}", level, level));
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
    imports: Vec<(String, String)>,
    exports: Vec<(String, String)>,
    scope: Option<String>,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxResult<()> {
    trio_conflict_error(build, sandbox, group, "add", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "add", Some("[NAMES]"));

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
        shell: Some(shell.unwrap_or(DEFAULT_SHELL.to_string())),
        scripts: scripts.into_iter().map(|(k, v)| (k, v.into())).collect(),
        imports: imports.into_iter().map(|(k, v)| (k, v.into())).collect(),
        exports: exports.into_iter().map(|(k, v)| (k, v.into())).collect(),
        scope,
    };

    config::add(&names, &component, path.as_deref(), config.as_deref()).await
}

pub async fn remove_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxResult<()> {
    trio_conflict_error(build, sandbox, group, "remove", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "remove", Some("[NAMES]"));
    config::remove(
        ComponentType::Sandbox,
        &names,
        path.as_deref(),
        config.as_deref(),
    )
    .await
}

pub async fn list_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxResult<()> {
    trio_conflict_error(build, sandbox, group, "list", None);
    unsupported_build_group_error(build, group, "list", None);
    let names = config::list(ComponentType::Sandbox, path.as_deref(), config.as_deref()).await?;
    for name in names {
        println!("{}", name);
    }

    Ok(())
}

pub async fn init_subcommand(
    path: Option<PathBuf>,
    path_with_flag: Option<PathBuf>,
) -> MicrosandboxResult<()> {
    let path = match (path, path_with_flag) {
        (Some(path), None) => Some(path),
        (None, Some(path)) => Some(path),
        (Some(_), Some(_)) => {
            MicrosandboxArgs::command()
                .override_usage(usage("init", Some("[PATH]"), None))
                .error(
                    ErrorKind::ArgumentConflict,
                    format!(
                        "cannot specify path both as a positional argument and with `{}` or `{}` flag",
                        "--path".placeholder(),
                        "-p".placeholder()
                    ),
                )
                .exit();
        }
        (None, None) => None,
    };

    menv::initialize(path).await?;

    Ok(())
}

pub async fn run_subcommand(
    sandbox: bool,
    build: bool,
    name: String,
    path: Option<PathBuf>,
    config: Option<String>,
    detach: bool,
    exec: Option<String>,
    args: Vec<String>,
) -> MicrosandboxResult<()> {
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

    unsupported_build_group_error(build, sandbox, "run", Some("[NAME]"));

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
    path: Option<PathBuf>,
    config: Option<String>,
    detach: bool,
    args: Vec<String>,
) -> MicrosandboxResult<()> {
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

    unsupported_build_group_error(build, sandbox, &script, Some("[NAME]"));

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
    .await
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
) -> MicrosandboxResult<()> {
    let (image, script) = parse_name_and_script(&name);
    let image = image.parse::<Reference>()?;

    if matches!((script, &exec), (Some(_), Some(_))) {
        MicrosandboxArgs::command()
            .override_usage(usage("tmp", Some("[NAME[~SCRIPT]]"), Some("<ARGS>")))
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
    .await
}

pub async fn up_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxResult<()> {
    trio_conflict_error(build, sandbox, group, "up", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "up", Some("[NAMES]"));

    orchestra::up(names, path.as_deref(), config.as_deref()).await
}

pub async fn down_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    names: Vec<String>,
    path: Option<PathBuf>,
    config: Option<String>,
) -> MicrosandboxResult<()> {
    trio_conflict_error(build, sandbox, group, "down", Some("[NAMES]"));
    unsupported_build_group_error(build, group, "down", Some("[NAMES]"));

    orchestra::down(names, path.as_deref(), config.as_deref()).await
}

/// Handle the `log` subcommand to show logs for a specific sandbox
pub async fn log_subcommand(
    sandbox: bool,
    build: bool,
    group: bool,
    name: String,
    project_dir: Option<PathBuf>,
    config_file: Option<String>,
    follow: bool,
    tail: Option<usize>,
) -> MicrosandboxResult<()> {
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

    // Load the configuration to get canonical paths
    let (_, canonical_project_dir, config_file) =
        config::load_config(project_dir.as_deref(), config_file.as_deref()).await?;

    // Construct log file path using the hierarchical structure: <project_dir>/.menv/log/<config>/<sandbox>.log
    let log_path = canonical_project_dir
        .join(MICROSANDBOX_ENV_DIR)
        .join(LOG_SUBDIR)
        .join(&config_file)
        .join(format!("{}.log", name));

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

/// Handles the clean subcommand, which removes the .menv directory from a project
pub async fn clean_subcommand(
    _sandbox: bool,
    name: Option<String>,
    global: bool,
    all: bool,
    path: Option<PathBuf>,
    config: Option<String>,
    force: bool,
) -> MicrosandboxResult<()> {
    if global || all {
        // Global cleanup - clean the microsandbox home directory
        home::clean().await?;
        tracing::info!("global microsandbox home directory cleaned");
    }

    if !global || all {
        // Local project cleanup
        if let Some(sandbox_name) = name {
            // Clean specific sandbox if sandbox name is provided
            tracing::info!("cleaning sandbox: {}", sandbox_name);
            menv::clean(path, config.as_deref(), Some(&sandbox_name), force).await?;
        } else {
            // Clean the entire .menv directory if no sandbox is specified
            tracing::info!("cleaning entire project environment");
            menv::clean(path, None, None, force).await?;
        }
    }

    Ok(())
}

pub async fn server_start_subcommand(
    port: Option<u16>,
    path: Option<PathBuf>,
    disable_default: bool,
    secure: bool,
    key: Option<String>,
    detach: bool,
) -> MicrosandboxResult<()> {
    if !secure && key.is_some() {
        MicrosandboxArgs::command()
            .override_usage(usage("server start", Some("[OPTIONS]"), None))
            .error(
                ErrorKind::InvalidValue,
                format!(
                    "cannot specify `{}` flag without `{}` flag",
                    "--key".literal(),
                    "--secure".literal(),
                ),
            )
            .exit();
    }

    server::start(port, path, disable_default, secure, key, detach).await
}

pub async fn server_keygen_subcommand(expire: Option<String>) -> MicrosandboxResult<()> {
    // Convert the string duration to chrono::Duration
    let duration = if let Some(expire_str) = expire {
        Some(parse_duration_string(&expire_str)?)
    } else {
        None
    };

    server::keygen(duration).await
}

/// Handle the self subcommand, which manages microsandbox itself
pub async fn self_subcommand(action: SelfAction) -> MicrosandboxResult<()> {
    match action {
        SelfAction::Upgrade => {
            MicrosandboxArgs::command()
                .override_usage(usage("self", Some("upgrade"), None))
                .error(
                    ErrorKind::InvalidValue,
                    "Upgrade functionality is not yet implemented",
                )
                .exit();
        }
        SelfAction::Uninstall => {
            // Clean the home directory first
            home::clean().await?;

            // Then uninstall the binaries and libraries
            toolchain::uninstall().await?;
        }
    }

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

/// Parse a duration string like "1s", "1m", "3h", "2d" into a chrono::Duration
fn parse_duration_string(duration_str: &str) -> MicrosandboxResult<chrono::Duration> {
    let duration_str = duration_str.trim();

    if duration_str.is_empty() {
        return Err(MicrosandboxError::InvalidArgument(
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
        return Err(MicrosandboxError::InvalidArgument(format!(
            "Invalid duration format: {}. Expected format like 1s, 2m, 3h, 4d, 5w, 6mo, 7y",
            duration_str
        )));
    }

    let value: i64 = value_str.parse().map_err(|_| {
        MicrosandboxError::InvalidArgument(format!(
            "Invalid numeric value in duration: {}",
            value_str
        ))
    })?;

    // Safety check for very large numbers
    if value < 0 || value > 8760 {
        // 8760 is the number of hours in a year
        return Err(MicrosandboxError::InvalidArgument(format!(
            "Duration value too large or negative: {}. Maximum allowed is 8760 hours (1 year)",
            value
        )));
    }

    match unit {
        "s" => Ok(chrono::Duration::seconds(value)),
        "m" => Ok(chrono::Duration::minutes(value)),
        "h" => Ok(chrono::Duration::hours(value)),
        "d" => Ok(chrono::Duration::days(value)),
        "w" => Ok(chrono::Duration::weeks(value)),
        "mo" => {
            // Approximate a month as 30 days
            Ok(chrono::Duration::days(value * 30))
        }
        "y" => {
            // Approximate a year as 365 days
            Ok(chrono::Duration::days(value * 365))
        }
        "" => Ok(chrono::Duration::hours(value)), // Default to hours if no unit specified
        _ => Err(MicrosandboxError::InvalidArgument(format!(
            "Invalid duration unit: {}. Expected one of: s, m, h, d, w, mo, y",
            unit
        ))),
    }
}
