//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

use thiserror::Error;

/// The result of a microsandbox-cli related operation.
pub type MicrosandboxCliResult<T> = Result<T, MicrosandboxCliError>;

/// An error that occurred during a file system operation.
#[derive(pretty_error_debug::Debug, Error)]
pub enum MicrosandboxCliError {
    /// An I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Error returned from the microsandbox-server crate
    #[error(transparent)]
    Server(#[from] microsandbox_server::MicrosandboxServerError),

    /// Error returned from the microsandbox-core crate
    #[error(transparent)]
    Core(#[from] microsandbox_core::MicrosandboxError),

    /// Invalid argument
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Not found
    #[error("not found: {0}")]
    NotFound(String),

    /// Process wait error
    #[error("process wait error: {0}")]
    ProcessWaitError(String),
}
