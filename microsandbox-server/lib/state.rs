//! Application state management for the microsandbox server.
//!
//! This module handles:
//! - Global application state
//! - Configuration state management
//! - Thread-safe state sharing
//!
//! The module provides:
//! - Thread-safe application state container
//! - State initialization and access methods
//! - Configuration state management

use std::sync::Arc;

use getset::Getters;

use crate::config::Config;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Application state structure
#[derive(Clone, Getters)]
#[getset(get = "pub with_prefix")]
pub struct AppState {
    /// The application configuration
    config: Arc<Config>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl AppState {
    /// Create a new application state instance
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}
