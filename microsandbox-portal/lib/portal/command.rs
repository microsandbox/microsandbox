//! Command execution for the microsandbox portal.
//!
//! This module provides functionality for executing system commands in a sandboxed environment.
//! It handles:
//! - Spawning and managing command processes using tokio::process::Command
//! - Streaming stdout and stderr output in real-time
//! - Managing command lifecycle and termination
//! - Providing a secure execution environment for system commands
//!
//! # Architecture
//!
//! The architecture follows a similar pattern to the code evaluation system:
//!
//! 1. A command handler receives execution requests
//! 2. Commands are executed in a controlled environment
//! 3. Output is streamed back to the caller
//!
//! # Security Considerations
//!
//! All commands are executed with carefully controlled permissions and environment
//! variables to maintain system security. Command execution is isolated to prevent
//! damage to the host system.

use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    time::{sleep, Duration},
};
use uuid::Uuid;

use crate::portal::repl::types::Stream;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Error types that can occur during command operations
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// Error spawning the command
    #[error("Failed to spawn command: {0}")]
    SpawnError(String),

    /// Error during command execution
    #[error("Command execution error: {0}")]
    ExecutionError(String),

    /// Timeout during execution
    #[error("Command timeout after {0} seconds")]
    Timeout(u64),

    /// Command environment unavailable
    #[error("Command environment unavailable: {0}")]
    Unavailable(String),
}

/// A single line of output from command execution
#[derive(Debug, Clone)]
pub struct CommandLine {
    /// Stream type (stdout/stderr)
    pub stream: Stream,

    /// Line content
    pub text: String,
}

/// Response from a command execution
#[derive(Debug)]
pub enum CommandResp {
    /// A line of output
    Line {
        /// Unique identifier for the execution
        id: String,

        /// Stream type (stdout/stderr)
        stream: Stream,

        /// Line content
        text: String,
    },

    /// Execution completed successfully
    Done {
        /// Unique identifier for the execution
        id: String,

        /// Exit code from the command
        exit_code: i32,
    },

    /// Execution resulted in an error
    Error {
        /// Unique identifier for the execution
        id: String,

        /// Error message
        message: String,
    },
}

/// Command executor handle
///
/// This is the primary interface that clients use to execute system commands
/// in a controlled environment.
#[derive(Clone)]
pub struct CommandHandle {
    cmd_sender: Sender<CommandRequest>,
}

// Implement Debug for CommandHandle
impl fmt::Debug for CommandHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandHandle")
            .field("cmd_sender", &"<SENDER>")
            .finish()
    }
}

/// Request for command execution
struct CommandRequest {
    id: String,
    command: String,
    args: Vec<String>,
    resp_tx: Sender<CommandResp>,
    done_tx: oneshot::Sender<Result<i32, CommandError>>,
    timeout: Option<u64>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl CommandHandle {
    /// Creates a new command handle
    pub fn new() -> Self {
        let (cmd_sender, mut cmd_receiver) = mpsc::channel::<CommandRequest>(100);

        // Start the command executor in a background task
        tokio::spawn(async move {
            while let Some(req) = cmd_receiver.recv().await {
                let CommandRequest {
                    id,
                    command,
                    args,
                    resp_tx,
                    done_tx,
                    timeout,
                } = req;

                // Execute the command in a separate task
                tokio::spawn(async move {
                    let result = execute_command(id, command, args, resp_tx.clone(), timeout).await;
                    let _ = done_tx.send(result);
                });
            }
        });

        Self { cmd_sender }
    }

    /// Executes a command and streams the output
    ///
    /// # Parameters
    ///
    /// * `command` - The command to execute
    /// * `args` - Arguments to pass to the command
    /// * `timeout` - Optional timeout in seconds after which execution will be cancelled
    ///
    /// # Returns
    ///
    /// A tuple containing the exit code and a vector of output lines
    pub async fn execute<S: Into<String>>(
        &self,
        command: S,
        args: Vec<String>,
        timeout: Option<u64>,
    ) -> Result<(i32, Vec<CommandLine>), CommandError> {
        let command = command.into();

        // Generate a unique execution ID
        let execution_id = Uuid::new_v4().to_string();

        // Channels for communication
        let (resp_tx, mut resp_rx) = mpsc::channel::<CommandResp>(100);
        let (line_tx, mut line_rx) = mpsc::channel::<CommandLine>(100);
        let (done_tx, done_rx) = oneshot::channel::<Result<i32, CommandError>>();

        // Send the command execution request
        self.cmd_sender
            .send(CommandRequest {
                id: execution_id,
                command,
                args,
                resp_tx,
                done_tx,
                timeout,
            })
            .await
            .map_err(|_| CommandError::Unavailable("Command executor not available".to_string()))?;

        // Process responses in a separate task
        let process_handle = tokio::spawn(async move {
            let mut exit_code = 0;

            while let Some(resp) = resp_rx.recv().await {
                match resp {
                    CommandResp::Line {
                        id: _,
                        stream,
                        text,
                    } => {
                        let _ = line_tx.send(CommandLine { stream, text }).await;
                    }
                    CommandResp::Done {
                        id: _,
                        exit_code: code,
                    } => {
                        exit_code = code;
                        break;
                    }
                    CommandResp::Error { id: _, message } => {
                        let _ = line_tx
                            .send(CommandLine {
                                stream: Stream::Stderr,
                                text: format!("Error: {}", message),
                            })
                            .await;
                        break;
                    }
                }
            }

            exit_code
        });

        // Collect all output lines
        let mut lines = Vec::new();
        while let Some(line) = line_rx.recv().await {
            lines.push(line);
        }

        // Wait for processing to complete
        let _exit_code = process_handle.await.unwrap_or(1);

        // Wait for execution completion
        let result = done_rx
            .await
            .map_err(|_| CommandError::ExecutionError("Command execution failed".to_string()))??;

        Ok((result, lines))
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Creates a new command executor handle
pub fn create_command_executor() -> CommandHandle {
    CommandHandle::new()
}

/// Executes a system command and streams the output
async fn execute_command(
    id: String,
    command: String,
    args: Vec<String>,
    resp_tx: Sender<CommandResp>,
    timeout: Option<u64>,
) -> Result<i32, CommandError> {
    // Spawn the command process
    let mut process = Command::new(&command)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| CommandError::SpawnError(format!("Failed to spawn command: {}", e)))?;

    // Get stdout and stderr handles
    let stdout = process
        .stdout
        .take()
        .ok_or_else(|| CommandError::ExecutionError("Failed to capture stdout".to_string()))?;

    let stderr = process
        .stderr
        .take()
        .ok_or_else(|| CommandError::ExecutionError("Failed to capture stderr".to_string()))?;

    // Track active processing
    let processing = Arc::new(Mutex::new(true));

    // Start stdout handler
    let stdout_reader = BufReader::new(stdout);
    let stdout_resp_tx = resp_tx.clone();
    let stdout_id = id.clone();
    let stdout_processing = Arc::clone(&processing);

    let stdout_handle = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if *stdout_processing.lock().unwrap() {
                let _ = stdout_resp_tx
                    .send(CommandResp::Line {
                        id: stdout_id.clone(),
                        stream: Stream::Stdout,
                        text: line,
                    })
                    .await;
            } else {
                break;
            }
        }
    });

    // Start stderr handler
    let stderr_reader = BufReader::new(stderr);
    let stderr_resp_tx = resp_tx.clone();
    let stderr_id = id.clone();
    let stderr_processing = Arc::clone(&processing);

    let stderr_handle = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if *stderr_processing.lock().unwrap() {
                let _ = stderr_resp_tx
                    .send(CommandResp::Line {
                        id: stderr_id.clone(),
                        stream: Stream::Stderr,
                        text: line,
                    })
                    .await;
            } else {
                break;
            }
        }
    });

    // Set a timeout for the command execution if specified
    let process_wait = async {
        match process.wait().await {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(1);
                let _ = resp_tx
                    .send(CommandResp::Done {
                        id: id.clone(),
                        exit_code,
                    })
                    .await;
                Ok(exit_code)
            }
            Err(e) => {
                let _ = resp_tx
                    .send(CommandResp::Error {
                        id: id.clone(),
                        message: format!("Command execution failed: {}", e),
                    })
                    .await;
                Err(CommandError::ExecutionError(format!(
                    "Failed to wait for command: {}",
                    e
                )))
            }
        }
    };

    // Execute with timeout only if specified
    let result = match timeout {
        Some(timeout_secs) => {
            let timeout_duration = Duration::from_secs(timeout_secs);
            tokio::select! {
                result = process_wait => result,
                _ = sleep(timeout_duration) => {
                    // Kill the process on timeout
                    let _ = process.kill().await;
                    let _ = resp_tx
                        .send(CommandResp::Error {
                            id: id.clone(),
                            message: format!("Command timed out after {} seconds", timeout_secs),
                        })
                        .await;
                    Err(CommandError::Timeout(timeout_secs))
                }
            }
        }
        None => {
            // No timeout, just wait for the process to complete
            process_wait.await
        }
    };

    // Signal to output handlers to stop
    {
        let mut guard = processing.lock().unwrap();
        *guard = false;
    }

    // Wait for output handlers to complete
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    result
}
