//! Code evaluation engines for multiple programming languages.
//!
//! This module provides a unified system for evaluating code in various programming
//! languages (Rust, Python, JavaScript) in a sandboxed environment. It follows a
//! plugin-based architecture where language engines can be conditionally included
//! based on feature flags.
//!
//! # Primary Components
//!
//! - Engine implementations for each supported language
//! - A unified interface for interacting with all engines
//! - Type definitions for evaluation requests and responses
//! - Error handling structures for various failure modes
//!
//! # Feature Flags
//!
//! The module uses feature flags to control which language engines are included:
//!
//! - `rust`: Enables Rust code evaluation via evcxr
//! - `python`: Enables Python code evaluation
//! - `javascript`: Enables JavaScript/Node.js code evaluation
//!
//! # Usage
//!
//! To use the code evaluation system, start the engines and then evaluate code:
//!
//! ```no_run
//! use microsandbox_portal::code::{start_engines, Language};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start all enabled engines
//!     let handle = start_engines()?;
//!
//!     // Evaluate code in different languages
//!     #[cfg(feature = "python")]
//!     let python_result = handle.eval("print('Hello from Python')", Language::Python)?;
//!
//!     #[cfg(feature = "nodejs")]
//!     let js_result = handle.eval("console.log('Hello from JavaScript')", Language::Node)?;
//!
//!     // Shutdown engines when done
//!     handle.shutdown()?;
//!     Ok(())
//! }
//! ```
//!
//! Each evaluation returns a vector of output lines, with information about whether
//! they were sent to stdout or stderr.

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "nodejs")]
pub mod nodejs;

pub mod engine;
pub mod types;

pub use engine::*;
pub use types::*;
