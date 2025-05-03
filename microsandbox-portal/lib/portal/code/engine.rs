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
//! - `javascript`: Enables the Node.js engine
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

/// ```
use crossbeam_channel::bounded;
use std::thread;
use uuid::Uuid;

#[cfg(feature = "javascript")]
use super::node;
#[cfg(feature = "python")]
use super::python;
#[cfg(feature = "rust")]
use super::rust;

use super::types::{Cmd, Engine, EngineError, EngineHandle, Language, Line, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Internal Types
//--------------------------------------------------------------------------------------------------

/// All available REPL engines
///
/// This struct holds instances of each language engine that has been
/// enabled through feature flags. Each engine implements the `Engine` trait.
struct Engines {
    #[cfg(feature = "python")]
    python: Box<dyn Engine>,
    #[cfg(feature = "javascript")]
    node: Box<dyn Engine>,
    #[cfg(feature = "rust")]
    rust: Box<dyn Engine>,
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
    ///
    /// # Returns
    ///
    /// A vector of output lines from the evaluation.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if the evaluation fails or if the reactor
    /// thread is not available.
    pub fn eval<S: Into<String>>(
        &self,
        code: S,
        language: Language,
    ) -> Result<Vec<Line>, EngineError> {
        let id = Uuid::new_v4().to_string();
        let code = code.into();

        // Create bounded channels for receiving results
        let (_resp_sender, resp_receiver) = bounded::<Resp>(100);
        let (line_sender, line_receiver) = bounded::<Line>(100);

        // Send evaluation command to reactor
        self.cmd_sender
            .send(Cmd::Eval {
                id: id.clone(),
                code,
                language,
            })
            .map_err(|_| EngineError::Unavailable("Reactor thread not available".to_string()))?;

        // Process responses in a separate thread
        thread::spawn(move || {
            while let Ok(resp) = resp_receiver.recv() {
                match resp {
                    Resp::Line {
                        id: _,
                        stream,
                        text,
                    } => {
                        let _ = line_sender.send(Line { stream, text });
                    }
                    Resp::Done { id: _ } => {
                        break;
                    }
                    Resp::Error { id: _, message } => {
                        let _ = line_sender.send(Line {
                            stream: Stream::Stderr,
                            text: format!("Error: {}", message),
                        });
                        break;
                    }
                }
            }
            drop(line_sender); // Close channel when done
        });

        // Collect all lines
        let mut lines = Vec::new();
        while let Ok(line) = line_receiver.recv() {
            lines.push(line);
        }

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
    pub fn shutdown(&self) -> Result<(), EngineError> {
        self.cmd_sender
            .send(Cmd::Shutdown)
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
pub fn start_engines() -> Result<EngineHandle, EngineError> {
    let (cmd_sender, cmd_receiver) = bounded::<Cmd>(100);

    // Spawn reactor thread
    thread::spawn(move || {
        let mut engines = initialize_engines().expect("Failed to initialize engines");

        // Process commands until shutdown
        while let Ok(cmd) = cmd_receiver.recv() {
            match cmd {
                Cmd::Eval { id, code, language } => {
                    let (resp_sender, _) = bounded::<Resp>(100);
                    match language {
                        #[cfg(feature = "python")]
                        Language::Python => {
                            if let Err(e) = engines.python.eval(id.clone(), code, &resp_sender) {
                                let _ = resp_sender.send(Resp::Error {
                                    id,
                                    message: e.to_string(),
                                });
                            }
                        }
                        #[cfg(feature = "javascript")]
                        Language::Node => {
                            if let Err(e) = engines.node.eval(id.clone(), code, &resp_sender) {
                                let _ = resp_sender.send(Resp::Error {
                                    id,
                                    message: e.to_string(),
                                });
                            }
                        }
                        #[cfg(feature = "rust")]
                        Language::Rust => {
                            if let Err(e) = engines.rust.eval(id.clone(), code, &resp_sender) {
                                let _ = resp_sender.send(Resp::Error {
                                    id,
                                    message: e.to_string(),
                                });
                            }
                        }
                        #[cfg(not(any(
                            feature = "python",
                            feature = "javascript",
                            feature = "rust"
                        )))]
                        _ => {
                            let _ = resp_sender.send(Resp::Error {
                                id,
                                message: "Unsupported language".to_string(),
                            });
                        }
                    }
                }
                Cmd::Shutdown => {
                    // Shutdown all engines
                    #[cfg(feature = "python")]
                    engines.python.shutdown();
                    #[cfg(feature = "javascript")]
                    engines.node.shutdown();
                    #[cfg(feature = "rust")]
                    engines.rust.shutdown();
                    break;
                }
            }
        }
    });

    Ok(EngineHandle { cmd_sender })
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
fn initialize_engines() -> Result<Engines, EngineError> {
    #[cfg(feature = "python")]
    let mut python_engine = python::create_engine()?;
    #[cfg(feature = "javascript")]
    let mut node_engine = node::create_engine()?;
    #[cfg(feature = "rust")]
    let mut rust_engine = rust::create_engine()?;

    // Initialize each engine
    #[cfg(feature = "python")]
    python_engine.initialize()?;
    #[cfg(feature = "javascript")]
    node_engine.initialize()?;
    #[cfg(feature = "rust")]
    rust_engine.initialize()?;

    Ok(Engines {
        #[cfg(feature = "python")]
        python: python_engine,
        #[cfg(feature = "javascript")]
        node: node_engine,
        #[cfg(feature = "rust")]
        rust: rust_engine,
    })
}
