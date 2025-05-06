//! JSON-RPC payload structures for microsandbox portal.

use serde::{Deserialize, Serialize};
use serde_json::Value;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// JSON-RPC version - always "2.0"
pub const JSONRPC_VERSION: &str = "2.0";

//--------------------------------------------------------------------------------------------------
// Types: JSON-RPC Structures
//--------------------------------------------------------------------------------------------------

/// JSON-RPC request structure
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version, must be "2.0"
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Optional parameters for the method
    #[serde(default)]
    pub params: Value,

    /// Request ID
    pub id: Value,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version, always "2.0"
    pub jsonrpc: String,

    /// Result of the method execution (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error details (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Response ID (same as request ID)
    pub id: Value,
}

/// JSON-RPC error structure
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Optional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

//--------------------------------------------------------------------------------------------------
// Types: REST API Requests
//--------------------------------------------------------------------------------------------------

/// Request parameters for executing code in a REPL environment
#[derive(Debug, Deserialize, Serialize)]
pub struct SandboxReplRunParams {
    /// Code to be executed
    pub code: String,

    /// Programming language to use for execution
    pub language: String,
}

/// Request parameters for retrieving output from a previous REPL execution
#[derive(Debug, Deserialize, Serialize)]
pub struct SandboxReplGetOutputParams {
    /// Unique identifier for the execution
    pub execution_id: String,
}

/// Request parameters for executing a shell command
#[derive(Debug, Deserialize, Serialize)]
pub struct SandboxCommandExecuteParams {
    /// Command to execute
    pub command: String,

    /// Optional arguments for the command
    #[serde(default)]
    pub args: Vec<String>,
}

/// Request parameters for retrieving output from a previous command execution
#[derive(Debug, Deserialize, Serialize)]
pub struct SandboxCommandGetOutputParams {
    /// Unique identifier for the command execution
    pub execution_id: String,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: String, params: Value, id: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            params,
            id,
        }
    }
}

impl JsonRpcResponse {
    /// Create a new successful JSON-RPC response
    pub fn success(result: Value, id: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create a new error JSON-RPC response
    pub fn error(error: JsonRpcError, id: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}
