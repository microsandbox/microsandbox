//! Request handlers for the microsandbox portal JSON-RPC server.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use tracing::debug;

use crate::{
    error::PortalError,
    payload::{
        JsonRpcError, JsonRpcRequest, JsonRpcResponse, SandboxCommandExecuteParams,
        SandboxReplRunParams, JSONRPC_VERSION,
    },
    portal::command::create_command_executor,
    state::SharedState,
};

#[cfg(any(feature = "python", feature = "nodejs"))]
use crate::portal::repl::{start_engines, Language};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Handles JSON-RPC requests
pub async fn json_rpc_handler(
    State(state): State<SharedState>,
    req: Json<JsonRpcRequest>,
) -> Result<impl IntoResponse, PortalError> {
    let request = req.0;
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
        "sandbox.repl.run" => {
            // Call the sandbox_run_impl function
            match sandbox_run_impl(state, request.params).await {
                Ok(result) => {
                    // Create JSON-RPC response with success
                    Ok((StatusCode::OK, Json(JsonRpcResponse::success(result, id))))
                }
                Err(e) => {
                    // Use our helper function to create the error response
                    Ok(create_error_response(e, id))
                }
            }
        }
        "sandbox.command.execute" => {
            // Call the sandbox_command_run_impl function
            match sandbox_command_run_impl(state, request.params).await {
                Ok(result) => {
                    // Create JSON-RPC response with success
                    Ok((StatusCode::OK, Json(JsonRpcResponse::success(result, id))))
                }
                Err(e) => {
                    // Use our helper function to create the error response
                    Ok(create_error_response(e, id))
                }
            }
        }
        _ => {
            let error = PortalError::MethodNotFound(format!("Method not found: {}", method));
            Ok(create_error_response(error, id))
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions: Implementations
//--------------------------------------------------------------------------------------------------

/// Implementation for sandbox run method
async fn sandbox_run_impl(_state: SharedState, params: Value) -> Result<Value, PortalError> {
    debug!(?params, "Sandbox run method called");

    // Deserialize parameters using the structured type
    let params: SandboxReplRunParams = serde_json::from_value(params)
        .map_err(|e| PortalError::JsonRpc(format!("Invalid parameters: {}", e)))?;

    // Convert language string to Language enum
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let language;

    match params.language.to_lowercase().as_str() {
        #[cfg(feature = "python")]
        "python" => language = Language::Python,
        #[cfg(feature = "nodejs")]
        "node" | "nodejs" | "javascript" => language = Language::Node,
        _ => {
            // Check if we're being asked for a language that is supported but not enabled via features
            let error_msg = match params.language.to_lowercase().as_str() {
                "python" => {
                    "Python language support is not enabled. Recompile with --features python"
                        .to_string()
                }
                "node" | "nodejs" | "javascript" => {
                    "Node.js language support is not enabled. Recompile with --features nodejs"
                        .to_string()
                }
                _ => format!("Unsupported language: {}", params.language),
            };
            return Err(PortalError::JsonRpc(error_msg));
        }
    };

    // Get or initialize engine handle
    // With tokio::sync::Mutex, we can safely .await while holding the lock
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let engine_handle = {
        // Get the current engine handle if it exists
        let mut lock = _state.engine_handle.lock().await;

        if let Some(ref handle) = *lock {
            handle.clone()
        } else {
            // Otherwise initialize a new engine
            let handle = start_engines()
                .await
                .map_err(|e| PortalError::Internal(format!("Failed to start engines: {}", e)))?;

            // Store the new handle in the shared state
            *lock = Some(handle.clone());

            handle
        }
    };

    #[cfg(any(feature = "python", feature = "nodejs"))]
    debug!("Language: {}", params.language);

    // Use a temporary identifier for evaluation
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let temp_id = uuid::Uuid::new_v4().to_string();

    // Execute the code in REPL
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let lines = engine_handle
        .eval(&params.code, language, &temp_id, params.timeout)
        .await
        .map_err(|e| PortalError::Internal(format!("REPL execution failed: {}", e)))?;

    #[cfg(any(feature = "python", feature = "nodejs"))]
    debug!("REPL execution produced {} output lines", lines.len());

    // Convert the lines to a format suitable for JSON
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let output_lines: Vec<Value> = lines
        .iter()
        .map(|line| {
            json!({
                "stream": match line.stream {
                    crate::portal::repl::Stream::Stdout => "stdout",
                    crate::portal::repl::Stream::Stderr => "stderr",
                },
                "text": line.text,
            })
        })
        .collect();

    // Construct the result JSON object with explicit String conversions
    #[cfg(any(feature = "python", feature = "nodejs"))]
    let result = json!({
        "status": "success".to_string(),
        "language": params.language.to_string(),
        "output": output_lines,
    });

    #[cfg(any(feature = "python", feature = "nodejs"))]
    debug!("Returning result with output: {}", result);

    #[cfg(any(feature = "python", feature = "nodejs"))]
    Ok(result)
}

/// Implementation for sandbox command execute method
async fn sandbox_command_run_impl(state: SharedState, params: Value) -> Result<Value, PortalError> {
    debug!(?params, "Sandbox command execute method called");

    // Deserialize parameters using the structured type
    let params: SandboxCommandExecuteParams = serde_json::from_value(params)
        .map_err(|e| PortalError::JsonRpc(format!("Invalid parameters: {}", e)))?;

    // Get or initialize command executor handle
    let cmd_handle = {
        // Get the current command handle if it exists
        let mut lock = state.command_handle.lock().await;

        if let Some(ref handle) = *lock {
            handle.clone()
        } else {
            // Otherwise initialize a new command executor
            let handle = create_command_executor();

            // Store the new handle in the shared state
            *lock = Some(handle.clone());

            handle
        }
    };

    // Execute the command
    let (exit_code, output_lines) = cmd_handle
        .execute(&params.command, params.args.clone(), params.timeout)
        .await
        .map_err(|e| PortalError::Internal(format!("Command execution failed: {}", e)))?;

    // Convert the output lines
    let formatted_lines = output_lines
        .iter()
        .map(|line| {
            json!({
                "stream": match line.stream {
                    crate::portal::repl::Stream::Stdout => "stdout",
                    crate::portal::repl::Stream::Stderr => "stderr",
                },
                "text": line.text,
            })
        })
        .collect::<Vec<Value>>();

    // Construct the result JSON object
    let result = json!({
        "command": params.command,
        "args": params.args,
        "exit_code": exit_code,
        "success": exit_code == 0,
        "output": formatted_lines,
    });

    debug!("Returning command result with output: {}", result);

    Ok(result)
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Helper function to create a JSON-RPC error response from a PortalError
fn create_error_response(error: PortalError, id: Value) -> (StatusCode, Json<JsonRpcResponse>) {
    // Determine appropriate JSON-RPC error code
    let code = match &error {
        PortalError::JsonRpc(_) => -32600,        // Invalid Request
        PortalError::MethodNotFound(_) => -32601, // Method not found
        PortalError::Parse(_) => -32700,          // Parse error
        PortalError::Internal(_) => -32603,       // Internal error
    };

    let json_rpc_error = JsonRpcError {
        code,
        message: error.to_string(),
        data: None,
    };

    // Return the properly formatted error response
    (
        StatusCode::BAD_REQUEST,
        Json(JsonRpcResponse::error(json_rpc_error, id)),
    )
}
