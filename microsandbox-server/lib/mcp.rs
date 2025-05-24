//! Model Context Protocol (MCP) implementation for microsandbox server.
//!
//! This module implements MCP endpoints served at the `/mcp` endpoint.
//! MCP is essentially JSON-RPC with specific method names and schemas.
//!
//! The module provides:
//! - MCP server initialization and capabilities
//! - Tool definitions for sandbox operations
//! - Prompt templates for common sandbox tasks
//! - Integration with existing sandbox management functions

use serde_json::json;
use tracing::{debug, error};

use crate::{
    error::ServerError,
    handler::{
        forward_rpc_to_portal, sandbox_get_metrics_impl, sandbox_start_impl, sandbox_stop_impl,
    },
    payload::{
        JsonRpcError, JsonRpcRequest, JsonRpcResponse, SandboxMetricsGetParams, SandboxStartParams,
        SandboxStopParams, JSONRPC_VERSION,
    },
    state::AppState,
    ServerResult,
};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// MCP protocol version
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Server information
const SERVER_NAME: &str = "microsandbox-server";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

//--------------------------------------------------------------------------------------------------
// Functions: Handlers
//--------------------------------------------------------------------------------------------------

/// Handle MCP initialize request
pub async fn handle_mcp_initialize(
    _state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    debug!("Handling MCP initialize request");

    let result = json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    });

    Ok(JsonRpcResponse::success(result, request.id))
}

/// Handle MCP list tools request
pub async fn handle_mcp_list_tools(
    _state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    debug!("Handling MCP list tools request");

    let tools = json!({
        "tools": [
            {
                "name": "sandbox_start",
                "description": "Start a new sandbox with specified configuration. This creates an isolated environment for code execution. IMPORTANT: Always stop the sandbox when done to prevent it from running indefinitely and consuming resources.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sandbox": {
                            "type": "string",
                            "description": "Name of the sandbox to start"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace for the sandbox"
                        },
                        "config": {
                            "type": "object",
                            "description": "Sandbox configuration",
                            "properties": {
                                "image": {
                                    "type": "string",
                                    "description": "Docker image to use"
                                },
                                "memory": {
                                    "type": "integer",
                                    "description": "Memory limit in MiB"
                                },
                                "cpus": {
                                    "type": "integer",
                                    "description": "Number of CPUs"
                                },
                                "volumes": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Volume mounts"
                                },
                                "ports": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Port mappings"
                                },
                                "envs": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Environment variables"
                                }
                            }
                        }
                    },
                    "required": ["sandbox", "namespace"]
                }
            },
            {
                "name": "sandbox_stop",
                "description": "Stop a running sandbox and clean up its resources. CRITICAL: Always call this when you're finished with a sandbox to prevent resource leaks and indefinite running. Failing to stop sandboxes will cause them to consume system resources unnecessarily.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sandbox": {
                            "type": "string",
                            "description": "Name of the sandbox to stop"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace of the sandbox"
                        }
                    },
                    "required": ["sandbox", "namespace"]
                }
            },
            {
                "name": "sandbox_run_code",
                "description": "Execute code in a running sandbox. PREREQUISITES: The target sandbox must be started first using sandbox_start - this will fail if the sandbox is not running. TIMING: Code execution is synchronous and may take time depending on complexity. Long-running code will block until completion or timeout.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sandbox": {
                            "type": "string",
                            "description": "Name of the sandbox (must be already started)"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace of the sandbox"
                        },
                        "code": {
                            "type": "string",
                            "description": "Code to execute"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (e.g., 'python', 'nodejs')"
                        }
                    },
                    "required": ["sandbox", "namespace", "code", "language"]
                }
            },
            {
                "name": "sandbox_run_command",
                "description": "Execute a command in a running sandbox. PREREQUISITES: The target sandbox must be started first using sandbox_start - this will fail if the sandbox is not running. TIMING: Command execution is synchronous and may take time depending on the command complexity. Long-running commands will block until completion or timeout.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sandbox": {
                            "type": "string",
                            "description": "Name of the sandbox (must be already started)"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace of the sandbox"
                        },
                        "command": {
                            "type": "string",
                            "description": "Command to execute"
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Command arguments"
                        }
                    },
                    "required": ["sandbox", "namespace", "command"]
                }
            },
            {
                "name": "sandbox_get_metrics",
                "description": "Get metrics and status for sandboxes including CPU usage, memory consumption, and running state. This tool can check the status of any sandbox regardless of whether it's running or not",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sandbox": {
                            "type": "string",
                            "description": "Optional specific sandbox name to get metrics for"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace to query (use '*' for all namespaces)"
                        }
                    },
                    "required": ["namespace"]
                }
            }
        ]
    });

    Ok(JsonRpcResponse::success(tools, request.id))
}

/// Handle MCP list prompts request
pub async fn handle_mcp_list_prompts(
    _state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    debug!("Handling MCP list prompts request");

    let prompts = json!({
        "prompts": [
            {
                "name": "create_python_sandbox",
                "description": "Create a Python development sandbox",
                "arguments": [
                    {
                        "name": "sandbox_name",
                        "description": "Name for the new sandbox",
                        "required": true
                    },
                    {
                        "name": "namespace",
                        "description": "Namespace for the sandbox",
                        "required": true
                    }
                ]
            },
            {
                "name": "create_node_sandbox",
                "description": "Create a Node.js development sandbox",
                "arguments": [
                    {
                        "name": "sandbox_name",
                        "description": "Name for the new sandbox",
                        "required": true
                    },
                    {
                        "name": "namespace",
                        "description": "Namespace for the sandbox",
                        "required": true
                    }
                ]
            }
        ]
    });

    Ok(JsonRpcResponse::success(prompts, request.id))
}

/// Handle MCP get prompt request
pub async fn handle_mcp_get_prompt(
    _state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    debug!("Handling MCP get prompt request");

    let params = request.params.as_object().ok_or_else(|| {
        ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
            "Request parameters must be an object".to_string(),
        ))
    })?;

    let prompt_name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
            "Missing required 'name' parameter".to_string(),
        ))
    })?;

    let arguments = params.get("arguments").and_then(|v| v.as_object());

    let result = match prompt_name {
        "create_python_sandbox" => {
            let sandbox_name = arguments
                .and_then(|args| args.get("sandbox_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("python-sandbox");
            let namespace = arguments
                .and_then(|args| args.get("namespace"))
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            json!({
                "description": "Create a Python development sandbox",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Create a Python sandbox named '{}' in namespace '{}' using the sandbox_start tool with the following configuration:\n\n\
                                - Image: microsandbox/python\n\
                                - Memory: 512 MiB\n\
                                - CPUs: 1\n\
                                - Working directory: /workspace\n\n\
                                This will set up a Python development environment ready for code execution.",
                                sandbox_name, namespace
                            )
                        }
                    }
                ]
            })
        }
        "create_node_sandbox" => {
            let sandbox_name = arguments
                .and_then(|args| args.get("sandbox_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("node-sandbox");
            let namespace = arguments
                .and_then(|args| args.get("namespace"))
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            json!({
                "description": "Create a Node.js development sandbox",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Create a Node.js sandbox named '{}' in namespace '{}' using the sandbox_start tool with the following configuration:\n\n\
                                - Image: microsandbox/node\n\
                                - Memory: 512 MiB\n\
                                - CPUs: 1\n\
                                - Working directory: /workspace\n\n\
                                This will set up a Node.js development environment ready for JavaScript execution.",
                                sandbox_name, namespace
                            )
                        }
                    }
                ]
            })
        }
        _ => {
            return Err(ServerError::NotFound(format!(
                "Prompt '{}' not found",
                prompt_name
            )));
        }
    };

    Ok(JsonRpcResponse::success(result, request.id))
}

/// Handle MCP call tool request
pub async fn handle_mcp_call_tool(
    state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    debug!("Handling MCP call tool request");

    let params = request.params.as_object().ok_or_else(|| {
        ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
            "Request parameters must be an object".to_string(),
        ))
    })?;

    let tool_name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
            "Missing required 'name' parameter".to_string(),
        ))
    })?;

    let arguments = params.get("arguments").ok_or_else(|| {
        ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
            "Missing required 'arguments' parameter".to_string(),
        ))
    })?;

    // Convert MCP tool calls to our internal JSON-RPC calls
    let internal_method = match tool_name {
        "sandbox_start" => "sandbox.start",
        "sandbox_stop" => "sandbox.stop",
        "sandbox_run_code" => "sandbox.repl.run",
        "sandbox_run_command" => "sandbox.command.run",
        "sandbox_get_metrics" => "sandbox.metrics.get",
        _ => {
            return Err(ServerError::NotFound(format!(
                "Tool '{}' not found",
                tool_name
            )));
        }
    };

    // Create internal JSON-RPC request
    let internal_request = JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        method: internal_method.to_string(),
        params: arguments.clone(),
        id: request.id.clone(),
    };

    // Handle the request using our existing infrastructure
    let internal_response = if matches!(internal_method, "sandbox.repl.run" | "sandbox.command.run")
    {
        // These need to be forwarded to the portal
        match forward_rpc_to_portal(state, internal_request).await {
            Ok((_, json_response)) => json_response.0,
            Err(e) => {
                error!("Failed to forward request to portal: {}", e);
                return Ok(JsonRpcResponse::error(
                    JsonRpcError {
                        code: -32603,
                        message: format!("Internal error: {}", e),
                        data: None,
                    },
                    request.id,
                ));
            }
        }
    } else {
        // These are handled locally - call the handler functions directly
        match internal_method {
            "sandbox.start" => {
                let params: SandboxStartParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        return JsonRpcResponse::error(
                            JsonRpcError {
                                code: -32602,
                                message: format!("Invalid parameters: {}", e),
                                data: None,
                            },
                            request.id.clone(),
                        );
                    })
                    .unwrap();

                match sandbox_start_impl(state, params).await {
                    Ok(result) => JsonRpcResponse::success(json!(result), request.id.clone()),
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError {
                            code: -32603,
                            message: format!("Sandbox start failed: {}", e),
                            data: None,
                        },
                        request.id.clone(),
                    ),
                }
            }
            "sandbox.stop" => {
                let params: SandboxStopParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        return JsonRpcResponse::error(
                            JsonRpcError {
                                code: -32602,
                                message: format!("Invalid parameters: {}", e),
                                data: None,
                            },
                            request.id.clone(),
                        );
                    })
                    .unwrap();

                match sandbox_stop_impl(state, params).await {
                    Ok(result) => JsonRpcResponse::success(json!(result), request.id.clone()),
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError {
                            code: -32603,
                            message: format!("Sandbox stop failed: {}", e),
                            data: None,
                        },
                        request.id.clone(),
                    ),
                }
            }
            "sandbox.metrics.get" => {
                let params: SandboxMetricsGetParams = serde_json::from_value(arguments.clone())
                    .map_err(|e| {
                        return JsonRpcResponse::error(
                            JsonRpcError {
                                code: -32602,
                                message: format!("Invalid parameters: {}", e),
                                data: None,
                            },
                            request.id.clone(),
                        );
                    })
                    .unwrap();

                match sandbox_get_metrics_impl(state, params).await {
                    Ok(result) => JsonRpcResponse::success(json!(result), request.id.clone()),
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError {
                            code: -32603,
                            message: format!("Get metrics failed: {}", e),
                            data: None,
                        },
                        request.id.clone(),
                    ),
                }
            }
            _ => JsonRpcResponse::error(
                JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", internal_method),
                    data: None,
                },
                request.id.clone(),
            ),
        }
    };

    // Convert the response to MCP format
    let mcp_result = if let Some(result) = internal_response.result {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                }
            ]
        })
    } else if let Some(error) = internal_response.error {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("Error: {}", error.message)
                }
            ],
            "isError": true
        })
    } else {
        json!({
            "content": [
                {
                    "type": "text",
                    "text": "No result returned"
                }
            ]
        })
    };

    Ok(JsonRpcResponse::success(mcp_result, request.id))
}

/// Handle MCP methods
pub async fn handle_mcp_method(
    state: AppState,
    request: JsonRpcRequest,
) -> ServerResult<JsonRpcResponse> {
    match request.method.as_str() {
        "initialize" => handle_mcp_initialize(state, request).await,
        "tools/list" => handle_mcp_list_tools(state, request).await,
        "tools/call" => handle_mcp_call_tool(state, request).await,
        "prompts/list" => handle_mcp_list_prompts(state, request).await,
        "prompts/get" => handle_mcp_get_prompt(state, request).await,
        _ => Err(ServerError::NotFound(format!(
            "MCP method '{}' not found",
            request.method
        ))),
    }
}
