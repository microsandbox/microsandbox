//! Network configuration types and fluent builders.

pub mod builder;
mod host;
mod resolver;
mod types;

//--------------------------------------------------------------------------------------------------
// Re-Exports
//--------------------------------------------------------------------------------------------------

pub use builder::*;
pub use host::*;
pub use resolver::*;
pub use types::*;
