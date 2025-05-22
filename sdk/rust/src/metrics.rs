use std::error::Error;
use std::sync::Arc;

use serde_json::json;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::base::SandboxBase;

/// Metrics interface for the Microsandbox Rust SDK.
pub struct Metrics {
    /// Base sandbox implementation
    base: Arc<Mutex<SandboxBase>>,
}

impl Metrics {
    /// Create a new Metrics instance
    pub fn new(base: Arc<Mutex<SandboxBase>>) -> Self {
        Self { base }
    }

    /// Internal method to fetch current metrics from the server
    async fn get_metrics(&self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        // Check if sandbox is started
        let is_started = {
            let base = self.base.lock().await;
            base.is_started
        };

        if !is_started {
            return Err(Box::new(crate::SandboxError::NotStarted));
        }

        // Extract sandbox details
        let (server_url, namespace, sandbox_name, api_key) = {
            let base = self.base.lock().await;
            (
                base.server_url.clone(),
                base.namespace.clone(),
                base.name.clone(),
                base.api_key.clone(),
            )
        };

        // Build request payload
        let request_id = Uuid::new_v4().to_string();
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "sandbox.metrics.get",
            "params": {
                "namespace": namespace,
                "sandbox": sandbox_name,
            },
            "id": request_id,
        });

        // Create HTTP client
        let client = reqwest::Client::new();
        let mut req_builder = client
            .post(&format!("{}/api/v1/rpc", server_url))
            .json(&payload)
            .header("Content-Type", "application/json");

        // Add API key if present
        if let Some(key) = api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
        }

        // Send request
        let response = req_builder
            .send()
            .await
            .map_err(|e| Box::new(crate::SandboxError::RequestFailed(e.to_string())))?;

        // Check status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Box::new(crate::SandboxError::RequestFailed(format!(
                "Failed to get sandbox metrics: {} - {}",
                status, error_text
            ))));
        }

        // Parse response
        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Box::new(crate::SandboxError::InvalidResponse(e.to_string())))?;

        // Check for errors in response
        if let Some(error) = response_data.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(Box::new(crate::SandboxError::RequestFailed(format!(
                "Failed to get sandbox metrics: {}",
                message
            ))));
        }

        // Extract result and sandboxes array
        let result = response_data.get("result").ok_or_else(|| {
            crate::SandboxError::InvalidResponse("Missing 'result' field".to_string())
        })?;

        let sandboxes = result
            .get("sandboxes")
            .and_then(|s| s.as_array())
            .ok_or_else(|| {
                crate::SandboxError::InvalidResponse("Missing 'sandboxes' array".to_string())
            })?;

        // We expect exactly one sandbox in the response (our own)
        if sandboxes.is_empty() {
            return Ok(json!({}));
        }

        // Return the first (and should be only) sandbox data
        Ok(sandboxes[0].clone())
    }

    /// Get all metrics for the current sandbox
    ///
    /// Returns a JSON object containing all metrics for the sandbox:
    /// ```json
    /// {
    ///   "name": "sandbox-name",
    ///   "namespace": "namespace",
    ///   "running": true,
    ///   "cpu_usage": 0.5,
    ///   "memory_usage": 128,
    ///   "disk_usage": 1024
    /// }
    /// ```
    pub async fn all(&self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        self.get_metrics().await
    }

    /// Get CPU usage percentage for the current sandbox
    ///
    /// Returns CPU usage as a percentage (0-100) or None if not available.
    /// May return 0.0 for idle sandboxes or when metrics are not precise.
    pub async fn cpu(&self) -> Result<Option<f32>, Box<dyn Error + Send + Sync>> {
        let metrics = self.get_metrics().await?;
        Ok(metrics
            .get("cpu_usage")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32))
    }

    /// Get memory usage for the current sandbox
    ///
    /// Returns memory usage in MiB or None if not available
    pub async fn memory(&self) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        let metrics = self.get_metrics().await?;
        Ok(metrics.get("memory_usage").and_then(|v| v.as_u64()))
    }

    /// Get disk usage for the current sandbox
    ///
    /// Returns disk usage in bytes or None if not available
    pub async fn disk(&self) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        let metrics = self.get_metrics().await?;
        Ok(metrics.get("disk_usage").and_then(|v| v.as_u64()))
    }

    /// Check if the sandbox is currently running
    ///
    /// Returns true if the sandbox is running, false otherwise
    pub async fn is_running(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let metrics = self.get_metrics().await?;
        Ok(metrics
            .get("running")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}
