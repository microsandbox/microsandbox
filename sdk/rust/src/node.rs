//! Node.js-specific sandbox implementation

use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::command::Command;
use crate::{BaseSandbox, Execution, SandboxBase, SandboxOptions, StartOptions};

/// Node.js-specific sandbox for executing JavaScript code
pub struct NodeSandbox {
    /// Base sandbox implementation
    base: Arc<Mutex<SandboxBase>>,
}

impl NodeSandbox {
    /// Create a new Node.js sandbox with a name
    pub async fn create(name: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let options = SandboxOptions::builder().name(name).build();
        Self::create_with_options(options).await
    }

    /// Create a new Node.js sandbox with options
    pub async fn create_with_options(
        options: SandboxOptions,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let base = SandboxBase::new(&options);

        // Create sandbox
        let sandbox = Self {
            base: Arc::new(Mutex::new(base)),
        };

        Ok(sandbox)
    }

    /// Get the command interface for executing shell commands
    pub async fn command(&self) -> Result<Command, Box<dyn Error + Send + Sync>> {
        Ok(Command::new(self.base.clone()))
    }
}

#[async_trait]
impl BaseSandbox for NodeSandbox {
    async fn get_default_image(&self) -> String {
        "appcypher/msb-nodejs".to_string()
    }

    async fn is_started(&self) -> bool {
        let base = self.base.lock().await;
        base.is_started
    }

    async fn run(&self, code: &str) -> Result<Execution, Box<dyn Error + Send + Sync>> {
        // Check if sandbox is started
        let is_started = {
            let base = self.base.lock().await;
            base.is_started
        };

        if !is_started {
            return Err(Box::new(crate::SandboxError::NotStarted));
        }

        // Execute code
        let base = self.base.lock().await;
        base.run_code("node", code).await
    }

    async fn start(
        &mut self,
        options: Option<StartOptions>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let opts = options.unwrap_or_default();

        // Get default image
        let default_image = self.get_default_image().await;
        let image = opts.image.or_else(|| Some(default_image));

        let mut base = self.base.lock().await;
        base.start_sandbox(image, opts.memory, opts.cpus, opts.timeout)
            .await
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Check if already stopped
        let is_started = {
            let base = self.base.lock().await;
            base.is_started
        };

        if !is_started {
            return Ok(());
        }

        // Stop sandbox
        let mut base = self.base.lock().await;
        base.stop_sandbox().await
    }
}
