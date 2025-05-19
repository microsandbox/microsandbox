//! Core engine management for code evaluation.
//!
//! This module implements the central management system for the REPL engines.
//! It provides a unified interface for interacting with language-specific engines
//! through the `EngineHandle` type, and manages the lifecycle of each engine.
//!
//! # Architecture
//!
//! The architecture follows a reactor pattern, where:
//!
//! 1. A central reactor thread listens for commands on a channel
//! 2. Each command is dispatched to the appropriate language engine
//! 3. Results are sent back through response channels
//!
//! The system is designed to be extensible, allowing for additional language
//! engines to be added with minimal changes to the core architecture.
//!
//! # Feature Flags
//!
//! The module uses feature flags to conditionally include language engines:
//!
//! - `python`: Enables the Python engine
//! - `nodejs`: Enables the Node.js engine
//! - `rust`: Enables the Rust engine
//!
//! # Thread Safety
//!
//! All components are designed to be thread-safe, using message passing for
//! communication between threads and thread-safe wrappers around shared state.
//!
//! # Example
//!
//! ```no_run
//! use microsandbox_portal::code::{start_engines, Language};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start the engines
//!     let handle = start_engines()?;
//!
//!     // Evaluate Python code
//!     #[cfg(feature = "python")]
//!     let result = handle.eval("print('Hello, world!')", Language::Python)?;
//!
//!     // Shutdown
//!     handle.shutdown()?;
//!     Ok(())
//! }
//! ```

use tokio::sync::mpsc;

#[cfg(feature = "nodejs")]
use super::nodejs;
#[cfg(feature = "python")]
use super::python;

use super::types::{Cmd, EngineError, EngineHandle, Language, Line, Resp, Stream};

#[cfg(any(feature = "python", feature = "nodejs"))]
use super::types::Engine;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// All available REPL engines
///
/// This struct holds instances of each language engine that has been
/// enabled through feature flags. Each engine implements the `Engine` trait.
struct Engines {
    #[cfg(feature = "python")]
    python: Box<dyn Engine>,
    #[cfg(feature = "nodejs")]
    nodejs: Box<dyn Engine>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl EngineHandle {
    /// Evaluates code in the specified language
    ///
    /// This method sends a command to the reactor thread to evaluate the
    /// provided code in the specified language, and then collects the
    /// output lines.
    ///
    /// # Parameters
    ///
    /// * `code` - The code to evaluate
    /// * `language` - The language to use for evaluation
    /// * `execution_id` - A unique identifier for this evaluation
    /// * `timeout` - Optional timeout in seconds after which evaluation will be cancelled
    ///
    /// # Returns
    ///
    /// A vector of output lines from the evaluation.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if the evaluation fails or if the reactor
    /// thread is not available.
    pub async fn eval<S: Into<String>>(
        &self,
        code: S,
        language: Language,
        execution_id: S,
        timeout: Option<u64>,
    ) -> Result<Vec<Line>, EngineError> {
        let code = code.into();
        let execution_id = execution_id.into();
        // Create channel for receiving results
        let (resp_tx, mut resp_rx) = mpsc::channel::<Resp>(100);
        let (line_tx, mut line_rx) = mpsc::channel::<Line>(100);

        // Send evaluation command to reactor using the provided execution_id
        self.cmd_sender
            .send(Cmd::Eval {
                id: execution_id,
                code,
                language,
                resp_tx,
                timeout,
            })
            .await
            .map_err(|_| EngineError::Unavailable("Reactor thread not available".to_string()))?;

        // Process responses in a separate task
        let process_handle = tokio::spawn(async move {
            while let Some(resp) = resp_rx.recv().await {
                match resp {
                    Resp::Line {
                        id: _,
                        stream,
                        text,
                    } => {
                        let _ = line_tx.send(Line { stream, text }).await;
                    }
                    Resp::Done { id: _ } => {
                        break;
                    }
                    Resp::Error { id: _, message } => {
                        let _ = line_tx
                            .send(Line {
                                stream: Stream::Stderr,
                                text: format!("Error: {}", message),
                            })
                            .await;
                        break;
                    }
                }
            }
        });

        // Collect all lines
        let mut lines = Vec::new();
        while let Some(line) = line_rx.recv().await {
            lines.push(line);
        }

        // Wait for processing to complete
        let _ = process_handle.await;

        Ok(lines)
    }

    /// Shuts down all engines and the reactor
    ///
    /// This method sends a shutdown command to the reactor thread, which
    /// will then shut down all language engines and terminate.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if the reactor thread is not available.
    pub async fn shutdown(&self) -> Result<(), EngineError> {
        self.cmd_sender
            .send(Cmd::Shutdown)
            .await
            .map_err(|_| EngineError::Unavailable("Reactor thread not available".to_string()))?;
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Start all supported REPL engines and return a handle
///
/// This function initializes all the language engines that have been enabled
/// through feature flags and starts the reactor thread that manages them.
/// It returns a handle that can be used to interact with the engines.
///
/// # Returns
///
/// An `EngineHandle` that can be used to evaluate code and shut down the engines.
///
/// # Errors
///
/// Returns an `EngineError` if any of the engines fail to initialize.
pub async fn start_engines() -> Result<EngineHandle, EngineError> {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Cmd>(100);

    // Spawn reactor task
    tokio::spawn(async move {
        // Initialize engines asynchronously
        #[cfg(any(feature = "python", feature = "nodejs"))]
        let mut engines = initialize_engines()
            .await
            .expect("Failed to initialize engines");

        // Process commands until shutdown
        #[cfg(any(feature = "python", feature = "nodejs"))]
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                Cmd::Eval {
                    id,
                    code,
                    language,
                    resp_tx,
                    timeout,
                } => match language {
                    #[cfg(feature = "python")]
                    Language::Python => {
                        if let Err(e) = engines
                            .python
                            .eval(id.clone(), code, &resp_tx, timeout)
                            .await
                        {
                            let _ = resp_tx
                                .send(Resp::Error {
                                    id,
                                    message: e.to_string(),
                                })
                                .await;
                        }
                    }
                    #[cfg(feature = "nodejs")]
                    Language::Node => {
                        if let Err(e) = engines
                            .nodejs
                            .eval(id.clone(), code, &resp_tx, timeout)
                            .await
                        {
                            let _ = resp_tx
                                .send(Resp::Error {
                                    id,
                                    message: e.to_string(),
                                })
                                .await;
                        }
                    }
                },
                Cmd::Shutdown => {
                    // Shutdown all engines
                    #[cfg(feature = "python")]
                    engines.python.shutdown().await;
                    #[cfg(feature = "nodejs")]
                    engines.nodejs.shutdown().await;
                    break;
                }
            }
        }
    });

    Ok(EngineHandle { cmd_sender: cmd_tx })
}

/// Initialize all engines
///
/// This function creates and initializes instances of each language engine
/// that has been enabled through feature flags.
///
/// # Returns
///
/// An `Engines` struct containing the initialized engines.
///
/// # Errors
///
/// Returns an `EngineError` if any of the engines fail to initialize.
async fn initialize_engines() -> Result<Engines, EngineError> {
    #[cfg(feature = "python")]
    let mut python_engine = python::create_engine()?;
    #[cfg(feature = "nodejs")]
    let mut nodejs_engine = nodejs::create_engine()?;

    // Initialize each engine asynchronously
    #[cfg(feature = "python")]
    python_engine.initialize().await?;
    #[cfg(feature = "nodejs")]
    nodejs_engine.initialize().await?;

    Ok(Engines {
        #[cfg(feature = "python")]
        python: python_engine,
        #[cfg(feature = "nodejs")]
        nodejs: nodejs_engine,
    })
}
