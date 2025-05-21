//! Management module for the microsandbox server.
//!
//! This module provides functionality for managing the microsandbox server lifecycle, including:
//! - Starting and stopping the server
//! - API key generation and management
//! - Process management and signal handling
//! - Server configuration and state management
//!
//! The module implements core server management features such as:
//! - Secure server key generation and storage
//! - PID file management for process tracking
//! - Signal handling for graceful shutdown
//! - JWT-based API key generation and formatting

use std::{path::PathBuf, process::Stdio};

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
#[cfg(feature = "cli")]
use microsandbox_utils::term;
use microsandbox_utils::{
    env, DEFAULT_MSBSERVER_EXE_PATH, MSBSERVER_EXE_ENV_VAR, NAMESPACES_SUBDIR, SERVER_KEY_FILE,
    SERVER_PID_FILE,
};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use tokio::{fs, process::Command};

use crate::{MicrosandboxServerError, MicrosandboxServerResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Prefix for the API key
pub const API_KEY_PREFIX: &str = "msb_";

/// Length of the server key
const SERVER_KEY_LENGTH: usize = 32;

#[cfg(feature = "cli")]
const START_SERVER_MSG: &str = "Start sandbox server";

#[cfg(feature = "cli")]
const STOP_SERVER_MSG: &str = "Stop sandbox server";

#[cfg(feature = "cli")]
const KEYGEN_MSG: &str = "Generate new API key";

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Claims for the JWT token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Expiration time
    pub exp: u64,

    /// Issued at time
    pub iat: u64,

    /// Namespace
    pub namespace: String,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Start the sandbox server
pub async fn start(
    key: Option<String>,
    port: Option<u16>,
    namespace_dir: Option<PathBuf>,
    dev_mode: bool,
    detach: bool,
    reset_key: bool,
) -> MicrosandboxServerResult<()> {
    // Ensure microsandbox home directory exists
    let microsandbox_home_path = env::get_microsandbox_home_path();
    fs::create_dir_all(&microsandbox_home_path).await?;

    // Ensure namespace directory exists
    let namespace_path = microsandbox_home_path.join(NAMESPACES_SUBDIR);
    fs::create_dir_all(&namespace_path).await?;

    #[cfg(feature = "cli")]
    let start_server_sp = term::create_spinner(START_SERVER_MSG.to_string(), None, None);

    // Check if PID file exists, indicating a server might be running
    let pid_file_path = microsandbox_home_path.join(SERVER_PID_FILE);
    if pid_file_path.exists() {
        // Read PID from file
        let pid_str = fs::read_to_string(&pid_file_path).await?;
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is actually running
            let process_running = unsafe { libc::kill(pid, 0) == 0 };

            if process_running {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_server_sp);

                #[cfg(feature = "cli")]
                println!(
                    "A sandbox server is already running (PID: {}) - Use {} to stop it",
                    pid,
                    console::style("msb server stop").yellow()
                );

                tracing::info!(
                    "A sandbox server is already running (PID: {}). Use 'msb server stop' to stop it",
                    pid
                );

                return Ok(());
            } else {
                // Process not running, clean up stale PID file
                tracing::warn!("found stale PID file for process {}. Cleaning up.", pid);
                clean(&pid_file_path).await?;
            }
        } else {
            // Invalid PID in file, clean up
            tracing::warn!("found invalid PID in server.pid file. Cleaning up.");
            clean(&pid_file_path).await?;
        }
    }

    // Get the path to the msbrun executable
    let msbserver_path = microsandbox_utils::path::resolve_env_path(
        MSBSERVER_EXE_ENV_VAR,
        &*DEFAULT_MSBSERVER_EXE_PATH,
    )
    .map_err(|e| {
        #[cfg(feature = "cli")]
        term::finish_with_error(&start_server_sp);
        e
    })?;

    let mut command = Command::new(msbserver_path);

    if dev_mode {
        command.arg("--dev");
    }

    if let Some(port) = port {
        command.arg("--port").arg(port.to_string());
    }

    if let Some(namespace_dir) = namespace_dir {
        command.arg("--path").arg(namespace_dir);
    }

    // Handle secure non-dev mode
    if !dev_mode {
        // Create a key file with either the provided key or a generated one
        let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

        // Store if a key was provided before consuming the option
        let key_provided = key.is_some();

        let server_key = if let Some(key) = key {
            // Use the provided key
            command.arg("--key").arg(&key);
            key
        } else if key_file_path.exists() && !reset_key {
            // Use existing key file if it exists and reset_key is not set
            let existing_key = fs::read_to_string(&key_file_path).await.map_err(|e| {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_server_sp);

                MicrosandboxServerError::StartError(format!(
                    "failed to read existing key file {}: {}",
                    key_file_path.display(),
                    e
                ))
            })?;
            command.arg("--key").arg(&existing_key);
            existing_key
        } else {
            // Generate a new random key
            let generated_key = generate_random_key();
            command.arg("--key").arg(&generated_key);
            generated_key
        };

        // Write the key to file (if it's a new key or we're resetting)
        if !key_file_path.exists() || key_provided || reset_key {
            fs::write(&key_file_path, &server_key).await.map_err(|e| {
                #[cfg(feature = "cli")]
                term::finish_with_error(&start_server_sp);

                MicrosandboxServerError::StartError(format!(
                    "failed to write key file {}: {}",
                    key_file_path.display(),
                    e
                ))
            })?;

            tracing::info!("created server key file at {}", key_file_path.display());
        }
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
        #[cfg(feature = "cli")]
        term::finish_with_error(&start_server_sp);

        MicrosandboxServerError::StartError(format!("failed to spawn server process: {}", e))
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
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_server_sp);

            MicrosandboxServerError::StartError(format!(
                "failed to write PID file {}: {}",
                pid_file_path.display(),
                e
            ))
        })?;

    #[cfg(feature = "cli")]
    start_server_sp.finish();

    if detach {
        return Ok(());
    }

    // Set up signal handlers for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| {
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_server_sp);

            MicrosandboxServerError::StartError(format!("failed to set up signal handlers: {}", e))
        })?;

    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|e| {
            #[cfg(feature = "cli")]
            term::finish_with_error(&start_server_sp);

            MicrosandboxServerError::StartError(format!("failed to set up signal handlers: {}", e))
        })?;

    // Wait for either child process to exit or signal to be received
    tokio::select! {
        status = child.wait() => {
            if !status.as_ref().map_or(false, |s| s.success()) {
                tracing::error!(
                    "child process — sandbox server — exited with status: {:?}",
                    status
                );

                // Clean up PID file if process fails
                clean(&pid_file_path).await?;

                #[cfg(feature = "cli")]
                term::finish_with_error(&start_server_sp);

                return Err(MicrosandboxServerError::StartError(format!(
                    "child process — sandbox server — failed with exit status: {:?}",
                    status
                )));
            }

            // Clean up PID file on successful exit
            clean(&pid_file_path).await?;
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

            // Clean up PID file after signal
            clean(&pid_file_path).await?;

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

            // Clean up PID file after signal
            clean(&pid_file_path).await?;

            // Exit with a message
            tracing::info!("server terminated by SIGINT signal");
        }
    }

    Ok(())
}

/// Stop the sandbox server
pub async fn stop() -> MicrosandboxServerResult<()> {
    let microsandbox_home_path = env::get_microsandbox_home_path();
    let pid_file_path = microsandbox_home_path.join(SERVER_PID_FILE);

    #[cfg(feature = "cli")]
    let stop_server_sp = term::create_spinner(STOP_SERVER_MSG.to_string(), None, None);

    // Check if PID file exists
    if !pid_file_path.exists() {
        #[cfg(feature = "cli")]
        term::finish_with_error(&stop_server_sp);

        return Err(MicrosandboxServerError::StopError(
            "server is not running (PID file not found)".to_string(),
        ));
    }

    // Read PID from file
    let pid_str = fs::read_to_string(&pid_file_path).await?;
    let pid = pid_str.trim().parse::<i32>().map_err(|_| {
        MicrosandboxServerError::StopError("invalid PID found in server.pid file".to_string())
    })?;

    // Send SIGTERM to the process
    unsafe {
        if libc::kill(pid, libc::SIGTERM) != 0 {
            // If process doesn't exist, clean up PID file and return error
            if std::io::Error::last_os_error().raw_os_error().unwrap() == libc::ESRCH {
                // Delete only the PID file
                clean(&pid_file_path).await?;

                #[cfg(feature = "cli")]
                term::finish_with_error(&stop_server_sp);

                return Err(MicrosandboxServerError::StopError(
                    "server process not found (stale PID file removed)".to_string(),
                ));
            }

            #[cfg(feature = "cli")]
            term::finish_with_error(&stop_server_sp);

            return Err(MicrosandboxServerError::StopError(format!(
                "failed to stop server process (PID: {})",
                pid
            )));
        }
    }

    // Clean up just the PID file
    clean(&pid_file_path).await?;

    #[cfg(feature = "cli")]
    stop_server_sp.finish();

    tracing::info!("stopped sandbox server process (PID: {})", pid);

    Ok(())
}

/// Generate a new API key (JWT token)
pub async fn keygen(
    expire: Option<Duration>,
    namespace: String,
) -> MicrosandboxServerResult<String> {
    let microsandbox_home_path = env::get_microsandbox_home_path();
    let key_file_path = microsandbox_home_path.join(SERVER_KEY_FILE);

    #[cfg(feature = "cli")]
    let keygen_sp = term::create_spinner(KEYGEN_MSG.to_string(), None, None);

    // Check if server key file exists
    if !key_file_path.exists() {
        #[cfg(feature = "cli")]
        term::finish_with_error(&keygen_sp);

        return Err(MicrosandboxServerError::KeyGenError(
            "Server key file not found. Make sure the server is running in secure mode."
                .to_string(),
        ));
    }

    // Read the server key
    let server_key = fs::read_to_string(&key_file_path).await.map_err(|e| {
        #[cfg(feature = "cli")]
        term::finish_with_error(&keygen_sp);

        MicrosandboxServerError::KeyGenError(format!(
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
        namespace,
    };

    // Encode the token
    let jwt_token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(server_key.as_bytes()),
    )
    .map_err(|e| {
        #[cfg(feature = "cli")]
        term::finish_with_error(&keygen_sp);

        MicrosandboxServerError::KeyGenError(format!("Failed to generate token: {}", e))
    })?;

    // Convert the JWT token to our custom API key format
    let custom_token = convert_jwt_to_api_key(&jwt_token)?;

    // Store the token information for output
    let token_str = custom_token.clone();
    let expiry_str = expiry.to_rfc3339();

    #[cfg(feature = "cli")]
    keygen_sp.finish();

    tracing::info!(
        "Generated API token with namespace {} and expiry {}",
        claims.namespace,
        expiry_str
    );

    #[cfg(feature = "cli")]
    {
        println!("Token: {}", console::style(&token_str).cyan());
        println!("Token expires: {}", console::style(&expiry_str).cyan());
        println!("Namespace: {}", console::style(&claims.namespace).cyan());
    }

    Ok(token_str)
}

/// Clean up the PID file
pub async fn clean(pid_file_path: &PathBuf) -> MicrosandboxServerResult<()> {
    // Clean up PID file
    if pid_file_path.exists() {
        fs::remove_file(pid_file_path).await?;
        tracing::info!("removed server PID file at {}", pid_file_path.display());
    }

    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Generate a random key for JWT token signing
fn generate_random_key() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(SERVER_KEY_LENGTH)
        .map(char::from)
        .collect()
}

/// Convert a standard JWT token to our custom API key format
/// Takes a standard JWT token (<header>.<payload>.<signature>) and returns
/// our custom API key format (<API_KEY_PREFIX><full_jwt_token>)
pub fn convert_jwt_to_api_key(jwt_token: &str) -> MicrosandboxServerResult<String> {
    // Create custom API key format: API_KEY_PREFIX + full JWT token
    Ok(format!("{}{}", API_KEY_PREFIX, jwt_token))
}
