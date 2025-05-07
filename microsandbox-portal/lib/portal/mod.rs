//! Core functionality for the microsandbox portal system.
//!
//! The portal module provides the interface between the microsandbox environment
//! and external systems. It handles code execution in REPL environments, command execution, and file
//! system operations in a controlled, sandboxed manner.
//!
//! # Core Components
//!
//! The portal consists of several submodules:
//!
//! - `repl`: Provides multi-language REPL engines for interactive code execution
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
//! # Examples
//!
//! ## REPL Code Execution
//!
//! ```no_run
//! use microsandbox_portal::repl::{start_engines, Language};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize REPL engines
//!     let engines = start_engines()?;
//!
//!     // Execute Python code in REPL
//!     #[cfg(feature = "python")]
//!     let result = engines.eval("print('Hello from microsandbox!')", Language::Python)?;
//!
//!     engines.shutdown()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Command Execution
//!
//! ```no_run
//! use microsandbox_portal::command::create_command_executor;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a command executor
//!     let cmd_handle = create_command_executor();
//!
//!     // Execute a system command
//!     let (exit_code, output) = cmd_handle.execute("ls", vec!["-la"]).await?;
//!
//!     // Process the output
//!     for line in output {
//!         println!("[{}] {}",
//!                  if line.stream == microsandbox_portal::repl::Stream::Stdout { "stdout" } else { "stderr" },
//!                  line.text);
//!     }
//!
//!     Ok(())
//! }
//! ```

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

pub mod command;
pub mod fs;
pub mod repl;
