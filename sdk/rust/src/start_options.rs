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
