//! Core functionality for the microsandbox portal system.
//!
//! The portal module provides the interface between the microsandbox environment
//! and external systems. It handles code execution, command execution, and file
//! system operations in a controlled, sandboxed manner.
//!
//! # Core Components
//!
//! The portal consists of several submodules:
//!
//! - `code`: Provides multi-language REPL engines for code evaluation
//! - `command`: Handles sandboxed execution of system commands
//! - `fs`: Manages secure file system operations
//!
//! # Architecture
//!
//! The portal system follows a modular architecture where each submodule handles
//! a specific aspect of the sandboxed environment. All operations are designed
//! with security as the primary consideration.
//!
//! # Feature Flags
//!
//! The functionality of the portal can be customized using various feature flags:
//!
//! - Language-specific features: `python`, `javascript`, `rust`
//! - Security features: Various flags controlling isolation levels
//!
//! # Example
//!
//! ```no_run
//! use microsandbox_portal::code::{start_engines, Language};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize code evaluation engines
//!     let engines = start_engines()?;
//!
//!     // Evaluate Python code
//!     #[cfg(feature = "python")]
//!     let result = engines.eval("print('Hello from microsandbox!')", Language::Python)?;
//!
//!     engines.shutdown()?;
//!     Ok(())
//! }
//! ```

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

pub mod code;
pub mod command;
pub mod fs;
