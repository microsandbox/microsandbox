//! Error handling module for the microsandbox server.
//!
//! This module provides comprehensive error handling functionality including:
//! - Custom error types for server operations
//! - Error codes and responses for API communication
//! - Authentication and authorization error handling
//! - Validation error handling
//!
//! The module implements:
//! - Error types with detailed error messages
//! - HTTP status code mapping
//! - Serializable error responses for API clients
//! - Structured error codes for frontend handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use microsandbox_utils::MicrosandboxUtilsError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// The result of a server-related operation.
pub type MicrosandboxServerResult<T> = Result<T, MicrosandboxServerError>;

/// Error returned when an unexpected internal error occurs
#[derive(Error, Debug)]
pub enum MicrosandboxServerError {
    /// Error returned when the server fails to start
    #[error("Server failed to start: {0}")]
    StartError(String),

    /// Error returned when the server fails to stop
    #[error("Server failed to stop: {0}")]
    StopError(String),

    /// Error returned when the server key fails to generate
    #[error("Server key failed to generate: {0}")]
    KeyGenError(String),

    /// Error returned when the server configuration fails
    #[error("Server configuration failed: {0}")]
    ConfigError(String),

    /// Error returned when an I/O error occurs
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Error returned from the microsandbox-utils crate
    #[error(transparent)]
    Utils(#[from] MicrosandboxUtilsError),
}

/// Represents all possible errors that can occur in the application
#[derive(Error, Debug)]
pub enum ServerError {
    /// Error returned when authentication fails
    #[error("Authentication failed: {0}")]
    Authentication(AuthenticationError),

    /// Error returned when a user doesn't have permission to access a resource
    #[error("Authorization failed: {0}")]
    AuthorizationError(AuthorizationError),

    /// Error returned when a requested resource is not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Error returned when a database operation fails
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Error returned when request validation fails (e.g., invalid input format)
    #[error("Validation error: {0}")]
    ValidationError(ValidationError),

    /// Error returned when an unexpected internal error occurs
    #[error("Internal server error: {0}")]
    InternalError(String),
}

/// Error code structure to be sent to frontend
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Authentication error codes
    /// Error returned when credentials provided don't match our records
    InvalidCredentials = 1001,
    /// Error returned when a user attempts to login with an unconfirmed email
    EmailNotConfirmed = 1002,
    /// Error returned when there have been too many failed login attempts
    TooManyLoginAttempts = 1003,
    /// Error returned when an authentication token is invalid
    InvalidToken = 1004,
    /// Error returned when an authentication token has expired
    ExpiredToken = 1005,
    /// Error returned when a token is required but not provided
    TokenRequired = 1006,
    /// Error returned when attempting to register with an email that already exists
    EmailAlreadyExists = 1007,
    /// Error returned when a user tries to sign in with password but should use Google
    UseGoogleLogin = 1008,
    /// Error returned when a user tries to sign in with password but should use GitHub
    UseGithubLogin = 1009,
    /// Error returned when a user tries to use OAuth but should use email/password
    UseEmailLogin = 1010,
    /// Error returned when a user's email is not verified with their OAuth provider
    EmailNotVerified = 1011,

    // Validation error codes
    /// Error returned when input fails validation rules
    InvalidInput = 2001,
    /// Error returned when password doesn't meet strength requirements
    PasswordTooWeak = 2002,
    /// Error returned when email format is invalid
    EmailInvalid = 2003,
    /// Error returned when a confirmation token is invalid or has expired
    InvalidOrExpiredConfirmationToken = 2004,

    // Authorization error codes
    /// Error returned when a user is denied access to a resource
    AccessDenied = 3001,
    /// Error returned when a user doesn't have sufficient permissions for an action
    InsufficientPermissions = 3002,

    // Resource error codes
    /// Error returned when a requested resource cannot be found
    ResourceNotFound = 4001,

    // Server error codes
    /// Error returned when a database operation fails
    DatabaseError = 5001,
    /// Error returned when an unexpected server error occurs
    InternalServerError = 5002,
}

/// Represents different types of authentication failures
#[derive(Error, Debug)]
pub enum AuthenticationError {
    /// Security-sensitive authentication failures that shouldn't reveal details
    #[error("Invalid credentials")]
    InvalidCredentials(String),

    /// User-facing authentication errors that can be shown directly
    #[error("{0}")]
    ClientError(String),

    /// Email not confirmed
    #[error("Email not confirmed")]
    EmailNotConfirmed,

    /// Too many login attempts
    #[error("Too many login attempts")]
    TooManyAttempts,

    /// Invalid or expired token
    #[error("Invalid or expired token")]
    InvalidToken(String),

    /// Email already registered
    #[error("Email already registered")]
    EmailAlreadyExists,

    /// Should use Google login instead
    #[error("Use Google login")]
    UseGoogleLogin,

    /// Should use GitHub login instead
    #[error("Use GitHub login")]
    UseGithubLogin,

    /// Should use email/password login instead
    #[error("Use email/password login")]
    UseEmailLogin,

    /// Email not verified with provider
    #[error("Email not verified")]
    EmailNotVerified,
}

/// Represents validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Generic validation error
    #[error("{0}")]
    InvalidInput(String),

    /// Password not strong enough
    #[error("Password is too weak")]
    PasswordTooWeak(String),

    /// Email format invalid
    #[error("Email is invalid")]
    EmailInvalid(String),

    /// Invalid or expired confirmation token
    #[error("Invalid or expired confirmation token")]
    InvalidConfirmationToken,
}

/// Represents authorization errors
#[derive(Error, Debug)]
pub enum AuthorizationError {
    /// Access denied
    #[error("Access denied")]
    AccessDenied(String),

    /// Insufficient permissions
    #[error("Insufficient permissions")]
    InsufficientPermissions(String),
}

/// Response structure for errors
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: Option<u32>,
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl IntoResponse for ServerError {
    /// Converts the ServerError into an HTTP response with appropriate status code
    /// and JSON error message.
    ///
    /// ## Returns
    ///
    /// Returns an HTTP response containing:
    /// - Appropriate HTTP status code based on the error type
    /// - JSON body with an "error" field containing the error message
    /// - And an optional "code" field with a numeric error code for the frontend
    fn into_response(self) -> Response {
        // Log the actual error with details
        error!(error = ?self, "API error occurred");

        let (status, error_message, error_code) = match self {
            ServerError::Authentication(auth_error) => {
                match auth_error {
                    AuthenticationError::InvalidCredentials(_details) => {
                        // Generic message for security-sensitive auth failures
                        error!(details = ?_details, "Authentication error");
                        (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string(), Some(ErrorCode::InvalidCredentials as u32))
                    }
                    AuthenticationError::ClientError(details) => {
                        // Safe to show these messages to users
                        error!(details = ?details, "User-facing authentication error");
                        (StatusCode::UNAUTHORIZED, details, None)
                    }
                    AuthenticationError::EmailNotConfirmed => {
                        (StatusCode::UNAUTHORIZED, "Email not confirmed".to_string(), Some(ErrorCode::EmailNotConfirmed as u32))
                    }
                    AuthenticationError::TooManyAttempts => {
                        (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts, please try again later".to_string(), Some(ErrorCode::TooManyLoginAttempts as u32))
                    }
                    AuthenticationError::InvalidToken(details) => {
                        error!(details = ?details, "Invalid token");
                        (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string(), Some(ErrorCode::InvalidToken as u32))
                    }
                    AuthenticationError::EmailAlreadyExists => {
                        (StatusCode::CONFLICT, "Email already registered".to_string(), Some(ErrorCode::EmailAlreadyExists as u32))
                    }
                    AuthenticationError::UseGoogleLogin => {
                        (StatusCode::UNAUTHORIZED, "This email is registered with Google. Please use 'Sign in with Google' instead.".to_string(), Some(ErrorCode::UseGoogleLogin as u32))
                    }
                    AuthenticationError::UseGithubLogin => {
                        (StatusCode::UNAUTHORIZED, "This email is registered with GitHub. Please use 'Sign in with GitHub' instead.".to_string(), Some(ErrorCode::UseGithubLogin as u32))
                    }
                    AuthenticationError::UseEmailLogin => {
                        (StatusCode::UNAUTHORIZED, "This email is already registered. Please login with your password.".to_string(), Some(ErrorCode::UseEmailLogin as u32))
                    }
                    AuthenticationError::EmailNotVerified => {
                        (StatusCode::UNAUTHORIZED, "Email not verified with the provider".to_string(), Some(ErrorCode::EmailNotVerified as u32))
                    }
                }
            }
            ServerError::AuthorizationError(auth_error) => match auth_error {
                AuthorizationError::AccessDenied(details) => {
                    error!(details = ?details, "Access denied");
                    (
                        StatusCode::FORBIDDEN,
                        "Access denied".to_string(),
                        Some(ErrorCode::AccessDenied as u32),
                    )
                }
                AuthorizationError::InsufficientPermissions(details) => {
                    error!(details = ?details, "Insufficient permissions");
                    (
                        StatusCode::FORBIDDEN,
                        "Insufficient permissions".to_string(),
                        Some(ErrorCode::InsufficientPermissions as u32),
                    )
                }
            },
            ServerError::NotFound(details) => (
                StatusCode::NOT_FOUND,
                details,
                Some(ErrorCode::ResourceNotFound as u32),
            ),
            ServerError::DatabaseError(details) => {
                error!(details = ?details, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    Some(ErrorCode::DatabaseError as u32),
                )
            }
            ServerError::ValidationError(validation_error) => match validation_error {
                ValidationError::InvalidInput(details) => (
                    StatusCode::BAD_REQUEST,
                    details,
                    Some(ErrorCode::InvalidInput as u32),
                ),
                ValidationError::PasswordTooWeak(details) => (
                    StatusCode::BAD_REQUEST,
                    details,
                    Some(ErrorCode::PasswordTooWeak as u32),
                ),
                ValidationError::EmailInvalid(details) => (
                    StatusCode::BAD_REQUEST,
                    details,
                    Some(ErrorCode::EmailInvalid as u32),
                ),
                ValidationError::InvalidConfirmationToken => (
                    StatusCode::BAD_REQUEST,
                    "Invalid or expired confirmation token".to_string(),
                    Some(ErrorCode::InvalidOrExpiredConfirmationToken as u32),
                ),
            },
            ServerError::InternalError(details) => {
                error!(details = ?details, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    Some(ErrorCode::InternalServerError as u32),
                )
            }
        };

        let body = Json(ErrorResponse {
            error: error_message,
            code: error_code,
        });

        (status, body).into_response()
    }
}
