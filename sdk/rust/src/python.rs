//! Python-specific sandbox implementation

use std::error::Error;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{BaseSandbox, Command, Execution, Metrics, SandboxBase, SandboxOptions, StartOptions};

/// Python-specific sandbox for executing Python code
pub struct PythonSandbox {
    /// Base sandbox implementation
    base: Arc<Mutex<SandboxBase>>,
}

impl PythonSandbox {
    /// Create a new Python sandbox
    pub async fn create(options: SandboxOptions) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut base = SandboxBase::new(&options);

        // Create sandbox
        let sandbox = Self {
            base: Arc::new(Mutex::new(base)),
        };

        // Start sandbox with default options
        sandbox.start(None).await?;

        Ok(sandbox)
    }

    /// Get the command interface for executing shell commands
    pub fn command(&self) -> Command {
        let base = self.base.lock().unwrap();
        Command::new(&base)
    }

    /// Get the metrics interface for retrieving resource usage metrics
    pub fn metrics(&self) -> Metrics {
        let base = self.base.lock().unwrap();
        Metrics::new(&base)
    }
}

#[async_trait]
impl BaseSandbox for PythonSandbox {
    async fn get_default_image(&self) -> String {
        "appcypher/msb-python".to_string()
    }

    async fn run(&self, code: &str) -> Result<Execution, Box<dyn Error + Send + Sync>> {
        let base = self.base.lock().unwrap();
        base.run_code("python", code).await
    }

    async fn start(
        &mut self,
        options: Option<StartOptions>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let opts = options.unwrap_or_default();
        let image = opts.image.or_else(|| Some(self.get_default_image().await));

        let mut base = self.base.lock().unwrap();
        base.start_sandbox(image, opts.memory, opts.cpus, opts.timeout)
            .await
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut base = self.base.lock().unwrap();
        base.stop_sandbox().await
    }
}
