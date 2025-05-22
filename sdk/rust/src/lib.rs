//! Microsandbox Rust SDK
//!
//! A Rust SDK for the Microsandbox project that provides secure sandbox environments
//! for executing untrusted code. This SDK allows you to create isolated environments
//! for running code with controlled access to system resources.

use async_trait::async_trait;

// Re-export common types
pub use base::SandboxBase;
pub use builder::SandboxOptions;
pub use command::Command;
pub use error::SandboxError;
pub use execution::Execution;
pub use node::NodeSandbox;
pub use python::PythonSandbox;
pub use start_options::StartOptions;

mod base;
mod builder;
mod command;
mod error;
mod execution;
mod node;
mod python;
mod start_options;

/// Base trait for sandbox implementations
#[async_trait]
pub trait BaseSandbox: Send + Sync {
    /// Get the default Docker image for this sandbox type
    async fn get_default_image(&self) -> String;

    /// Execute code in the sandbox
    async fn run(&self, code: &str) -> Result<Execution, Box<dyn std::error::Error + Send + Sync>>;

    /// Run code, automatically starting the sandbox if needed
    async fn run_or_start(
        &mut self,
        code: &str,
    ) -> Result<Execution, Box<dyn std::error::Error + Send + Sync>> {
        // Check if sandbox is started
        let is_started = self.is_started().await;

        if !is_started {
            // Start sandbox
            self.start(None).await?;
        }

        // Run code
        self.run(code).await
    }

    /// Check if the sandbox is started
    async fn is_started(&self) -> bool {
        false // Override in implementations
    }

    /// Start the sandbox container
    async fn start(
        &mut self,
        options: Option<StartOptions>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Stop the sandbox container
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
