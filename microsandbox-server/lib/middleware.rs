//! Middleware components for the microsandbox server.
//!
//! This module handles:
//! - Request/response middleware
//! - Authentication and authorization
//! - Request tracing and logging
//! - Error handling
//!
//! The module provides:
//! - Middleware components for common operations
//! - Authentication middleware for API security
//! - Logging and tracing middleware

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    middleware::Next,
    response::IntoResponse,
};

use crate::state::AppState;

//--------------------------------------------------------------------------------------------------
// Middleware Functions
//--------------------------------------------------------------------------------------------------

/// Proxy middleware for forwarding requests to a target service
pub async fn proxy_middleware(
    State(_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    // Default to passing the request to the next handler
    // This middleware can be extended to implement actual proxying logic
    next.run(req).await
}

/// Convert a URI to a proxied URI targeting a sandbox
pub fn proxy_uri(original_uri: Uri, namespace: &str, sandbox_name: &str) -> Uri {
    // In a real implementation, you would:
    // 1. Look up the sandbox's address from a registry or state
    // 2. Construct a new URI that points to the sandbox
    // 3. Return the new URI for proxying

    // For demonstration purposes, we'll construct a simple URI
    // In production, you would get this from a sandbox registry
    let target_host = format!("sandbox-{}.{}.internal", sandbox_name, namespace);

    let uri_string = if let Some(path_and_query) = original_uri.path_and_query() {
        format!("http://{}:{}{}", target_host, 8080, path_and_query)
    } else {
        format!("http://{}:{}/", target_host, 8080)
    };

    // Try to parse the string into a URI
    // In case of errors, fallback to a default URI
    uri_string
        .parse()
        .unwrap_or_else(|_| "http://localhost:8080/".parse().unwrap())
}

/// Log incoming requests
pub async fn logging_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Log the request
    tracing::info!("Request: {} {}", method, uri);

    // Process the request
    let response = next.run(req).await;

    // Log the response
    tracing::info!("Response: {} {}: {}", method, uri, response.status());

    Ok(response)
}
