//! Server management for the Microsandbox runtime.
//!
//! This module provides functionality for managing the Microsandbox sandbox server.
//! The sandbox server is responsible for orchestrating and managing multiple
//! sandbox instances, providing a centralized control mechanism.
//!
//! Key features include:
//! - Starting the server with configurable options (port, namespace path, etc.)
//! - Stopping the server and cleaning up resources
//!
//! The server uses a PID file to track the running process and supports
//! detached mode for running as a background service.

use std::{path::PathBuf, process::Stdio};

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distr::Alphanumeric, Rng};
use tokio::{fs, process::Command};

#[cfg(feature = "cli-viz")]
use crate::utils::viz;
use crate::{
    config::DEFAULT_MSBRUN_EXE_PATH,
    server::Claims,
    utils::{self, MSBRUN_EXE_ENV_VAR, SERVER_KEY_FILE, SERVER_PID_FILE},
    MicrosandboxError, MicrosandboxResult,
};
#[cfg(feature = "cli-viz")]
use console::style;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Length of the server key
const SERVER_KEY_LENGTH: usize = 32;

/// Prefix for the API key
pub const API_KEY_PREFIX: &str = "msb_";

#[cfg(feature = "cli-viz")]
const START_SERVER_MSG: &str = "Start sandbox server";

#[cfg(feature = "cli-viz")]
const STOP_SERVER_MSG: &str = "Stop sandbox server";

#[cfg(feature = "cli-viz")]
const KEYGEN_MSG: &str = "Generate new API key";

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Start the sandbox server
pub async fn start(
    port: Option<u16>,
    path: Option<PathBuf>,
    disable_default: bool,
    secure: bool,
    key: Option<String>,
    detach: bool,
) -> MicrosandboxResult<()> {
    // Ensure microsandbox home directory exists
    let microsandbox_home_path = utils::get_microsandbox_home_path();
    fs::create_dir_all(&microsandbox_home_path).await?;

    #[cfg(feature = "cli-viz")]
    let start_server_sp = viz::create_spinner(START_SERVER_MSG.to_string(), None, None);

    // Check if PID file exists, indicating a server might be running
    let pid_file_path = microsandbox_home_path.join(SERVER_PID_FILE);
    if pid_file_path.exists() {
        // Read PID from file
        let pid_str = fs::read_to_string(&pid_file_path).await?;
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is actually running
            let process_running = unsafe { libc::kill(pid, 0) == 0 };

            if process_running {
                #[cfg(feature = "cli-viz")]
                viz::finish_with_error(&start_server_sp);

                #[cfg(feature = "cli-viz")]
                println!(
                    "A sandbox server is already running (PID: {}) - Use {} to stop it",
                    pid,
                    style("msb server stop").yellow()
                );

                tracing::info!(
                    "A sandbox server is already running (PID: {}). Use 'msb server stop' to stop it",
                    pid
                );

                return Ok(());
            } else {
                // Process not running, clean up stale PID file
                tracing::warn!("found stale PID file for process {}. Cleaning up.", pid);
                let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);
                cleanup_server_files(&pid_file_path, &key_file_path).await?;
            }
        } else {
            // Invalid PID in file, clean up
            tracing::warn!("found invalid PID in server.pid file. Cleaning up.");
            let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);
            cleanup_server_files(&pid_file_path, &key_file_path).await?;
        }
    }

    // Get the path to the msbrun executable
    let msbrun_path =
        microsandbox_utils::path::resolve_env_path(MSBRUN_EXE_ENV_VAR, &*DEFAULT_MSBRUN_EXE_PATH)
            .map_err(|e| {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&start_server_sp);
            e
        })?;

    let mut command = Command::new(msbrun_path);
    command.arg("server");

    // Store the port for later use in the success message
    let server_port = port.unwrap_or(8080); // Default port is 8080 if not specified

    if let Some(port) = port {
        command.arg("--port").arg(port.to_string());
    }

    if let Some(path) = path {
        command.arg("--path").arg(path);
    }

    if disable_default {
        command.arg("--disable-default");
    }

    // Handle secure mode and key
    if secure {
        // Create a key file with either the provided key or a generated one
        let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

        let server_key = if let Some(key) = key {
            command.arg("--key").arg(&key);
            key
        } else {
            // Generate a random key
            let generated_key = generate_random_key();
            command.arg("--key").arg(&generated_key);
            generated_key
        };

        // Write the key to file
        fs::write(&key_file_path, &server_key).await.map_err(|e| {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&start_server_sp);

            MicrosandboxError::SandboxServerError(format!(
                "failed to write key file {}: {}",
                key_file_path.display(),
                e
            ))
        })?;

        tracing::info!("created server key file at {}", key_file_path.display());
    }

    if detach {
        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        // TODO: Redirect to log file
        // Redirect the i/o to /dev/null
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
        command.stdin(Stdio::null());
    }

    // Only pass RUST_LOG if it's set in the environment
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        tracing::debug!("using existing RUST_LOG: {:?}", rust_log);
        command.env("RUST_LOG", rust_log);
    }

    let mut child = command.spawn().map_err(|e| {
        #[cfg(feature = "cli-viz")]
        viz::finish_with_error(&start_server_sp);

        MicrosandboxError::SandboxServerError(format!("failed to spawn server process: {}", e))
    })?;

    let pid = child.id().unwrap_or(0);
    tracing::info!("started sandbox server process with PID: {}", pid);

    // Create PID file
    let pid_file_path = microsandbox_home_path.join(SERVER_PID_FILE);

    // Ensure microsandbox home directory exists
    fs::create_dir_all(&microsandbox_home_path).await?;

    // Write PID to file
    fs::write(&pid_file_path, pid.to_string())
        .await
        .map_err(|e| {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&start_server_sp);

            MicrosandboxError::SandboxServerError(format!(
                "failed to write PID file {}: {}",
                pid_file_path.display(),
                e
            ))
        })?;

    #[cfg(feature = "cli-viz")]
    start_server_sp.finish();

    // Show success message with server address
    #[cfg(feature = "cli-viz")]
    println!(
        "Started sandbox server at {} (PID: {})",
        style(format!("http://localhost:{}", server_port))
            .cyan()
            .underlined(),
        pid
    );

    tracing::info!(
        "Started sandbox server at http://localhost:{} (PID: {})",
        server_port,
        pid
    );

    if detach {
        return Ok(());
    }

    let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

    // Set up signal handlers for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&start_server_sp);

            MicrosandboxError::SandboxServerError(format!(
                "failed to set up signal handlers: {}",
                e
            ))
        })?;

    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|e| {
            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&start_server_sp);

            MicrosandboxError::SandboxServerError(format!(
                "failed to set up signal handlers: {}",
                e
            ))
        })?;

    // Wait for either child process to exit or signal to be received
    tokio::select! {
        status = child.wait() => {
            if !status.as_ref().map_or(false, |s| s.success()) {
                tracing::error!(
                    "child process — sandbox server — exited with status: {:?}",
                    status
                );

                // Clean up files if process fails
                cleanup_server_files(&pid_file_path, &key_file_path).await?;

                #[cfg(feature = "cli-viz")]
                viz::finish_with_error(&start_server_sp);

                return Err(MicrosandboxError::SandboxServerError(format!(
                    "child process — sandbox server — failed with exit status: {:?}",
                    status
                )));
            }

            // Clean up both files on successful exit
            cleanup_server_files(&pid_file_path, &key_file_path).await?;
        }
        _ = sigterm.recv() => {
            tracing::info!("received SIGTERM signal");

            // Send SIGTERM to child process
            if let Err(e) = child.kill().await {
                tracing::error!("failed to send SIGTERM to child process: {}", e);
            }

            // Wait for child to exit after sending signal
            if let Err(e) = child.wait().await {
                tracing::error!("error waiting for child after SIGTERM: {}", e);
            }

            // Clean up files after signal
            cleanup_server_files(&pid_file_path, &key_file_path).await?;

            // Exit with a message
            tracing::info!("server terminated by SIGTERM signal");
        }
        _ = sigint.recv() => {
            tracing::info!("received SIGINT signal");

            // Send SIGTERM to child process
            if let Err(e) = child.kill().await {
                tracing::error!("failed to send SIGTERM to child process: {}", e);
            }

            // Wait for child to exit after sending signal
            if let Err(e) = child.wait().await {
                tracing::error!("error waiting for child after SIGINT: {}", e);
            }

            // Clean up files after signal
            cleanup_server_files(&pid_file_path, &key_file_path).await?;

            // Exit with a message
            tracing::info!("server terminated by SIGINT signal");
        }
    }

    Ok(())
}

/// Stop the sandbox server
pub async fn stop() -> MicrosandboxResult<()> {
    let microsandbox_home_path = utils::get_microsandbox_home_path();
    let pid_file_path = microsandbox_home_path.join(SERVER_PID_FILE);
    let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

    #[cfg(feature = "cli-viz")]
    let stop_server_sp = viz::create_spinner(STOP_SERVER_MSG.to_string(), None, None);

    // Check if PID file exists
    if !pid_file_path.exists() {
        #[cfg(feature = "cli-viz")]
        viz::finish_with_error(&stop_server_sp);

        return Err(MicrosandboxError::SandboxServerError(
            "server is not running (PID file not found)".to_string(),
        ));
    }

    // Read PID from file
    let pid_str = fs::read_to_string(&pid_file_path).await?;
    let pid = pid_str.trim().parse::<i32>().map_err(|_| {
        MicrosandboxError::SandboxServerError("invalid PID found in server.pid file".to_string())
    })?;

    // Send SIGTERM to the process
    unsafe {
        if libc::kill(pid, libc::SIGTERM) != 0 {
            // If process doesn't exist, clean up PID file and return error
            if std::io::Error::last_os_error().raw_os_error().unwrap() == libc::ESRCH {
                // Delete PID and key files
                cleanup_server_files(&pid_file_path, &key_file_path).await?;

                #[cfg(feature = "cli-viz")]
                viz::finish_with_error(&stop_server_sp);

                return Err(MicrosandboxError::SandboxServerError(
                    "server process not found (stale PID file removed)".to_string(),
                ));
            }

            #[cfg(feature = "cli-viz")]
            viz::finish_with_error(&stop_server_sp);

            return Err(MicrosandboxError::SandboxServerError(format!(
                "failed to stop server process (PID: {})",
                pid
            )));
        }
    }

    // Clean up both PID and key files
    cleanup_server_files(&pid_file_path, &key_file_path).await?;

    #[cfg(feature = "cli-viz")]
    stop_server_sp.finish();

    #[cfg(feature = "cli-viz")]
    println!("Stopped sandbox server (PID: {})", pid);

    tracing::info!("stopped sandbox server process (PID: {})", pid);

    Ok(())
}

/// Generate a new API key (JWT token)
pub async fn keygen(expire: Option<Duration>) -> MicrosandboxResult<()> {
    let microsandbox_home_path = utils::get_microsandbox_home_path();
    let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

    #[cfg(feature = "cli-viz")]
    let keygen_sp = viz::create_spinner(KEYGEN_MSG.to_string(), None, None);

    // Check if server key file exists
    if !key_file_path.exists() {
        #[cfg(feature = "cli-viz")]
        viz::finish_with_error(&keygen_sp);

        return Err(MicrosandboxError::SandboxServerError(
            "Server key file not found. Make sure the server is running in secure mode."
                .to_string(),
        ));
    }

    // Read the server key
    let server_key = fs::read_to_string(&key_file_path).await.map_err(|e| {
        #[cfg(feature = "cli-viz")]
        viz::finish_with_error(&keygen_sp);

        MicrosandboxError::SandboxServerError(format!(
            "Failed to read server key file {}: {}",
            key_file_path.display(),
            e
        ))
    })?;

    // Determine token expiration (default: 24 hours)
    let expire = expire.unwrap_or(Duration::hours(24));

    // Generate JWT token with the specified expiration
    let now = Utc::now();
    let expiry = now + expire;

    let claims = Claims {
        exp: expiry.timestamp() as u64,
        iat: now.timestamp() as u64,
    };

    // Encode the token
    let jwt_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(server_key.as_bytes()),
    )
    .map_err(|e| {
        #[cfg(feature = "cli-viz")]
        viz::finish_with_error(&keygen_sp);

        MicrosandboxError::SandboxServerError(format!("Failed to generate token: {}", e))
    })?;

    // Convert the JWT token to our custom API key format
    let custom_token = convert_jwt_to_api_key(&jwt_token)?;

    // Store the token information for output
    let token_str = custom_token.clone();
    let expiry_str = expiry.to_rfc3339();

    #[cfg(feature = "cli-viz")]
    keygen_sp.finish();

    tracing::info!("Generated API token with expiry {}", expiry_str);

    #[cfg(feature = "cli-viz")]
    {
        println!("Generated new API token:");
        println!("{}", style(&token_str).cyan());
        println!("Token expires: {}", expiry_str);
    }

    Ok(())
}

/// Generate a random key for JWT token signing
fn generate_random_key() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(SERVER_KEY_LENGTH)
        .map(char::from)
        .collect()
}

/// Helper function to clean up server-related files
async fn cleanup_server_files(
    pid_file_path: &PathBuf,
    key_file_path: &PathBuf,
) -> MicrosandboxResult<()> {
    // Clean up PID file
    if pid_file_path.exists() {
        fs::remove_file(pid_file_path).await?;
        tracing::info!("removed server PID file at {}", pid_file_path.display());
    }

    // Clean up key file
    if key_file_path.exists() {
        fs::remove_file(key_file_path).await?;
        tracing::info!("removed server key file at {}", key_file_path.display());
    }

    Ok(())
}

/// Convert a standard JWT token to our custom API key format
/// Takes a standard JWT token (<header>.<payload>.<signature>) and returns
/// our custom API key format (<API_KEY_PREFIX_<payload>.<signature>)
pub fn convert_jwt_to_api_key(jwt_token: &str) -> MicrosandboxResult<String> {
    let parts: Vec<&str> = jwt_token.split('.').collect();
    if parts.len() != 3 {
        return Err(MicrosandboxError::SandboxServerError(
            "Invalid JWT token format".to_string(),
        ));
    }

    // Create custom API key format: API_KEY_PREFIX.payload.signature
    Ok(format!("{}{}.{}", API_KEY_PREFIX, parts[1], parts[2]))
}
