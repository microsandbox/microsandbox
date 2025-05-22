use std::error::Error;
use std::fmt;

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

    /// Invalid response received from server
    InvalidResponse(String),

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
            SandboxError::InvalidResponse(msg) => {
                write!(f, "Invalid response from server: {}", msg)
            }
            SandboxError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for SandboxError {}
