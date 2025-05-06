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

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{handler, middleware as app_middleware, state::AppState};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Create a new router with the given state
pub fn create_router(state: AppState) -> Router {
    // Create REST API routes - only health endpoint remains here
    let rest_api = Router::new().route("/health", get(handler::health));

    // Create JSON-RPC routes with authentication - a single endpoint that handles all RPC methods
    // This now mirrors the structure used in microsandbox-portal
    let rpc_api = Router::new()
        .route("/", post(handler::json_rpc_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_middleware::auth_middleware,
        ));

    // Combine all routes with logging middleware
    Router::new()
        .nest("/api/v1", rest_api)
        .nest("/api/v1/rpc", rpc_api)
        .layer(middleware::from_fn(app_middleware::logging_middleware))
        .with_state(state)
}
