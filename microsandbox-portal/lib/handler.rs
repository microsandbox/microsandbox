//! Request handlers for the microsandbox portal JSON-RPC server.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use tracing::debug;

use crate::{
    error::PortalError,
    payload::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, JSONRPC_VERSION},
};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// SharedState for the server
#[derive(Clone, Debug, Default)]
pub struct SharedState {
    /// Indicates if the server is ready to process requests
    pub ready: bool,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Handles JSON-RPC requests
pub async fn json_rpc_handler(
    State(state): State<SharedState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<impl IntoResponse, PortalError> {
    debug!(?request, "Received JSON-RPC request");

    // Check for required JSON-RPC fields
    if request.jsonrpc != JSONRPC_VERSION {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid or missing jsonrpc version field".to_string(),
            data: None,
        };
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::error(error, request.id.clone())),
        ));
    }

    let method = request.method.as_str();
    let id = request.id.clone();

    match method {
        "sandbox.run" => {
            // Call the sandbox_run_impl function
            let result = sandbox_run_impl(state, request.params).await?;

            // Create JSON-RPC response with success
            Ok((
                StatusCode::OK,
                Json(JsonRpcResponse::success(json!(result), id)),
            ))
        }
        "sandbox.command.run" => {
            // Call the sandbox_command_run_impl function
            let result = sandbox_command_run_impl(state, request.params).await?;

            // Create JSON-RPC response with success
            Ok((
                StatusCode::OK,
                Json(JsonRpcResponse::success(json!(result), id)),
            ))
        }
        _ => {
            let error = JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", method),
                data: None,
            };
            Ok((
                StatusCode::NOT_FOUND,
                Json(JsonRpcResponse::error(error, id)),
            ))
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions: Implementations
//--------------------------------------------------------------------------------------------------

/// Implementation for sandbox run method
async fn sandbox_run_impl(_state: SharedState, params: Value) -> Result<Value, PortalError> {
    debug!(?params, "Sandbox run method called");

    // This is just a placeholder implementation
    let result = json!({
        "success": true,
        "message": "Sandbox run method called successfully",
        "params": params,
    });

    Ok(result)
}

/// Implementation for sandbox command run method
async fn sandbox_command_run_impl(
    _state: SharedState,
    params: Value,
) -> Result<Value, PortalError> {
    debug!(?params, "Sandbox command run method called");

    // This is just a placeholder implementation
    let result = json!({
        "success": true,
        "message": "Sandbox command run method called successfully",
        "params": params,
    });

    Ok(result)
}
