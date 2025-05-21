// ABOUTME: Error types for transparent-classroom-photos-grabber-rs
// ABOUTME: Centralizes all error handling for the application

use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    /// Environment variable errors
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),

    /// Errors from the IO subsystem
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Generic application errors
    #[error("{0}")]
    Generic(String),
}

/// Create a new generic error with a message
pub fn generic_error<S: Into<String>>(message: S) -> AppError {
    AppError::Generic(message.into())
}
