//! Rust REPL engine implementation using the `evcxr` **library** (in-process).
//!
//! This version is dramatically simpler than the old subprocess design – we just
//! keep a single `evcxr::EvalContext` alive in a dedicated worker thread and
//! forward messages between that thread and the async world via channels.
//!
//! The context preserves state automatically, so multi-step / stateful
//! evaluation works out of the box.

use async_trait::async_trait;
use evcxr::EvalContext;
use rand::{distr::Alphanumeric, Rng};
use std::sync::{mpsc as stdmpsc, Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout as tokio_timeout;
use tokio::time::{sleep, Duration};

use super::types::{Engine, EngineError, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Command sent from async world to the blocking worker that owns `EvalContext`.
struct EvalCmd {
    /// Unique identifier for the evaluation
    id: String,

    /// Code to evaluate
    code: String,

    /// Channel to send responses back to caller
    resp_tx: mpsc::Sender<Resp>,

    /// One-shot channel to signal completion
    done_tx: oneshot::Sender<Result<(), EngineError>>,

    /// Optional timeout in seconds
    timeout: Option<u64>,
}

/// Helper struct to track execution status
struct ExecutionStatus {
    eoe_marker: String,
    completed: bool,
}

/// Rust engine implementation using an in-process `EvalContext`.
pub struct RustEngine {
    /// Channel to send commands to the worker thread
    /// None means not initialized or already shut down
    cmd_tx: Option<mpsc::Sender<EvalCmd>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl RustEngine {
    /// Creates a new, uninitialized Rust engine
    pub fn new() -> Self {
        Self { cmd_tx: None }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

#[async_trait]
impl Engine for RustEngine {
    async fn initialize(&mut self) -> Result<(), EngineError> {
        // Create channel for sending commands to worker thread
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<EvalCmd>(32);
        self.cmd_tx = Some(cmd_tx);

        // Spawn a blocking worker thread to own the non-Send EvalContext
        std::thread::spawn(move || {
            // This hook is mandatory to prevent fork-bombs when using evcxr as a library
            evcxr::runtime_hook();

            // Create the evaluation context and get its output channels
            let (mut ctx, outputs) = match EvalContext::new() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[evcxr-worker] Failed to create EvalContext: {e}");
                    return;
                }
            };

            // Set optimization level to 0 for faster compilation
            if let Err(e) = ctx.set_opt_level("0") {
                eprintln!("[evcxr-worker] Failed to set optimization level: {e}");
            }

            // Create a channel to bridge standard output from evcxr to async world
            let (out_tx, out_rx) = stdmpsc::channel::<(Stream, String)>();

            // Create execution status tracking
            let execution_status = Arc::new(Mutex::new(None::<ExecutionStatus>));
            let exec_status = Arc::clone(&execution_status);

            let stdout_rx = outputs.stdout;
            let stderr_rx = outputs.stderr;

            // Spawn helper thread for capturing stdout
            {
                let out_tx = out_tx.clone();
                let stdout_exec_status = Arc::clone(&execution_status);
                std::thread::spawn(move || {
                    for event in stdout_rx {
                        // Convert StdoutEvent to String using Debug formatting
                        let output = format!("{:?}", event);

                        // Check if this is an end-of-execution marker
                        let mut is_marker = false;
                        {
                            let mut status_guard = stdout_exec_status.lock().unwrap();
                            if let Some(status) = status_guard.as_mut() {
                                if output.trim() == status.eoe_marker {
                                    status.completed = true;
                                    is_marker = true;
                                }
                            }
                        }

                        // Only send if not a marker
                        if !is_marker {
                            let _ = out_tx.send((Stream::Stdout, output));
                        }
                    }
                });
            }

            // Spawn helper thread for capturing stderr
            {
                let stderr_exec_status = Arc::clone(&execution_status);
                std::thread::spawn(move || {
                    for event in stderr_rx {
                        // stderr events are already Strings
                        let mut is_marker = false;
                        {
                            let mut status_guard = stderr_exec_status.lock().unwrap();
                            if let Some(status) = status_guard.as_mut() {
                                if event.trim() == status.eoe_marker {
                                    status.completed = true;
                                    is_marker = true;
                                }
                            }
                        }

                        // Only send if not a marker
                        if !is_marker {
                            let _ = out_tx.send((Stream::Stderr, event));
                        }
                    }
                });
            }

            // Create a new Tokio runtime for this thread to handle async operations
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

            // Main loop: process evaluation commands
            while let Some(cmd) = rt.block_on(cmd_rx.recv()) {
                let EvalCmd {
                    id,
                    code,
                    resp_tx,
                    done_tx,
                    timeout: timeout_opt,
                } = cmd;

                // Generate a unique end-of-execution marker
                let eoe_marker = format!(
                    "eoe_{}",
                    rand::rng()
                        .sample_iter(&Alphanumeric)
                        .take(20)
                        .map(char::from)
                        .collect::<String>()
                );

                // Set the current execution status
                {
                    let mut status_guard = exec_status.lock().unwrap();
                    *status_guard = Some(ExecutionStatus {
                        eoe_marker: eoe_marker.clone(),
                        completed: false,
                    });
                }

                // Add the EOE marker to the end of the code
                let mut code_with_marker = code;
                // Add a println for the end-of-execution marker
                code_with_marker.push_str(&format!("\nprintln!(\"{}\");", eoe_marker));

                // Evaluate the code with evcxr
                let eval_res = ctx
                    .eval(&code_with_marker)
                    .map(|_| ())
                    .map_err(|e| EngineError::Evaluation(e.to_string()));

                // Drain all available output (non-blocking)
                let mut output_lines = Vec::new();
                while let Ok((stream, text)) = out_rx.try_recv() {
                    output_lines.push((stream, text));
                }

                // Send all accumulated output
                for (stream, text) in output_lines {
                    let _ = resp_tx.blocking_send(Resp::Line {
                        id: id.clone(),
                        stream,
                        text,
                    });
                }

                // Wait for the EOE marker or timeout
                if eval_res.is_ok() {
                    // Create wait future to monitor completion status
                    let wait_future = async {
                        let mut completed = false;
                        while !completed {
                            {
                                let status_guard = exec_status.lock().unwrap();
                                if let Some(status) = status_guard.as_ref() {
                                    completed = status.completed;
                                }
                            }
                            sleep(Duration::from_millis(50)).await;
                        }
                    };

                    // Apply timeout only if specified
                    let timeout_result = match timeout_opt {
                        Some(timeout_secs) => {
                            let timeout_duration = Duration::from_secs(timeout_secs);
                            rt.block_on(async {
                                tokio_timeout(timeout_duration, wait_future).await
                            })
                        }
                        None => {
                            // No timeout, just wait for completion
                            rt.block_on(wait_future);
                            Ok(())
                        }
                    };

                    if let Err(_) = timeout_result {
                        // Timeout occurred
                        let timeout_secs = timeout_opt.unwrap(); // Safe to unwrap since we're in Some branch
                        let _ = resp_tx.blocking_send(Resp::Error {
                            id: id.clone(),
                            message: format!("Execution timed out after {} seconds", timeout_secs),
                        });
                        let _ = done_tx.send(Err(EngineError::Timeout(timeout_secs)));
                    } else {
                        // Signal that evaluation is complete
                        let _ = resp_tx.blocking_send(Resp::Done { id: id.clone() });
                        let _ = done_tx.send(eval_res);
                    }
                } else {
                    // Evaluation error - signal completion with error
                    let _ = resp_tx.blocking_send(Resp::Done { id: id.clone() });
                    let _ = done_tx.send(eval_res);
                }

                // Clear current execution status
                {
                    let mut status_guard = exec_status.lock().unwrap();
                    *status_guard = None;
                }
            }
        });

        Ok(())
    }

    async fn eval(
        &mut self,
        id: String,
        code: String,
        sender: &mpsc::Sender<Resp>,
        timeout: Option<u64>,
    ) -> Result<(), EngineError> {
        // Get command channel or return error if engine not initialized
        let tx = self
            .cmd_tx
            .as_ref()
            .ok_or_else(|| EngineError::Unavailable("Rust engine not initialized".into()))?;

        // Create one-shot channel for completion notification
        let (done_tx, done_rx) = oneshot::channel();

        // Send evaluation command to worker thread
        tx.send(EvalCmd {
            id,
            code,
            resp_tx: sender.clone(),
            done_tx,
            timeout,
        })
        .await
        .map_err(|_| EngineError::Unavailable("Rust worker thread gone".into()))?;

        // Wait for evaluation to complete
        done_rx
            .await
            .map_err(|_| EngineError::Unavailable("Rust eval cancelled".into()))?
    }

    async fn shutdown(&mut self) {
        // Drop the sender to allow worker thread to exit its loop and clean up
        self.cmd_tx.take();
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Creates a new Rust engine instance
///
/// # Returns
///
/// A boxed implementation of the `Engine` trait
///
/// # Errors
///
/// This function cannot fail directly, but the engine may fail to initialize
/// later if evcxr cannot be loaded.
pub fn create_engine() -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(RustEngine::new()))
}
