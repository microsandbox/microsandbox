//! Request handlers for the microsandbox server.
//!
//! This module implements:
//! - API endpoint handlers
//! - Request processing logic
//! - Response formatting
//!
//! The module provides:
//! - Handler functions for API routes
//! - Request validation and processing
//! - Response generation and error handling

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use microsandbox_core::management::{menv, orchestra};
use microsandbox_utils::{DEFAULT_CONFIG, MICROSANDBOX_CONFIG_FILENAME};
use serde_yaml;
use std::path::PathBuf;
use tokio::fs as tokio_fs;

use crate::{
    error::ServerError,
    middleware,
    payload::{
        JsonRpcResponse, RegularMessageResponse, SandboxStartRequest, SandboxStatusRequest,
        SandboxStopRequest,
    },
    state::AppState,
    SandboxStatus, SandboxStatusResponse, ServerResult,
};

//--------------------------------------------------------------------------------------------------
// Functions: REST API Handlers
//--------------------------------------------------------------------------------------------------

/// Handler for health check
pub async fn health() -> ServerResult<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(RegularMessageResponse {
            message: "Service is healthy".to_string(),
        }),
    ))
}

//--------------------------------------------------------------------------------------------------
// Functions: JSON-RPC Handlers
//--------------------------------------------------------------------------------------------------

/// Main JSON-RPC handler that dispatches to the appropriate method
pub async fn json_rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> ServerResult<impl IntoResponse> {
    // Extract method field from the request
    let method = payload.get("method").and_then(|m| m.as_str());

    // Check for required JSON-RPC fields
    if payload.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Invalid or missing jsonrpc version field".to_string(),
            ),
        ));
    }

    let id = payload.get("id").cloned();
    let id_value = id.and_then(|i| i.as_u64());

    match method {
        Some("sandbox.start") => {
            // Parse the params into a SandboxStartRequest
            let params = payload.get("params").ok_or_else(|| {
                ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                    "Missing params field".to_string(),
                ))
            })?;

            let start_request: SandboxStartRequest = serde_json::from_value(params.clone())
                .map_err(|e| {
                    ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                        format!("Invalid params for sandbox.start: {}", e),
                    ))
                })?;

            // Access validation can be done here using the headers in the original request
            // We can extract the API key from headers and validate it has access to the
            // requested namespace
            // This is now handled by the auth middleware

            // Call the sandbox_up_impl function
            let result = sandbox_start_impl(state, start_request).await?;

            // Create JSON-RPC response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: serde_json::to_value(result).map_err(|e| {
                    ServerError::InternalError(format!("JSON serialization error: {}", e))
                })?,
                id: id_value,
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Some("sandbox.stop") => {
            // Parse the params into a SandboxStopRequest
            let params = payload.get("params").ok_or_else(|| {
                ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                    "Missing params field".to_string(),
                ))
            })?;

            let stop_request: SandboxStopRequest =
                serde_json::from_value(params.clone()).map_err(|e| {
                    ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                        format!("Invalid params for sandbox.stop: {}", e),
                    ))
                })?;

            // Call the sandbox_down_impl function
            let result = sandbox_stop_impl(state, stop_request).await?;

            // Create JSON-RPC response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: serde_json::to_value(result).map_err(|e| {
                    ServerError::InternalError(format!("JSON serialization error: {}", e))
                })?,
                id: id_value,
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Some("sandbox.getStatus") => {
            // Parse the params into a SandboxStatusRequest
            let params = payload.get("params").ok_or_else(|| {
                ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                    "Missing params field".to_string(),
                ))
            })?;

            let status_request: SandboxStatusRequest = serde_json::from_value(params.clone())
                .map_err(|e| {
                    ServerError::ValidationError(crate::error::ValidationError::InvalidInput(
                        format!("Invalid params for sandbox.getStatus: {}", e),
                    ))
                })?;

            // Call the sandbox_status_impl function with state and request
            let result = sandbox_get_status_impl(state.clone(), status_request).await?;

            // Create JSON-RPC response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: serde_json::to_value(result).map_err(|e| {
                    ServerError::InternalError(format!("JSON serialization error: {}", e))
                })?,
                id: id_value,
            };

            Ok((StatusCode::OK, Json(response)))
        }
        Some(unknown_method) => Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(format!(
                "Unknown method: {}",
                unknown_method
            )),
        )),
        None => Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput("Missing method field".to_string()),
        )),
    }
}

/// Implementation for starting a sandbox
async fn sandbox_start_impl(state: AppState, params: SandboxStartRequest) -> ServerResult<String> {
    // Validate sandbox name and namespace
    validate_sandbox_name(&params.sandbox)?;
    validate_namespace(&params.namespace)?;

    let namespace_dir = state
        .get_config()
        .get_namespace_dir()
        .join(&params.namespace);
    let config_file = MICROSANDBOX_CONFIG_FILENAME;
    let config_path = namespace_dir.join(config_file);
    let sandbox = &params.sandbox;

    // Create namespace directory if it doesn't exist
    if !namespace_dir.exists() {
        tokio_fs::create_dir_all(&namespace_dir)
            .await
            .map_err(|e| {
                ServerError::InternalError(format!("Failed to create namespace directory: {}", e))
            })?;

        // Initialize microsandbox environment
        menv::initialize(Some(namespace_dir.clone()))
            .await
            .map_err(|e| {
                ServerError::InternalError(format!(
                    "Failed to initialize microsandbox environment: {}",
                    e
                ))
            })?;
    }

    // Check if we have a valid configuration to proceed with
    let has_config_in_request = params
        .config
        .as_ref()
        .and_then(|c| c.image.as_ref())
        .is_some();
    let has_existing_config_file = config_path.exists();

    if !has_config_in_request && !has_existing_config_file {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(format!(
                "No configuration provided and no existing configuration found for sandbox '{}'",
                sandbox
            )),
        ));
    }

    // If we're relying on existing config, verify that the sandbox exists in it
    if !has_config_in_request && has_existing_config_file {
        // Read the existing config
        let config_content = tokio_fs::read_to_string(&config_path).await.map_err(|e| {
            ServerError::InternalError(format!("Failed to read config file: {}", e))
        })?;

        // Parse the config as YAML
        let config_yaml: serde_yaml::Value =
            serde_yaml::from_str(&config_content).map_err(|e| {
                ServerError::InternalError(format!("Failed to parse config file: {}", e))
            })?;

        // Check if the sandboxes configuration exists and contains our sandbox
        let has_sandbox_config = config_yaml
            .get("sandboxes")
            .and_then(|sandboxes| sandboxes.get(sandbox))
            .is_some();

        if !has_sandbox_config {
            return Err(ServerError::ValidationError(
                crate::error::ValidationError::InvalidInput(format!(
                    "Sandbox '{}' not found in existing configuration",
                    sandbox
                )),
            ));
        }
    }

    // If config is provided and we have an image, we need to update the config file
    if let Some(config) = &params.config {
        if config.image.is_some() {
            // Ensure config file exists
            if !config_path.exists() {
                tokio_fs::write(&config_path, DEFAULT_CONFIG)
                    .await
                    .map_err(|e| {
                        ServerError::InternalError(format!("Failed to create config file: {}", e))
                    })?;
            }

            // Read the existing config
            let config_content = tokio_fs::read_to_string(&config_path).await.map_err(|e| {
                ServerError::InternalError(format!("Failed to read config file: {}", e))
            })?;

            // Parse the config as YAML
            let mut config_yaml: serde_yaml::Value = serde_yaml::from_str(&config_content)
                .map_err(|e| {
                    ServerError::InternalError(format!("Failed to parse config file: {}", e))
                })?;

            // Ensure sandboxes field exists
            if !config_yaml.is_mapping() {
                config_yaml = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
            }

            let config_map = config_yaml.as_mapping_mut().unwrap();
            if !config_map.contains_key(&serde_yaml::Value::String("sandboxes".to_string())) {
                config_map.insert(
                    serde_yaml::Value::String("sandboxes".to_string()),
                    serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                );
            }

            // Get or create the sandboxes mapping
            let sandboxes_map = config_map
                .get_mut(&serde_yaml::Value::String("sandboxes".to_string()))
                .unwrap()
                .as_mapping_mut()
                .unwrap();

            // Create sandbox entry
            let mut sandbox_map = serde_yaml::Mapping::new();

            // Set required image field
            if let Some(image) = &config.image {
                sandbox_map.insert(
                    serde_yaml::Value::String("image".to_string()),
                    serde_yaml::Value::String(image.clone()),
                );
            }

            // Set optional fields
            if let Some(memory) = config.memory {
                sandbox_map.insert(
                    serde_yaml::Value::String("memory".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(memory)),
                );
            }

            if let Some(cpus) = config.cpus {
                sandbox_map.insert(
                    serde_yaml::Value::String("cpus".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(cpus)),
                );
            }

            if !config.volumes.is_empty() {
                let volumes_array = config
                    .volumes
                    .iter()
                    .map(|v| serde_yaml::Value::String(v.clone()))
                    .collect::<Vec<_>>();
                sandbox_map.insert(
                    serde_yaml::Value::String("volumes".to_string()),
                    serde_yaml::Value::Sequence(volumes_array),
                );
            }

            if !config.ports.is_empty() {
                let ports_array = config
                    .ports
                    .iter()
                    .map(|p| serde_yaml::Value::String(p.clone()))
                    .collect::<Vec<_>>();
                sandbox_map.insert(
                    serde_yaml::Value::String("ports".to_string()),
                    serde_yaml::Value::Sequence(ports_array),
                );
            }

            if !config.envs.is_empty() {
                let envs_array = config
                    .envs
                    .iter()
                    .map(|e| serde_yaml::Value::String(e.clone()))
                    .collect::<Vec<_>>();
                sandbox_map.insert(
                    serde_yaml::Value::String("envs".to_string()),
                    serde_yaml::Value::Sequence(envs_array),
                );
            }

            if !config.depends_on.is_empty() {
                let depends_on_array = config
                    .depends_on
                    .iter()
                    .map(|d| serde_yaml::Value::String(d.clone()))
                    .collect::<Vec<_>>();
                sandbox_map.insert(
                    serde_yaml::Value::String("depends_on".to_string()),
                    serde_yaml::Value::Sequence(depends_on_array),
                );
            }

            if let Some(workdir) = &config.workdir {
                sandbox_map.insert(
                    serde_yaml::Value::String("workdir".to_string()),
                    serde_yaml::Value::String(workdir.clone()),
                );
            }

            if let Some(shell) = &config.shell {
                sandbox_map.insert(
                    serde_yaml::Value::String("shell".to_string()),
                    serde_yaml::Value::String(shell.clone()),
                );
            }

            if !config.scripts.is_empty() {
                let mut scripts_map = serde_yaml::Mapping::new();
                for (script_name, script) in &config.scripts {
                    scripts_map.insert(
                        serde_yaml::Value::String(script_name.clone()),
                        serde_yaml::Value::String(script.clone()),
                    );
                }
                sandbox_map.insert(
                    serde_yaml::Value::String("scripts".to_string()),
                    serde_yaml::Value::Mapping(scripts_map),
                );
            }

            if let Some(exec) = &config.exec {
                sandbox_map.insert(
                    serde_yaml::Value::String("exec".to_string()),
                    serde_yaml::Value::String(exec.clone()),
                );
            }

            if let Some(scope) = &config.scope {
                sandbox_map.insert(
                    serde_yaml::Value::String("scope".to_string()),
                    serde_yaml::Value::String(scope.clone()),
                );
            }

            // Replace or add the sandbox in the config
            sandboxes_map.insert(
                serde_yaml::Value::String(sandbox.clone()),
                serde_yaml::Value::Mapping(sandbox_map),
            );

            // Write the updated config back to the file
            let updated_config = serde_yaml::to_string(&config_yaml).map_err(|e| {
                ServerError::InternalError(format!("Failed to serialize config: {}", e))
            })?;

            tokio_fs::write(&config_path, updated_config)
                .await
                .map_err(|e| {
                    ServerError::InternalError(format!("Failed to write config file: {}", e))
                })?;
        }
    }

    // If sandbox is already running, stop it first
    if let Err(e) = orchestra::down(
        vec![sandbox.clone()],
        Some(&namespace_dir),
        Some(config_file),
    )
    .await
    {
        // Log the error but continue - this might just mean the sandbox wasn't running
        tracing::warn!("Error stopping sandbox {}: {}", sandbox, e);
    }

    // Start the sandbox
    orchestra::up(
        vec![sandbox.clone()],
        Some(&namespace_dir),
        Some(config_file),
    )
    .await
    .map_err(|e| {
        ServerError::InternalError(format!("Failed to start sandbox {}: {}", params.sandbox, e))
    })?;

    // Return success message
    Ok(format!("Sandbox {} started successfully", params.sandbox))
}

/// Implementation for stopping a sandbox
async fn sandbox_stop_impl(state: AppState, params: SandboxStopRequest) -> ServerResult<String> {
    // Validate sandbox name and namespace
    validate_sandbox_name(&params.sandbox)?;
    validate_namespace(&params.namespace)?;

    let namespace_dir = state
        .get_config()
        .get_namespace_dir()
        .join(&params.namespace);
    let config_file = MICROSANDBOX_CONFIG_FILENAME;
    let sandbox = &params.sandbox;

    // Verify that the namespace directory exists
    if !namespace_dir.exists() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(format!(
                "Namespace directory '{}' does not exist",
                params.namespace
            )),
        ));
    }

    // Verify that the config file exists
    let config_path = namespace_dir.join(config_file);
    if !config_path.exists() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(format!(
                "Configuration file not found for namespace '{}'",
                params.namespace
            )),
        ));
    }

    // Stop the sandbox using orchestra::down
    orchestra::down(
        vec![sandbox.clone()],
        Some(&namespace_dir),
        Some(config_file),
    )
    .await
    .map_err(|e| {
        ServerError::InternalError(format!("Failed to stop sandbox {}: {}", params.sandbox, e))
    })?;

    // Return success message
    Ok(format!("Sandbox {} stopped successfully", params.sandbox))
}

/// Implementation for sandbox status
async fn sandbox_get_status_impl(
    state: AppState,
    params: SandboxStatusRequest,
) -> ServerResult<SandboxStatusResponse> {
    // Validate namespace - special handling for '*' wildcard
    if params.namespace != "*" {
        validate_namespace(&params.namespace)?;
    }

    // Validate sandbox name if provided
    if let Some(sandbox) = &params.sandbox {
        validate_sandbox_name(sandbox)?;
    }

    let namespaces_dir = state.get_config().get_namespace_dir();

    // Check if the namespaces directory exists
    if !namespaces_dir.exists() {
        return Err(ServerError::InternalError(format!(
            "Namespaces directory '{}' does not exist",
            namespaces_dir.display()
        )));
    }

    // Get all sandboxes statuses based on the request
    let mut all_statuses = Vec::new();

    // If namespace is "*", get statuses from all namespaces
    if params.namespace == "*" {
        // Read namespaces directory
        let mut entries = tokio::fs::read_dir(&namespaces_dir).await.map_err(|e| {
            ServerError::InternalError(format!("Failed to read namespaces directory: {}", e))
        })?;

        // Process each namespace directory
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            ServerError::InternalError(format!("Failed to read namespace directory entry: {}", e))
        })? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let namespace = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Get statuses for this namespace, filtered by sandbox name if provided
            let sandbox_names = if let Some(sandbox) = &params.sandbox {
                vec![sandbox.clone()]
            } else {
                vec![]
            };

            match orchestra::status(sandbox_names, Some(&path), None).await {
                Ok(statuses) => {
                    for status in statuses {
                        // Convert from orchestra::SandboxStatus to our SandboxStatus
                        all_statuses.push(SandboxStatus {
                            namespace: namespace.clone(),
                            name: status.name,
                            running: status.running,
                            cpu_usage: status.cpu_usage,
                            memory_usage: status.memory_usage,
                            disk_usage: status.disk_usage,
                        });
                    }
                }
                Err(e) => {
                    // Log the error but continue with other namespaces
                    tracing::warn!("Error getting status for namespace {}: {}", namespace, e);
                }
            }
        }
    } else {
        // Get status for a specific namespace
        let namespace_dir = namespaces_dir.join(&params.namespace);

        // Check if the namespace directory exists
        if !namespace_dir.exists() {
            return Err(ServerError::ValidationError(
                crate::error::ValidationError::InvalidInput(format!(
                    "Namespace directory '{}' does not exist",
                    params.namespace
                )),
            ));
        }

        // Get statuses for this namespace, filtered by sandbox name if provided
        let sandbox_names = if let Some(sandbox) = &params.sandbox {
            vec![sandbox.clone()]
        } else {
            vec![]
        };

        match orchestra::status(sandbox_names, Some(&namespace_dir), None).await {
            Ok(statuses) => {
                for status in statuses {
                    // Convert from orchestra::SandboxStatus to our SandboxStatus
                    all_statuses.push(SandboxStatus {
                        namespace: params.namespace.clone(),
                        name: status.name,
                        running: status.running,
                        cpu_usage: status.cpu_usage,
                        memory_usage: status.memory_usage,
                        disk_usage: status.disk_usage,
                    });
                }
            }
            Err(e) => {
                return Err(ServerError::InternalError(format!(
                    "Error getting status for namespace {}: {}",
                    params.namespace, e
                )));
            }
        }
    }

    Ok(SandboxStatusResponse {
        sandboxes: all_statuses,
    })
}

//--------------------------------------------------------------------------------------------------
// Functions: Proxy Handlers
//--------------------------------------------------------------------------------------------------

/// Handler for proxy requests
pub async fn proxy_request(
    State(_state): State<AppState>,
    Path((namespace, sandbox, path)): Path<(String, String, PathBuf)>,
    req: Request<Body>,
) -> ServerResult<impl IntoResponse> {
    // In a real implementation, this would use the middleware::proxy_uri function
    // to determine the target URI and then forward the request

    let path_str = path.display().to_string();

    // Calculate target URI using our middleware function
    let original_uri = req.uri().clone();
    let _target_uri = middleware::proxy_uri(original_uri, &namespace, &sandbox);

    // In a production system, this handler would forward the request to the target URI
    // For now, we'll just return information about what would be proxied

    let response = format!(
        "Axum Proxy Request\n\nNamespace: {}\nSandbox: {}\nPath: {}\nMethod: {}\nHeaders: {:?}",
        namespace,
        sandbox,
        path_str,
        req.method(),
        req.headers()
    );

    let result = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Body::from(response))
        .unwrap();

    Ok(result)
}

/// Fallback handler for proxy requests
pub async fn proxy_fallback() -> ServerResult<impl IntoResponse> {
    Ok((StatusCode::NOT_FOUND, "Resource not found"))
}

//--------------------------------------------------------------------------------------------------
// Functions: Helpers
//--------------------------------------------------------------------------------------------------

/// Validates a sandbox name
fn validate_sandbox_name(name: &str) -> ServerResult<()> {
    // Check name length
    if name.is_empty() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput("Sandbox name cannot be empty".to_string()),
        ));
    }

    if name.len() > 63 {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Sandbox name cannot exceed 63 characters".to_string(),
            ),
        ));
    }

    // Check name characters
    let valid_chars = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if !valid_chars {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Sandbox name can only contain alphanumeric characters, hyphens, or underscores"
                    .to_string(),
            ),
        ));
    }

    // Must start with an alphanumeric character
    if !name.chars().next().unwrap().is_ascii_alphanumeric() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Sandbox name must start with an alphanumeric character".to_string(),
            ),
        ));
    }

    Ok(())
}

/// Validates a namespace
fn validate_namespace(namespace: &str) -> ServerResult<()> {
    // Check namespace length
    if namespace.is_empty() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput("Namespace cannot be empty".to_string()),
        ));
    }

    if namespace.len() > 63 {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Namespace cannot exceed 63 characters".to_string(),
            ),
        ));
    }

    // Check for wildcard namespace - only valid for queries, not for creation
    if namespace == "*" {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Wildcard namespace (*) is not valid for sandbox creation".to_string(),
            ),
        ));
    }

    // Check namespace characters
    let valid_chars = namespace
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if !valid_chars {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Namespace can only contain alphanumeric characters, hyphens, or underscores"
                    .to_string(),
            ),
        ));
    }

    // Must start with an alphanumeric character
    if !namespace.chars().next().unwrap().is_ascii_alphanumeric() {
        return Err(ServerError::ValidationError(
            crate::error::ValidationError::InvalidInput(
                "Namespace must start with an alphanumeric character".to_string(),
            ),
        ));
    }

    Ok(())
}
