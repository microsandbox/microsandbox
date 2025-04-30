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
// Types: REST API Requests
//--------------------------------------------------------------------------------------------------

/// Request payload for starting a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStartRequest {
    /// Sandbox name
    pub sandbox: String,

    /// Optional namespace
    pub namespace: String,

    /// Optional sandbox configuration
    pub config: Option<SandboxConfig>,
}

/// Request payload for stopping a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStopRequest {
    /// Sandbox name
    pub sandbox: String,

    /// Optional namespace
    pub namespace: String,
}

/// Request payload for getting sandbox status
#[derive(Debug, Deserialize)]
pub struct SandboxStatusRequest {
    /// Optional sandbox name - if not provided, all sandboxes in the namespace will be included
    pub sandbox: Option<String>,

    /// Namespace - use "*" to get status from all namespaces
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

    /// The network scope for the sandbox
    pub scope: Option<String>,
}

//--------------------------------------------------------------------------------------------------
// Types: JSON-RPC Payloads
//--------------------------------------------------------------------------------------------------

/// Generic JSON-RPC request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest<T> {
    /// JSON-RPC version
    pub jsonrpc: String,

    /// Method to call
    pub method: String,

    /// Parameters for the method
    pub params: T,

    /// Request ID
    pub id: Option<u64>,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,

    /// Result of the operation
    pub result: Value,

    /// Request ID
    pub id: Option<u64>,
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
