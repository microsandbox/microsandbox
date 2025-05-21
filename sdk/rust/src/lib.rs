//! Microsandbox Rust SDK
//!
//! A Rust SDK for the Microsandbox project that provides secure sandbox environments
//! for executing untrusted code. This SDK allows you to create isolated environments
//! for running code with controlled access to system resources.

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

// Re-export common types
pub use builder::SandboxOptions;
pub use execution::Execution;
pub use node::NodeSandbox;
pub use python::PythonSandbox;

mod builder;
mod command;
mod execution;
mod metrics;
mod node;
mod python;

/// Base trait for sandbox implementations
#[async_trait::async_trait]
pub trait BaseSandbox {
    /// Get the default Docker image for this sandbox type
    async fn get_default_image(&self) -> String;

    /// Execute code in the sandbox
    async fn run(&self, code: &str) -> Result<Execution, Box<dyn Error + Send + Sync>>;

    /// Start the sandbox container
    async fn start(
        &mut self,
        options: Option<StartOptions>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Stop the sandbox container
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Options for starting a sandbox
#[derive(Debug, Clone)]
pub struct StartOptions {
    /// Docker image to use for the sandbox
    pub image: Option<String>,
    /// Memory limit in MB
    pub memory: u32,
    /// CPU limit
    pub cpus: f32,
    /// Maximum time in seconds to wait for the sandbox to start
    pub timeout: f32,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            image: None,
            memory: 512,
            cpus: 1.0,
            timeout: 180.0,
        }
    }
}

/// Common error types for the Microsandbox SDK
#[derive(Debug)]
pub enum SandboxError {
    /// The sandbox has not been started
    NotStarted,
    /// The request to the server failed
    RequestFailed(String),
    /// The server returned an error
    ServerError(String),
    /// The sandbox timed out
    Timeout(String),
    /// An error occurred with the HTTP client
    HttpError(String),
    /// General error
    General(String),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::NotStarted => write!(f, "Sandbox is not started. Call start() first."),
            SandboxError::RequestFailed(msg) => {
                write!(f, "Failed to communicate with Microsandbox server: {}", msg)
            }
            SandboxError::ServerError(msg) => write!(f, "Server error: {}", msg),
            SandboxError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            SandboxError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SandboxError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for SandboxError {}

/// Base implementation for sandbox types
pub struct SandboxBase {
    /// URL of the Microsandbox server
    server_url: String,
    /// Namespace for the sandbox
    namespace: String,
    /// Name of the sandbox
    name: String,
    /// API key for Microsandbox server authentication
    api_key: Option<String>,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Whether the sandbox has been started
    is_started: bool,
}

impl SandboxBase {
    /// Create a new sandbox base
    pub fn new(options: &SandboxOptions) -> Self {
        // Get server URL from options, environment, or default
        let server_url = options
            .server_url
            .clone()
            .or_else(|| env::var("MSB_SERVER_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:5555".to_string());

        // Get API key from options or environment
        let api_key = options
            .api_key
            .clone()
            .or_else(|| env::var("MSB_API_KEY").ok());

        // Generate a random name if not provided
        let name = options.name.clone().unwrap_or_else(|| {
            format!(
                "sandbox-{}",
                Uuid::new_v4().to_string().split('-').next().unwrap()
            )
        });

        Self {
            server_url,
            namespace: options
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            name,
            api_key,
            client: reqwest::Client::new(),
            is_started: false,
        }
    }

    /// Make a JSON-RPC request to the Microsandbox server
    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        // Create headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(api_key) = &self.api_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))?,
            );
        }

        // Create request body
        let request_data = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": Uuid::new_v4().to_string(),
        });

        // Send request
        let response = self
            .client
            .post(&format!("{}/api/v1/rpc", self.server_url))
            .headers(headers)
            .json(&request_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Box::new(SandboxError::RequestFailed(error_text)));
        }

        // Parse response
        let response_data: Value = response.json().await?;

        if let Some(error) = response_data.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            return Err(Box::new(SandboxError::ServerError(error_msg)));
        }

        // Extract and deserialize result
        let result =
            serde_json::from_value(response_data.get("result").cloned().unwrap_or(Value::Null))?;

        Ok(result)
    }

    /// Start the sandbox container
    pub async fn start_sandbox(
        &mut self,
        image: Option<String>,
        memory: u32,
        cpus: f32,
        timeout: f32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.is_started {
            return Ok(());
        }

        let params = json!({
            "namespace": self.namespace,
            "sandbox": self.name,
            "config": {
                "image": image,
                "memory": memory,
                "cpus": cpus.round() as i32,
            }
        });

        // Set client timeout to be slightly longer than the server timeout
        let client_timeout = Duration::from_secs_f32(timeout + 30.0);
        let client = reqwest::Client::builder().timeout(client_timeout).build()?;

        let request_data = json!({
            "jsonrpc": "2.0",
            "method": "sandbox.start",
            "params": params,
            "id": Uuid::new_v4().to_string(),
        });

        // Create headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(api_key) = &self.api_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))?,
            );
        }

        // Send request
        let response = match client
            .post(&format!("{}/api/v1/rpc", self.server_url))
            .headers(headers)
            .json(&request_data)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    return Err(Box::new(SandboxError::Timeout(format!(
                        "Timed out waiting for sandbox to start after {} seconds",
                        timeout
                    ))));
                }
                return Err(Box::new(SandboxError::HttpError(e.to_string())));
            }
        };

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Box::new(SandboxError::RequestFailed(error_text)));
        }

        // Parse response
        let response_data: Value = response.json().await?;

        if let Some(error) = response_data.get("error") {
            let error_msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            return Err(Box::new(SandboxError::ServerError(error_msg)));
        }

        // Check for warning in result
        if let Some(result) = response_data.get("result") {
            if let Some(result_str) = result.as_str() {
                if result_str.contains("timed out waiting") {
                    eprintln!("Sandbox start warning: {}", result_str);
                }
            }
        }

        self.is_started = true;
        Ok(())
    }

    /// Stop the sandbox container
    pub async fn stop_sandbox(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.is_started {
            return Ok(());
        }

        let params = json!({
            "namespace": self.namespace,
            "sandbox": self.name,
        });

        let result: Value = self.make_request("sandbox.stop", params).await?;
        self.is_started = false;

        Ok(())
    }

    /// Execute code in the sandbox
    pub async fn run_code(
        &self,
        language: &str,
        code: &str,
    ) -> Result<Execution, Box<dyn Error + Send + Sync>> {
        if !self.is_started {
            return Err(Box::new(SandboxError::NotStarted));
        }

        let params = json!({
            "sandbox": self.name,
            "namespace": self.namespace,
            "language": language,
            "code": code,
        });

        let result: HashMap<String, Value> = self.make_request("sandbox.repl.run", params).await?;
        Ok(Execution::new(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add tests here
}
