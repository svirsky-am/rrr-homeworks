//! Custom error types for the streaming quotes project.
//!
//! This module defines all error types used throughout the application,
//! providing consistent error handling and reporting.

use thiserror::Error;

/// Main error type for the streaming quotes project.
#[derive(Error, Debug)]
pub enum QuoteError {
    /// Error when binding to a network address fails.
    #[error("Failed to bind to {addr}: {source}")]
    BindError {
        /// The address that failed to bind.
        addr: String,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error when connecting to a server fails.
    #[error("Failed to connect to {addr}: {source}")]
    ConnectError {
        /// The server address.
        addr: String,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error when parsing a socket address fails.
    #[error("Invalid address format: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// Error when an unsupported ticker is requested.
    #[error("Unsupported ticker symbol: {0}")]
    UnsupportedTicker(String),

    /// Error when client command parsing fails.
    #[error("Invalid command format: {0}")]
    InvalidCommand(String),

    /// Error when client times out due to inactivity.
    #[error("Client timeout: no activity for {seconds} seconds")]
    ClientTimeout {
        /// Number of seconds of inactivity.
        seconds: u64,
    },

    /// Error when sending data over UDP fails.
    #[error("Failed to send data: {0}")]
    SendError(#[from] std::io::Error),

    /// Error when parsing stock quote data fails.
    #[error("Failed to parse quote data: {0}")]
    ParseError(String),

    /// Error when argument parsing fails.
    #[error("Invalid argument: {0}")]
    ArgumentError(String),

    /// Error when required argument is missing.
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Error argument --filer-list or --filer-file is missing.
    #[error("Missing required argument --filer-list or --filer-file")]
    MissingFilterArgument,

    /// Error argument --filer-list or --filer-file is missing.
    #[error("Just only one required argument --filer-list or --filer-file")]
    BothFiltersProvided,
}

/// Result type alias for quote operations.
pub type QuoteResult<T> = Result<T, QuoteError>;
