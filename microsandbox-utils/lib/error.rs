//! `microsandbox_utils::error` is a module containing error utilities for the microsandbox project.

use std::{
    error::Error,
    fmt::{self, Display},
};

use thiserror::Error;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// The result of a microsandbox-utils-related operation.
pub type MicrosandboxUtilsResult<T> = Result<T, MicrosandboxUtilsError>;

/// An error that occurred during a file system operation.
#[derive(pretty_error_debug::Debug, Error)]
pub enum MicrosandboxUtilsError {
    /// An error that occurred when validating paths
    #[error("path validation error: {0}")]
    PathValidation(String),

    /// An error that occurred when resolving a file
    #[error("file not found at: {0}\nSource: {1}")]
    FileNotFound(String, String),

    /// An error that occurred when performing an IO operation
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error that occurred during a runtime operation
    #[error("runtime error: {0}")]
    Runtime(String),

    /// An error from the nix crate
    #[error("nix error: {0}")]
    NixError(#[from] nix::Error),

    /// Custom error.
    #[error("Custom error: {0}")]
    Custom(#[from] AnyError),
}

/// An error that can represent any error.
#[derive(Debug)]
pub struct AnyError {
    error: anyhow::Error,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl MicrosandboxUtilsError {
    /// Creates a new `Err` result.
    pub fn custom(error: impl Into<anyhow::Error>) -> MicrosandboxUtilsError {
        MicrosandboxUtilsError::Custom(AnyError {
            error: error.into(),
        })
    }
}

impl AnyError {
    /// Downcasts the error to a `T`.
    pub fn downcast<T>(&self) -> Option<&T>
    where
        T: Display + fmt::Debug + Send + Sync + 'static,
    {
        self.error.downcast_ref::<T>()
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Creates an `Ok` `MicrosandboxUtilsResult`.
#[allow(non_snake_case)]
pub fn Ok<T>(value: T) -> MicrosandboxUtilsResult<T> {
    Result::Ok(value)
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl PartialEq for AnyError {
    fn eq(&self, other: &Self) -> bool {
        self.error.to_string() == other.error.to_string()
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Error for AnyError {}
