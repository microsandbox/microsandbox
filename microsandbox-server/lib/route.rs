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
    // Create REST API routes
    let rest_api = Router::new()
        .route("/sandbox-start", post(handler::sandbox_start))
        .route("/sandbox-stop", post(handler::sandbox_stop))
        .route("/sandbox-config", get(handler::sandbox_config))
        .route("/health", get(handler::health))
        .route("/system-status", get(handler::system_status))
        .route("/sandbox-status", get(handler::sandbox_status));

    // Create JSON-RPC routes
    let rpc_api = Router::new().route("/run", post(handler::run_code));

    // Create proxy routes - directly on the root path as requested
    let proxy_routes = Router::new()
        .route(
            "/{namespace}/{sandbox_name}/{*path}",
            get(handler::proxy_request),
        )
        .fallback(handler::proxy_fallback)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_middleware::proxy_middleware,
        ));

    // Combine all routes with logging middleware
    Router::new()
        .nest("/api/v1", rest_api)
        .nest("/api/v1/rpc", rpc_api)
        .merge(proxy_routes) // Use merge instead of nest to have the proxy routes directly on root
        .layer(middleware::from_fn(app_middleware::logging_middleware))
        .with_state(state)
}
