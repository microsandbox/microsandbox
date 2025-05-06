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
use std::sync::mpsc as stdmpsc;
use tokio::sync::{mpsc, oneshot};

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
            let stdout_rx = outputs.stdout;
            let stderr_rx = outputs.stderr;

            // Spawn helper thread for capturing stdout
            {
                let out_tx = out_tx.clone();
                std::thread::spawn(move || {
                    for event in stdout_rx {
                        // Convert StdoutEvent to String using Debug formatting
                        let _ = out_tx.send((Stream::Stdout, format!("{:?}", event)));
                    }
                });
            }

            // Spawn helper thread for capturing stderr
            std::thread::spawn(move || {
                for event in stderr_rx {
                    // stderr events are already Strings
                    let _ = out_tx.send((Stream::Stderr, event));
                }
            });

            // Create a new Tokio runtime for this thread to handle async operations
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

            // Main loop: process evaluation commands
            while let Some(cmd) = rt.block_on(cmd_rx.recv()) {
                let EvalCmd {
                    id,
                    code,
                    resp_tx,
                    done_tx,
                } = cmd;

                // Evaluate the code with evcxr
                let eval_res = ctx
                    .eval(&code)
                    .map(|_| ())
                    .map_err(|e| EngineError::Evaluation(e.to_string()));

                // Drain all available output (non-blocking)
                while let Ok((stream, text)) = out_rx.try_recv() {
                    let _ = resp_tx.blocking_send(Resp::Line {
                        id: id.clone(),
                        stream,
                        text,
                    });
                }

                // Signal that evaluation is complete
                let _ = resp_tx.blocking_send(Resp::Done { id: id.clone() });
                let _ = done_tx.send(eval_res);
            }
        });

        Ok(())
    }

    async fn eval(
        &mut self,
        id: String,
        code: String,
        sender: &mpsc::Sender<Resp>,
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
