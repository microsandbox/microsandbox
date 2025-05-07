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
use tokio::sync::RwLock;

use getset::Getters;

use crate::{
    config::Config,
    port::{PortManager, LOCALHOST_IP},
    ServerError, ServerResult,
};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Application state structure
#[derive(Clone, Getters)]
#[getset(get = "pub with_prefix")]
pub struct AppState {
    /// The application configuration
    config: Arc<Config>,

    /// The port manager for handling sandbox port assignments
    port_manager: Arc<RwLock<PortManager>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl AppState {
    /// Create a new application state instance
    pub fn new(config: Arc<Config>, port_manager: Arc<RwLock<PortManager>>) -> Self {
        Self {
            config,
            port_manager,
        }
    }

    /// Get a sandbox's portal URL
    ///
    /// Returns an error if no port is assigned for the given sandbox
    pub async fn get_portal_url_for_sandbox(
        &self,
        namespace: &str,
        sandbox_name: &str,
    ) -> ServerResult<String> {
        let port_manager = self.port_manager.read().await;
        let key = format!("{}/{}", namespace, sandbox_name);

        if let Some(port) = port_manager.get_port(&key) {
            Ok(format!("http://{}:{}", LOCALHOST_IP, port))
        } else {
            Err(ServerError::InternalError(format!(
                "No portal port assigned for sandbox {}",
                key
            )))
        }
    }
}
