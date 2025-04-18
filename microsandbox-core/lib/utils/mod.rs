//! Utility functions and types.

pub mod conversion;
pub mod env;
pub mod file;
pub mod path;

// The CLI visualisation utilities (progress bars, spinners, etc.) are only
// compiled when the `cli-viz` feature is enabled so we gate the module here.
#[cfg(feature = "cli-viz")]
pub mod viz;

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

pub use conversion::*;
pub use env::*;
pub use file::*;
pub use path::*;
