//! Metrics interface for sandboxes

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

use crate::SandboxBase;
use crate::SandboxError;

/// Resource usage metrics for a sandbox
#[derive(Debug, Clone)]
pub struct SandboxMetrics {
    /// CPU usage as a percentage
    cpu_usage: f64,
    /// Memory usage in bytes
    memory_usage: u64,
    /// Disk usage in bytes
    disk_usage: u64,
    /// Network usage in bytes
    network_usage: u64,
}

impl SandboxMetrics {
    /// Create a new metrics instance from data
    fn new(metrics_data: HashMap<String, Value>) -> Self {
        let cpu_usage = metrics_data
            .get("cpu_usage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let memory_usage = metrics_data
            .get("memory_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let disk_usage = metrics_data
            .get("disk_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let network_usage = metrics_data
            .get("network_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_usage,
        }
    }

    /// Get the CPU usage as a percentage
    pub fn cpu_usage(&self) -> f64 {
        self.cpu_usage
    }

    /// Get the memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage
    }

    /// Get the disk usage in bytes
    pub fn disk_usage(&self) -> u64 {
        self.disk_usage
    }

    /// Get the network usage in bytes
    pub fn network_usage(&self) -> u64 {
        self.network_usage
    }
}

/// Metrics interface for retrieving sandbox resource usage metrics
pub struct Metrics<'a> {
    sandbox: &'a SandboxBase,
}

impl<'a> Metrics<'a> {
    /// Create a new metrics instance
    pub(crate) fn new(sandbox: &'a SandboxBase) -> Self {
        Self { sandbox }
    }

    /// Get current resource usage metrics for the sandbox
    pub async fn get(&self) -> Result<SandboxMetrics, Box<dyn Error + Send + Sync>> {
        if !self.sandbox.is_started {
            return Err(Box::new(SandboxError::NotStarted));
        }

        let params = serde_json::json!({
            "sandbox": self.sandbox.name,
            "namespace": self.sandbox.namespace,
        });

        let result: HashMap<String, Value> = self
            .sandbox
            .make_request("sandbox.metrics.get", params)
            .await?;

        Ok(SandboxMetrics::new(result))
    }
}
