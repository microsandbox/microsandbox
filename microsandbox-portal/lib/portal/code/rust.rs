//! Rust REPL engine implementation using evcxr.
//!
//! This module provides a Rust code evaluation engine that leverages the `evcxr` crate
//! to execute Rust code within a persistent context. This allows for stateful evaluations
//! where variables and functions defined in previous evaluations remain available.
//!
//! # Features
//!
//! - Interactive Rust code evaluation
//! - Stateful execution context
//! - Streaming stdout and stderr output
//! - Proper resource cleanup on shutdown
//!
//! # Implementation Details
//!
//! The implementation creates a dedicated `EvalContext` from the evcxr crate and manages
//! it within a thread-safe container. It uses separate threads to handle stdout and stderr
//! outputs, which are then forwarded to the caller through a response channel.
//!
//! # Examples
//!
//! ```no_run
//! use microsandbox_portal::code::{Language, EngineHandle};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = microsandbox_portal::code::start_engines()?;
//!
//!     // Evaluate some Rust code
//!     let output = engine.eval("println!(\"Hello, Rust!\");", Language::Rust)?;
//!
//!     // Print the output
//!     for line in output {
//!         println!("{}: {}", line.stream, line.text);
//!     }
//!
//!     engine.shutdown()?;
//!     Ok(())
//! }
//! ```

use crossbeam_channel::{bounded, Sender};
use evcxr::{EvalContext, StdoutEvent};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::types::{Engine, EngineError, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Rust engine implementation using evcxr
pub struct RustEngine {
    ctx: Arc<Mutex<Option<EvalContext>>>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    shutdown_signal: Option<Sender<()>>,
    active_eval: Arc<Mutex<Option<(String, Sender<Resp>)>>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl RustEngine {
    /// Creates a new RustEngine instance with uninitialized state.
    ///
    /// This creates the basic structure but does not initialize the evaluation context.
    /// Call `initialize()` to set up the engine before use.
    fn new() -> Self {
        RustEngine {
            ctx: Arc::new(Mutex::new(None)),
            stdout_thread: None,
            stderr_thread: None,
            shutdown_signal: None,
            active_eval: Arc::new(Mutex::new(None)),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Engine for RustEngine {
    fn initialize(&mut self) -> Result<(), EngineError> {
        // Initialize the evcxr runtime
        evcxr::runtime_hook();

        // Create a new evaluation context
        let (ctx, outputs) = EvalContext::new().map_err(|e| {
            EngineError::Initialization(format!("Failed to create Rust eval context: {}", e))
        })?;

        // Create a channel for shutdown signaling
        let (shutdown_tx, shutdown_rx) = bounded::<()>(1);
        self.shutdown_signal = Some(shutdown_tx);

        // Store the context
        let ctx_mutex = Arc::clone(&self.ctx);
        *ctx_mutex.lock().unwrap() = Some(ctx);

        // Start stdout handler thread
        let stdout = outputs.stdout;
        let shutdown_rx_stdout = shutdown_rx.clone();
        let active_eval_stdout = Arc::clone(&self.active_eval);

        self.stdout_thread = Some(thread::spawn(move || {
            loop {
                // Check if shutdown was requested
                if shutdown_rx_stdout.try_recv().is_ok() {
                    break;
                }

                if let Ok(event) = stdout.recv_timeout(Duration::from_millis(100)) {
                    match event {
                        StdoutEvent::Line(line) => {
                            // Send to active evaluation if one exists
                            if let Some((id, sender)) = active_eval_stdout.lock().unwrap().as_ref()
                            {
                                let _ = sender.send(Resp::Line {
                                    id: id.clone(),
                                    stream: Stream::Stdout,
                                    text: line,
                                });
                            }
                        }
                        StdoutEvent::ExecutionComplete => {
                            // Execution is complete, nothing to do here
                        }
                    }
                }
            }
        }));

        // Start stderr handler thread
        let stderr = outputs.stderr;
        let shutdown_rx_stderr = shutdown_rx;
        let active_eval_stderr = Arc::clone(&self.active_eval);

        self.stderr_thread = Some(thread::spawn(move || {
            loop {
                // Check if shutdown was requested
                if shutdown_rx_stderr.try_recv().is_ok() {
                    break;
                }

                // For stderr channel, we use a different approach since there seems to be
                // a type mismatch in the API
                match stderr.recv_timeout(Duration::from_millis(100)) {
                    Ok(line) => {
                        // In evcxr 0.19, stderr might use a different type than StdoutEvent
                        // so we handle it directly as a String
                        if let Some((id, sender)) = active_eval_stderr.lock().unwrap().as_ref() {
                            let _ = sender.send(Resp::Line {
                                id: id.clone(),
                                stream: Stream::Stderr,
                                text: line,
                            });
                        }
                    }
                    Err(_) => {
                        // Timeout or channel closed
                    }
                }
            }
        }));

        // Initialize with some basic setup
        if let Some(ctx) = &mut *self.ctx.lock().unwrap() {
            // Setup initial environment
            ctx.eval("let mut s = String::new();").map_err(|e| {
                EngineError::Initialization(format!("Failed to initialize Rust environment: {}", e))
            })?;
        }

        Ok(())
    }

    fn eval(&mut self, id: String, code: String, sender: &Sender<Resp>) -> Result<(), EngineError> {
        // Store the current evaluation
        {
            let mut active_eval = self.active_eval.lock().unwrap();
            *active_eval = Some((id.clone(), sender.clone()));
        }

        // Clone the sender for use in threads
        let sender = sender.clone();

        // Get the eval context
        let ctx_arc = Arc::clone(&self.ctx);
        let active_eval = Arc::clone(&self.active_eval);

        // Spawn a thread to handle evaluation
        thread::spawn(move || {
            let result = {
                let mut ctx_guard = ctx_arc.lock().unwrap();
                let ctx = ctx_guard.as_mut().unwrap();
                ctx.eval(&code)
            };

            match result {
                Ok(eval_outputs) => {
                    // Process any output from the evaluation
                    // Check for text/plain content which is the most common
                    if let Some(output_text) = eval_outputs.content_by_mime_type.get("text/plain") {
                        if !output_text.is_empty() {
                            let _ = sender.send(Resp::Line {
                                id: id.clone(),
                                stream: Stream::Stdout,
                                text: output_text.clone(),
                            });
                        }
                    }

                    // Mark evaluation as complete
                    let _ = sender.send(Resp::Done { id: id.clone() });
                }
                Err(e) => {
                    // Send error message
                    let _ = sender.send(Resp::Error {
                        id: id.clone(),
                        message: e.to_string(),
                    });
                }
            }

            // Clear the active evaluation
            let mut active_eval_guard = active_eval.lock().unwrap();
            *active_eval_guard = None;
        });

        Ok(())
    }

    fn shutdown(&mut self) {
        // Signal shutdown
        if let Some(tx) = self.shutdown_signal.take() {
            let _ = tx.send(());
        }

        // Drop the context to free resources
        if let Ok(mut guard) = self.ctx.lock() {
            *guard = None;
        }

        // Wait for threads to complete
        if let Some(handle) = self.stdout_thread.take() {
            let _ = handle.join();
        }

        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Creates a new Rust engine instance.
///
/// This function is used by the engine manager to create an instance of the Rust
/// evaluation engine. It returns a boxed trait object that can be used to evaluate
/// Rust code.
///
/// # Returns
///
/// A boxed `Engine` trait object for evaluating Rust code.
///
/// # Errors
///
/// Returns an `EngineError` if the engine could not be created.
pub fn create_engine() -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(RustEngine::new()))
}
