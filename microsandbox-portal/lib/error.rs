//! Error handling for microsandbox portal.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::payload::JsonRpcError;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Main error type for microsandbox portal
#[derive(Debug, Error)]
pub enum PortalError {
    /// Error related to JSON-RPC protocol
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    /// Method not found
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Error during parsing
    #[error("Parse error: {0}")]
    Parse(String),
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl IntoResponse for PortalError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            PortalError::JsonRpc(message) => {
                let error = JsonRpcError {
                    code: -32600,
                    message,
                    data: None,
                };
                (StatusCode::BAD_REQUEST, error)
            }
            PortalError::MethodNotFound(message) => {
                let error = JsonRpcError {
                    code: -32601,
                    message,
                    data: None,
                };
                (StatusCode::NOT_FOUND, error)
            }
            PortalError::Parse(message) => {
                let error = JsonRpcError {
                    code: -32700,
                    message,
                    data: None,
                };
                (StatusCode::BAD_REQUEST, error)
            }
            PortalError::Internal(message) => {
                let error = JsonRpcError {
                    code: -32603,
                    message,
                    data: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
        };

        (status, Json(error_response)).into_response()
    }
}
