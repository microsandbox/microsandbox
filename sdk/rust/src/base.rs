use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;

use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{Execution, SandboxError, SandboxOptions};

/// Base implementation for sandbox types
pub struct SandboxBase {
    /// URL of the Microsandbox server
    pub(crate) server_url: String,

    /// Namespace for the sandbox
    pub(crate) namespace: String,

    /// Name of the sandbox
    pub(crate) name: String,

    /// API key for Microsandbox server authentication
    pub(crate) api_key: Option<String>,

    /// HTTP client for API requests
    pub(crate) client: reqwest::Client,

    /// Whether the sandbox has been started
    pub(crate) is_started: bool,
}

impl SandboxBase {
    /// Create a new sandbox base
    pub fn new(options: &SandboxOptions) -> Self {
        // Try to load .env file if MSB_API_KEY is not set
        if env::var("MSB_API_KEY").is_err() {
            // Ignore errors if .env file doesn't exist
            let _ = dotenv();
        }

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
    pub(crate) async fn make_request<T: for<'de> Deserialize<'de>>(
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

        let _result: Value = self.make_request("sandbox.stop", params).await?;
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
