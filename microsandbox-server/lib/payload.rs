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

//--------------------------------------------------------------------------------------------------
// Types: REST API Requests
//--------------------------------------------------------------------------------------------------

/// Request payload for starting a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStartRequest {
    /// Sandbox name
    pub sandbox_name: String,

    /// Optional namespace
    pub namespace: String,

    /// Optional sandbox configuration
    pub config: Option<SandboxConfig>,
}

/// Request payload for stopping a sandbox
#[derive(Debug, Deserialize)]
pub struct SandboxStopRequest {
    /// Sandbox name
    pub sandbox_name: String,

    /// Optional namespace
    pub namespace: String,
}

//--------------------------------------------------------------------------------------------------
// Types: JSON-RPC Payloads
//--------------------------------------------------------------------------------------------------

/// JSON-RPC request for running code in a sandbox
#[derive(Debug, Deserialize)]
pub struct RunCodeRequest {
    /// Code to execute
    pub code: String,

    /// Namespace for the sandbox
    pub namespace: String,

    /// Sandbox name
    pub sandbox_name: String,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse<T> {
    /// JSON-RPC version
    pub jsonrpc: String,

    /// Result of the operation
    pub result: T,

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
pub struct SandboxStatus {}

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
