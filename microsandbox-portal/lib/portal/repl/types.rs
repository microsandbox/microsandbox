//! Types and interfaces for the code evaluation engines.
//!
//! This module provides the fundamental types, traits, and error definitions used by
//! the microsandbox-portal code evaluation system. It defines the language-agnostic
//! interfaces that each specific language implementation must conform to.
//!
//! # Architecture
//!
//! The code evaluation system is built around a few key abstractions:
//!
//! - `Language`: An enum representing the supported programming languages
//! - `EngineHandle`: A handle for interacting with the language engines
//! - `Engine`: A trait defining the interface each language engine must implement
//! - `Resp` and `Line`: Types for representing evaluation output
//!
//! Each specific language implementation (Python, Node.js, Rust) provides its own
//! engine that implements the `Engine` trait, but users interact with the system
//! through the `EngineHandle` which provides a unified interface.
//!
//! # Error Handling
//!
//! The module defines an `EngineError` type that encapsulates the various error
//! conditions that can occur during engine initialization, code evaluation, etc.
//!
//! # Thread Safety
//!
//! The design accounts for concurrent use by leveraging thread-safe primitives and
//! message passing through channels to communicate between components.

use thiserror::Error;
use tokio::sync::mpsc::Sender;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Supported programming languages for evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Python language support
    #[cfg(feature = "python")]
    Python,

    /// Node.js/JavaScript support
    #[cfg(feature = "nodejs")]
    Node,

    /// Rust language support
    #[cfg(feature = "rust")]
    Rust,
}

/// Stream type for output lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// Standard output stream
    Stdout,

    /// Standard error stream
    Stderr,
}

/// A single line of output from code evaluation
#[derive(Debug, Clone)]
pub struct Line {
    /// Stream type (stdout/stderr)
    pub stream: Stream,

    /// Line content
    pub text: String,
}

/// Handle for interacting with the REPL engines
///
/// This is the primary interface that clients use to evaluate code in
/// various languages. It communicates with the engine manager through
/// a command channel.
#[derive(Clone)]
pub struct EngineHandle {
    pub(crate) cmd_sender: Sender<Cmd>,
}

/// Error types that can occur during engine operations
///
/// This enum encapsulates the various error conditions that can occur
/// during engine initialization, code evaluation, and other operations.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Error during engine initialization
    #[error("Failed to initialize engine: {0}")]
    Initialization(String),

    /// Error during code evaluation
    #[error("Evaluation error: {0}")]
    Evaluation(String),

    /// Timeout during evaluation
    #[error("Evaluation timeout after {0} seconds")]
    Timeout(u64),

    /// Engine unavailable (shutdown or crashed)
    #[error("Engine unavailable: {0}")]
    Unavailable(String),
}

/// Command sent to the reactor thread
///
/// These commands are sent from the `EngineHandle` to the reactor thread
/// to perform operations like code evaluation and engine shutdown.
#[derive(Debug)]
pub(crate) enum Cmd {
    /// Evaluate code with ID and code string
    Eval {
        id: String,
        code: String,
        language: Language,
        resp_tx: Sender<Resp>,
    },

    /// Shutdown the reactor and all engines
    Shutdown,
}

/// Response from an engine evaluation
///
/// These responses are sent from the engine back to the client to
/// provide evaluation results, output lines, or error messages.
#[derive(Debug)]
pub enum Resp {
    /// A line of output from evaluation
    Line {
        /// Unique identifier for the evaluation
        id: String,

        /// Stream type (stdout/stderr)
        stream: Stream,

        /// Line content
        text: String,
    },

    /// Evaluation completed successfully
    Done {
        /// Unique identifier for the evaluation
        id: String,
    },

    /// Evaluation resulted in an error
    Error {
        /// Unique identifier for the evaluation
        id: String,

        /// Error message
        message: String,
    },
}

//--------------------------------------------------------------------------------------------------
// Traits
//--------------------------------------------------------------------------------------------------

/// Trait defining common engine operations
///
/// This trait must be implemented by each language-specific engine.
/// It defines the core operations that all engines must support.
#[async_trait::async_trait]
pub trait Engine: Send + 'static {
    /// Initialize the engine
    ///
    /// This method is called when the engine is first created to set up
    /// any necessary resources, start the evaluation context, etc.
    async fn initialize(&mut self) -> Result<(), EngineError>;

    /// Evaluate code and send responses through the channel
    ///
    /// This method evaluates the given code and sends output and status
    /// messages through the provided response channel.
    ///
    /// # Parameters
    ///
    /// * `id` - A unique identifier for this evaluation
    /// * `code` - The code to evaluate
    /// * `sender` - A channel for sending evaluation responses
    async fn eval(
        &mut self,
        id: String,
        code: String,
        sender: &Sender<Resp>,
    ) -> Result<(), EngineError>;

    /// Shutdown the engine
    ///
    /// This method is called when the engine is being shut down to clean up
    /// resources, terminate processes, etc.
    async fn shutdown(&mut self);
}

// -------------------------------------------------------------------------------------------------
// Trait Implementations
// -------------------------------------------------------------------------------------------------

// Add Debug implementation for EngineHandle
impl std::fmt::Debug for EngineHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineHandle")
            .field("cmd_sender", &"<channel>")
            .finish()
    }
}
