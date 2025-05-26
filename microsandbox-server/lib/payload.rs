//! Request and response payload definitions for the microsandbox server.
//!
//! This module defines the data structures for:
//! - API request payloads for sandbox operations
//! - API response payloads for operation results
//! - Error response structures and types
//! - Status message formatting
//!
//! The module implements:
//! - Request/response serialization and deserialization
//! - Structured error responses with type categorization
//! - Success message formatting for sandbox operations
//! - Detailed error information handling

use serde::{Deserialize, Serialize};
use serde_json::Value;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// JSON-RPC version - always "2.0"
pub const JSONRPC_VERSION: &str = "2.0";

//--------------------------------------------------------------------------------------------------
// Types: JSON-RPC Payloads
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

    /// Request ID (optional for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// JSON-RPC notification structure (no id field, no response expected)
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version, must be "2.0"
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Optional parameters for the method
    #[serde(default)]
    pub params: Value,
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

    /// Response ID (same as request ID, optional for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
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

/// JSON-RPC response or notification result
/// This enum allows handlers to return either a response (for regular requests)
/// or no response (for notifications) while maintaining type safety
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResponseOrNotification {
    /// A regular JSON-RPC response
    Response(JsonRpcResponse),

    /// A processed notification (no response should be sent)
    Notification(ProcessedNotification),
}

/// Represents a processed JSON-RPC notification (no response expected)
#[derive(Debug, Serialize)]
pub struct ProcessedNotification {
    /// Indicates this was a notification that was processed
    #[serde(skip)]
    pub processed: bool,
}

//--------------------------------------------------------------------------------------------------
// Types: Server Operations
//--------------------------------------------------------------------------------------------------

/// Request payload for starting a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStartParams {
    /// Sandbox name
    pub sandbox: String,

    /// Optional namespace
    pub namespace: String,

    /// Optional sandbox configuration
    pub config: Option<SandboxConfig>,
}

/// Request payload for stopping a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStopParams {
    /// Sandbox name
    pub sandbox: String,

    /// Optional namespace
    pub namespace: String,
}

/// Request payload for getting sandbox metrics
#[derive(Debug, Deserialize)]
pub struct SandboxMetricsGetParams {
    /// Optional sandbox name - if not provided, all sandboxes in the namespace will be included
    pub sandbox: Option<String>,

    /// Namespace - use "*" to get metrics from all namespaces
    pub namespace: String,
}

/// Configuration for a sandbox
/// Similar to microsandbox-core's Sandbox but with optional fields for update operations
#[derive(Debug, Deserialize)]
pub struct SandboxConfig {
    /// The image to use (optional for updates)
    pub image: Option<String>,

    /// The amount of memory in MiB to use
    pub memory: Option<u32>,

    /// The number of vCPUs to use
    pub cpus: Option<u8>,

    /// The volumes to mount
    #[serde(default)]
    pub volumes: Vec<String>,

    /// The ports to expose
    #[serde(default)]
    pub ports: Vec<String>,

    /// The environment variables to use
    #[serde(default)]
    pub envs: Vec<String>,

    /// The sandboxes to depend on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// The working directory to use
    pub workdir: Option<String>,

    /// The shell to use (optional for updates)
    pub shell: Option<String>,

    /// The scripts that can be run
    #[serde(default)]
    pub scripts: std::collections::HashMap<String, String>,

    /// The exec command to run
    pub exec: Option<String>,
    // SECURITY: Needs networking namespacing to be implemented
    // /// The network scope for the sandbox
    // pub scope: Option<String>,
}

//--------------------------------------------------------------------------------------------------
// Types: Portal-mirrored RPC Payloads
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
pub struct SandboxCommandRunParams {
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
            id: Some(id),
        }
    }

    /// Create a new JSON-RPC notification (no response expected)
    pub fn new_notification(method: String, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            params,
            id: None,
        }
    }

    /// Check if this is a notification (no id field)
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

impl ProcessedNotification {
    /// Create a new processed notification marker
    pub fn processed() -> Self {
        Self { processed: true }
    }
}

impl JsonRpcResponseOrNotification {
    /// Create a successful response
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self::Response(JsonRpcResponse::success(result, id))
    }

    /// Create an error response
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Self {
        Self::Response(JsonRpcResponse::error(error, id))
    }

    /// Create a response from a JsonRpcResponse
    pub fn response(response: JsonRpcResponse) -> Self {
        Self::Response(response)
    }

    /// Create a notification result (no response)
    pub fn notification(notification: ProcessedNotification) -> Self {
        Self::Notification(notification)
    }

    /// Create a no-response result for notifications (deprecated - use notification() instead)
    pub fn no_response() -> Self {
        Self::Notification(ProcessedNotification::processed())
    }
}

impl JsonRpcResponse {
    /// Create a new successful JSON-RPC response
    pub fn success(result: Value, id: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create a new error JSON-RPC response
    pub fn error(error: JsonRpcError, id: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Types: Responses
//--------------------------------------------------------------------------------------------------

/// Response type for regular message responses
#[derive(Debug, Serialize)]
pub struct RegularMessageResponse {
    /// Message indicating the status of the sandbox operation
    pub message: String,
}

/// System status response
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {}

/// Sandbox status response
#[derive(Debug, Serialize)]
pub struct SandboxStatusResponse {
    /// List of sandbox statuses
    pub sandboxes: Vec<SandboxStatus>,
}

/// Sandbox configuration response
#[derive(Debug, Serialize)]
pub struct SandboxConfigResponse {}

/// Status of an individual sandbox
#[derive(Debug, Serialize)]
pub struct SandboxStatus {
    /// Namespace the sandbox belongs to
    pub namespace: String,

    /// The name of the sandbox
    pub name: String,

    /// Whether the sandbox is running
    pub running: bool,

    /// CPU usage percentage
    pub cpu_usage: Option<f32>,

    /// Memory usage in MiB
    pub memory_usage: Option<u64>,

    /// Disk usage of the RW layer in bytes
    pub disk_usage: Option<u64>,
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl axum::response::IntoResponse for JsonRpcResponseOrNotification {
    fn into_response(self) -> axum::response::Response {
        match self {
            JsonRpcResponseOrNotification::Response(response) => {
                (axum::http::StatusCode::OK, axum::Json(response)).into_response()
            }
            JsonRpcResponseOrNotification::Notification(_notification) => {
                // For JSON-RPC notifications, send HTTP 200 with empty body
                // This satisfies the HTTP protocol requirement while sending no JSON-RPC response
                axum::http::StatusCode::OK.into_response()
            }
        }
    }
}
