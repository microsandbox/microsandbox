//! Server module for Microsandbox's remote management functionality.
//!
//! This module implements a reverse proxy server that enables remote management of Microsandbox sandboxes
//! through a REST API. It provides a secure and controlled way to:
//! - Start and stop sandboxes
//! - Monitor sandbox status and health
//! - Apply configuration changes
//! - View logs and metrics

mod api;
mod data;

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

pub use api::*;
pub use data::*;
