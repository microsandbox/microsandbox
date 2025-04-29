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
        JsonRpcResponse, RegularMessageResponse, RunCodeRequest, SandboxStartRequest,
        SandboxStopRequest,
    },
    state::AppState,
    SandboxConfigResponse, SandboxStatus, SandboxStatusResponse, ServerResult,
    SystemStatusResponse,
};

//--------------------------------------------------------------------------------------------------
// REST API Handlers
//--------------------------------------------------------------------------------------------------

/// Handler for starting a sandbox
pub async fn sandbox_up(
    State(state): State<AppState>,
    Json(payload): Json<SandboxStartRequest>,
) -> ServerResult<impl IntoResponse> {
    let namespace_dir = state
        .get_config()
        .get_namespace_dir()
        .join(&payload.namespace);
    let config_file = MICROSANDBOX_CONFIG_FILENAME;
    let config_path = namespace_dir.join(config_file);
    let sandbox = &payload.sandbox;

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
    let has_config_in_request = payload
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
    if let Some(config) = &payload.config {
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
        ServerError::InternalError(format!("Failed to start sandbox {}: {}", payload.sandbox, e))
    })?;

    Ok((
        StatusCode::OK,
        Json(RegularMessageResponse {
            message: format!("Sandbox {} started successfully", payload.sandbox),
        }),
    ))
}

/// Handler for stopping a sandbox
pub async fn sandbox_down(
    State(_state): State<AppState>,
    Json(payload): Json<SandboxStopRequest>,
) -> ServerResult<impl IntoResponse> {
    // TODO: Implement sandbox stop logic
    Ok((
        StatusCode::OK,
        Json(RegularMessageResponse {
            message: format!("Sandbox stop requested for: {}", payload.sandbox),
        }),
    ))
}

/// Handler for health check
pub async fn health() -> ServerResult<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(RegularMessageResponse {
            message: "Service is healthy".to_string(),
        }),
    ))
}

/// Handler for system status
pub async fn system_status(State(_state): State<AppState>) -> ServerResult<impl IntoResponse> {
    let status = SystemStatusResponse {};

    Ok((StatusCode::OK, Json(status)))
}

/// Handler for sandbox configuration
pub async fn sandbox_config(State(_state): State<AppState>) -> ServerResult<impl IntoResponse> {
    let response = SandboxConfigResponse {};

    Ok((StatusCode::OK, Json(response)))
}

/// Handler for sandbox status
pub async fn sandbox_status(State(_state): State<AppState>) -> ServerResult<impl IntoResponse> {
    // TODO: Implement actual sandbox status logic
    let sandbox1 = SandboxStatus {};
    let sandbox2 = SandboxStatus {};

    let response = SandboxStatusResponse {
        sandboxes: vec![sandbox1, sandbox2],
    };

    Ok((StatusCode::OK, Json(response)))
}

//--------------------------------------------------------------------------------------------------
// JSON-RPC Handlers
//--------------------------------------------------------------------------------------------------

/// Handler for running code in a sandbox
pub async fn run_code(
    State(_state): State<AppState>,
    Json(payload): Json<RunCodeRequest>,
) -> ServerResult<impl IntoResponse> {
    // TODO: Implement code execution logic
    let result = format!(
        "Code execution requested in sandbox: {} (namespace: {})",
        payload.sandbox, payload.namespace
    );

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result,
        id: Some(1),
    };

    Ok((StatusCode::OK, Json(response)))
}

//--------------------------------------------------------------------------------------------------
// Proxy Handlers
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
