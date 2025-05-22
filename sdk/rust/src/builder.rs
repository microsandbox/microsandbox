//! Builder pattern implementation for sandbox options

/// Options for creating a sandbox
#[derive(Debug, Clone)]
pub struct SandboxOptions {
    /// URL of the Microsandbox server
    pub(crate) server_url: Option<String>,

    /// Namespace for the sandbox
    pub(crate) namespace: Option<String>,

    /// Name of the sandbox
    pub(crate) name: Option<String>,

    /// API key for Microsandbox server authentication
    pub(crate) api_key: Option<String>,
}

/// Builder for sandbox options
#[derive(Debug, Clone, Default)]
pub struct SandboxOptionsBuilder {
    server_url: Option<String>,
    namespace: Option<String>,
    name: Option<String>,
    api_key: Option<String>,
}

impl SandboxOptions {
    /// Create a new builder for SandboxOptions
    pub fn builder() -> SandboxOptionsBuilder {
        SandboxOptionsBuilder::default()
    }
}

impl SandboxOptionsBuilder {
    /// Set the server URL
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    /// Set the namespace
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Set the sandbox name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Build the SandboxOptions
    pub fn build(self) -> SandboxOptions {
        SandboxOptions {
            server_url: self.server_url,
            namespace: self.namespace,
            name: self.name,
            api_key: self.api_key,
        }
    }
}
