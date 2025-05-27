//! Python engine implementation for code execution in a sandboxed environment.
//!
//! This module provides a Python-based code execution engine that:
//! - Runs Python code in an interactive subprocess
//! - Captures and streams stdout/stderr output
//! - Manages process lifecycle and cleanup
//! - Provides non-blocking evaluation of Python code
//!
//! The engine uses Python's interactive mode with customized settings to
//! disable prompts and ensure unbuffered output for real-time streaming.

use async_trait::async_trait;
use rand::{distr::Alphanumeric, Rng};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    time::{sleep, timeout as tokio_timeout, Duration},
};

use super::types::{Engine, EngineError, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Python engine implementation using subprocess
pub struct PythonEngine {
    process_control_tx: Option<Sender<ProcessControl>>,
    eval_tx: Option<Sender<EvalRequest>>,
}

/// Commands for controlling the Python process
enum ProcessControl {
    Shutdown,
}

/// Request for code evaluation
struct EvalRequest {
    id: String,
    code: String,
    resp_tx: Sender<Resp>,
    done_tx: oneshot::Sender<Result<(), EngineError>>,
    timeout: Option<u64>,
}

/// Helper struct to track execution status
struct ExecutionStatus {
    id: String,
    sender: Sender<Resp>,
    eoe_marker: String,
    completed: bool,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl PythonEngine {
    fn new() -> Self {
        PythonEngine {
            process_control_tx: None,
            eval_tx: None,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

#[async_trait]
impl Engine for PythonEngine {
    async fn initialize(&mut self) -> Result<(), EngineError> {
        // Create channels for process control and evaluation requests
        let (process_control_tx, mut process_control_rx) = mpsc::channel::<ProcessControl>(10);
        let (eval_tx, mut eval_rx) = mpsc::channel::<EvalRequest>(10);

        self.process_control_tx = Some(process_control_tx);
        self.eval_tx = Some(eval_tx);

        // Start the Python process manager in a separate task
        tokio::spawn(async move {
            // Start Python process with interactive mode
            // -q: hide banner, -u: unbuffered, -i: interactive, clear prompts
            let mut process = match Command::new("python3")
                .args(&["-q", "-u", "-i", "-c", "import sys; sys.ps1=sys.ps2=''"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to start Python process: {}", e);
                    return;
                }
            };

            // Get stdin handle
            let mut stdin = match process.stdin.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Python stdin");
                    return;
                }
            };

            // Get stdout and stderr handles
            let stdout = match process.stdout.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Python stdout");
                    return;
                }
            };

            let stderr = match process.stderr.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Python stderr");
                    return;
                }
            };

            // Current execution status
            let execution_status = Arc::new(Mutex::new(None::<ExecutionStatus>));

            // Start stdout handler in a separate task
            let stdout_reader = BufReader::new(stdout);
            let (stdout_done_tx, mut stdout_done_rx) = mpsc::channel::<()>(1);
            let stdout_exec_status = Arc::clone(&execution_status);

            tokio::task::spawn_blocking(move || {
                let mut lines_future = stdout_reader.lines();
                let runtime = tokio::runtime::Handle::current();

                loop {
                    // Use the runtime to execute the async call in the blocking thread
                    let line_result = runtime.block_on(lines_future.next_line());

                    match line_result {
                        Ok(Some(line)) => {
                            // Check if this is an end-of-execution marker line
                            let mut should_send = true;

                            {
                                let mut status_guard = stdout_exec_status.lock().unwrap();
                                if let Some(status) = status_guard.as_mut() {
                                    // Check if this line is our end-of-execution marker
                                    if line.trim() == status.eoe_marker {
                                        should_send = false;
                                        status.completed = true;

                                        // Signal completion
                                        let id = status.id.clone();
                                        let sender = status.sender.clone();
                                        runtime.block_on(async {
                                            let _ = sender.send(Resp::Done { id }).await;
                                        });
                                    }
                                }
                            }

                            // Send line if it's not an EOE marker
                            if should_send {
                                if let Some(status) = stdout_exec_status.lock().unwrap().as_ref() {
                                    // Use block_on to send the message
                                    let _ = runtime.block_on(status.sender.send(Resp::Line {
                                        id: status.id.clone(),
                                        stream: Stream::Stdout,
                                        text: line,
                                    }));
                                }
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(_) => break,   // Error reading
                    }
                }

                // Signal that we're done
                let _ = runtime.block_on(stdout_done_tx.send(()));
            });

            // Start stderr handler in a separate task
            let stderr_reader = BufReader::new(stderr);
            let (stderr_done_tx, mut stderr_done_rx) = mpsc::channel::<()>(1);
            let stderr_exec_status = Arc::clone(&execution_status);

            tokio::task::spawn_blocking(move || {
                let mut lines_future = stderr_reader.lines();
                let runtime = tokio::runtime::Handle::current();

                loop {
                    // Use the runtime to execute the async call in the blocking thread
                    let line_result = runtime.block_on(lines_future.next_line());

                    match line_result {
                        Ok(Some(line)) => {
                            if let Some(status) = stderr_exec_status.lock().unwrap().as_ref() {
                                // Use block_on to send the message
                                let _ = runtime.block_on(status.sender.send(Resp::Line {
                                    id: status.id.clone(),
                                    stream: Stream::Stderr,
                                    text: line,
                                }));
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(_) => break,   // Error reading
                    }
                }

                // Signal that we're done
                let _ = runtime.block_on(stderr_done_tx.send(()));
            });

            // Process control and evaluation loop
            loop {
                tokio::select! {
                    Some(ctrl) = process_control_rx.recv() => {
                        match ctrl {
                            ProcessControl::Shutdown => {
                                break;
                            }
                        }
                    }
                    Some(eval_req) = eval_rx.recv() => {
                        let EvalRequest { id, code, resp_tx, done_tx, timeout } = eval_req;

                        // Generate a unique end-of-execution marker
                        // This should be unique enough to not appear in normal output
                        let eoe_marker = format!("eoe_{}", rand::rng()
                            .sample_iter(&Alphanumeric)
                            .take(20)
                            .map(char::from)
                            .collect::<String>());

                        // Set as current execution
                        {
                            let mut status_guard = execution_status.lock().unwrap();
                            *status_guard = Some(ExecutionStatus {
                                id: id.clone(),
                                sender: resp_tx.clone(),
                                eoe_marker: eoe_marker.clone(),
                                completed: false,
                            });
                        }

                        // Execute the code with timeout
                        let exec_status = Arc::clone(&execution_status);

                        let result = async {
                            // Prepare code with EOE marker
                            // Ensure code ends with a newline for proper execution
                            let mut code_with_marker = match code.chars().last() {
                                Some('\n') => code,
                                _ => format!("{}\n", code),
                            };

                            // Add print statement for EOE marker
                            code_with_marker.push_str(&format!("\nprint('{}')\n", eoe_marker));

                            // Write code to Python process
                            stdin.write_all(code_with_marker.as_bytes()).await.map_err(|e| {
                                EngineError::Evaluation(format!("Failed to send code to Python: {}", e))
                            })?;

                            // Flush to ensure code is processed
                            stdin.flush().await.map_err(|e| {
                                EngineError::Evaluation(format!("Failed to flush code to Python: {}", e))
                            })?;

                            // Create wait future that will complete when the EOE marker is detected
                            let wait_future = async {
                                let mut completed = false;
                                while !completed {
                                    if let Some(status) = exec_status.lock().unwrap().as_ref() {
                                        completed = status.completed;
                                    }
                                    sleep(Duration::from_millis(50)).await;
                                }
                            };

                            // Apply timeout only if specified
                            match timeout {
                                Some(timeout_secs) => {
                                    let timeout_duration = Duration::from_secs(timeout_secs);
                                    if let Err(_) = tokio_timeout(timeout_duration, wait_future).await {
                                        // Timeout occurred
                                        let _ = resp_tx.send(Resp::Error {
                                            id: id.clone(),
                                            message: format!("Execution timed out after {} seconds", timeout_secs),
                                        }).await;
                                        return Err(EngineError::Timeout(timeout_secs));
                                    }
                                },
                                None => {
                                    // No timeout, just wait for completion
                                    wait_future.await;
                                }
                            }

                            Ok(())
                        }.await;

                        // Clear current execution
                        {
                            let mut status_guard = execution_status.lock().unwrap();
                            *status_guard = None;
                        }

                        // Signal completion to caller
                        let _ = done_tx.send(result);
                    }
                    _ = stdout_done_rx.recv() => {
                        eprintln!("Python stdout handler exited");
                        break;
                    }
                    _ = stderr_done_rx.recv() => {
                        eprintln!("Python stderr handler exited");
                        break;
                    }
                }
            }

            // Cleanup: kill the process
            let _ = process.kill().await;
            let _ = process.wait().await;
        });

        // Wait a bit for initialization
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    async fn eval(
        &mut self,
        id: String,
        code: String,
        sender: &Sender<Resp>,
        timeout: Option<u64>,
    ) -> Result<(), EngineError> {
        let eval_tx = self
            .eval_tx
            .as_ref()
            .ok_or_else(|| EngineError::Unavailable("Python engine not initialized".to_string()))?;

        // Create a oneshot channel for the result
        let (done_tx, done_rx) = oneshot::channel();

        // Send evaluation request
        eval_tx
            .send(EvalRequest {
                id,
                code,
                resp_tx: sender.clone(),
                done_tx,
                timeout,
            })
            .await
            .map_err(|_| EngineError::Unavailable("Python process channel closed".to_string()))?;

        // Wait for completion
        done_rx
            .await
            .map_err(|_| EngineError::Unavailable("Python evaluation cancelled".to_string()))?
    }

    async fn shutdown(&mut self) {
        if let Some(tx) = self.process_control_tx.take() {
            // Send shutdown command
            let _ = tx.send(ProcessControl::Shutdown).await;
        }

        // Clear channels
        self.eval_tx = None;

        // Wait a bit for clean shutdown
        sleep(Duration::from_millis(100)).await;
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Create a new Python engine instance
pub fn create_engine() -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(PythonEngine::new()))
}
