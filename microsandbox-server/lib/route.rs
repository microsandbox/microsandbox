//! Router configuration for the microsandbox server.
//!
//! This module handles:
//! - API route definitions
//! - Router configuration and setup
//! - Request routing and handling
//!
//! The module provides:
//! - Router creation and configuration
//! - Route handlers and middleware integration
//! - State management for routes

use axum::Router;

use crate::state::AppState;

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Create a new router with the given state
pub fn create_router(_state: AppState) -> Router {
    let router = Router::new();
    router
}
